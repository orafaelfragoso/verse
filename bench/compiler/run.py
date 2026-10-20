#!/usr/bin/env python3
"""Measure the M0 front end; build and input generation are outside timed runs."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import statistics
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]


def generated(lines):
    chunks = [f"fn function_{i}(x: Int): Int\n    x + {i}\n\n" for i in range(lines // 3)]
    chunks.extend("# padding\n" for _ in range(lines % 3))
    return "".join(chunks)


def measure(binary, path, stage, runs):
    timings = []
    memory = []
    record = {}
    with tempfile.TemporaryDirectory(prefix="verse-bench-") as directory:
        peak_file = Path(directory) / "peak"
        for _ in range(runs):
            command = [str(binary), stage, str(path)]
            if Path("/usr/bin/time").exists():
                command = ["/usr/bin/time", "-f", "%M", "-o", str(peak_file), *command]
            record = json.loads(subprocess.check_output(command, text=True))
            timings.append(record["elapsed_ns"])
            if peak_file.exists():
                memory.append(int(peak_file.read_text().strip()))
    median = statistics.median(timings)
    spread = (max(timings) - min(timings)) / median * 100
    return {**record, "input": path.name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest(), "runs": runs, "samples_ns": timings, "median_ns": median, "spread_percent": spread, "peak_rss_kib": max(memory) if memory else None, "gate_eligible": spread <= 3}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runs", type=int, default=10)
    parser.add_argument("--sizes", type=int, nargs="+", default=[1000, 10000, 100000])
    parser.add_argument("--output", type=Path, default=ROOT / "dist/bench/compiler.json")
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    if args.runs < 10:
        parser.error("performance §9.2 requires at least ten runs")
    subprocess.run(["cargo", "build", "--locked", "--release", "-p", "verse-cli", "--example", "compiler-bench"], cwd=ROOT, check=True)
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version=1"], cwd=ROOT))
    binary = Path(metadata["target_directory"]) / "release/examples/compiler-bench"
    inputs = ROOT / "dist/bench/inputs"
    inputs.mkdir(parents=True, exist_ok=True)
    records = []
    for size in args.sizes:
        if size < 3:
            parser.error("project sizes must be at least three lines")
        path = inputs / f"project-{size}.vs"
        path.write_text(generated(size))
        for stage in ["lex", "parse", "ast", "format"]:
            records.append(measure(binary, path, stage, args.runs))
    pathological = inputs / "single-function-50000.vs"
    pathological.write_text("fn pathological()\n" + "".join(f"    value_{i} = {i}\n" for i in range(50000)))
    for stage in ["lex", "parse", "ast", "format"]:
        records.append(measure(binary, pathological, stage, args.runs))
    for path in sorted((ROOT / "bench/compiler/pathological").glob("*.vs")):
        for stage in ["lex", "parse", "ast", "format"]:
            records.append(measure(binary, path, stage, args.runs))
    result = {"rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(), "platform": platform.platform(), "cpu": platform.processor(), "frequency_pinned": False, "limiter": "CPU and allocation; lex, parse, arena lowering and formatting run without I/O in their timed region. Peak RSS includes source loading and process overhead.", "results": records}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    regressions = []
    baseline_path = args.baseline or ROOT / "bench/compiler/baseline.json"
    if baseline_path.exists():
        baseline = json.loads(baseline_path.read_text())
        comparable = all(result[key] == baseline.get(key) for key in ("platform", "cpu", "rustc"))
        before = {(r["sha256"], r["stage"]): r for r in baseline["results"]} if comparable else {}
        for after in records:
            old = before.get((after["sha256"], after["stage"]))
            if old and old["gate_eligible"] and after["gate_eligible"] and after["median_ns"] > old["median_ns"] * 1.05:
                regressions.append(f"{after['input']} {after['stage']}: more than 5% regression")
    if args.json:
        print(json.dumps({**result, "regressions": regressions}))
    else:
        for record in records:
            print(f"{record['input']:24} {record['stage']:6} {record['median_ns'] / 1e6:9.3f} ms, spread {record['spread_percent']:.1f}%, peak {record['peak_rss_kib']} KiB")
        print(f"results: {args.output}")
        for regression in regressions:
            print(regression)
    return bool(regressions)


if __name__ == "__main__":
    raise SystemExit(main())
