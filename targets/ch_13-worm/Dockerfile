# Isolated lab for the Chapter 13 self-propagating worm.
#
# CONTAINMENT DESIGN -- read this before running anything.
#
# The worm is `spread(host_port)`: it connects to a *supplied* SSH target,
# brute-forces it with a three-entry wordlist (root/password, root/admin,
# root/root), uploads an agent binary, and executes it remotely. It does NOT
# scan, discover, or propagate on its own -- every hop needs a host:port handed
# to it as an argument.
#
# Therefore the containment is exact rather than approximate:
#   * `--network none` -- the container has loopback and nothing else. Outbound
#     connections are impossible, so no hop can reach a host other than itself.
#   * The SSH server lives *inside* the same container. The "remote" host the
#     worm infects is the container it is already running in.
#   * No volume mounts, no shared filesystem with the host.
#   * `cap-drop=ALL` and `--security-opt no-new-privileges` limit what a
#     successful infection can do even in principle.
#   * The root password is set to `password`, one of the book's own three
#     wordlist entries. No real credential is exposed.
#
# The wordlist is the book's, unmodified -- the point of the exercise is that
# the worm works, not that the wordlist is effective.

FROM ubuntu:24.04
# Not 20.04: the agent is built on the codespace, whose glibc is 2.38, and a
# 20.04 image (glibc 2.31) cannot load it. musl static linking was tried first
# and fails on ssh2's vendored libssh2/openssl.

RUN apt-get update -qq && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y -qq openssh-server sudo >/dev/null 2>&1 && \
    rm -rf /var/lib/apt/lists/*

# The book's own wordlist entry -- the entire credential surface of this lab.
RUN echo 'root:password' | chpasswd

RUN mkdir -p /run/sshd /root/.ssh && \
    ssh-keygen -A >/dev/null 2>&1 && \
    sed -i 's/^#\?PermitRootLogin.*/PermitRootLogin yes/' /etc/ssh/sshd_config && \
    sed -i 's/^#\?PasswordAuthentication.*/PasswordAuthentication yes/' /etc/ssh/sshd_config && \
    sed -i 's/^#\?UsePAM.*/UsePAM no/' /etc/ssh/sshd_config && \
    echo 'PermitRootLogin yes' >> /etc/ssh/sshd_config && \
    echo 'PasswordAuthentication yes' >> /etc/ssh/sshd_config && \
    echo 'PermitEmptyPasswords no' >> /etc/ssh/sshd_config

# The agent and its bundle. The bundle is the book's own mechanism: install()
# extracts bundle.zip into the data dir, and spread() reads agent.linux_x86_64
# from there. Copying it in exercises extract_bundle(), not just the happy path.
COPY agent /opt/lab/agent
COPY bundle.zip /opt/lab/bundle.zip

RUN chmod 755 /opt/lab/agent /opt/lab/bundle.zip

# Start sshd, wait for it, then run the worm against loopback only.
CMD bash -c '\
  /usr/sbin/sshd && \
  for i in $(seq 1 30); do \
    (exec 3<>/dev/tcp/127.0.0.1/22) 2>/dev/null && break; \
    sleep 1; \
  done && \
  echo "=== sshd up on 127.0.0.1:22 ===" && \
  cd /opt/lab && \
  echo "=== uname -a on the target ===" && \
  uname -a && \
  echo "=== running the worm against 127.0.0.1:22 ===" && \
  ./agent 127.0.0.1:22; \
  echo "=== WORM_EXIT=$? ==="; \
  echo "=== what landed in /tmp ==="; \
  ls -la /tmp; \
  echo "=== data dir (install() + bundle extraction) ==="; \
  ls -la /root/.local/share/bhr_ch13 2>/dev/null; \
  echo "=== any second agent process? ==="; \
  ps aux | grep -c "[a]gent" || true; \
  sleep 600'
