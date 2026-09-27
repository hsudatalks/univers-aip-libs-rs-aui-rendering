#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
lib_cargo="${CARGO:-cargo}"
"${lib_cargo}" build --locked --release
