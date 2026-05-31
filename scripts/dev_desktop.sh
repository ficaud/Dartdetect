#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Starting DartDetect desktop app (Tauri) ..."
(
	cd "${ROOT_DIR}/dart_desktop"
	cargo tauri dev
)
