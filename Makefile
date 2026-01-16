.PHONY: all clean run build-software build-hardware linux run-linux

all: build-software build-hardware

build-software:
	@echo "[Root] Building Bare Metal Software..."
	$(MAKE) -C software

build-hardware:
	@echo "[Root] Building CPU Simulator..."
	cd hardware && cargo build --release

test:
	@echo "[Root] Testing CPU Simulator..."
	cd hardware && cargo test --release

linux:
	@echo "[Root] Building Linux (This takes time)..."
	$(MAKE) -C software linux

# Run Bare Metal
run: all
	@echo "[Root] Booting Bare Metal RISC-V System..."
	./hardware/target/release/riscv-emulator \
		--config hardware/configs/default.toml \
		--kernel software/bin/kernel/kernel.bin
		--disk software/disk.img

# Run Linux
run-linux: build-hardware
	@echo "[Root] Booting Linux..."
	@if [ ! -f software/linux/output/Image ]; then \
		echo "Error: Linux Image not found. Run 'make linux' first."; \
		exit 1; \
	fi
	./hardware/target/release/riscv-emulator \
		--config hardware/configs/linux.toml \
		--kernel software/linux/output/Image \
		--disk software/linux/output/disk.img \
		--dtb software/linux/system.dtb

clean:
	$(MAKE) -C software clean
	rm -rf hardware/target
