#!/bin/bash
set -e

echo "=== 1. Updating System Packages ==="
sudo apt-get update

echo "=== 2. Installing Linux Graphics Libraries for Slint ==="
# These are required by Slint's backend to compile UI elements on Linux
sudo apt-get install -y \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxkbcommon-dev \
    libfontconfig1-dev \
    libasound2-dev

echo "=== 3. Installing Command Line Utilities ==="
# Installs 'tree' directory visualizer
sudo apt-get install -y tree

echo "=== 4. Updating Rust & Cargo Toolchain ==="
# Ensures rustc and cargo are up-to-date inside the container
rustup update stable

echo "=== Setup Complete! ==="
