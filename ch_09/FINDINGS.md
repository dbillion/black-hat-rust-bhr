# Chapter 9 — Phishing: bitsquatting, a credential harvester, and mail sending

## Summary

Three of the four exercises are Rust programs that build and run here. The
fourth, `evil_twin`, is a Raspberry Pi access-point attack: it needs a WiFi
radio that supports AP mode, and a codespace has none. That is an
environment limit, not a code defect, and it is reported as a gap.

## Steps

1. Built and ran `dnsquat`, comparing its output byte-for-byte with the book's
   README example.
2. Built the `phishing` workspace (`webapp` + `server` + `common`).
3. Started the credential-harvesting server and drove it with real HTTP:
   a login POST, a static-page GET, and a read of the resulting database.
4. Type-checked `emails` (lettre + rusoto_ses + tera).
5. Read `evil_twin` and established what hardware it needs.

## Mistakes and what caused them

| What happened | Cause |
|---|---|
| `ls '$d'` failed inside the SSH command | `\$d` was not expanded by the remote shell — the escaping was mine, not the repo's. Fixed by naming each directory. |
| `emails` initially showed no output from the filtered check | The grep pattern was exhausting the output before `Finished` arrived. Re-ran without the filter. |
| The codespace hit `No space left on device` during a `git pull` | Six Docker images plus build cache from Ch5–Ch8 filled the 32 GB disk. Removed the images of finished chapters; 2.4 GB recovered. |

## Fixes

| Change | Result |
|---|---|
| Removed `bhr-ch8-shell`, `bhr-ch8-shellcode`, `bhr-cve-2021-3156`, `seleniarm/standalone-chromium` images | disk 100% → 93%, `git pull` works again |

No source changes were needed in this chapter: the book's code built and ran as
written. That is worth stating plainly rather than manufacturing a defect to
report.

## Measured results

**`dnsquat`** — `cargo run -- domain .com`, output identical to the book's
README, 26 candidates, in this order and with these exact bytes:

```
eomain.com  fomain.com  lomain.com  tomain.com  dnmain.com  dmmain.com
dkmain.com  dgmain.com  dolain.com  dooain.com  doiain.com  doeain.com
do-ain.com  domcin.com  domein.com  domiin.com  domqin.com  domahn.com
domakn.com  domamn.com  domaan.com  domayn.com  domaio.com  domail.com
domaij.com  domaif.com
```

One bit flip per character position, eight bits per position, filtered by
`is_valid` (alphanumeric or `-`) and by a case-insensitive inequality against the
original. The algorithm is bitsquatting: each candidate differs from the real
domain by exactly one bit, so it survives a typo or a corrupted memory cell while
still resolving to an attacker's host.

The no-argument case prints `Usage: dnsquat domain .com` and returns — correct
behaviour, not a crash.

**`phishing` server** — built in 2m05s, `cargo check -p server` clean:

| Request | Result |
|---|---|
| `POST /api/login` `{"email":"victim@example.com","password":"hunter2hunter2"}` | `{"ok":true}`, HTTP 200 |
| `GET /` | serves `public/index.html`, HTTP 200 |
| Credential row written | `('victim@example.com', 'hunter2hunter2', '2026-11-09T03:14:45.797307219+00:00')` |

The server listens on `0.0.0.0:8080`, takes `--port` and `--directory`, creates
`db.sqlite` if missing, runs `server/db/0001_init.sql` through `sqlx::migrate!`,
and inserts into WAL mode with `synchronous = Normal`. The credential is stored
in plaintext with a UTC timestamp.

**`emails`** — `cargo check` clean, no warnings.

## Unverified

- `webapp` (the Yew WASM front end) — **not built**. `wasm-pack` is not
  installed, and building it would add the `wasm32-unknown-unknown` target plus
  a Node toolchain. The server half is fully exercised; the front end is not.
- `evil_twin` — **not run**. Needs a Raspberry Pi with a radio in AP mode,
  `hostapd`, `dnsmasq`, and `iptables`. None of that exists here, and it should
  not: the script sets up a fake access point, which is the kind of thing that
  must never be exercised on a shared network.
- `emails` was type-checked but never executed, because executing it sends real
  mail to `credule@kerkour.com` through hardcoded credentials.

## Recommendation

`dnsquat` is a good self-contained teaching example and safe to run anywhere.
The `phishing` server is worth running against localhost only: it binds
`0.0.0.0` by default, so `--port` and the network it is attached to are the two
things to get right before starting it.
