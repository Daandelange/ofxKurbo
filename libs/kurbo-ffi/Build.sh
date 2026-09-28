#!/bin/bash

# Build lib
echo "Compiling library..."
RUSTFLAGS='-C target-feature=+crt-static' cargo build --release

# Copy files to OF plugin structure
echo "Copying files..."
mkdir -p ./lib/osx
mkdir -p ./lib/linux64
mv ./target/release/libkurbo_ffi.dylib ./lib/osx/libkurbo_ffi.dylib 2>/dev/null || true
mv ./target/release/libkurbo_ffi.a ./lib/osx/libkurbo_ffi.a 2>/dev/null || true
mv ./target/release/libkurbo_ffi.so ./lib/linux64/libkurbo_ffi.so 2>/dev/null || true

# Generate header file
echo "Generating header file..."
cbindgen --config ./cbindgen.toml --crate kurbo-ffi --output include/kurbo-ffi.h