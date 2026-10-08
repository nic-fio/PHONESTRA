#!/usr/bin/env python3
"""Turns a small Markdown file (headings, paragraphs, numbered and bulleted
lists, **bold**, `code`, [links](url)) into a light page that is easy to print:
LICENSE.md into licence.html, CHANGELOG.md into changes.html.
Usage: licence-page.py FILE.md [TITLE DESCRIPTION PAGE] > page.html
(without the three arguments: the licence page)."""
import html, re, sys


def inline(t):
    t = html.escape(t, quote=False)
    t = re.sub(r"\*\*(.+?)\*\*", r"<strong>\1</strong>", t)
    t = re.sub(r"`(.+?)`", r"<code>\1</code>", t)
    t = re.sub(r"\[(.+?)\]\((.+?)\)", r'<a href="\2">\1</a>', t)
    t = re.sub(r"(phonestra@nicfio\.it)", r'<a href="mailto:\1">\1</a>', t)
    return t


def body(md):
    out, para, lst = [], [], None

    def flush():
        nonlocal para, lst
        if para:
            out.append("<p>%s</p>" % inline(" ".join(para)))
            para = []
        if lst:
            tag, items = lst
            out.append("<%s>%s</%s>" % (tag, "".join("<li>%s</li>" % inline(i) for i in items), tag))
            lst = None

    for line in md.splitlines():
        s = line.strip()
        m = re.match(r"(#+)\s+(.*)", s)
        if m:
            flush()
            n = len(m.group(1))
            out.append("<h%d>%s</h%d>" % (n, inline(m.group(2)), n))
        elif re.match(r"(\d+\.|-)\s+", s):
            if para:
                flush()
            tag = "ol" if s[0].isdigit() else "ul"
            if not lst or lst[0] != tag:
                flush()
                lst = (tag, [])
            lst[1].append(re.sub(r"^(\d+\.|-)\s+", "", s))
        elif s and lst and line.startswith("  "):
            lst[1][-1] += " " + s
        elif s:
            if lst:
                flush()
            para.append(s)
        else:
            flush()
    flush()
    return "\n".join(out)


md = open(sys.argv[1]).read()
if len(sys.argv) == 5:
    title, description, page = sys.argv[2:5]
else:
    title = "Phonestra Freeware Licence"
    description = "The licence of Phonestra: free to use and to give away, not to sell or include in commercial products."
    page = "licence.html"
print("""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>%s</title>
<meta name="description" content="%s">
<link rel="canonical" href="https://phonestra.nicfio.it/%s">
<style>
body{margin:0;background:#f4f6fa;color:#111827;font:16px/1.65 Cantarell,"Segoe UI",Roboto,Arial,sans-serif}
main{max-width:760px;margin:0 auto;padding:48px 24px 64px;background:#fff;box-shadow:0 1px 30px rgba(15,23,42,.08)}
nav{max-width:760px;margin:0 auto;padding:18px 24px;font-size:15px}
a{color:#0050c0}
h1{font-size:32px;line-height:1.15;margin:0 0 18px;color:#0a2a6b}
h2{font-size:20px;margin:34px 0 10px;color:#0a2a6b}
ol,ul{padding-left:1.4em}li{margin:6px 0}
code{font-family:"DejaVu Sans Mono",Consolas,monospace;font-size:.88em;background:#eef2f7;padding:.08em .35em;border-radius:4px}
@media (max-width:640px){main{padding:32px 16px 48px}nav{padding:14px 16px}h1{font-size:26px}}
@media print{body{background:#fff}main{box-shadow:none}nav{display:none}}
</style>
</head>
<body>
<nav><a href="./">&larr; Phonestra</a></nav>
<main>
%s
</main>
</body>
</html>""" % (html.escape(title), html.escape(description), page, body(md)))
