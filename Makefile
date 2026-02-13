include openvino.conf

DOCKER_IMAGE := capi-builder:latest
OPENVINO_URL := https://storage.openvinotoolkit.org/repositories/openvino_genai/packages/$(OPENVINO_SHORT)/linux/openvino_genai_ubuntu22_$(OPENVINO_VERSION)_x86_64.tar.gz
LIBS_DIR := $(shell pwd)/libs
OPENVINO_DIR := $(LIBS_DIR)/openvino
SIDECAR_BIN_DIR := $(shell pwd)/capi-ui/src-tauri/bin

.PHONY: all build run download-deps clean-deps bundle prepare-libs

all: build

$(OPENVINO_DIR):
	@echo "Downloading OpenVINO $(OPENVINO_VERSION)..."
	@mkdir -p $(LIBS_DIR)
	@curl -L $(OPENVINO_URL) | tar -xz -C $(LIBS_DIR)
	@mv $(LIBS_DIR)/openvino_genai_ubuntu22_$(OPENVINO_VERSION)_x86_64 $(OPENVINO_DIR)
	@echo "OpenVINO installed in $(OPENVINO_DIR)"

download-deps: $(OPENVINO_DIR)

prepare-libs: download-deps
	@echo "Preparing essential OpenVINO libraries..."
	@mkdir -p target/release/lib
	@cp $(OPENVINO_DIR)/runtime/lib/intel64/libopenvino*.so* target/release/lib/
	@cp $(OPENVINO_DIR)/runtime/lib/intel64/cache.json target/release/lib/ 2>/dev/null || true
	@cp $(OPENVINO_DIR)/runtime/lib/intel64/libopenvino_tokenizers.so target/release/lib/ 2>/dev/null || true
	@cp $(OPENVINO_DIR)/runtime/lib/intel64/plugins.xml target/release/lib/ 2>/dev/null || true
	@cp $(OPENVINO_DIR)/runtime/3rdparty/tbb/lib/libtbb.so* target/release/lib/
	@cp $(OPENVINO_DIR)/runtime/3rdparty/tbb/lib/libhwloc.so* target/release/lib/
	@echo "Essential libraries staged in target/release/lib/"

build: download-deps
	@echo "Building Capi..."
	@OPENVINO_ROOT=$(OPENVINO_DIR) cargo build --release

run: build prepare-libs
	@echo "Running Capi..."
	@LD_LIBRARY_PATH=target/release/lib \
	./target/release/capi $(ARGS)

clean-deps:
	@rm -rf $(LIBS_DIR)
	@rm -rf target/release/lib
	@echo "Dependencies removed."

prepare-sidecar:
	@echo "Comparing sidecar binary..."
	@mkdir -p $(SIDECAR_BIN_DIR)
	@cargo build --release --bin capi-server
	@cp target/release/capi-server $(SIDECAR_BIN_DIR)/capi-server-x86_64-unknown-linux-gnu

run-ui: download-deps prepare-sidecar
	@echo "Installing UI dependencies..."
	@cd capi-ui && npm install
	@echo "Running Capi UI..."
	cd capi-ui && \
	OPENVINO_ROOT=$(OPENVINO_DIR) \
	LD_LIBRARY_PATH=$(OPENVINO_DIR)/runtime/lib/intel64:$(OPENVINO_DIR)/runtime/3rdparty/tbb/lib \
	npm run tauri dev

bundle: download-deps prepare-libs
	@echo "Building Capi..."
	@OPENVINO_ROOT=$(OPENVINO_DIR) cargo build --release --bin capi-server --bin capi-engine --bin capi
	@echo "Preparing Sidecar..."
	@mkdir -p $(SIDECAR_BIN_DIR)
	@cp target/release/capi-server $(SIDECAR_BIN_DIR)/capi-server-x86_64-unknown-linux-gnu
	@echo "Building UI Bundle..."
	@cd capi-ui && npm install && \
	OPENVINO_ROOT=$(OPENVINO_DIR) \
	LD_LIBRARY_PATH=$(OPENVINO_DIR)/runtime/lib/intel64:$(OPENVINO_DIR)/runtime/3rdparty/tbb/lib \
	npm run tauri build
	@echo "Creating Portable Tarball..."
	@rm -rf target/release/capi-linux-x64
	@mkdir -p target/release/capi-linux-x64/bin
	@mkdir -p target/release/capi-linux-x64/bin/lib
	@cp target/release/capi target/release/capi-linux-x64/bin/capi
	@cp target/release/capi-engine target/release/capi-linux-x64/bin/capi-engine
	@cp target/release/capi-server target/release/capi-linux-x64/bin/capi-server
	@cp target/release/capi-ui target/release/capi-linux-x64/bin/capi-ui
	@cp target/release/lib/* target/release/capi-linux-x64/bin/lib/
	@cd target/release && tar -czf capi-linux-x64.tar.gz capi-linux-x64
	@echo "Portable tarball created at target/release/capi-linux-x64.tar.gz"

docker-build:
	docker build -t $(DOCKER_IMAGE) -f Dockerfile.build .

docker-run-test:
	docker run --rm --user $(shell id -u):$(shell id -g) -v $(shell pwd):/app -e HOME=/tmp $(DOCKER_IMAGE) cargo test --workspace --exclude capi-ui --all-targets --all-features

docker-run-bundle:
	docker run --rm --user $(shell id -u):$(shell id -g) -v $(shell pwd):/app $(DOCKER_IMAGE) bash -c "\
		mkdir -p /app/libs && \
		ln -sfn /opt/openvino /app/libs/openvino && \
		make bundle OPENVINO_DIR=/app/libs/openvino"
