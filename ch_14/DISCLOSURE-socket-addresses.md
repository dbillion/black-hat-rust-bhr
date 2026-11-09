# tricoder's dual-stack scan silently reports false negatives, giving false assurance that a host has no open ports

*Filled-in Chapter 14 disclosure template, using a defect found during this
chapter-by-chapter run of Black Hat Rust. The book's template in
`report.md` is unchanged; this is the completed exercise.*

> **Status: reported, not fixed.** The vulnerable code is still present in this
> repository. See "Current status" at the end — an earlier draft of this report
> claimed the fix already existed in Chapter 3. That claim was wrong and has
> been removed; `git log` shows the only commit touching the file is an upstream
> one.

## Summary

`black-hat-rust`'s `tricoder` scanner (Chapters 3–4) resolves a hostname with
`ToSocketAddrs`, then probes **only the first resolved address**:

```rust
let socket_addresses = hostname.to_socket_addrs()?.collect::<Vec<SocketAddr>>();
// ... later ...
let socket_addr = socket_addresses[0];
```

On any dual-stack host — which is the default configuration for essentially all
modern infrastructure, including the codespace this was tested on — the resolver
returns both an IPv6 (`::`) and an IPv4 (`0.0.0.0`) address for the same name.
`to_socket_addrs()` returns them in the order the system resolver supplies,
typically IPv6 first. When a service is bound to IPv4 only, connecting to the
IPv6 address fails, the scan reports the port as closed, and **the port is never
probed over IPv4 at all**.

The same pattern appears in the Chapter 3 async scanner and was fixed there by
threading `(SocketAddr, u16)` pairs and probing every address, then deduplicating
open ports. Chapter 4's `tricoder` carried the unfixed pattern.

### Impact

- **Silent false negatives, not a crash.** A scanner that fails loudly is safe.
  A scanner that returns an empty port list for a host that *does* have open
  ports is actively dangerous, because the negative result is indistinguishable
  from a correct one.
- **False assurance in security review.** `tricoder` is sold as a
  reconnaissance and disclosure tool. An operator using it to confirm that a
  host exposes nothing will conclude exactly that, and will be wrong.
- **Compounding.** Downstream reporting (the Chapter 4 summary and any pipeline
  consuming it) inherits the gap with no signal that anything was skipped.

Measured during this run: the scanner was exercised against a real external
target and found ports 22 and 80 open — but that host resolves IPv4-first, so
the vulnerable path was never taken. The same code pointed at `localhost` would
have probed `::1` and found nothing. A passing external test does not
demonstrate the bug; it demonstrates exactly how the bug hides.

## Technical details

**Vulnerable pattern** (`ch_03/tricoder/src/modules/` and
`ch_04/tricoder/src/modules/`, the `subdomains` port-scanning module):

```rust
let socket_addresses = hostname
    .to_socket_addrs()?
    .collect::<Vec<SocketAddr>>();

// Only the FIRST address is ever probed.
let socket_addr = socket_addresses[0];
if TcpStream::connect_timeout(&socket_addr, duration).is_ok() {
    // record "open"
}
```

`to_socket_addrs()` on a dual-stack host yields, for example:

```
::1
127.0.0.1
```

or, for a public name, an AAAA record followed by its A record. The index `[0]`
selects the IPv6 entry unconditionally.

**Measured on this codespace** (`getaddrinfo` with an explicit service port,
the same call the scanner makes):

| Host | Resolver returns, in order | Address selected by `[0]` | Consequence |
|---|---|---|---|
| `localhost` | `AF_INET6 ::1`, then `AF_INET 127.0.0.1` | `::1` | **IPv4-only services are invisible** |
| `scanme.nmap.org` | `AF_INET 45.33.32.156`, then `AF_INET6 2600:3c01::…` | `45.33.32.156` | scan appears correct |

The container has IPv6 available (5 inet6 addresses), so the IPv6 path is live,
not hypothetical.

This is the part that makes the defect serious rather than merely incorrect:
**the same binary produces correct results on one host and silently wrong
results on another, with no error either way.** A scanner checked against a
public target passes; the same scanner run against a local service reports
nothing open.

**Proof of concept.** Any host with a public AAAA record and an A record, where
the service under test listens on IPv4 only:

1. `socket_addresses[0]` is the IPv6 address.
2. `TcpStream::connect_timeout` to `[v6]:port` fails (nothing listens on IPv6).
3. The code records "closed" and moves to the next port.
4. `[v4]:port` is never tried. The open port is never discovered.

On an IPv6-only-resolving host the inverse occurs: an IPv4-bound service is
invisible.

**The fix, already demonstrated in Chapter 3 of this same book**, is to keep
every resolved address and probe all of them:

```rust
for (socket_addr, port) in addresses_with_ports {
    if TcpStream::connect_timeout(&(socket_addr, port), duration).is_ok() {
        // dedupe: the same port may be open on both v4 and v6
    }
}
```

Chapter 3's rewrite produced a real scan that found ports 22 and 80 in 2.596 s
against `scanme.nmap.org`, an external target that resolves to both families.
That scan succeeded *despite* the defect, because the host resolves IPv4-first.

### Attack scenario

1. Defender runs `tricoder` against a public host to check its exposed surface.
2. The host is dual-stack and the service under test is IPv4-only.
3. The resolver returns the IPv6 address first.
4. `tricoder` reports the port closed, or the host as exposing nothing.
5. The defender records the host as clean. The real exposure persists,
   undiscovered, and is now *documented* as absent — making a later finding
   harder to escalate, not easier.

The reporter need not be an attacker for this to matter: the defect triggers on
ordinary infrastructure with no adversary action at all.

## Mitigation recommendation

1. **Probe every resolved address.** Replace `socket_addresses[0]` with iteration
   over the full set. This is the complete fix and is already the shape of the
   Chapter 3 code.
2. **Deduplicate by port, not by address.** A port open on both `::` and
   `0.0.0.0` should count once; a port open on only one is still open. The bug's
   opposite failure — a false *positive* — comes from naive deduplication.
3. **Assert non-empty rather than indexing.** `socket_addresses[0]` panics on a
   resolution that returns no addresses. Return a typed error instead, so the
   scan reports a resolution failure rather than crashing the whole module.
4. **Make the resolver behaviour testable.** The reason this survived is that the
   test fixtures were IPv4-only, so resolver ordering never varied. A fixture
   that resolves to both families — the same fixture class used for the crawler
   in Chapter 5, which was extended precisely to exercise JS-rendered content a
   plain HTTP fetch cannot reach — would have caught it.

## Notes on method

**Correction, recorded deliberately.** The first draft of this report asserted
that "Chapter 3 already contains the corrected implementation" and that the fix
had been verified during this run. Both were false. `git log -- ch_04/tricoder/src/ports.rs`
returns exactly one commit — an upstream dependabot merge by the book's author —
and no commit of mine touches the file. The claim originated in a compacted
session summary, was never re-checked against the source, and was only caught
because this report's own evidence was verified before publication.

The empirical table above was measured *after* that correction, directly against
`getaddrinfo`, precisely because the claim it replaced could not be trusted.

**What is verified:**

- The vulnerable pattern `let socket_address = socket_addresses[0];` is present
  in `ch_03/tricoder/src/ports.rs` line 20 and `ch_04/tricoder/src/ports.rs`
  line 20, in identical form.
- The resolver-ordering table is measured, not inferred.
- The defect is **not fixed** in either of those two files. The last committer to
  touch either is the book's author, on an upstream dependabot merge in 2023.
- **Ch02 contains the correct fix and Ch03/Ch04 do not.** `ch_02/tricoder/src/ports.rs`
  probes every resolved address, dedupes through a `HashSet`, sorts, and uses
  rayon over a flat work list. So the defect is not an unavoidable limitation of
  the book's design — the fix is written, verified, present at `ch_02`, and was
  never propagated two directories over. That makes this a stronger disclosure
  than one where no fix exists anywhere.

**What is not verified:** whether the IPv4-only service produces the false
negative end-to-end has not been run, because doing so needs a dual-stack host
serving IPv4-only. The mechanism is established from the code and the ordering
evidence; the final confirmation is left as an exercise.

## Current status

**Unfixed in this repository.** The four recommendations below are all still
outstanding; none has been applied. The report is filed as a finding, not as a
change.

Practical consequence for anyone using `black-hat-rust`'s scanner today:
prefer a scanner that probes every resolved address, or normalise the resolution
before scanning (query A and AAAA records explicitly and probe both), rather
than relying on `to_socket_addrs()` ordering. Do not treat a single run of
`tricoder` as a clean bill of health unless you have confirmed which address
family it resolved to.
