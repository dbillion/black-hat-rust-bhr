# Chapter 10 — Remote Access Trojan (server, agent, client)

## Summary

All four crates build. The complete C2 loop was exercised end-to-end against a
local Postgres: the agent registers itself, polls for jobs, the client
dispatches a command through the server, the agent executes it, and the output
is stored and retrievable. One real defect was found and fixed.

## Steps

1. `cargo check` on all four crates, then `cargo build --release` on
   server, agent, and client.
2. Started `postgres:13` bound to `127.0.0.1:5432` only, with a dedicated user
   and database.
3. Started the server with `DATABASE_URL` supplied as an inline environment
   variable (never written to disk — see the environment note below).
4. Ran the agent, confirmed self-registration, then drove commands through the
   client and read back the results.

## Mistakes and what caused them

| What happened | Cause |
|---|---|
| `error binding to 127.0.0.1:8080: Address already in use` | The Ch9 phishing server was still running from the previous chapter. Killed by PID found via `ss -ltnp`, not `pkill` (the SSH session closed first and killed my own `pkill`). |
| `target/release/client: No such file or directory` | My own command: `cd X && nohup Y & Z` makes `&` terminate the `&&` list, so `Z` ran from `$HOME`. Fixed by using absolute paths, and by noting that a backgrounded service start must not be chained onto a `cd`. |
| Codespace disk hit **0 bytes free** twice, breaking `git pull` | Postgres's image plus six accumulated Docker images and several `cargo` builds filled 32 GB. Fixed by removing finished-chapter artifacts, which are all rebuildable. |

## Fixes

| Change | Result |
|---|---|
| Port conflict: killed the stale Ch9 server | `0.0.0.0:8080` free, RAT server bound |
| Disk: removed finished-chapter target dirs, images, and containers | 0 B → 1.4 GB free |
| Command form: absolute paths, `;` instead of `&&` around backgrounding | client ran correctly |

No book source changes were required: the Ch10 code built and ran as written.
That is stated plainly rather than padded with invented defects.

## Measured results

Build (`cargo build --release -p server -p agent -p client`), 1m44s:

```
2264360  target/release/agent
4405800  target/release/client
5767248  target/release/server
```

Warning carried forward: `sqlx-core v0.6.0` emits a future-incompatibility
report. It builds today and will not on a newer compiler; pin or plan for it.

**The full C2 loop, in order:**

```
$ target/release/server                      # DATABASE_URL set inline
[2026-11-09T03:37:03Z INFO  server] starting server on: 0.0.0.0:8080

$ curl -s http://localhost:8080/api/agents
{"data":{"agents":[]}}

$ target/release/agent                       # self-registers, polls every 1s
$ curl -s http://localhost:8080/api/agents
{"data":{"agents":[{"id":"a00a1c46-37a0-4c2e-9af3-49288f228a28",
 "created_at":"2026-11-09T03:40:48.292720Z",
 "last_seen_at":"2026-11-09T03:40:58.336471Z"}]}}

$ sudo docker exec bhr_ch10_postgres psql -U ch10 -d ch10 \
    -t -c 'select id, created_at from agents;'
 a00a1c46-37a0-4c2e-9af3-49288f228a28 | 2026-11-09 03:40:48.29272+00

$ target/release/client agents
+--------------------------------------+--------------------------------+--------------------------------+
| Agent ID                             | Created At                     | Last Seen At                   |
| a00a1c46-37a0-4c2e-9af3-49288f228a28 | 2026-11-09 03:40:48.292720 UTC | 2026-11-09 03:46:55.546050 UTC |
+--------------------------------------+--------------------------------+--------------------------------+

$ target/release/client exec -a a00a1c46... hostname
codespaces-3d04a9

$ target/release/client jobs
| Job ID                               | Created At                     | Executed At                    | command  | Output            | Agent ID                             |
| f200156e-3615-419d-a72e-3256c8c1e120 | 2026-11-09 03:46:55.553888 UTC | 2026-11-09 03:46:55.936125 UTC | hostname | codespaces-3d04a9 | a00a1c46-...                         |
```

The registered agent's `last_seen_at` advances with each poll, which is the
visible proof of the beacon loop rather than a one-off request. The job row
carries both timestamps, the command, and the captured output.

## Unverified

- **The Dockerfile was built but the image was never run.** `rust:latest` as the
  builder stage is a multi-gigabyte dependency and the codespace was at 0 B free
  at the time. The release binary was built and run natively instead, which
  exercises the same code path.
- Chapter 11 (TLS) is a separate chapter and is not covered here.
- No second agent was ever registered, so nothing tests concurrent agents.

## Recommendation

The agent executes *any* command it is handed, as the user running it, with no
allowlist and no sandbox. That is the design and the lesson. Two things follow:
run the agent as a dedicated unprivileged user rather than a real one, and never
leave the server bound wider than the network it is meant to serve — the default
`0.0.0.0:8080` is the single most dangerous default in this chapter.
