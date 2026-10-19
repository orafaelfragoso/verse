# Architecture 20. First-party packages

Status: **draft 1, every signature is a sketch
(non-normative)**. Nothing here is designed in detail or implemented; the
package APIs are settled in milestone M9 ([11](../roadmap.md)). Labels follow
the [README](README.md) conventions: **Fact**, **Decision**, **Assumption**,
**Target**, **Proposal** (collected in §20.7).

The five packages are `finance`, `ml`, `web`, `gui` and `iot` (Fact: P6-1,
spec §15.2 and §16.3). They are ordinary packages: added with `verse add`,
pinned in `verse.lock`, checked against `verse.api`, shaken like any code
([06](stdlib-and-shaking.md), [19](../reference/cli-and-manifest.md)). They get no
compiler privileges and no syntax.

## 20.1 Rules every package follows

| Rule | Source |
|------|--------|
| **One obvious name.** No second spelling of a standard library idea: `finance` uses `math.Decimal`, `web` builds on `http.serve`, every package uses `time` for dates and durations | 06 §6.2, P6-1 |
| **No stored references.** Values are owned; borrows are second-class (spec §5.3, §9.3). Long-lived structures refer to parts by `Int` index (the arena pattern of spec §4.10) or by `Shared`/`Guarded` handles (spec §11.4) | spec |
| **Failure in the type.** Anything that can fail returns `T \| Error` or a named error type (spec §8, §15) | spec |
| **Resources are `Cleanup` values** used with `with` (spec §13): files, devices, windows, pins | spec |
| **Capabilities are declared** in `[library] capabilities` and are the most any module may reach; a program pays only for what it reaches (06 §6.5) | 06 |
| **Per-target availability is data** in `[library.targets]`, never conditions in source (P6-7) | 06 §6.9 |
| **Each package has a size budget** in `[library] size_budget`, checked by `verse publish` against its examples (Target; no numbers are set yet) | 06 §6.9 |

How to read the signature tables: each row is `fn name(params): Return`, as
it would appear in `verse.api`. Examples are valid Verse 0.1 apart from the
package names themselves, which do not exist yet. Where an example uses a
standard library function whose exact signature the spec does not give, the comment
says *assumed*.

## 20.2 `finance`

**Purpose.** Exact money arithmetic, rates and day counts, business-day
calendars, and time series of values over dates.

**Capabilities:** none. Every function is pure; "today" is passed in by the
caller (from `time`), which keeps finance code testable and replayable
(Decision). **Targets:** all, including `--no-heap` for `Money` and rounding
(Assumption: `math.Decimal` is fixed-size; calendars and series allocate).

### Types (sketch)

```verse
import Decimal from math

export type Money
    export amount: Decimal
    export currency: Currency

export type Currency
    export code: Text                     # ISO 4217, "EUR"
    export minor_units: Int               # 2 for EUR, 0 for JPY

type Rounding = HalfEven | HalfUp | HalfDown | Up | Down

type DayCount = Act360 | Act365 | ActAct | Thirty360

type Roll = Following | ModifiedFollowing | Preceding | Unadjusted
```

`Money` is `Copy` if `Decimal` is (Assumption), so passing it costs nothing
and needs no `.copy()`.

### Functions (sketch)

| Signature | Notes |
|-----------|-------|
| `fn Money.parse(text: Text, currency: Text): Money \| Error` | `"19.99"`; rejects more decimals than the currency allows |
| `fn Money.zero(currency: Text): Money` | |
| `fn add(self, other: Money): Money \| Error` | different currencies are an error, so `Money` does **not** implement `Add`: `+` would have to panic or hide a conversion (Decision) |
| `fn sub(self, other: Money): Money \| Error` | |
| `fn times(self, factor: Decimal): Money` | full precision; round explicitly |
| `fn round(self, places: Int = 2, mode: Rounding = HalfEven): Money` | the rounding rule is always visible at the call or defaulted to banker's rounding |
| `fn allocate(self, parts: List<Int>): List<Money>` | splits by ratio so the parts sum exactly to the whole (largest-remainder) |
| `fn convert(self, to: Text, rate: Decimal): Money` | conversion only with an explicit rate; no hidden rate source |
| `fn Rate.annual(percent: Decimal): Rate` | |
| `fn accrue(self, principal: Money, from: Date, to: Date, basis: DayCount): Money` | `Rate` method; `Date` from `time` |
| `fn Calendar.named(name: Text): Calendar \| Error` | `"TARGET"`, `"NYSE"`, … from bundled holiday data |
| `fn is_business_day(self, day: Date): Bool` | |
| `fn add_business_days(self, day: Date, count: Int): Date` | |
| `fn roll(self, day: Date, rule: Roll): Date` | |
| `fn Series.new(): Series<T>` | ordered by date; inserting an earlier date keeps the order |
| `fn insert(mut self, day: Date, value: T)` | |
| `fn window(self, from: Date, to: Date): Series<T>` | an owned copy, not a view, so it can be stored |
| `fn returns(self): Series<Decimal>` | for `Series<Money>` and `Series<Decimal>` |

### Example

```verse
import Money, Calendar from finance
import Date from time

prices = [Money.parse("19.99", currency: "EUR")?, Money.parse("5.01", currency: "EUR")?]
mut total = Money.zero("EUR")
for price in prices
    total = total.add(price)?
print(total.round())                              # 25.00 EUR

invoiced = Date.parse("2026-10-16")?              # assumed time constructor
target = Calendar.named("TARGET")?
print(target.add_business_days(invoiced, 30))
```

### Shaking

The example reaches `Money.parse`, `zero`, `add`, `round`, `Show` for `Money`
and `Date`, `Calendar.named`, `add_business_days`, and the TARGET holiday
data. Holiday tables are **module constants, one per calendar** (lazy, spec
§12.4), so `Calendar.named` must not be one big `match` over every calendar
name, or every table would be reachable. Decision: calendars are looked up
in a `Map<Text, fn(): Calendar>` built from one constant per calendar, and the
size report shows each table separately. Assumption: a lookup by run-time text
reaches every table in that map, so a program that uses one calendar still
carries all bundled tables. `Calendar.named` stays the only spelling (P20-1,
ruled: one spelling per meaning); the cost is measured in M9 before any
second form is considered.

### Ownership notes

- `Series<T>` owns its points; `window` returns an owned copy because a view
  of a series could not be stored in a report record.
- No `Money` arithmetic allocates, so finance code works inside tight loops
  and on boards.

## 20.3 `ml`

**Purpose.** Tensors, the common operations, reverse-mode automatic
differentiation, loading trained models, inference, and device selection.

**Capabilities:** none for CPU work; `gpu` for device work (P6-6, slice
`sys-gpu`); `fs` only if `load_model` is reached. **Targets:** CPU on every
target with a heap; `gpu` where the target file grants it; not under
`--no-heap` (tensors allocate).

### Types (sketch)

```verse
type Shape
    export dims: List<Int>

type DType = F32 | F64 | I32 | U8
```

`Tensor` is an opaque owned type: a shape, a dtype and one buffer. It is
`Copyable` (`.copy()` duplicates the buffer, explicitly) and **not** `Copy`,
so every expensive duplicate is visible in the source (spec §4.3).

### Functions (sketch)

| Signature | Notes |
|-----------|-------|
| `fn Tensor.zeros(dims: List<Int>, dtype: DType = F32): Tensor` | |
| `fn Tensor.from_list(values: List<List<Float>>): Tensor \| Error` | ragged rows are an error |
| `fn Tensor.random(dims: List<Int>, seed: Int): Tensor` | always seeded: same seed, same tensor, on every tier (replay, 08 §8.5) |
| `fn matmul(self, other: Tensor): Tensor \| Error` | shapes are checked at run time; the error names both shapes |
| `fn add(self, other: Tensor): Tensor \| Error` | broadcasting as in NumPy |
| `fn map(self, f: fn(Float): Float): Tensor` | |
| `fn sum(self): Float`, `fn argmax(self): Int` | |
| `fn row(self, index: Int): Tensor` | a view (spec §9.3): readable, bindable, not storable; `.copy()` to keep |
| `fn scale(mut self, factor: Float)` | in-place variants take `mut self`, so the change is visible at the call |
| `fn load_model(path: Text): Model \| Error` | a documented subset of ONNX (Assumption; format choice is P20-3) |
| `fn run(self, input: Tensor): Tensor \| Error` | `Model` method; inference only |
| `fn Tape.new(): Tape` | autodiff, below |
| `fn param(mut self, value: Tensor): Var`, `fn input(mut self, value: Tensor): Var` | |
| `fn matmul(mut self, a: Var, b: Var): Var \| Error`, `fn mean(mut self, x: Var): Var` | each operation records a node |
| `fn backward(mut self, loss: Var): Gradients \| Error` | |
| `fn open_gpu(index: Int = 0): Gpu \| Error` | `Gpu` is a `Cleanup` resource; needs `gpu` |
| `fn upload(mut self, value: Tensor): GpuTensor \| Error` | `Gpu` method |
| `fn download(mut self, value: GpuTensor): Tensor \| Error` | |

### Autodiff without stored references

A tape owns every recorded node; a `Var` is a `Copy` record holding an `Int`
index into its tape. That is the index-based arena the spec recommends for
graphs (§4.10), so there is no reference from one node to another and no
`Shared` cycle to leak.

```verse
import Tensor, Tape from ml

mut tape = Tape.new()
w = tape.param(Tensor.random([3, 1], seed: 7))
x = tape.input(Tensor.from_list([[1.0, 2.0, 3.0]])?)
y = tape.matmul(x, w)?
loss = tape.mean(y)
grads = tape.backward(loss)?
print(grads.get(w))
```

A `Var` used with a different tape is an error value naming both tapes
(checked by a tape id inside `Var`), not undefined behaviour.

### Devices

```verse
import Tensor, open_gpu from ml

with open_gpu()? as mut gpu
    a = gpu.upload(Tensor.zeros([1024, 1024]))?
    b = gpu.matmul(a, a)?
    result = gpu.download(b)?
    print(result.sum())
```

`GpuTensor` is also an index into the `Gpu` resource that owns device memory,
so it cannot outlive the device in a harmful way: using it after the `with`
block, or with another device, is an error value. The capability appears
where the resource is acquired.

### Shaking

Operations are written in Verse over a small set of kernels, so a program
that only runs inference reaches `load_model`, the operators the model's file
uses (looked up by name, so all operators of the supported subset are
reachable from `load_model`: Assumption, and the reason P20-3 proposes a
generator that turns a model file into Verse code with only its operators),
and the `fs` slice. Autodiff code is reached only from `Tape` methods.

## 20.4 `web`

**Purpose.** Routing, middleware, typed request helpers and templates **on
top of `http.serve`** (Fact: P6-1). `web` has no server loop of its own.

**Capabilities:** `net` (through `http`), `fs` only if static files are
served, `clock` only if a time-based middleware is reached. **Targets:** OS
targets and WASI with sockets; not the browser target (it serves nothing),
not microcontrollers without a network stack (06 §6.8).

### Types (sketch)

```verse
import Request, Response from http

alias Handler = fn(Request): Response | Error              # as spec §4.8
alias Route = fn(Request, Params): Response | Error
alias Middleware = fn(Request, Handler): Response | Error
```

`Request` and `Response(status, headers, body)` are `http`'s types (spec
§15.2); `web` adds no second request type.

### Functions (sketch)

| Signature | Notes |
|-----------|-------|
| `fn Router.new(): Router` | |
| `fn get(take self, pattern: Text, route: Route): Router` | also `post`, `put`, `delete`; builder methods take and return the router |
| `fn wrap(take self, middleware: Middleware): Router` | applied in the order written |
| `fn handler(take self): Handler` | the router moves into the returned function value (spec §5.4: an escaping lambda moves its captures) |
| `fn int(self, name: Text): Int \| Error` | `Params` method: `/users/:id` → `params.int("id")` |
| `fn text(self, name: Text): Text?` | |
| `fn json(value: ToJson): Response` | status 200, `content-type: application/json` |
| `fn text_response(body: Text, status: Int = 200): Response` | |
| `fn not_found(): Response` | |
| `fn static_files(dir: Text): Route` | needs `fs` |

### Example

```verse
import serve, Request, Response from http
import Router, Params, json, text_response from web

type User
    id: Int
    name: Text

fn show_user(_request: Request, params: Params): Response | Error
    id = params.int("id")?
    json(User(id:, name: "Ana"))

fn health(_request: Request, _params: Params): Response | Error
    text_response("ok")

router = Router.new()
    .get("/users/:id", show_user)
    .get("/health", health)
serve(port: 8080, handler: router.handler())?    # assumed serve signature
```

The unused parameters start with `_`, which keeps them without the
unused-binding warning (spec §3.9).

### Templates without new syntax

Templates are **generated code**, not a runtime language: a first-party
generator package ([spec §16.6](../language/16-tooling.md#166-generators))
turns `templates/user.html` into `generated/templates/templates.vs` with one
typed function per template (`fn user_page(user: User): Text`). A missing
field is a compile error in the generated file, which the reader can open.
The project lists it under `[generate]`
([19 §19.3](../reference/cli-and-manifest.md#generate-and-tool)), and `verse generate` refreshes it.

### Shaking and ownership

- A program reaches only the routing table, the routes it registers, and the
  `http` and `net` code `serve` reaches. Unused helpers and the template generator's
  runtime support are absent.
- Each request is handled in its own task (Assumption about `serve`). The
  handler's captured router is shared read-only across tasks, so
  `handler()` wraps it in a `Shared<Router>` (spec §11.4). Mutable
  application state goes in a `Guarded<T>` that each route receives through
  the router (`Router.with_state(state: Guarded<T>)`), never in a global
  (module constants are immutable, spec §12.4).

## 20.5 `gui`

**Purpose.** Windows, widgets, layout and input events for desktop and
browser targets.

**Capabilities:** `surface` (P6-6, slice `sys-surface`), plus `clock` for
animations if reached. **Targets:** Windows, macOS and Linux desktops through
their platform layer (`std-os`), and the browser through `std-web` (canvas or
DOM: Assumption, decided in M9). Not servers, not microcontrollers in 0.1
(`[library.targets] unavailable`).

### State model that fits ownership

Widgets do not hold callbacks that capture and mutate state, because that
needs shared mutable references. Instead the program owns one `Model` value,
and events are **messages** (a variant type), as in the Elm architecture:

1. `view(model)` borrows the model and returns a widget tree, an owned value
   with no references into the model.
2. A click produces the `Msg` value stored in the widget.
3. `update(model, msg)` returns the next model.

```verse
import run_app, column, row, button, label, Widget from gui

type Msg = Increment | Decrement

type Model
    count: Int = 0

fn update(model: Model, msg: Msg): Model
    match msg
        Increment -> model.copy(count: model.count + 1)
        Decrement -> model.copy(count: model.count - 1)

fn view(model: Model): Widget<Msg>
    column([
        label("Count: {model.count}"),
        row([button("-", on_click: Decrement), button("+", on_click: Increment)]),
    ])

run_app(title: "Counter", init: Model(), update:, view:)?
```

### Functions (sketch)

| Signature | Notes |
|-----------|-------|
| `fn run_app<M, E>(title: Text, init: M, update: fn(M, E): M, view: fn(M): Widget<E>): Nothing \| Error` | owns the model; returns when the window closes; `E` is the program's message type |
| `fn label<E>(text: Text): Widget<E>` | `E` is inferred from where the widget is used (no explicit type arguments, spec §10.5) |
| `fn button<E>(text: Text, on_click: E): Widget<E>` | |
| `fn text_input<E>(value: Text, on_change: fn(Text): E): Widget<E>` | the message carries the new text |
| `fn column<E>(children: List<Widget<E>>, spacing: Int = 8): Widget<E>` | also `row`, `stack` |
| `fn padded<E>(child: Widget<E>, all: Int): Widget<E>` | layout is composition, not properties on every widget |
| `fn every<E>(interval: Duration, msg: E): Subscription<E>` | timed messages (`clock`) |

Long work (a download) is a task: `update` may return a command that the
runtime spawns, and the task's result comes back as a message
(`Update(model:, commands:)`; exact shape P20-4). Tasks never touch the model.

### Per-target availability

| Target family | Platform layer | Status |
|---------------|----------------|--------|
| Windows, macOS, Linux desktop | `std-os` with `sys-surface` per OS | planned for M9 |
| Browser (`wasm32-unknown-unknown`) | `std-web`, canvas or DOM | planned for M9 (Assumption: canvas first) |
| WASI | none: WASI has no surface | unavailable |
| Microcontrollers with a display | would need a `--no-heap` widget set | not in 0.1 |

### Ownership notes

- The model is moved into `run_app` and owned by it; `view` gets a borrow and
  `update` returns a new owned model, so no widget, task or handler ever holds
  a reference into the model.
- Widget trees are rebuilt by `view` and compared with the previous tree by
  the runtime (Assumption: a diffing renderer, as in Elm and React), so the
  program never mutates widgets in place.
- A `text_input`'s current text lives in the model, not in the widget: one
  owner for each piece of state.

### Shaking

A counter app reaches `run_app`, three widget kinds, layout for `column` and
`row`, text rendering, and the `surface` slice. Widgets that are never
constructed are absent (each widget kind is its own type behind
`Widget<Msg>`; Assumption: the renderer dispatches by trait type, so only
converted types × called slots are reached, 06 §6.4).

## 20.6 `iot`

**Purpose.** GPIO, I2C, SPI, UART and timers on microcontrollers and
single-board computers, inside the embedded profile ([05 §5.7](runtime.md#57-embedded-profile)).

**Capabilities:** `gpio`, `i2c`, `spi`, `uart` (slices `sys-gpio`, `sys-i2c`,
`sys-spi`, `sys-uart`, 05 §5.1), and `clock` for delays and timers. Each
program reaches only the buses it uses. **Targets:** Cortex-M, ESP32, RISC-V
boards (04 §4.2); Linux boards through `std-os` where the target file grants
`gpio` (Assumption); not desktops without peripherals, not the browser.

### Types (sketch)

```verse
import Instant from time

type Level = High | Low

type Edge
    export rising: Bool               # false for a falling edge
    export at: Instant                # time.Instant
```

`Board`, `Pin`, `I2c`, `Spi` and `Uart` are opaque `Cleanup` resources: when
one goes out of scope the peripheral is released (pin back to input, bus
disabled).

### Functions (sketch)

| Signature | Notes |
|-----------|-------|
| `fn Board.take(): Board \| Error` | once per program; a second call is an error value |
| `fn output_pin(mut self, number: Int): Pin \| Error` | `Board` method; a pin already taken is an error naming its first owner |
| `fn input_pin(mut self, number: Int): Pin \| Error` | |
| `fn set(mut self, level: Level)`, `fn toggle(mut self)`, `fn read(self): Level` | `Pin` methods; no heap |
| `fn edges(mut self, capacity: Int): Receiver<Edge> \| Error` | interrupts arrive as channel messages |
| `fn i2c(mut self, bus: Int, speed_hz: Int): I2c \| Error` | |
| `fn read_register(mut self, address: Int, register: Int): Int \| Error` | `I2c`; one byte, no buffer |
| `fn write_bytes(mut self, address: Int, data: Bytes): Nothing \| Error` | data from a `b"…"` literal or a byte view (spec §4.1) |
| `fn uart(mut self, port: Int, baud: Int): Uart \| Error` | also `spi(…)` |

### Example: blinky and a button

```verse
import Board, Edge from iot
import Receiver from task
import sleep from time

fn watch(take edges: Receiver<Edge>)
    for edge in edges
        if edge.rising
            print("pressed")

mut board = Board.take()?
with board.output_pin(13)? as mut led, board.input_pin(2)? as mut button
    spawn watch(button.edges(capacity: 4)?)
    while true
        led.toggle()
        sleep(500.millis())
```

On a board, `print` goes to the UART console if the target grants `stdio`.

### Interrupts map to tasks

An interrupt handler inside the runtime only records the event and wakes the
receiving task on the cooperative executor (05 §5.3). User code never runs in
interrupt context, so it can use the whole language, and "may not block"
(05 §5.7) is enforced by construction (Decision, agreed as P20-5; 05's "handlers written in Verse" is
still to align).

### No-heap notes

- `Pin`, `I2c`, `Uart`, `Level`, `Edge` and the register functions allocate
  nothing, so blinky passes `--no-heap` (05 §5.7) if the channel behind
  `edges` is placed in a static region sized by `capacity` (Assumption: needs
  build-time sizing; P20-6).
- The spec has **no fixed-capacity mutable byte buffer**: `Bytes` is
  immutable and `List<Int>` allocates. Bulk SPI and UART reads therefore need
  a new type (P20-6). Until then, `iot` offers byte-at-a-time reads and
  writes from literals and views.

### Shaking

Blinky reaches `Board.take`, `output_pin`, `toggle`, `sleep`, the `gpio` and
`clock` slices, and `core`. The I2C, SPI and UART code and slices are absent.
The Target in [09](performance.md) (blinky under 16 KB flash on Cortex-M)
applies to this program.

## 20.7 Proposals and rulings

| # | Proposal | Touches | Ruling |
|---|----------|---------|----------------|
| P20-1 | `finance` offers only `Calendar.named(text)`; no one-function-per-calendar form | one obvious name (§1.4), 06 | ruled: keep only `named`; measure table sizes in M9 |
| P20-2 | `Money` does not implement `Add`/`Sub`; mixing currencies is an error value from `add`/`sub` | spec §10 operator traits | agreed |
| P20-3 | `ml` model loading: a `generator` that turns a model file into Verse code (only its operators are reached), plus the run-time `load_model` for dynamic models | spec §16.6, 06 | agreed; the generated file is visible and committed like any generator output (spec §16.6); file format chosen in M9 |
| P20-4 | `gui` uses the message model (Elm architecture); long work runs as tasks whose results return as messages | `gui` API, spec §11 | agreed |
| P20-5 | `iot` interrupts only wake tasks; no user code runs in interrupt context | 05 §5.7 | agreed |
| P20-6 | A fixed-capacity mutable byte buffer type for no-heap I/O (in the standard library, since `iot` and `net` would both use it), and build-time sizing of channels on `--no-heap` targets | spec §4.1, §15.2, 05 §5.7 | 0.2 item (spec §18.15): capacity visible where the buffer is made, writing past it returns an `Error`, one name in the standard library; channel sizing still open |
| P20-7 | `web` templates are generated code from a first-party generator package, not a run-time template language | spec §16.6, §1.4 | agreed |
| P20-8 | `web` routes receive shared state through `Router.with_state(Guarded<T>)` | `web` API | agreed |

## 20.8 Conflicts with other documents

1. [05 §5.7](runtime.md#57-embedded-profile) registers interrupt handlers
   "through the `gpio` or `timer` capability", but neither [05 §5.1](runtime.md#51-feature-slices)
   nor §5.6 has a `timer` capability or slice. This document uses `clock` for
   timers.
2. The spec gives `http.serve` and `Response.body` no signatures (§15.2),
   so every `web` example depends on an assumed `serve(port:, handler:)`.
