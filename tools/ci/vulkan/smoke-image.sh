#!/usr/bin/env bash
# Proves the software Vulkan adapter and a functional X11 server.
set -euo pipefail
test -r /usr/share/vulkan/icd.d/lvp_icd.json
test "$VK_DRIVER_FILES" = /usr/share/vulkan/icd.d/lvp_icd.json
test "$VK_ICD_FILENAMES" = "$VK_DRIVER_FILES"
test "$WGPU_BACKEND" = vulkan
command -v rustc
command -v cargo
command -v xdotool
rustc +stable --version
cargo +stable --version
vulkaninfo --summary | tee /tmp/runenwerk-vulkan-summary.txt
grep -Eq 'deviceName[[:space:]]*=[[:space:]]*llvmpipe' /tmp/runenwerk-vulkan-summary.txt
xvfb-run --auto-servernum --server-args="-screen 0 1280x720x24" \
  sh -c 'test -n "$DISPLAY"; xdpyinfo -display "$DISPLAY" >/dev/null'
echo "PASS: Mesa Lavapipe adapter and Xvfb display are functional"
