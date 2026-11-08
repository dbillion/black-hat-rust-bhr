"""Controlled target for Chapter 3: a local crt.sh-format endpoint plus real
listeners, so the async scanner can be measured without crt.sh in the path.

Run: python3 fixture_target.py
"""
import http.server
import json
import socket
import threading
import time

PORTS = [80, 443, 8000, 8080, 8443]


class CrtShFixture(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_GET(self):
        body = json.dumps([{"name_value": "localhost"}]).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass


def main():
    socks = []
    for port in PORTS:
        s = socket.socket()
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind(("127.0.0.1", port))
        s.listen(16)
        socks.append(s)

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 8099), CrtShFixture)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    print(f"fixture on 8099, listeners on {PORTS}", flush=True)
    time.sleep(3600)


if __name__ == "__main__":
    main()
