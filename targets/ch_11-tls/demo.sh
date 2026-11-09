#!/bin/sh
# Run the Chapter 11 RAT (end-to-end encrypted) against a local Postgres.
#
# Containment: everything stays on this one codespace. The server binds
# 127.0.0.1 (the book's own choice in server/src/main.rs line 35), Postgres is
# bound to 127.0.0.1 by the docker -p flag, and agent and client both talk to
# 127.0.0.1:8080. Nothing is published to any network.
set -u

PORT=8080
DIR=/workspaces/black-hat-rust-bhr/ch_11
# The key that actually matches the agent's hardcoded expectation. env.example
# ships a different, stale value -- see FINDINGS.md.
export DATABASE_URL='postgres://ch11:black_hat_rust@localhost:5432/ch11?sslmode=disable'
export CLIENT_IDENTITY_PUBLIC_KEY='xQ6gstFLtTbDC06LDb5dAQap+fXVG45BnRZj0L5th+M='
export PORT="$PORT"

echo "=== 1. server ==="
"$DIR/target/release/server" > /tmp/ch11_server.log 2>&1 &
for _ in $(seq 1 25); do
    curl -sf "http://127.0.0.1:$PORT/api/agents" > /dev/null 2>&1 && break
    sleep 1
done
echo "log: $(cat /tmp/ch11_server.log)"
echo "listening: $(ss -ltn | grep ":$PORT " | tr -s ' ' | cut -d' ' -f4)"
echo "agents (empty): $(curl -s "http://127.0.0.1:$PORT/api/agents")"

echo "=== 2. agent registers, generates its own identity + prekey ==="
"$DIR/target/release/agent" > /tmp/ch11_agent.log 2>&1 &
sleep 4
AGENT_ID=$(curl -s "http://127.0.0.1:$PORT/api/agents" | grep -oP '"id":"\K[^"]+' | head -1)
echo "AGENT_ID=$AGENT_ID"

echo "=== 3. agent record: identity public key + public prekey ==="
curl -s "http://127.0.0.1:$PORT/api/agents"

echo
echo "=== 4. client agents ==="
"$DIR/target/release/client" agents 2>&1 | head -8

echo "=== 5. client exec -- encrypted round trip ==="
"$DIR/target/release/client" exec -a "$AGENT_ID" 'hostname' 2>&1 | head -8
sleep 5

echo "=== 6. client exec again (proves the loop repeats) ==="
"$DIR/target/release/client" exec -a "$AGENT_ID" 'id -un' 2>&1 | head -8
sleep 5

echo "=== 7. agent-side persisted config (its own keys) ==="
find "$HOME/.config" "$HOME/.local/share" -name 'ch_11*' 2>/dev/null | head -3

echo "=== 8. stopped ==="
for p in $(pgrep -f 'release/(server|agent)'); do kill "$p" 2>/dev/null || true; done
echo done
