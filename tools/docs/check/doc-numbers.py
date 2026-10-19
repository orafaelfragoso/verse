"""Check architecture/reference document numbers.
- Every docs/architecture/*.md and docs/reference/*.md except README.md has the title `# NN. Title`
  with a unique NN, and so does docs/roadmap.md.
- A bare `[NN]` link never points at a spec chapter (those are cited as `spec §N`).
- Every link `[NN ...](file.md...)` names the target doc's number, and a `§N.x` in the link text
  starts with that number.
- Every unlinked `NN §M.x` mention has M == NN and NN is an existing doc.
- docs/README.md and docs/architecture/README.md list every numbered doc as `NN` next to a link
  to it, and link no missing file."""
import os, re, sys
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', '..'))
DIRS = ['docs/architecture', 'docs/reference']
num, bad = {}, 0
def err(msg):
    global bad; bad += 1; print(msg)
for d in DIRS:
    for f in sorted(os.listdir(d)):
        if not f.endswith('.md') or f == 'README.md': continue
        p = os.path.join(d, f); title = open(p).readline().rstrip('\n')
        m = re.match(r'# (\d\d)\. \S', title)
        if not m: err(f'{p}:1: title is not `# NN. Title`: {title}'); continue
        if m.group(1) in num.values(): err(f'{p}:1: number {m.group(1)} used twice')
        num[os.path.normpath(p)] = m.group(1)
p = 'docs/roadmap.md'; m = re.match(r'# (\d\d)\. \S', open(p).readline())
if not m: err(f'{p}:1: title is not `# NN. Title`')
else: num[p] = m.group(1)
by_num = {v: k for k, v in num.items()}
files = ['README.md'] + sorted(os.path.join(d, f) for d, _, fs in os.walk('docs') for f in fs if f.endswith('.md'))
LINK = re.compile(r'\[(\d\d)\b([^\]]*)\]\(([^)#\s]+\.md)(#[^)]*)?\)')
MENTION = re.compile(r'(?<![\w.§/\[])(\d\d) §(\d+)(?:\.\d+)*')
for f in files:
    fence = False
    for no, line in enumerate(open(f), 1):
        if line.startswith('```'): fence = not fence; continue
        if fence: continue
        for n, rest, target, _ in LINK.findall(line):
            dest = os.path.normpath(os.path.join(os.path.dirname(f), target))
            if dest.startswith('docs/language/'):
                err(f'{f}:{no}: bare [{n}] links to spec chapter {target}; write it as spec §{int(n)}'); continue
            if dest not in num: err(f'{f}:{no}: [{n}...] links to unnumbered {target}'); continue
            if num[dest] != n: err(f'{f}:{no}: link text {n} but {target} is {num[dest]}')
            s = re.search(r'§(\d+)', rest)
            if s and s.group(1).zfill(2) != num[dest]: err(f'{f}:{no}: §{s.group(1)} in link to doc {num[dest]}')
        for n, sec in MENTION.findall(LINK.sub('', line)):
            if n not in by_num: err(f'{f}:{no}: mention of doc {n}, which does not exist')
            elif sec.zfill(2) != n: err(f'{f}:{no}: `{n} §{sec}` section does not match doc number')
for idx in ['docs/README.md', 'docs/architecture/README.md']:
    text = open(idx).read()
    for p, n in sorted(num.items(), key=lambda kv: kv[1]):
        rel = os.path.relpath(p, os.path.dirname(idx))
        if not re.search(rf'\b{n}\b[^\n]*\]\({re.escape(rel)}\)', text):
            err(f'{idx}: no `{n}` entry linking {rel}')
print(f'{len(num)} numbered docs; problems: {bad}')
sys.exit(1 if bad else 0)
