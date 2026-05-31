#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

SERVER_PID=""
UI_PID=""

stop_process_tree() {
	local pid="$1"

	if [[ -z "${pid}" ]] || ! kill -0 "${pid}" 2>/dev/null; then
		return
	fi

	# Stop children first (npm -> node/vite), then parent.
	pkill -TERM -P "${pid}" 2>/dev/null || true
	kill -TERM "${pid}" 2>/dev/null || true

	# Give processes a short grace period before forcing.
	for _ in {1..20}; do
		if ! kill -0 "${pid}" 2>/dev/null; then
			break
		fi
		sleep 0.1
	done

	if kill -0 "${pid}" 2>/dev/null; then
		pkill -KILL -P "${pid}" 2>/dev/null || true
		kill -KILL "${pid}" 2>/dev/null || true
	fi

	wait "${pid}" 2>/dev/null || true
}

cleanup() {
	stop_process_tree "${UI_PID}"
	stop_process_tree "${SERVER_PID}"
}

trap cleanup EXIT INT TERM

echo "Starting dart_server on http://localhost:8080 ..."
(
	cd "${ROOT_DIR}"
	cargo run --manifest-path dart_server/Cargo.toml
) &
SERVER_PID=$!

echo "Starting UI on http://localhost:1420 ..."
(
	cd "${ROOT_DIR}/dart_ui"
	npm run dev
) &
UI_PID=$!

while true; do
	if ! kill -0 "${SERVER_PID}" 2>/dev/null; then
		break
	fi

	if ! kill -0 "${UI_PID}" 2>/dev/null; then
		break
	fi

	sleep 0.2
done
