"""Local replicas for Chapter 5's crawler.

Serves three sites that mirror the shape of the book's targets so the crawl
loop can be exercised without touching a third party:

  :8081  quotes, static HTML, paginated  (the reqwest + select path)
  :8082  quotes, DOM assembled by JS    (the fantoccini path)
  :8097  GitHub-shaped public_members   (JSON API, paginated)
  :8099  crt.sh-shaped entries          (certificate transparency)

Run: python3 server.py   (or via docker compose)
"""
import http.server
import json
import re
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

QUOTES_PORT = 8000
QUOTES_JS_PORT = 8000
GITHUB_PORT = 8001
CRTSH_PORT = 8002

AUTHORS = ["Albert Einstein", "J. K. Rowling", "Jane Austen", "Marilyn Monroe"]
PAGE_SIZES = {1: 10, 2: 10, 3: 4}


def quote_items(page):
    out = []
    for i in range(PAGE_SIZES.get(page, 0)):
        n = (page - 1) * 10 + i
        out.append(
            {
                "text": f"A quote about testing number {n}.",
                "author": AUTHORS[n % len(AUTHORS)],
            }
        )
    return out


def quotes_html(page):
    items = quote_items(page)
    blocks = "\n".join(
        f"""      <div class="quote">
        <span class="text">&#8220;{it['text']}&#8221;</span>
        <span>by <small class="author">{it['author']}</small></span>
      </div>"""
        for it in items
    )
    nxt = f'href="/page/{page + 1}/"' if page in PAGE_SIZES else ""
    next_li = (
        f'<li class="next"><a {nxt}>Next <span class="icon-next"></span></a></li>'
        if nxt
        else ""
    )
    return f"""<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8"><title>Quotes to Scrape</title>
<link rel="stylesheet" href="/static/bootstrap.min.css"></head>
<body>
  <div class="container">
    <div class="row header-box"><div class="col-md-12"><h1>Quotes to Scrape</h1></div></div>
    <div class="row">
      <div class="col-md-8">
{blocks}
      </div>
    </div>
    <nav><ul class="pager">
      <li class="previous"><a href="/page/{max(page - 1, 1)}/">Previous</a></li>
      {next_li}
    </ul></nav>
  </div>
</body>
</html>"""


def github_members(page):
    return [
        {
            "login": f"user{page}_{i}",
            "id": page * 1000 + i,
            "node_id": f"MDQ6VXNlcg{page}{i}",
            "html_url": f"https://github.com/user{page}_{i}",
            "avatar_url": f"https://avatars.githubusercontent.com/u/{page}{i}?v=4",
        }
        for i in range(PAGE_SIZES.get(page, 0))
    ]


JS_PAGE = """<!DOCTYPE html>
<html lang="en"><head><meta charset="UTF-8"><title>Quotes to Scrape</title>
<script>window.__QUOTES__ = %s;</script></head>
<body>
  <div class="container">
    <div class="row"><h1>Quotes to Scrape</h1></div>
    <div class="row"><div class="col-md-8" id="quotes"></div></div>
    <nav><ul class="pager" id="pager"></ul></nav>
  </div>
<script>
(function () {
  var page = Number(new URLSearchParams(location.search).get('page') || 1);
  var quotes = window.__QUOTES__[String(page)] || [];
  var host = document.getElementById('quotes');
  quotes.forEach(function (q) {
    var d = document.createElement('div');
    d.className = 'quote';
    var s1 = document.createElement('span'); s1.className = 'text';
    s1.textContent = '\\u201c' + q.text + '\\u201d';
    var s2 = document.createElement('span');
    var small = document.createElement('small'); small.className = 'author';
    small.textContent = q.author;
    s2.appendChild(document.createTextNode('by ')); s2.appendChild(small);
    d.appendChild(s1); d.appendChild(s2); host.appendChild(d);
  });
  var pager = document.getElementById('pager');
  if (window.__QUOTES__[String(page + 1)]) {
    var li = document.createElement('li'); li.className = 'next';
    var a = document.createElement('a');
    a.setAttribute('href', '/js?page=' + (page + 1));
    a.textContent = 'Next'; li.appendChild(a); pager.appendChild(li);
  }
})();
</script>
</body>
</html>"""


class SiteHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def _send(self, status, body, ctype="text/html; charset=utf-8"):
        raw = body.encode() if isinstance(body, str) else body
        self.send_response(status)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def _page_of(self):
        match = re.search(r"[?&]page=(\d+)", self.path)
        return int(match.group(1)) if match else 1

    def do_GET(self):
        if self.path.startswith("/js"):
            data = {str(p): quote_items(p) for p in PAGE_SIZES}
            self._send(200, JS_PAGE % json.dumps(data))
            return

        match = re.match(r"^/page/(\d+)/$", self.path)
        if match:
            self._send(200, quotes_html(int(match.group(1))))
            return

        if self.path in ("/", "/page/1/"):
            self._send(200, quotes_html(1))
            return

        if self.path.startswith("/orgs/") or "public_members" in self.path:
            page = self._page_of()
            self._send(200, json.dumps(github_members(page)), "application/json")
            return

        if "crt.sh" in self.path or "output=json" in self.path:
            self._send(200, json.dumps([{"name_value": "localhost"}]), "application/json")
            return

        self._send(404, "<h1>404</h1>")

    def log_message(self, *args):
        pass


def serve(port, handler_cls):
    ThreadingHTTPServer(("0.0.0.0", port), handler_cls).serve_forever()


def main():
    sites = [
        ("quotes (static HTML at /, JS-rendered at /js)", QUOTES_PORT, SiteHandler),
        ("github-shaped public_members", GITHUB_PORT, SiteHandler),
        ("crt.sh-shaped entries", CRTSH_PORT, SiteHandler),
    ]
    threads = [
        threading.Thread(target=serve, args=(p, h), daemon=True) for _, p, h in sites
    ]
    for t in threads:
        t.start()
    for name, p, _ in sites:
        print(f"serving {name} on {p}", flush=True)
    threading.Event().wait()


if __name__ == "__main__":
    main()
