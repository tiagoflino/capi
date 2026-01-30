OPENVINO_VER := 2025.4.1.0
OPENVINO_URL := https://storage.openvinotoolkit.org/repositories/openvino_genai/packages/2025.4.1/linux/openvino_genai_ubuntu22_$(OPENVINO_VER)_x86_64.tar.gz
LIBS_DIR := $(shell pwd)/libs
OPENVINO_DIR := $(LIBS_DIR)/openvino

.PHONY: all build run download-deps clean-deps

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
