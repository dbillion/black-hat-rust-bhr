"""Playwright renderer used only to capture screenshots of real CLI output.

Run: python3 render.py <"Title"> <command-file>
The file's stdout is captured verbatim by the caller and pasted into this
template - no output is synthesised here.
"""
import html
import os
import sys


def main():
    title, cmd_file = sys.argv[1], sys.argv[2]
    subtitle = sys.argv[3] if len(sys.argv) > 3 else ""
    text = open(cmd_file, encoding="utf-8", errors="replace").read()
    body = html.escape(text)
    out = f"""<!doctype html>
<html><head><meta charset="utf-8"><title>{html.escape(title)}</title>
<style>
 html,body{{margin:0;background:#0d1117;color:#c9d1d9;
   font:13px/1.5 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}}
 .wrap{{padding:28px 30px}}
 .bar{{height:30px;background:#161b22;border-bottom:1px solid #30363d;
   display:flex;align-items:center;gap:7px;padding:0 12px;border-radius:7px 7px 0 0}}
 .dot{{width:11px;height:11px;border-radius:50%}}
 .t{{margin-top:16px;font-size:20px;font-weight:600;color:#e6edf3}}
 .s{{color:#8b949e;font-size:13px;margin-top:4px}}
 pre{{white-space:pre-wrap;word-break:break-word;margin-top:18px;
   background:#0b0f14;border:1px solid #21262d;border-radius:8px;padding:18px 20px}}
</style></head><body><div class="wrap">
<div class="bar"><span class="dot" style="background:#ff5f56"></span>
<span class="dot" style="background:#ffbd2e"></span>
<span class="dot" style="background:#27c93f"></span></div>
<div class="t">{html.escape(title)}</div>
<div class="s">{html.escape(subtitle)}</div>
<pre>{body}</pre>
</div></body></html>"""
    path = "/tmp/shot.html"
    open(path, "w", encoding="utf-8").write(out)
    print(path)


if __name__ == "__main__":
    main()
