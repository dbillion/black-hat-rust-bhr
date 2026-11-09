#!/bin/sh
# Build and run the isolated Chapter 13 worm lab.
#
# Containment is total: --network none gives the container loopback and nothing
# else, so the worm's only reachable target is the SSH server inside the same
# container. cap-drop=ALL and no-new-privileges mean a successful infection
# cannot escalate. No host mounts.
set -eu
NAME=bhr_ch13_worm
echo "=== build ==="
docker build -q -t bhr-ch13-worm targets/ch_13-worm
echo "=== run (network none, no caps, no new privs, no mounts) ==="
docker run --rm \
    --name "$NAME" \
    --network none \
    --cap-drop=ALL \
    --security-opt no-new-privileges \
    --pids-limit 256 \
    bhr-ch13-worm
echo "=== container gone ==="
docker ps -a --filter "name=$NAME" --format '{{.Names}}' || echo "(none)"
