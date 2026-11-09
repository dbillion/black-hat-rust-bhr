#!/bin/sh
# Run the Chapter 11 RAT (E2E-encrypted) end to end against a local Postgres.
#
# All three processes stay on this one codespace. Nothing is published to a
# network: the server binds 127.0.0.1 (which is the book's own choice in
# server/src/main.rs), Postgres is bound to 127.0.0.1 by the docker -p flag,
# and the agent and client both talk to 127.0.0.1:8080.
set -eu

PORT=8080
DIR=/workspaces/black-hat-rust-bhr/ch_11
export DATABASE_URL='postgres://ch11:black_hat_rust@localhost:5432/ch11?sslmode=disable'
export PORT="$PORT"

echo "=== 1. server ==="
# -p binds to the given address; the agent and client reach it at 127.0.0.1:8080.
ss -ltn | grep ":$PORT " && echo "(pre-existing listener on $PORT -- stop it first)" || true
"$DIR/target/release/server" > /tmp/ch11_server.log 2>&1 &
for _ in $(seq 1 20); do
    curl -sf "http://127.0.0.1:$PORT/api/agents" > /dev/null 2>&1 && break
    sleep 1
done
echo "server log: $(cat /tmp/ch11_server.log)"
echo "listening on: $(ss -ltn | grep ":$PORT " | tr -s ' ' | cut -d' ' -f4)"
echo "agents (empty): $(curl -s "http://127.0.0.1:$PORT/api/agents")"

echo "=== 2. agent registers and polls ==="
"$DIR/target/release/agent" > /tmp/ch11_agent.log 2>&1 &
sleep 3
AGENT_ID=$(curl -s "http://127.0.0.1:$PORT/api/agents" | grep -oP '"id":"\K[^"]+' | head -1)
echo "AGENT_ID=$AGENT_ID"

echo "=== 3. full agent record: identity public key + public prekey ==="
curl -s "http://127.0.0.1:$PORT/api/agents" | tr ',' '\n' | grep -E 'id|last_seen|public_prekey' | head -5

echo "=== 4. client: agents ==="
"$DIR/target/release/client" agents 2>&1 | head -8

echo "=== 5. client: exec (encrypted round trip) ==="
"$DIR/target/release/client" exec -a "$AGENT_ID" 'hostname' 2>&1 | head -6
sleep 4

echo "=== 6. client: jobs (decrypted output stored server-side) ==="
"$DIR/target/release/client" jobs 2>&1 | head -8

echo "=== 7. agent's persisted config (agent-side keys on disk) ==="
ls -la "$HOME/.config/ch_11" 2>/dev/null || find "$HOME" -name 'ch_11*' -maxdepth 4 2>/dev/null | head -3

echo "=== stopping ==="
for p in $(pgrep -f 'release/(server|agent)'); do kill "$p" 2>/dev/null || true; done
echo done
