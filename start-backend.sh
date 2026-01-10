#!/bin/bash

# Quick start script for VoIP Application on macOS/Linux

echo ""
echo "============================================"
echo "   VoIP Application - Quick Start"
echo "============================================"
echo ""

# Check for Rust
if ! command -v rustc &> /dev/null; then
    echo "ERROR: Rust is not installed!"
    echo "Please download from: https://rustup.rs/"
    exit 1
fi

echo "Rust found!"
echo ""
echo "Starting backend server..."
echo ""

cd "$(dirname "$0")/backend"

# Build if needed
if [ ! -f target/release/voip-backend ]; then
    echo "Building Rust backend (this may take a few minutes)..."
    cargo build --release
    if [ $? -ne 0 ]; then
        echo "ERROR: Build failed!"
        exit 1
    fi
fi

echo ""
echo "============================================"
echo "   Backend Server Starting on http://localhost:8080"
echo "============================================"
echo ""

cargo run --release
