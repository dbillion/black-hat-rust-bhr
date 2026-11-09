# Isolated target for Black Hat Rust Chapter 8 (shellcode).
#
# Chapter 8 is pure shellcode: write(2) "hello world", then exit(2), via raw
# syscalls with no libc. That makes it a better test target than any of the
# later chapters — no network, no propagation, no listener.
#
# The shellcode is linked into a standalone ELF with `-nostdlib` and a custom
# `shellcode.ld`, so the only thing to verify is that the bytes execute.
#
# Build:   docker build -t bhr-ch8 targets/ch_08-shellcode/
# Run:     docker run --rm --network none bhr-ch8-shellcode
#
# Safety: the run is `--network none` with no privileged flag and no host mount.
# The shellcode makes two syscalls (write, exit) and nothing else. There is no
# shell, no listener, no fork, and no file write. It cannot propagate, so the
# isolation here is belt-and-braces rather than the only thing standing between
# this and a compromised host.

FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update -qq && apt-get install -y --no-install-recommends \
        gcc \
        libc6-dev \
        make \
        binutils \
        ca-certificates \
        curl \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain stable --profile minimal --no-modify-path
ENV PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

COPY ch_08/ /opt/ch_08/

# cargo needs to run from the project directory for the relative linker script
# path (`-Wl,-T../shellcode.ld`) in .cargo/config.toml to resolve.
RUN cd /opt/ch_08/hello_world \
    && cargo build --release 2>&1 | tail -2 \
    && strip -s target/release/hello_world \
    && objcopy -O binary target/release/hello_world /opt/ch_08/shellcode.bin \
    && cp target/release/hello_world /hello_world \
    && ls -l /opt/ch_08/shellcode.bin \
    && xxd /opt/ch_08/shellcode.bin | head -4

CMD ["/hello_world"]
