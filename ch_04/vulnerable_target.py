"""Deliberately-vulnerable local target for Chapter 4.

Serves the two disclosures the book's HTTP modules look for (.git/HEAD and
.env) plus a crt.sh-format subdomain fixture, so the whole pipeline -
subdomain enumeration, DNS, port scan, 14 vulnerability modules - fires end to
end against something I control rather than a third party.

Run: python3 vulnerable_target.py
"""
import http.server
import json
import socket
import threading
import time

FIXTURE_PORT = 8099
FIXTURE_SUBDOMAIN = "localhost"

# disclosures the book's modules look for
LEAKS = {
    "/.git/HEAD": "ref: refs/heads/master\n",
    "/.env": "DB_PASSWORD=supersecret\nSECRET_KEY=notsosecret\n",
    "/.git/config": "[core]\n\trepositoryformatversion = 0\n",
}


class CrtShFixture(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def _json(self, body):
        raw = json.dumps(body).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def do_GET(self):
        if self.path.startswith("/cdx/"):
            self._json([[f"http://{FIXTURE_SUBDOMAIN}/"]])
        else:
            self._json([{"name_value": FIXTURE_SUBDOMAIN}])

    def log_message(self, *args):
        pass


class Leaky(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_GET(self):
        if self.path in LEAKS:
            body = LEAKS[self.path].encode()
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
        else:
            body = b"not found\n"
            self.send_response(404)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

    def log_message(self, *args):
        pass


def main():
    threading.Thread(
        target=http.server.ThreadingHTTPServer(
            ("127.0.0.1", FIXTURE_PORT), CrtShFixture
        ).serve_forever,
        daemon=True,
    ).start()
    threading.Thread(
        target=http.server.ThreadingHTTPServer(
            ("127.0.0.1", 8081), Leaky
        ).serve_forever,
        daemon=True,
    ).start()
    print(f"crt.sh fixture on {FIXTURE_PORT}, leaky http server on 8081", flush=True)
    time.sleep(3600)


if __name__ == "__main__":
    main()
