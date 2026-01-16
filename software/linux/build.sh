#!/bin/bash
set -e

# Configuration
BUILDROOT_VER="2024.08"
BUILDROOT_DIR="buildroot-${BUILDROOT_VER}"
OUTPUT_DIR="output"
DTS_FILE="system.dts"
DTB_FILE="system.dtb"
URL="https://buildroot.org/downloads/buildroot-${BUILDROOT_VER}.tar.gz"

# GCC Fixes for newer host compilers
export HOST_CFLAGS="-O2 -std=gnu11 -Wno-implicit-function-declaration -Wno-int-conversion -Wno-incompatible-pointer-types -Wno-return-type -Wno-error"

# 1. Download Buildroot
if [ ! -d "$BUILDROOT_DIR" ]; then
    echo "[Linux] Downloading Buildroot..."
    wget "$URL"
    tar -xf buildroot-${BUILDROOT_VER}.tar.gz
    rm buildroot-${BUILDROOT_VER}.tar.gz
fi

# 2. Configure Buildroot
echo "[Linux] Configuring Buildroot..."
cd $BUILDROOT_DIR

# Create Buildroot configuration
cat > configs/riscv_emu_defconfig <<EOF
BR2_riscv=y
BR2_RISCV_64=y
BR2_RISCV_ISA_RVC=y
BR2_RISCV_ABI_LP64D=y
BR2_LINUX_KERNEL=y
BR2_LINUX_KERNEL_CUSTOM_VERSION=y
BR2_LINUX_KERNEL_CUSTOM_VERSION_VALUE="6.6.44"
BR2_LINUX_KERNEL_USE_ARCH_DEFAULT_CONFIG=y
BR2_TARGET_OPENSBI=y
BR2_TARGET_OPENSBI_PLAT="generic"
# Explicitly set ISA for OpenSBI to INCLUDE 'c'
BR2_TARGET_OPENSBI_ADDITIONAL_VARIABLES="PLATFORM_RISCV_ISA=rv64imafdc_zifencei"
BR2_TARGET_ROOTFS_EXT2=y
BR2_TARGET_ROOTFS_EXT2_SIZE="60M"
BR2_PACKAGE_HOST_LINUX_HEADERS_CUSTOM_6_6=y
EOF

# Apply config
make riscv_emu_defconfig

# 3. Build System
echo "[Linux] Building Linux..."
make -j$(nproc) HOST_CFLAGS="$HOST_CFLAGS"

# 4. Copy Artifacts
cd ..
mkdir -p $OUTPUT_DIR
cp $BUILDROOT_DIR/output/images/Image $OUTPUT_DIR/
cp $BUILDROOT_DIR/output/images/rootfs.ext2 $OUTPUT_DIR/disk.img
cp $BUILDROOT_DIR/output/images/fw_jump.bin $OUTPUT_DIR/

# 5. Compile Device Tree
echo "[Linux] Compiling Device Tree..."
dtc -I dts -O dtb -o $DTB_FILE $DTS_FILE

echo "[Linux] Build Complete."
