"""Run rt_panic and drop_drain of the panic-path prototype in a Cortex-M4
emulator on a painted stack and print the deepest stack each one used.

  rustup target add thumbv7em-none-eabihf
  (cd tools/measure/panic-path && cargo build --release)
  uv run --with unicorn --with pyelftools tools/measure/stack-cortex-m.py \
      target/thumbv7em-none-eabihf/release/panic-path
"""
import struct, sys
from elftools.elf.elffile import ELFFile
from unicorn import *
from unicorn.arm_const import *

RAM, RAM_SIZE, TOP = 0x20000000, 128 * 1024, 0x20020000
PAINT_LO, PAINT = 0x20010000, b'\xaa'
STOP = 0x08070000

def load(path):
    elf = ELFFile(open(path, 'rb'))
    mu = Uc(UC_ARCH_ARM, UC_MODE_THUMB | UC_MODE_MCLASS)
    mu.mem_map(0x08000000, 512 * 1024)
    mu.mem_map(RAM, RAM_SIZE)
    mu.mem_map(0x40000000, 0x10000)
    for seg in elf.iter_segments():
        if seg['p_type'] == 'PT_LOAD' and seg['p_filesz']:
            mu.mem_write(seg['p_paddr'], seg.data())
            if seg['p_vaddr'] != seg['p_paddr']:
                mu.mem_write(seg['p_vaddr'], seg.data())
    syms = {s.name: s['st_value'] for s in elf.get_section_by_name('.symtab').iter_symbols()}
    out = bytearray()
    mu.hook_add(UC_HOOK_MEM_WRITE, lambda uc, a, addr, size, v, u: out.append(v & 0xff), begin=0x4000C000, end=0x4000C003)
    return mu, syms, out

def run(mu, fn, args):
    mu.mem_write(PAINT_LO, PAINT * (TOP - PAINT_LO))
    for r, v in zip([UC_ARM_REG_R0, UC_ARM_REG_R1], args):
        mu.reg_write(r, v)
    mu.reg_write(UC_ARM_REG_SP, TOP)
    mu.reg_write(UC_ARM_REG_LR, STOP | 1)
    mu.emu_start(fn | 1, STOP, count=200_000_000)
    assert mu.reg_read(UC_ARM_REG_PC) == STOP, f'run did not return: pc={mu.reg_read(UC_ARM_REG_PC):#x}'
    mem = mu.mem_read(PAINT_LO, TOP - PAINT_LO)
    first = next(i for i, b in enumerate(mem) if b != 0xaa)
    return TOP - (PAINT_LO + first)

def main(path):
    mu, syms, out = load(path)
    rec = 0x20008000
    walk = syms['WALK']
    worst = (0, None)
    sequences = {
        'recursion 4000 folded': [0] * 3999 + [1],
        'block of 8 repeated': list(range(3)) * 2 + ([0, 1, 2, 0, 1, 2, 2, 1] * 400),
        'no repeats': [i % 3 for i in range(64)],
    }
    for name, seq in sequences.items():
        mu.mem_write(walk, struct.pack(f'<{len(seq)}H', *seq))
        mu.mem_write(syms['WALK_LEN'], struct.pack('<I', len(seq)))
        for kind in range(11):
            for a, b in [(-2**63, 2**63 - 1), (7, 3)]:
                for site in range(3):
                    mu.mem_write(rec, struct.pack('<BxxxIIxxxxqq', kind, site, 4294967295, a, b))
                    out.clear()
                    used = run(mu, syms['rt_panic'], [rec])
                    assert out.decode().rstrip().endswith('`)'), out.decode()[-200:]
                    if used > worst[0]:
                        worst = (used, f'{name}, kind {kind}, site {site}, a={a}')
    print(f'rt_panic: {worst[0]} bytes ({worst[1]})')
    print('sample report:')
    mu.mem_write(walk, struct.pack('<4000H', *([0] * 3999 + [1])))
    mu.mem_write(syms['WALK_LEN'], struct.pack('<I', 4000))
    mu.mem_write(rec, struct.pack('<BxxxIIxxxxqq', 10, 0, 0, 10000, 0))
    out.clear(); run(mu, syms['rt_panic'], [rec]); print(out.decode())
    # a 3-level tree of nodes, each [file handle, left, right]
    nodes = [0x20009000 + 12 * i for i in range(15)]
    for i, n in enumerate(nodes):
        l = nodes[2 * i + 1] if 2 * i + 1 < 15 else 0
        r = nodes[2 * i + 2] if 2 * i + 2 < 15 else 0
        mu.mem_write(n, struct.pack('<III', 100 + i, l, r))
    out.clear()
    used = run(mu, syms['drop_drain'], [nodes[0], 0x2000A000])
    print(f'drop_drain: {used} bytes, released {len(out)} handles')

main(sys.argv[1])
