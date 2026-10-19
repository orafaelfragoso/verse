## 20. Complete example programs

Each program below is a complete, runnable entry file. Run one with
`verse file.vs`; run its tests with `verse test file.vs`, which checks the
functions and types but never runs the top-level program (§14.1) — so the
tests below read no files, touch no network, and never call `exit`.

### 20.1 CLI word counter

```verse
## wordcount — show the most common words in one or more text files.
##
## Usage: verse wordcount.vs [--top N] FILE...

import fs

type Options
    top: Int = 10
    files: List<Text> = []

## Reads command-line arguments into Options.
fn parse_args(raw: List<Text>)                   # returns Options | Error (inferred)
    mut options = Options()
    mut i = 0
    while i < raw.len()
        match raw[i]
            "--top" ->
                value = raw.get(i + 1).or_error("--top needs a number")?
                options.top = value.to_int().context("--top expects a number")?
                i += 2
            flag if flag.starts_with("--") ->
                return error("unknown option: {flag}")
            file ->
                options.files.push(file.copy())  # `raw` is borrowed: keep a copy
                i += 1
    if options.files.is_empty()
        return error("usage: wordcount [--top N] FILE...")
    options

## Counts words case-insensitively, ignoring surrounding punctuation.
fn count_words(text: Text): Map<Text, Int>
    mut counts: Map<Text, Int> = {:}
    for word in text.lower().words()
        clean = word.trim_matches(c -> not c.is_letter())
        continue if clean.is_empty()
        # `clean` is a view of `word`; the map must own its keys.
        counts[clean.copy()] = counts.get(clean, default: 0) + 1
    counts

# ---- program ----

options = parse_args(args)?

mut totals: Map<Text, Int> = {:}
mut word_total = 0
for path in options.files
    text = fs.read_text(path).context("reading {path}")?
    for (word, n) in count_words(text)
        totals[word] = totals.get(word, default: 0) + n
        word_total += n

file_label = if options.files.len() == 1 then "file" else "files"
print("{word_total} words, {totals.len()} distinct, in {options.files.len()} {file_label}")
print()

top = totals
    .entries()
    .sort_by(.value, descending: true)
    .prefix(options.top)

for (rank, entry) in top.enumerate()
    print("{rank + 1:>3}. {entry.key:<20} {entry.value:>6}")

# ---- tests ----

test "counting ignores case and punctuation"
    counts = count_words("The cat. The CAT, the end!")
    assert counts["the"] == 3
    assert counts["cat"] == 2
    assert "end!" not in counts

test "parse_args reads --top and files"
    options = parse_args(["--top", "3", "a.txt", "b.txt"])?
    assert options.top == 3
    assert options.files == ["a.txt", "b.txt"]

test "parse_args requires at least one file"
    assert parse_args([]) is _: Error
```

Sample run:

```
$ verse wordcount.vs --top 3 poem.txt
112 words, 74 distinct, in 1 file

  1. the                       9
  2. and                       6
  3. of                        4
```

### 20.2 Concurrent web fetcher

```verse
## fetch — download several pages at once and report on each.
##
## Usage: verse fetch.vs [URL...]

import get from http
import TaskPanicked from task
import time

type Report
    url: Text
    status: Int
    bytes: Int
    elapsed: time.Duration

## Fetches one page. Runs as an ordinary function; `spawn` makes it concurrent.
## Takes `url` because the report keeps it.
fn fetch(take url): Report | Error
    started = time.now()
    response = get(url, timeout: 10.seconds()).context("fetching {url}")?
    Report(
        url:,
        status: response.status,
        bytes: response.body.bytes().len(),
        elapsed: time.now() - started,
    )

fn describe(report)
    mark = if report.status < 400 then "ok " else "bad"
    "{mark} {report.status}  {report.url}  {report.bytes} bytes in {report.elapsed}"

# ---- program ----

defaults = ["https://example.com", "https://example.org", "https://example.net"]
urls = if args.is_empty() then defaults else args

print("Fetching {urls.len()} pages…")
started = time.now()

# Start every download at once; each `spawn` returns a Task immediately.
# `url` is borrowed from `urls`, so each task gets its own copy.
tasks = urls.map(url -> spawn fetch(url.copy()))

mut succeeded = 0
# `zip` can hand over the tasks only because `tasks` is at its last use here:
# a `Task` cannot be copied, and `wait` below takes each one.
for (url, task) in urls.zip(tasks)
    # Errors from the task surface here, at `.wait()`.
    match task.wait()
        report: Report ->
            succeeded += 1
            print("  {describe(report)}")
        crash: TaskPanicked ->
            # The panic and its trace were already printed when it happened.
            print("  crash    {url}  (bug in fetch: {crash.message()})")
        err: Error ->
            print("  err      {url}  {err.message()}")

print("{succeeded}/{urls.len()} succeeded in {time.now() - started}")
if succeeded < urls.len()
    exit(1)

# ---- tests ----

test "describe marks client errors as bad"
    report = Report(url: "x", status: 404, bytes: 0, elapsed: 5.millis())
    assert describe(report).starts_with("bad 404")
```

Because `fetch` is an ordinary function, the same code works sequentially
(`fetch(url)?`) or concurrently (`spawn fetch(url)`), with no `async` variant.
In both cases `fetch` takes its `url`, so a caller that still needs the text
passes `url.copy()`.
The three requests overlap, so the total time is close to the slowest single
request.

### 20.3 Shape areas with traits

```verse
## shapes — areas and perimeters with variants, records, traits and generics.

import math

type Shape = Circle(radius: Float) | Square(side: Float) | Rect(width: Float, height: Float)

type Triangle
    a: Float
    b: Float
    c: Float

## Anything with an area and a perimeter.
trait Measurable
    fn name(self): Text
    fn area(self): Float
    fn perimeter(self): Float

    fn summary(self): Text
        "{self.name():<10} area {self.area():>7.2}   perimeter {self.perimeter():>6.2}"

impl Measurable for Shape
    fn name(self): Text
        match self
            Circle(_) -> "circle"
            Square(_) -> "square"
            Rect(_, _) -> "rectangle"

    fn area(self): Float
        match self
            Circle(r) -> math.pi * r * r
            Square(s) -> s * s
            Rect(w, h) -> w * h

    fn perimeter(self): Float
        match self
            Circle(r) -> 2.0 * math.pi * r
            Square(s) -> 4.0 * s
            Rect(w, h) -> 2.0 * (w + h)

impl Triangle
    ## Builds a triangle, rejecting side lengths that cannot close.
    fn new(a: Float, b: Float, c: Float): Triangle | Error
        if a + b <= c or a + c <= b or b + c <= a
            return error("sides {a}, {b}, {c} cannot form a triangle")
        Triangle(a:, b:, c:)

impl Measurable for Triangle
    fn name(self): Text
        "triangle"

    fn area(self): Float
        # Heron's formula
        s = self.perimeter() / 2.0
        (s * (s - self.a) * (s - self.b) * (s - self.c)).sqrt()

    fn perimeter(self): Float
        self.a + self.b + self.c

## The item with the largest area, if there is one.
fn largest<T: Measurable>(take items: List<T>): T?
    # `items` is owned and at its last use, so the single element moves out
    # and the rest of the list is freed (§9.5).
    items.sort_by(.area(), descending: true).first()

# ---- program ----

shapes: List<Measurable> = [
    Circle(radius: 1.5),
    Square(side: 2.0),
    Rect(width: 3.0, height: 1.25),
    Triangle.new(3.0, 4.0, 5.0)?,
]

print("Shapes")
for shape in shapes
    print("  {shape.summary()}")

total = shapes.map(.area()).sum()
print("Total area: {total:.2}")

if largest(shapes) is biggest?             # takes `shapes`: its last use
    print("Largest:    {biggest.name()}")

# ---- tests ----

test "a square of side 2 has area 4"
    assert Square(side: 2.0).area() == 4.0

test "a 3-4-5 triangle has area 6"
    t = Triangle.new(3.0, 4.0, 5.0)?
    assert t.area() == 6.0
    assert t.perimeter() == 12.0

test "impossible triangles are rejected"
    assert Triangle.new(1.0, 1.0, 5.0) is _: Error
```

Output:

```
Shapes
  circle     area    7.07   perimeter   9.42
  square     area    4.00   perimeter   8.00
  rectangle  area    3.75   perimeter   8.50
  triangle   area    6.00   perimeter  12.00
Total area: 20.82
Largest:    circle
```

---

