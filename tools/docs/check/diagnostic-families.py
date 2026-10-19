"""Check the pass-to-family table of the diagnostics catalog.
Passes are the Stage column of 02 §2.2 and the backticked names of the 03 §3.4 pass list;
crates are the names in the 02 §2.9 layout. The table is 16 §16.2.1, families are 16.2.
Reports: a pass with no row, a row naming neither a pass nor a crate, a family of 16.2
with no row, and a defined V or R code that no row covers."""
import os, re, sys
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', '..'))
ARCH, IR = 'docs/architecture/compiler-architecture.md', 'docs/architecture/ir-and-passes.md'
CAT = 'docs/reference/diagnostics-catalog.md'

def section(path, head):
    out, on = [], False
    for line in open(path):
        if line.startswith('#'):
            if on: break
            on = line.startswith(head)
        elif on: out.append(line)
    return out

def cells(line):
    return [c.strip() for c in line.strip().strip('|').split('|')]

def rows(lines):
    return [cells(l) for l in lines if l.startswith('|') and not re.match(r'\|\s*-', l)][1:]

stages = {r[0] for r in rows(section(ARCH, '## 2.2 '))}
passes = {n for r in rows(section(IR, '## 3.4 ')) for n in re.findall(r'`([^`]+)`', r[1])}
crates = set(re.findall(r'^\s+(verse-[a-z]+)\s', ''.join(section(ARCH, '## 2.9 ')), re.M))
families = {r[1] for r in rows(section(CAT, '## 16.2 '))}

table = rows(section(CAT, '### 16.2.1 '))
named, covered_fam, covered = set(), set(), set()
bad = 0
def err(msg):
    global bad; bad += 1; print(msg)
for name_cell, codes in table:
    for name in (n.strip().strip('`') for n in name_cell.split(',')):
        if name not in stages | passes | crates:
            err(f'row names `{name}`, which is not a pass of 02 §2.2 or 03 §3.4 nor a crate of 02 §2.9')
        named.add(name)
    covered_fam.update(re.findall(r'\b[VR]\d*x+\b', codes))
    for lo, hi in re.findall(r'\b([VR]\d{4})(?: to ([VR]\d{4}))?\b', codes):
        for n in range(int(lo[1:]), int((hi or lo)[1:]) + 1):
            covered.add(f'{lo[0]}{n:04d}')
for p in sorted((stages | passes) - named):
    err(f'pass `{p}` has no row in 16.2.1')
for f in sorted(families - covered_fam):
    if not any(c.startswith(f.rstrip('x')) for c in covered):
        err(f'family {f} has no pass in 16.2.1')
def in_family(code):
    return any(code.startswith(f.rstrip('x')) and len(f) == len(code) for f in covered_fam)
for line in open(CAT):
    m = re.match(r'\|\s*([VR]\d{4})\s*\|', line)
    if m and not (m.group(1) in covered or in_family(m.group(1))):
        err(f'code {m.group(1)} is defined but no pass in 16.2.1 reports it')
print(f'passes: {len(stages)} stages, {len(passes)} optimizer passes; rows: {len(table)}; problems: {bad}')
sys.exit(1 if bad else 0)
