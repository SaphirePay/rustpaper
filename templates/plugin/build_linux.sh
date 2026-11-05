#!/usr/bin/env bash
# The Linux .so of this plugin, built in Docker (from Windows or anywhere). Run from this folder:
#   docker run --rm -v "<repo>/sdk:/sdk:ro" -v "$PWD:/src" rust:1.99.1-bookworm bash /src/build_linux.sh
# The .so lands in out/.
set -euo pipefail
rm -rf /work && cp -r /src /work && cd /work
sed -i 's#path = "[^"]*horizon-bukkit"#path = "/sdk/horizon-bukkit"#' Cargo.toml
cargo build --release
mkdir -p /src/out && cp target/release/lib*.so /src/out/
ls -la /src/out
