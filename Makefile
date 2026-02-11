OPENVINO_VER := 2025.4.1.0
OPENVINO_URL := https://storage.openvinotoolkit.org/repositories/openvino_genai/packages/2025.4.1/linux/openvino_genai_ubuntu22_$(OPENVINO_VER)_x86_64.tar.gz
LIBS_DIR := $(shell pwd)/libs
OPENVINO_DIR := $(LIBS_DIR)/openvino
SIDECAR_BIN_DIR := $(shell pwd)/capi-ui/src-tauri/bin

.PHONY: all build run download-deps clean-deps bundle

all: build

$(OPENVINO_DIR):
	@echo "Downloading OpenVINO $(OPENVINO_VER)..."
	@mkdir -p $(LIBS_DIR)
	@curl -L $(OPENVINO_URL) | tar -xz -C $(LIBS_DIR)
	@mv $(LIBS_DIR)/openvino_genai_ubuntu22_$(OPENVINO_VER)_x86_64 $(OPENVINO_DIR)
	@# Create symbolic links if needed or just rely on env vars
	@echo "OpenVINO installed in $(OPENVINO_DIR)"

download-deps: $(OPENVINO_DIR)

build: download-deps
	@echo "Building Capi..."
	@OPENVINO_ROOT=$(OPENVINO_DIR) cargo build --release

run: build
	@echo "Running Capi..."
	@LD_LIBRARY_PATH=$(OPENVINO_DIR)/runtime/lib/intel64:$(OPENVINO_DIR)/runtime/3rdparty/tbb/lib \
	OPENVINO_ROOT=$(OPENVINO_DIR) \
	./target/release/capi $(ARGS)

clean-deps:
	@rm -rf $(LIBS_DIR)
	@echo "Dependencies removed."

run-ui: download-deps
	@echo "Installing UI dependencies..."
	@cd capi-ui && npm install
	@echo "Running Capi UI..."
	cd capi-ui && \
	OPENVINO_ROOT=$(OPENVINO_DIR) \
	LD_LIBRARY_PATH=$(OPENVINO_DIR)/runtime/lib/intel64:$(OPENVINO_DIR)/runtime/3rdparty/tbb/lib \
	npm run tauri dev

bundle: download-deps
	@echo "Building Capi Server..."
	@OPENVINO_ROOT=$(OPENVINO_DIR) cargo build --release --bin capi-server
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
	@mkdir -p target/release/capi-linux-x64/lib
	@cp target/release/capi-ui target/release/capi-linux-x64/bin/capi
	@cp target/release/capi-server target/release/capi-linux-x64/bin/capi-server-x86_64-unknown-linux-gnu
	@cp -r $(OPENVINO_DIR) target/release/capi-linux-x64/lib/
	@echo '#!/bin/sh' > target/release/capi-linux-x64/bin/capi-wrapper
	@echo 'SCRIPT_DIR=$$(dirname "$$(readlink -f "$$0")")' >> target/release/capi-linux-x64/bin/capi-wrapper
	@echo 'INSTALL_DIR=$$(dirname "$$SCRIPT_DIR")' >> target/release/capi-linux-x64/bin/capi-wrapper
	@echo 'export OPENVINO_ROOT="$$INSTALL_DIR/lib/openvino"' >> target/release/capi-linux-x64/bin/capi-wrapper
	@echo 'export LD_LIBRARY_PATH="$$OPENVINO_ROOT/runtime/lib/intel64:$$OPENVINO_ROOT/runtime/3rdparty/tbb/lib:$$LD_LIBRARY_PATH"' >> target/release/capi-linux-x64/bin/capi-wrapper
	@echo 'exec "$$SCRIPT_DIR/capi" "$$@"' >> target/release/capi-linux-x64/bin/capi-wrapper
	@chmod +x target/release/capi-linux-x64/bin/capi-wrapper
	@cd target/release && tar -czf capi-linux-x64.tar.gz capi-linux-x64
	@echo "Portable tarball created at target/release/capi-linux-x64.tar.gz"
