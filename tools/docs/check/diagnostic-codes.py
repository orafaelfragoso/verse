"""Check diagnostic codes across the docs.
V and R codes are defined by a table row `| CODE |` (or `| A to B |`) in docs/reference/diagnostics-catalog.md;
D codes by an `[errors.Dxxxx]` table in docs/reference/events.toml.
Reports: codes referenced anywhere but never defined, codes defined twice,
and codes the catalog lists as unassigned are allowed."""
import os, re, sys
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', '..'))
CAT = 'docs/reference/diagnostics-catalog.md'
EV = 'docs/reference/events.toml'
CODE = re.compile(r'\b(?:V\d{4}|R\d{4}|D\d{4})\b')
files = ['README.md'] + sorted(os.path.join(d, f) for d, _, fs in os.walk('docs') for f in fs
                               if f.endswith(('.md', '.toml', '.ebnf')))
defined, unassigned = {}, set()
for no, line in enumerate(open(CAT), 1):
    m = re.match(r'\|\s*([VR])(\d{4})(?:\s+to\s+[VR](\d{4}))?\s*\|', line)
    if not m: continue
    lo = int(m.group(2)); hi = int(m.group(3) or lo)
    for n in range(lo, hi + 1):
        c = f'{m.group(1)}{n:04d}'
        defined.setdefault(c, []).append(no)
for line in open(CAT):
    if re.search(r'unassigned', line): unassigned.update(CODE.findall(line))
cur = None
for no, line in enumerate(open(EV), 1):
    m = re.match(r'\[errors\.(D\d{4})\]', line)
    if m: cur = m.group(1); defined.setdefault(cur, []).append(no); continue
    if line.startswith('['): cur = None
refs = {}
for f in files:
    for no, line in enumerate(open(f), 1):
        for c in CODE.findall(line):
            refs.setdefault(c, []).append(f'{f}:{no}')
bad = 0
for c, lines in sorted(defined.items()):
    if len(lines) > 1: bad += 1; print(f'defined twice: {c} at lines {lines}')
for c, where in sorted(refs.items()):
    if c not in defined and c not in unassigned:
        bad += 1; print(f'undefined: {c} referenced at {", ".join(where)}')
print(f'{len(defined)} codes defined, {len(refs)} referenced; problems: {bad}')
sys.exit(1 if bad else 0)
