# Isolated target for Black Hat Rust Chapter 8 shell shellcode.
#
# This shellcode makes one syscall: execve("/bin/sh", argv, NULL). It replaces
# the process with a shell -- no network, no listener, no file writes, no fork.
#
# The point of running it here rather than on the codespace directly is that it
# execs a real shell, so the container bounds what that shell can touch: no
# network, no host mount, no privileges, and an ephemeral filesystem.
#
# Build: docker build -f targets/ch_08-shell/Dockerfile -t bhr-ch8-shell .
# Run:   docker run --rm --network none --user victim bhr-ch8-shell

FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update -qq && apt-get install -y --no-install-recommends \
        gcc libc6-dev make binutils ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# A non-root user so the spawned shell is not root in the container.
RUN useradd -m -s /bin/bash victim

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain stable --profile minimal --no-modify-path
ENV PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

COPY ch_08/ /opt/ch_08/

# Rustup installs into /root, so build as root, then hand the binary to victim.
RUN cd /opt/ch_08/shell \
    && cargo build --release 2>&1 | tail -2 \
    && strip -s target/release/shell \
    && cp target/release/shell /shell \
    && chmod 755 /shell

USER victim
WORKDIR /home/victim

CMD ["/shell"]
