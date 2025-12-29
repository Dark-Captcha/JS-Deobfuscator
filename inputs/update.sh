#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(dirname "$0")"

declare -A FILES=(
  ["captcha.js"]="https://captcha.hsprotect.net/PXzC5j78di/captcha.js"
  ["main.min.js"]="https://client.hsprotect.net/PXzC5j78di/main.min.js"
)

fetch() {
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL -o "$2" "$1"
  elif command -v wget >/dev/null 2>&1; then
    wget -q "$1" -O "$2"
  else
    echo "Error: curl or wget required" >&2
    exit 1
  fi
}

for filename in "${!FILES[@]}"; do
  rm -f "${SCRIPT_DIR}/${filename}"
  fetch "${FILES[$filename]}" "${SCRIPT_DIR}/${filename}"
done
