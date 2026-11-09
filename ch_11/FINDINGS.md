# Chapter 11 — Encrypted RAT (the "TLS" chapter, which is not TLS)

## Summary

All four crates build and the full C2 loop runs with real cryptography: the
agent generates its own ed25519 identity keypair and an x25519 prekey on first
run, the client encrypts the command against that prekey, the agent decrypts and
executes it, and the result is encrypted back and decrypted client-side. Two
plaintexts (`hostname`, `id -un`) were recovered correctly.

The chapter is titled for TLS but ships no TLS at all. That is the single most
important finding here and it is not a criticism of the code — it is a
mislabel in the book, and the difference between "encrypted" and "transported
over TLS" is exactly the distinction a reader should leave with.

## Steps

1. `cargo check` then `cargo build --release` on all four crates.
2. Started `postgres:13` bound to `127.0.0.1:5432` with a dedicated user/db.
3. Started the server with `DATABASE_URL` and `CLIENT_IDENTITY_PUBLIC_KEY` as
   inline env vars (never written to disk).
4. Started the agent, which self-registered with a freshly generated keypair.
5. Drove two commands through the client and read the decrypted output.

## Mistakes and what caused them

| What happened | Cause |
|---|---|
| `password authentication failed for user "ch11"` | Ch10's Postgres container was still bound to 127.0.0.1:5432. Ch11's server reached *that* database. Removed all containers, started a dedicated one. |
| `Connection reset by peer (os error 104)` | Postgres was still running initdb moments after `pg_isready` first answered. Re-running once settled succeeded. |
| `Error: Internal error` from the server, no other detail | See the defect below — a required env var defaults to an empty string, and the resulting base64/ed25519 error is opaque. |
| `Found argument 'jobs' which wasn't expected` | Ch11's client CLI differs from Ch10's: it has `identity`, not `jobs`, and `exec` loads the client config. Read the CLI instead of assuming the previous chapter's shape. |
| `docker: Conflict. The container name "/bhr_ch11_postgres" is already in use` | A zombie container in `Created` state that `rm -f` did not clear. Cleared every container with `docker ps -aq` and used a fresh name. |
| Codespace disk hit 0 B again | Third occurrence. Cleared `ch_01`–`ch_09` and `ch_11` build trees; 2.6 GB recovered. |

## Defects found in the book's own code

**1. `CLIENT_IDENTITY_PUBLIC_KEY` is required but defaults to empty.**
`server/src/config.rs` reads it with `.ok().unwrap_or(String::new())`, then
`base64::decode("")` succeeds and `ed25519_dalek::PublicKey::from_bytes(&[])`
fails with `invalid length: expected length 32 for simple format, found 0`.
The server dies with `Error: Internal error` and no mention of the missing
variable. Compare `DATABASE_URL` four lines above, which *does* return a named
`NotFound` error. The fix is to read this one the same way.

**2. `env.example` ships a stale client key that does not match the code.**
Three different client keys appear in this chapter:

| Location | Value | Matches the client's private key? |
|---|---|---|
| `client/src/config.rs` `IDENTITY_PRIVATE_KEY` | `wToLgDfjCxFijRA+YKi6T9j7bTc/4grwoTRJZJs5DU8=` | (the private key itself) |
| `agent/src/config.rs` `CLIENT_IDENTITY_PUBLIC_KEY` | `xQ6gstFLtTbDC06LDb5dAQap+fXVG45BnRZj0L5th+M=` | **yes** |
| `env.example` `CLIENT_IDENTITY_PUBLIC_KEY` | `dFa0knTlWODvELy59rzMEvITKxcFj984R28T/K3r6tI=` | **no** |

Verified by deriving the ed25519 public key from the private seed with
Python's `cryptography`: the derived key equals the agent's constant exactly.
So the README's own instruction, `cp env.example .env`, configures the server
with a key the agent will reject — and the failure would surface as a decryption
error on the agent side, not as a clear "wrong key" message.

**3. No TLS, despite the chapter framing.** `server/src/main.rs` line 35 binds
plain HTTP: `warp::serve(routes).bind_with_graceful_shutdown(([127,0,0,1], port), ...)`.
There is no `TlsServer`, no certificate, and no `--tls` option anywhere in the
chapter. `ureq` is built with the `tls` feature but `SERVER_URL` is `http://`.
The confidentiality comes entirely from application-layer ChaCha20-Poly1305.

## Fixes

None applied to the source. Both defects are reported rather than fixed, because
fixing #2 means choosing which of the three keys is authoritative — that is the
author's call, not mine, and the one that works today is the one hardcoded in
the agent. The runner in `targets/ch_11-tls/demo.sh` sets the matching value so
the chapter can be exercised as written.

## Measured results

Build (`cargo build --release -p server -p agent -p client`), 1m48s:

```
2478672  target/release/agent
4596600  target/release/client
5870944  target/release/server
```

```
$ server@127.0.0.1   # binds 127.0.0.1, unlike Ch10's 0.0.0.0
$ agent
$ curl -s /api/agents
{"data":{"agents":[{
  "id":"ec01b810-a010-4ba4-9da9-409ff405b4aa",
  "created_at":"2026-11-09T04:19:23.210917Z",
  "last_seen_at":"2026-11-09T04:19:26.228944Z",
  "identity_public_key":[204,33,196,200,221,24,98,155,...],   // 32 bytes
  "public_prekey":[231,135,140,245,132,45,101,22,...],        // 32 bytes
  "public_prekey_signature":[216,162,123,140,...]}]}}         // 64 bytes

$ client agents
| Agent ID                             | Identity Public Key                    | Public Prekey                          |
| ec01b810-a010-4ba4-9da9-409ff405b4aa | zCHEyN0YYpuE3jpOF1mSUpqeLoYsx2XC6ST/N/MYc48= | 54eM9YQtZRYNRawekSbQ+vTRrKrs0EoXLS7whBXpe2Q= |

$ client exec -a ec01b810-a010-4ba4-9da9-409ff405b4aa 'hostname'
codespaces-3d04a9

$ client exec -a ec01b810-a010-4ba4-9da9-409ff405b4aa 'id -un'
codespace
```

The agent's identity key, prekey, and prekey signature are **freshly generated
on first run** — the values in this table came from that run, not from the book.
Two different plaintexts were recovered correctly, which proves the round trip
rather than a single lucky request.

## Unverified

- The Dockerfile image was not built (`rust:latest` builder exceeds the disk).
- Only one agent was registered, so nothing tests concurrent agents.
- The agent's persisted config (`~/.config/ch_11`) was not inspected after the
  run; the find at the end of the runner printed nothing, so its location is
  unconfirmed.

## Recommendation

Read this chapter as **end-to-end encryption, not transport security**, and take
the distinction with you. ChaCha20-Poly1305 with an X25519 prekey exchange means
the *server* cannot read the commands it is routing — which is the genuinely
interesting property, and stronger than what TLS would give you here, since the
C2 host never sees the plaintext either. What it does *not* give you is any
protection against a network observer: the traffic is plain HTTP. Both facts are
worth knowing, and the chapter's title gets one of them wrong.
