"""Simulate the drop stack of 15 §15.3.6 on JSON documents parsed into the
Verse type Json = Null | Bool | Num | Str(Text) | Arr(List<Json>) |
Obj(Map<Text, Json>), and report the most entries any document needs.

  find / -xdev -name '*.json' > corpus.txt
  python3 -I tools/measure/drop-depth.py corpus.txt
"""
import json, sys

def parts(v):
    # The parts of a buffer, in drop order; each is True if it lives in its
    # own heap allocation (a Text, List or Map buffer), else False.
    if isinstance(v, list):
        return v
    if isinstance(v, dict):
        out = []
        for k, x in v.items():
            out.append(k)
            out.append(x)
        return out
    return []

def heap(x):
    return isinstance(x, (str, list, dict))

def max_entries(root):
    if not heap(root):
        return 0
    stack = [[root, 0]]  # entry: buffer, next part index
    worst = 1
    while stack:
        top = stack[-1]
        ps = parts(top[0])
        i = top[1]
        if i >= len(ps):
            stack.pop()
            continue
        p = ps[i]
        last = i == len(ps) - 1
        if heap(p):
            if last:
                stack[-1] = [p, 0]  # free this buffer, continue with its last part
            else:
                top[1] = i + 1
                stack.append([p, 0])
        else:
            top[1] = i + 1
        worst = max(worst, len(stack))
    return worst

def depth(v):
    d, todo = 0, [(v, 1)]
    while todo:
        x, n = todo.pop()
        d = max(d, n)
        todo.extend((y, n + 1) for y in parts(x) if isinstance(y, (list, dict)))
    return d

results, failed = [], 0
for path in open(sys.argv[1]).read().split('\n'):
    if not path:
        continue
    try:
        doc = json.load(open(path, encoding='utf-8'))
    except Exception:
        failed += 1
        continue
    results.append((max_entries(doc), depth(doc), path))
results.sort()
n = len(results)
q = lambda f: results[min(n - 1, int(f * n))][0]
print(f'documents: {n} parsed, {failed} not JSON or unreadable')
print(f'entries: median {q(0.5)}, p99 {q(0.99)}, p99.9 {q(0.999)}, max {results[-1][0]}')
for lim in (8, 16, 32, 64):
    print(f'  documents needing more than {lim}: {sum(1 for r in results if r[0] > lim)}')
print('deepest nesting:', max(r[1] for r in results))
for r in results[-5:]:
    print(' ', r)
# a 100,000-cell Cons list and a degenerate tree, as Arr chains
chain = []
for _ in range(100_000):
    chain = [1, chain]
print('Cons list of 100,000 cells:', max_entries(chain))
left = []
for _ in range(1000):
    left = [left, 1]
print('left-deep tree of 1,000 levels:', max_entries(left))
