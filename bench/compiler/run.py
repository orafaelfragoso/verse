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
    parts = []
    index = 0
    while len(parts) < lines:
        pattern = index % 4
        if pattern == 0:
            parts.append(f"fn function_{index}(x: Int): Int\n    x + {index}\n\n")
        elif pattern == 1:
            parts.append(
                f"fn sum_{index}(x: Int): Int\n"
                f"    values = [x, x + 1, x + 2]\n"
                f"    total = 0\n"
                f"    for value in values\n"
                f"        total = total + value\n"
                f"    total\n\n"
            )
        elif pattern == 2:
            parts.append(
                f"fn choose_{index}(x: Int): Int\n"
                f"    if x > {index} then x else {index}\n\n"
            )
        else:
            parts.append(
                f"## Function {index} maps its input.\n"
                f"fn map_{index}(values: List<Int>): List<Int>\n"
                f"    values.map(value -> value + {index})\n\n"
            )
        index += 1
    return "".join(parts)


def measure(binary, path, stage, runs, iterations):
    timings = []
    run_medians = []
    memory = []
    with tempfile.TemporaryDirectory(prefix="verse-bench-") as directory:
        peak_file = Path(directory) / "peak"
        for _ in range(runs):
            command = [str(binary), stage, str(path), str(iterations)]
            if Path("/usr/bin/time").exists():
                command = ["/usr/bin/time", "-f", "%M", "-o", str(peak_file), *command]
            record = json.loads(subprocess.check_output(command, text=True))
            samples = record["samples_ns"]
            timings.extend(samples)
            run_medians.append(statistics.median(samples))
            if peak_file.exists():
                memory.append(int(peak_file.read_text().strip()))
    median = statistics.median(run_medians)
    quartiles = statistics.quantiles(run_medians, n=4)
    # Spread is the interquartile range of the per-run medians. Averaging the
    # inner iterations first keeps a scheduler hiccup inside one run from
    # disabling the gate, while the ten runs still expose real host noise.
    spread = (quartiles[2] - quartiles[0]) / median * 100
    return {
        "input": path.name,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "stage": stage,
        "runs": runs,
        "iterations": iterations,
        "samples_ns": timings,
        "run_medians_ns": run_medians,
        "median_ns": median,
        "spread_percent": spread,
        "peak_rss_kib": max(memory) if memory else None,
        "gate_eligible": len(run_medians) >= 10 and spread <= 3,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runs", type=int, default=10)
    parser.add_argument("--iterations", type=int, default=5)
    parser.add_argument("--sizes", type=int, nargs="+", default=[1000, 10000, 100000])
    parser.add_argument("--output", type=Path, default=ROOT / "dist/bench/compiler.json")
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    if args.runs < 10:
        parser.error("performance §9.2 requires at least ten runs")
    if args.iterations < 1:
        parser.error("iterations must be at least one")
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
            records.append(measure(binary, path, stage, args.runs, args.iterations))
    pathological = inputs / "single-function-50000.vs"
    pathological.write_text("fn pathological()\n" + "".join(f"    value_{i} = {i}\n" for i in range(50000)))
    for stage in ["lex", "parse", "ast", "format"]:
        records.append(measure(binary, pathological, stage, args.runs, args.iterations))
    for path in sorted((ROOT / "bench/compiler/pathological").glob("*.vs")):
        for stage in ["lex", "parse", "ast", "format"]:
            records.append(measure(binary, path, stage, args.runs, args.iterations))
    result = {
        "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
        "platform": platform.platform(),
        "cpu": platform.processor() or platform.machine(),
        "frequency_pinned": False,
        "limiter": "CPU and allocation; lex, parse, arena lowering and formatting run without I/O in their timed region. AST lowering is timed separately from parsing. Peak RSS includes source loading and process overhead.",
        "results": records,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    regressions = []
    baseline_path = args.baseline or ROOT / "bench/compiler/baseline.json"
    if baseline_path.exists():
        baseline = json.loads(baseline_path.read_text())
        # A baseline is comparable only on the same toolchain, CPU and
        # platform. A new kernel or libc invalidates its absolute timings even
        # when the source and compiler are unchanged.
        comparable = all(
            result[key] == baseline.get(key) for key in ("cpu", "rustc", "platform")
        )
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
