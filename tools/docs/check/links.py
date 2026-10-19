"""Check every relative Markdown link in docs/, README.md and .claude/skills/**/*.md.
Verifies the target file exists and, for .md targets, that the #anchor matches a
GitHub-style heading slug. Prints one line per broken link and a summary."""
import os, re, sys
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', '..'))
def slug(t):
    t = re.sub(r'`', '', t.strip().lower())
    return re.sub(r'[^\w\- ]', '', t).replace(' ', '-')
def anchors(path):
    seen, out, fence = {}, set(), False
    for line in open(path):
        if line.lstrip().startswith('```'): fence = not fence; continue
        m = None if fence else re.match(r'#{1,6} (.*?)\s*#*\s*$', line)
        if m:
            s = slug(m.group(1)); n = seen.get(s, 0); seen[s] = n + 1
            out.add(s if n == 0 else f'{s}-{n}')
        for a in re.findall(r'<a (?:name|id)="([^"]+)"', line): out.add(a)
    return out
files = ['README.md'] + sorted(os.path.join(d, f) for d, _, fs in os.walk('docs') for f in fs if f.endswith('.md'))
files += sorted(os.path.join(d, f) for d, _, fs in os.walk('.claude/skills') for f in fs if f.endswith('.md'))
LINK = re.compile(r'\]\(([^)\s]*)\)')
broken = total = 0
cache = {}
for f in files:
    fence = False
    for no, line in enumerate(open(f), 1):
        if line.lstrip().startswith('```'): fence = not fence; continue
        if fence: continue
        for target in LINK.findall(line):
            if re.match(r'[a-z]+:', target): continue
            total += 1
            path, _, anchor = target.partition('#')
            dest = os.path.normpath(os.path.join(os.path.dirname(f), path)) if path else f
            err = None
            if not os.path.exists(dest): err = 'missing file'
            elif anchor and dest.endswith('.md'):
                if dest not in cache: cache[dest] = anchors(dest)
                if anchor not in cache[dest]: err = 'missing anchor'
            if err:
                broken += 1; print(f'{f}:{no}: {err}: {target}')
print(f'checked {total} relative links in {len(files)} files; broken: {broken}')
sys.exit(1 if broken else 0)
