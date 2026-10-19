//! A prototype of verse-rt's panic path and drop drain on a `status-code`
//! Cortex-M target, for measuring their worst-case stack in an emulator.
#![no_std]
#![no_main]

use core::fmt::{self, Write};

const UART_DR: *mut u32 = 0x4000_C000 as *mut u32;

struct Uart;
impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            unsafe { core::ptr::write_volatile(UART_DR, b as u32) };
        }
        Ok(())
    }
}

#[repr(C)]
pub struct PanicRecord {
    kind: u8,
    site: u32,
    task: u32,
    vals: [i64; 2],
}

struct Site {
    func: u16,
    file: u16,
    line: u32,
    col: u32,
    len: u32,
    op: u8,
}

static NAMES: [&str; 3] = ["sum_to", "<script>", "best"];
static FILES: [&str; 2] = ["sum.vs", "scores.vs"];
static LINES: [&str; 3] = ["    n + sum_to(n - 1)", "print(sum_to(100000))", "    scores[i]"];
static SITES: [Site; 3] = [
    Site { func: 0, file: 0, line: 4, col: 9, len: 13, op: 0 },
    Site { func: 1, file: 0, line: 6, col: 7, len: 14, op: 1 },
    Site { func: 2, file: 1, line: 18, col: 5, len: 9, op: 2 },
];

// The call sites the walk finds, innermost first: the site words of the
// frames. A static array stands for the frames; reading it costs no stack.
#[unsafe(no_mangle)]
static mut WALK: [u16; 4096] = [0; 4096];
#[unsafe(no_mangle)]
static mut WALK_LEN: u32 = 0;

fn walk(i: usize) -> usize {
    unsafe { (*core::ptr::addr_of!(WALK))[i] as usize }
}

fn same_block(i: usize, k: usize, n: usize) -> bool {
    i + 2 * k <= n && (0..k).all(|j| walk(i + j) == walk(i + k + j))
}

fn line_of(w: &mut Uart, s: usize) -> fmt::Result {
    let site = &SITES[s];
    writeln!(w, "  in {} at {}:{}", NAMES[site.func as usize], FILES[site.file as usize], site.line)
}

/// Prints the folded stack lines with O(1) memory: at each position, the
/// shortest block of 1 to 8 lines that repeats right after itself is printed
/// three times and the rest becomes one line.
fn stack_lines(w: &mut Uart, n: usize) -> fmt::Result {
    let mut i = 0;
    while i < n {
        let mut folded = false;
        for k in 1..=8 {
            if !same_block(i, k, n) {
                continue;
            }
            let mut reps = 1;
            while same_block(i + (reps - 1) * k, k, n) {
                reps += 1;
            }
            if reps <= 3 {
                continue;
            }
            for _ in 0..3 {
                for j in 0..k {
                    line_of(w, walk(i + j))?;
                }
            }
            let r = reps - 3;
            let times = if r == 1 { "time" } else { "times" };
            if k == 1 {
                writeln!(w, "  [the call above repeats {} more {}]", r, times)?;
            } else {
                writeln!(w, "  [the {} calls above repeat {} more {}]", k, r, times)?;
            }
            i += reps * k;
            folded = true;
            break;
        }
        if !folded {
            line_of(w, walk(i))?;
            i += 1;
        }
    }
    Ok(())
}

fn message(w: &mut Uart, rec: &PanicRecord, site: &Site) -> fmt::Result {
    let [a, b] = rec.vals;
    let ops = ["+", "-", "*"];
    let f = NAMES[site.func as usize];
    match rec.kind {
        1 => write!(w, "{} {} {} overflows `Int`", a, ops[site.op as usize % 3], b),
        2 => write!(w, "cannot divide {} by zero", a),
        3 => write!(w, "index {} is out of range, the list has {} items", a, b),
        4 => write!(w, "assertion failed: `{}`", LINES[site.line as usize % 3].trim()),
        5 => write!(w, "`{}` is none, and the program expected a value", f),
        6 => write!(w, "the exponent {} is negative, and an `Int` power needs 0 or more", a),
        7 => write!(w, "`{}` cannot be converted to `Int32`", a),
        8 => write!(w, "task t{} ran out of stack in `{}`", rec.task, f),
        9 => write!(w, "the program ran out of memory"),
        _ => write!(w, "recursion is too deep in task t{}: {} calls in progress, the limit, in `{}`", rec.task, a, f),
    }
}

fn report(w: &mut Uart, rec: &PanicRecord) -> fmt::Result {
    let site = &SITES[rec.site as usize % SITES.len()];
    w.write_str("panic: ")?;
    message(w, rec, site)?;
    writeln!(w)?;
    writeln!(w, "  --> {}:{}:{}", FILES[site.file as usize], site.line, site.col)?;
    writeln!(w, "{:>2} | {}", site.line, LINES[site.func as usize])?;
    w.write_str("   | ")?;
    for _ in 1..site.col {
        w.write_char(' ')?;
    }
    for _ in 0..site.len {
        w.write_char('^')?;
    }
    writeln!(w, " this call")?;
    stack_lines(w, unsafe { WALK_LEN } as usize)?;
    let code = [0, 1, 3, 2, 6, 5, 11, 12, 4, 7, 4][rec.kind as usize % 11];
    writeln!(w, "explain: R{:04} (run `verse explain R{:04}`)", code, code)
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.rt_panic")]
pub extern "C" fn rt_panic(rec: &PanicRecord) -> u32 {
    let _ = report(&mut Uart, rec);
    1
}

// The drop drain: an entry holds a part, its glue and its next part index.
// Glue drops its inline parts in its own body and calls only `release` and
// `push`; it never calls another glue.
#[derive(Clone, Copy)]
pub struct Entry {
    part: *mut u32,
    glue: extern "C" fn(*mut u32, u32, &mut DropStack) -> u32,
    next: u32,
}

pub struct DropStack {
    len: usize,
    entries: [Entry; 8],
}

#[inline(never)]
extern "C" fn release(handle: u32) {
    unsafe { core::ptr::write_volatile(UART_DR, handle) };
}

/// Glue of a node `Node(file: File, left: Tree, right: Tree)`: releases the
/// inline handle, then asks the drain for its two heap parts in order.
extern "C" fn node_glue(p: *mut u32, next: u32, s: &mut DropStack) -> u32 {
    unsafe {
        if next == 0 {
            release(*p);
        }
        let child = *p.add(1 + next as usize) as *mut u32;
        if !child.is_null() && s.len < s.entries.len() {
            s.entries[s.len] = Entry { part: child, glue: node_glue, next: 0 };
            s.len += 1;
        }
        if next < 1 { next + 1 } else { u32::MAX }
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.drop_drain")]
pub extern "C" fn drop_drain(root: *mut u32, s: &mut DropStack) {
    s.entries[0] = Entry { part: root, glue: node_glue, next: 0 };
    s.len = 1;
    while s.len > 0 {
        let top = s.len - 1;
        let e = s.entries[top];
        let next = (e.glue)(e.part, e.next, s);
        if next == u32::MAX {
            s.entries.copy_within(top + 1..s.len, top);
            s.len -= 1;
        } else {
            s.entries[top].next = next;
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
