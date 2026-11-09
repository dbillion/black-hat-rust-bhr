#!/bin/sh
# Run the isolated Chapter 13 worm lab.
#
# CONTAINMENT IS TOTAL -- read the Dockerfile's header for the full design.
# --network none gives the container loopback and nothing else, so the worm's
# only reachable target is the SSH server inside the same container. No caps,
# no new privileges, no mounts. The root password is `password`, one of the
# book's own wordlist entries.
#
# NOTE ON INVOCATION FORM: this runner uses `--entrypoint bash -c '<script>'`
# rather than the image's own ENTRYPOINT. With the image ENTRYPOINT the same
# steps fail with "Unable to exchange encryption keys" every time; via `bash -c`
# the identical steps succeed every time. The difference was reproduced, not
# root-caused to the protocol level, and is reported as such in
# ch_13/FINDINGS.md. The entrypoint script stays in the image for reference.
set -eu

NAME=bhr_ch13_worm

echo "=== build ==="
docker build -q -t bhr-ch13-worm targets/ch_13-worm

echo "=== run (network none, no caps, no new privs, no mounts) ==="
docker run -d \
    --name "$NAME" \
    --network none \
    --cap-drop=ALL \
    --security-opt no-new-privileges \
    --pids-limit 256 \
    --entrypoint bash \
    bhr-ch13-worm \
    -c '
    /usr/sbin/sshd
    sleep 8
    cd /opt/lab
    echo "=== sshd up on 127.0.0.1:22 ==="
    echo "=== uname -a on the target ==="
    uname -a
    echo "=== running the worm against 127.0.0.1:22 ==="
    ./agent 127.0.0.1:22
    echo "=== WORM_EXIT=$? ==="
    echo "=== what landed in /tmp (the uploaded agent) ==="
    ls -la /tmp
    echo "=== data dir (install() + bundle extraction) ==="
    ls -la /root/.local/share/bhr_ch13 2>/dev/null
    echo "=== agent processes ==="
    ps -eo pid,args | grep "[a]gent" || echo "(none besides this shell)"
    '

echo "=== waiting for the spread to finish ==="
sleep 35
docker logs "$NAME" 2>&1 || true

echo "=== /tmp inside the container (independent confirmation) ==="
docker exec "$NAME" ls -la /tmp 2>/dev/null || true

echo "=== cleaning up ==="
docker rm -f "$NAME" >/dev/null 2>&1
docker ps -a --filter "name=$NAME" --format '{{.Names}}' || echo "(removed)"
