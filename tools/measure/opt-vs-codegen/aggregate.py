"""Compare MIR optimization with code generation in the rustc self-profiles
that run.sh writes: target/prof/<profile>-<backend>-<run>/<crate>-<pid>.mm_profdata.

  ./run.sh

MIR optimization is the self time of `optimized_mir` and of the `mir_pass_*`
labels of optimizing passes; LOWERING lists the lowering and lint passes left
out. The upper bound counts every `mir_pass_*` label.
rustc optimizes each generic item once; Verse optimizes each mono instance, so
the scaled column multiplies by instances (functions code-generated) per item.
"""
import collections, os, re, statistics, subprocess, sys

UNIT = {'ns': 1e-6, 'µs': 1e-3, 'us': 1e-3, 'ms': 1.0, 's': 1000.0}
ROW = re.compile(r'^\| (\S.*?) \.*\s*\| ([\d.]+)(ns|µs|us|ms|s)\s*\| [\d.]+\s*\| ([\d.]+)(ns|µs|us|ms|s)\s*\| (\d+)\s*\|')
LOWERING = re.compile(r'check_|lint|elaborate|promote|subtyper|derefer|post_analysis|'
                      r'abort_unwinding_calls|function_item_references|add_|cleanup_post_borrowck|'
                      r'remove_false_edges|remove_place_mention|ctfe_limit|remove_noop_landing_pads')
CODEGEN = {
    'llvm': ['codegen_module', 'LLVM_module_optimize', 'LLVM_module_codegen'],
    'cranelift': ['codegen fn', 'Compilation passes', 'write object file'],
}
INSTANCES = {'llvm': 'items_of_instance', 'cranelift': 'codegen fn'}

def rows(path):
    out = subprocess.run([os.path.expanduser('~/.cargo/bin/summarize'), 'summarize', path],
                         capture_output=True, text=True, check=True).stdout
    for line in out.splitlines():
        m = ROW.match(line)
        if m:
            yield m.group(1), float(m.group(2)) * UNIT[m.group(3)], float(m.group(4)) * UNIT[m.group(5)], int(m.group(6))

runs = collections.defaultdict(list)
for d in sorted(os.listdir(sys.argv[1])):
    cfg = d.rsplit('-', 1)[0]
    backend = cfg.split('-')[1]
    opt = upper = gen = items = inst = 0.0
    for f in os.listdir(os.path.join(sys.argv[1], d)):
        for label, self_ms, total_ms, count in rows(os.path.join(sys.argv[1], d, f)):
            if label == 'optimized_mir' or label.startswith('mir_pass_'):
                upper += self_ms
                if not LOWERING.search(label):
                    opt += self_ms
            if label == 'optimized_mir':
                items += count
            if label in CODEGEN[backend]:
                gen += total_ms
            if label == INSTANCES[backend]:
                inst += count
    assert gen > 0 and opt > 0, f'{d}: no codegen or MIR labels found'
    runs[cfg].append((opt, gen, opt / gen, opt * inst / items / gen, upper / gen))

for cfg, rs in runs.items():
    med = lambda i: statistics.median(r[i] for r in rs)
    rng = lambda i: f'{min(r[i] for r in rs):.1%} to {max(r[i] for r in rs):.1%}'
    print(f'{cfg}: {len(rs)} runs; MIR opt {med(0):.0f} ms, codegen {med(1):.0f} ms; '
          f'opt/codegen {med(2):.1%} ({rng(2)}); per instance {med(3):.1%} ({rng(3)}); '
          f'upper bound {med(4):.1%}')
