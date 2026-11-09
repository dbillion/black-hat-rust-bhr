# Isolated target for Black Hat Rust Chapter 8 reverse_tcp shellcode.
#
# This shellcode is the one exercise in the chapter that connects OUT: it opens
# a socket to 127.0.0.1:8042, dup2()s stdin/stdout/stderr onto it, and execve()s
# /bin/sh. Anything typed on the listening side runs as the victim user.
#
# Containment design -- this is the part that matters:
#   * The listener and the shellcode live in the SAME container, because the
#     shellcode targets 127.0.0.1. Nothing has to be routed or published.
#   * `--network none` removes eth0 entirely. Loopback still works inside the
#     container, so the connection succeeds, but no byte can leave the container.
#   * The listener runs as the same non-root `victim` user that runs the
#     shellcode, so the shell it spawns is container-victim and nothing more.
#   * No host mount, no --privileged, no published port.
#   * The container is --rm and ephemeral.
#
# Build: docker build -f targets/ch_08-reversetcp/Dockerfile -t bhr-ch8-reversetcp .
# Run:   docker run --rm --network none bhr-ch8-reversetcp
#
# The listener prints a banner and runs one command sent from the far side, so
# the whole interaction is captured in the container's own stdout.

FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update -qq && apt-get install -y --no-install-recommends \
        gcc libc6-dev make binutils ca-certificates curl netcat-openbsd \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash victim

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain stable --profile minimal --no-modify-path
ENV PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

COPY ch_08/ /opt/ch_08/

# Build the shellcode as root (rustup lives in /root), then hand it to victim.
RUN cd /opt/ch_08/reverse_tcp \
    && cargo build --release 2>&1 | tail -2 \
    && strip -s target/release/reverse_tcp \
    && objcopy -O binary target/release/reverse_tcp /opt/ch_08/shellcode.bin \
    && cp target/release/reverse_tcp /reverse_tcp \
    && chmod 755 /reverse_tcp /opt/ch_08/shellcode.bin

# The whole demonstration, run as the non-root victim user:
#   1. a listener binds 127.0.0.1:8042 inside this container
#   2. the shellcode connects to it and hands over a shell
#   3. one command runs on that shell, proving who it is
#   4. both sides exit
USER victim
WORKDIR /home/victim

COPY targets/ch_08-reversetcp/demo.sh /opt/demo.sh
RUN chmod +x /opt/demo.sh

CMD ["/opt/demo.sh"]
