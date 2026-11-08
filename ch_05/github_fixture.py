"""Local GitHub-shaped fixture for Chapter 5's github spider.

Serves three pages of public_members so the crawler's pagination logic can be
exercised end to end (page 1 full -> fetches page 2, page 2 full -> fetches
page 3, page 3 short -> stops). No real GitHub API traffic.

Run: python3 github_fixture.py
"""

import http.server
import json
import re
import threading
import time

# third page is short, so the crawl stops there
PAGE_SIZES = {1: 100, 2: 100, 3: 7}


def members(page, count):
    out = []
    for i in range(count):
        out.append(
            {
                "login": f"user{page}_{i}",
                "id": page * 1000 + i,
                "node_id": f"MDQ6VXNlcg{page}{i}",
                "html_url": f"https://github.com/user{page}_{i}",
                "avatar_url": f"https://avatars.githubusercontent.com/u/{page}{i}?v=4",
            }
        )
    return out


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_GET(self):
        match = re.search(r"[?&]page=(\d+)", self.path)
        page = int(match.group(1)) if match else 1
        body = json.dumps(members(page, PAGE_SIZES.get(page, 0))).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass


def main():
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 8097), Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    print("github fixture on 8097, page sizes:", PAGE_SIZES, flush=True)
    time.sleep(3600)


if __name__ == "__main__":
    main()
