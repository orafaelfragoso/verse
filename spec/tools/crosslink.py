"""Rewrite in-page spec links (#anchor) in the part files into cross-file links
(NN-name.md#anchor) when the anchor lives in another part. Idempotent.
build.sh undoes this with a sed rule so the combined spec keeps #anchor links."""
import re, sys, os
os.chdir(os.path.dirname(os.path.abspath(__file__)) + '/..')
PARTS = ['00-README.md'] + sorted(f for f in os.listdir('.') if re.match(r'\d\d-.*\.md$', f) and f not in ('00-README.md', '17-grammar.md')) + ['grammar.ebnf.md']

def slug(t):
    v = re.sub(r'[^\w\- ]', '', t.strip().lower())
    return v.replace(' ', '-')

owner, seen = {}, {}
for f in PARTS:
    fence = False
    for line in open(f):
        if line.startswith('```'): fence = not fence
        m = None if fence else re.match(r'(#{1,6}) (.*)', line)
        if m:
            s = slug(m.group(2)); n = seen.get(s, 0); seen[s] = n + 1
            owner[s if n == 0 else f'{s}-{n}'] = f

unresolved, changed = [], 0
for f in PARTS:
    src = open(f).read()
    def fix(m):
        global changed
        target, anchor = m.group(1), m.group(2)
        home = owner.get(anchor)
        if home is None:
            unresolved.append((f, anchor)); return m.group(0)
        new = f'](#{anchor})' if home == f else f']({home}#{anchor})'
        if new != m.group(0): changed += 1
        return new
    out = re.sub(r'\]\(((?:[\w.-]+\.md)?)#([\w-]+)\)', fix, src)
    if out != src: open(f, 'w').write(out)
print(f'links rewritten: {changed}')
for f, a in unresolved: print(f'UNRESOLVED {f}: #{a}')
