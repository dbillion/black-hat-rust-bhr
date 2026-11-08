"""Local controlled target for verifying Chapter 2's tricoder.

Provides:
  * a crt.sh-format JSON fixture on 127.0.0.1:8099
  * real TCP listeners on well-known high ports from the common-100 list

Used so the DNS + rayon port-scanning path can be exercised end to end
without scanning any third-party infrastructure.
"""
import http.server
import json
import socket
import threading
import time

PORTS = [1521, 5432, 6379, 8443, 9443]


class CrtShFixture(http.server.BaseHTTPRequestHandler):
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
        s.listen(8)
        socks.append(s)

    server = http.server.HTTPServer(("127.0.0.1", 8099), CrtShFixture)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    print(f"fixture on 8099, listeners on {PORTS}", flush=True)
    time.sleep(600)


if __name__ == "__main__":
    main()
