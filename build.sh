#!/bin/bash

set -e

TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-gnu"
)

echo "Building Elgato Ring Light Controller for multiple platforms..."

for target in "${TARGETS[@]}"; do
    echo "Building for $target..."
    
    if rustup target list | grep -q "$target (installed)"; then
        cargo build --release --target "$target"
        echo "✓ Built for $target"
    else
        echo "⚠ Target $target not installed. Run: rustup target add $target"
    fi
done

echo "Creating release packages..."
mkdir -p releases

for target in "${TARGETS[@]}"; do
    if [ -d "target/$target/release" ]; then
        case "$target" in
            *linux*)
                tar -czf "releases/elgato-controller-$target.tar.gz" \
                    -C "target/$target/release" elgato-cli elgato-dbus-service
                ;;
            *darwin*)
                tar -czf "releases/elgato-controller-$target.tar.gz" \
                    -C "target/$target/release" elgato-cli
                ;;
            *windows*)
                if [ -f "target/$target/release/elgato-cli.exe" ]; then
                    zip -j "releases/elgato-controller-$target.zip" \
                        "target/$target/release/elgato-cli.exe"
                fi
                ;;
        esac
    fi
done

echo "Build complete! Packages available in ./releases/"