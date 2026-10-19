//! Measures the stack that native unwinding needs on a hosted target: a panic
//! whose hook walks the stack (unresolved, like a trace-map lookup) and prints
//! a report, then unwinds through frames whose drops run, as cleanup blocks do.
//!
//!   cargo run --release --manifest-path tools/measure/unwind-path/Cargo.toml -- walk
//!
//! The argument picks the hook: `report` prints only the report, `walk` also
//! walks the stack without resolving it, `symbolize` resolves it with DWARF.
use std::backtrace::Backtrace;
use std::hint::black_box;
use std::io::Write;
use std::panic;

const PAINT: u8 = 0xaa;
const REGION: usize = 256 * 1024;

#[inline(never)]
fn paint() -> usize {
    let mut buf = [0u8; REGION];
    for b in buf.iter_mut() {
        unsafe { std::ptr::write_volatile(b, PAINT) };
    }
    black_box(&buf);
    buf.as_ptr() as usize
}

#[inline(never)]
fn used_below(top: usize, low: usize) -> usize {
    // Reads the dead stack below the caller; the paint marks untouched bytes.
    let mut addr = low;
    while addr < top && unsafe { std::ptr::read_volatile(addr as *const u8) } == PAINT {
        addr += 1;
    }
    top - addr
}

struct File(i64);
impl Drop for File {
    fn drop(&mut self) {
        black_box(self.0);
    }
}

#[inline(never)]
fn deep(n: u32, at: &mut usize) -> i64 {
    let f = File(n as i64);
    if n == 0 {
        let sp = 0u8;
        *at = &sp as *const u8 as usize;
        let v: Vec<i64> = black_box(vec![1, 2, 3]);
        return v[black_box(7)] + f.0;
    }
    deep(n - 1, at) + f.0
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    panic::set_hook(Box::new(move |info| {
        let frames = match mode.as_str() {
            "walk" => {
                let mut n = 0;
                backtrace::trace(|f| {
                    black_box(f.ip());
                    n += 1;
                    true
                });
                n
            }
            "symbolize" => format!("{:?}", Backtrace::force_capture()).matches("fn:").count(),
            _ => 0,
        };
        let mut err = std::io::stderr().lock();
        let _ = writeln!(err, "panic: {}\n  --> {}\n  ({} frames walked)", info.payload_as_str().unwrap_or(""), info.location().unwrap(), frames);
    }));
    let worst = std::thread::Builder::new()
        .stack_size(4 << 20)
        .spawn(|| {
            let mut worst = 0;
            for _ in 0..5 {
                let low = paint();
                let mut at = 0usize;
                let r = panic::catch_unwind(panic::AssertUnwindSafe(|| deep(black_box(20), &mut at)));
                assert!(r.is_err(), "the index must panic");
                let used = used_below(at, low);
                worst = worst.max(used);
            }
            worst
        })
        .unwrap()
        .join()
        .unwrap();
    println!("hosted panic path below the panicking frame: {worst} bytes");
}
