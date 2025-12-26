use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::result::Result;
use std::path::Path;
use zilog_z80::cpu::CPU;
use crate::constantes::RAM_LEN_MAX;
use crate::cpu_exec::CpuRunState;

/// gestiona snapshots  .z80 versión 1 (ZX Spectrum 48K)
pub struct Z80Snapshot {
    // Registros
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,

    pub af_: u16,
    pub bc_: u16,
    pub de_: u16,
    pub hl_: u16,

    pub ix: u16,
    pub iy: u16,

    pub sp: u16,
    pub pc: u16,

    pub i: u8,
    pub r: u8,

    pub iff1: bool,
    pub iff2: bool,
    pub im: u8,

    pub border: u8,

    /// RAM 48K (0x4000–0xFFFF)
    pub ram: Vec<u8>,
}

impl Z80Snapshot {
    pub fn load(path: &Path) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).map_err(|e| e.to_string())?;

        if data.len() < 30 {
            //panic!("Fichero .z80 demasiado pequeño");
            return Err("Fichero .z80 demasiado pequeño".into());
        }

        // -------------------------
        // HEADER (30 bytes)
        // -------------------------
        let a = data[0];
        let f = data[1];
        let af = ((a as u16) << 8) | (f as u16);

        let bc = u16::from_le_bytes([data[2], data[3]]);
        let hl = u16::from_le_bytes([data[4], data[5]]);
        let pc = u16::from_le_bytes([data[6], data[7]]);
        let sp = u16::from_le_bytes([data[8], data[9]]);
        let i = data[10];
        let r = data[11];

        let flags = data[12];
        let border = flags & 0x07;
        let compressed = (flags & 0x20) != 0;

        let de = u16::from_le_bytes([data[13], data[14]]);
        let bc_ = u16::from_le_bytes([data[15], data[16]]);
        let de_ = u16::from_le_bytes([data[17], data[18]]);
        let hl_ = u16::from_le_bytes([data[19], data[20]]);

        let a_ = data[21];
        let f_ = data[22];
        let af_ = ((a_ as u16) << 8) | (f_ as u16);

        let iy = u16::from_le_bytes([data[23], data[24]]);
        let ix = u16::from_le_bytes([data[25], data[26]]);
        let iff1 = data[27] != 0;
        let iff2 = data[28] != 0;
        let im = data[29] & 0x03;

        if pc == 0 {  // Es version 2 o 3
            let (pos, real_pc) = read_extended_header(&data)?;

            let pages = read_ram_blocks(&data, pos)?;

            return Ok(Z80SnapshotV23 {
                af,
                bc,
                de,
                hl,
                af_,
                bc_,
                de_,
                hl_,
                ix,
                iy,
                sp,
                pc: real_pc,
                i,
                r,
                iff1,
                iff2,
                im,
                border,
                //compressed: true,
                ram_pages: pages,
            }.into());
        }

        // -------------------------
        // RAM
        // -------------------------
        let mut ram = Vec::with_capacity(48 * 1024);
        let mut pos = 30;

        if !compressed {
            if data.len() < pos + RAM_LEN_MAX {
                return Err("RAM incompleta en snapshot v1".into());
            }
            ram.extend_from_slice(&data[pos..pos + RAM_LEN_MAX]);
        } else {
            while ram.len() < RAM_LEN_MAX {
                let b = data[pos];
                pos += 1;

                if b == 0xED && data[pos] == 0xED {
                    pos += 1;
                    let count = data[pos] as usize;
                    pos += 1;
                    let value = data[pos];
                    pos += 1;

                    for _ in 0..count {
                        ram.push(value);
                    }
                } else {
                    ram.push(b);
                }
            }
        }

        Ok(Self {
            af: ((a as u16) << 8) | (f as u16),
            bc,
            de,
            hl,

            af_: ((a_ as u16) << 8) | (f_ as u16),
            bc_,
            de_,
            hl_,

            ix,
            iy,

            sp,
            pc,

            i,
            r,

            iff1,
            iff2,
            im,

            border,
            ram,
        })
    }
}

/// Aplica un snapshot .z80 v1 a la CPU
pub fn apply_z80(cpu: &mut CPU, run_state: &mut CpuRunState, snap: &Z80Snapshot) {
    // -------------------------
    // RAM 0x4000–0xFFFF
    // -------------------------
    for (i, b) in snap.ram.iter().enumerate() {
        cpu.bus.write_byte(0x4000 + i as u16, *b);
    }

    // -------------------------
    // Registros principales
    // -------------------------
    cpu.reg.set_af(snap.af);
    cpu.reg.set_bc(snap.bc);
    cpu.reg.set_de(snap.de);
    cpu.reg.set_hl(snap.hl);

    cpu.alt.set_af(snap.af_);
    cpu.alt.set_bc(snap.bc_);
    cpu.alt.set_de(snap.de_);
    cpu.alt.set_hl(snap.hl_);

    cpu.reg.ixh = (snap.ix >> 8) as u8;
    cpu.reg.ixl = (snap.ix & 0xFF) as u8;
    cpu.reg.iyh = (snap.iy >> 8) as u8;
    cpu.reg.iyl = (snap.iy & 0xFF) as u8;

    cpu.reg.sp = snap.sp;
    cpu.reg.pc = snap.pc;

    cpu.reg.i = snap.i;
    cpu.reg.r = snap.r;

    // -------------------------
    // Interrupciones
    // -------------------------
    run_state.iff1 = snap.iff1;
    run_state.iff1_pending = false;
    run_state.im = snap.im;
    run_state.halted = false;
    run_state.allow_interrupts = true;
    run_state.t_states = 0;

    // -------------------------
    // Border (opcional)
    // -------------------------
    //cpu.bus.border = snap.border;
}

/// Snapshot .z80 versión 2 o 3 (48K por ahora)
pub struct Z80SnapshotV23 {
    // ======================
    // Registros CPU (igual que v1)
    // ======================
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,

    pub af_: u16,
    pub bc_: u16,
    pub de_: u16,
    pub hl_: u16,

    pub ix: u16,
    pub iy: u16,

    pub sp: u16,
    pub pc: u16, // ⬅️ PC REAL (desde cabecera extendida)

    pub i: u8,
    pub r: u8,

    pub iff1: bool,
    pub iff2: bool,
    pub im: u8,

    pub border: u8,

    // // ======================
    // // Info extendida
    // // ======================
    // pub machine_type: u8,   // 48K / 128K / etc
    //pub compressed: bool,
    //
    // // ======================
    // // RAM por páginas
    // // ======================
    // /// (page, data)
    // pub pages: Vec<(u8, Vec<u8>)>,
    // RAM por páginas
    pub ram_pages: HashMap<u8, Vec<u8>>,

}
impl From<Z80SnapshotV23> for Z80Snapshot {
    fn from(v: Z80SnapshotV23) -> Self {
        let mut ram = vec![0u8; 48 * 1024];

        // Page 4 → 0x8000
        if let Some(p) = v.ram_pages.get(&4) {
            ram[0x0000..0x4000].copy_from_slice(p);
        }

        // Page 5 → 0xC000
        if let Some(p) = v.ram_pages.get(&5) {
            ram[0x4000..0x8000].copy_from_slice(p);
        }

        // Page 8 → 0x4000
        if let Some(p) = v.ram_pages.get(&8) {
            ram[0x8000..0xC000].copy_from_slice(p);
        }

        Z80Snapshot {
            af: v.af,
            bc: v.bc,
            de: v.de,
            hl: v.hl,

            af_: v.af_,
            bc_: v.bc_,
            de_: v.de_,
            hl_: v.hl_,

            ix: v.ix,
            iy: v.iy,
            sp: v.sp,
            pc: v.pc,

            i: v.i,
            r: v.r,
            iff1: v.iff1,
            iff2: v.iff2,
            im: v.im,

            border: v.border,
            ram,
        }
    }
}

/*pub enum Z80AnySnapshot {
    V1(Z80Snapshot),
    V23(Z80SnapshotV23),
}*/

fn read_extended_header(data: &[u8]) -> Result<(usize, u16), String> {
    if data.len() < 32 {
        return Err("Fichero .z80 demasiado pequeño".into());
    }

    // data empieza en offset 30
    let header_len = u16::from_le_bytes([data[30], data[31]]) as usize;

    //let ext_start = 32;
    //let ext_end = ext_start + header_len;

    if data.len() < 32 + header_len {
        return Err("Cabecera extendida incompleta".into());
    }

    // PC REAL está en los primeros 2 bytes de la cabecera extendida
    let pc = u16::from_le_bytes([data[32], data[33]]);

    Ok((32 + header_len, pc))
}

/*fn read_memory_blocks(
    data: &[u8],
    mut pos: usize,
) -> Result<Vec<(u8, Vec<u8>)>, String> {
    let mut pages = Vec::new();

    while pos < data.len() {
        let size = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        let page = data[pos];
        pos += 1;

        let mut block = Vec::new();

        if size == 0xFFFF {
            // Bloque sin comprimir: 16 KB exactos
            block.extend_from_slice(&data[pos..pos + 0x4000]);
            pos += 0x4000;
        } else {
            // Bloque comprimido (RLE)
            let end = pos + size;
            while pos < end {
                let b = data[pos];
                pos += 1;

                if b == 0xED && data[pos] == 0xED {
                    pos += 1;
                    let count = data[pos] as usize;
                    pos += 1;
                    let value = data[pos];
                    pos += 1;

                    for _ in 0..count {
                        block.push(value);
                    }
                } else {
                    block.push(b);
                }
            }
        }

        pages.push((page, block));
    }

    Ok(pages)
}*/

/*fn load_v23(data: &[u8]) -> Result<Z80SnapshotV23, String> {
    let (ram_start, pc) = read_extended_header(data)?;

    // if machine_type != 0 {
    //     return Err("Solo ZX Spectrum 48K soportado por ahora".into());
    // }

    let ram_pages = read_ram_blocks(&data, ram_start)?;

    // Reusar parsing de registros del header base (bytes 0..30)
    // (idéntico a v1 excepto PC)

    let a = data[0];
    let f = data[1];
    let bc = u16::from_le_bytes([data[2], data[3]]);
    let hl = u16::from_le_bytes([data[4], data[5]]);
    let pc = u16::from_le_bytes([data[6], data[7]]);
    let sp = u16::from_le_bytes([data[8], data[9]]);
    let i = data[10];
    let r = data[11];

    let flags = data[12];
    let border = flags & 0x07;

    let de = u16::from_le_bytes([data[13], data[14]]);
    let bc_ = u16::from_le_bytes([data[15], data[16]]);
    let de_ = u16::from_le_bytes([data[17], data[18]]);
    let hl_ = u16::from_le_bytes([data[19], data[20]]);
    let a_ = data[21];
    let f_ = data[22];

    let iy = u16::from_le_bytes([data[23], data[24]]);
    let ix = u16::from_le_bytes([data[25], data[26]]);

    let iff1 = data[27] != 0;
    let iff2 = data[28] != 0;
    let im = data[29] & 0x03;

    // ✅ AQUÍ se crean los registros compuestos
    let af = ((a as u16) << 8) | f as u16;
    let af_ = ((a_ as u16) << 8) | f_ as u16;

    Ok(Z80SnapshotV23 {
        af,
        bc,
        de,
        hl,

        af_,
        bc_,
        de_,
        hl_,

        ix,
        iy,

        sp,
        pc,

        i,
        r,

        iff1,
        iff2,
        im,

        border,

        //machine_type,
        //compressed: true, // en v2/v3 SIEMPRE hay bloques

        ram_pages,
    })
}*/

fn read_ram_blocks(
    data: &[u8],
    mut pos: usize,
) -> Result<HashMap<u8, Vec<u8>>, String> {
    use std::collections::HashMap;

    let mut pages = HashMap::new();

    while pos + 3 <= data.len() {
        let len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        let page = data[pos + 2];
        pos += 3;

        let mut buf = Vec::with_capacity(16 * 1024);

        if len == 0xFFFF {
            buf.extend_from_slice(&data[pos..pos + 16384]);
            pos += 16384;
        } else {
            let end = pos + len;
            while pos < end {
                let b = data[pos];
                pos += 1;

                if b == 0xED && data[pos] == 0xED {
                    pos += 1;
                    let count = data[pos] as usize;
                    pos += 1;
                    let value = data[pos];
                    pos += 1;

                    buf.extend(std::iter::repeat(value).take(count));
                } else {
                    buf.push(b);
                }
            }
        }

        if buf.len() != 16 * 1024 {
            return Err(format!("Página {} tamaño inválido: {}", page, buf.len()));
        }
        pages.insert(page, buf);
    }

    Ok(pages)
}

pub fn apply_z80_v23(
    cpu: &mut CPU,
    run_state: &mut CpuRunState,
    snap: &Z80SnapshotV23,
) {
    for (page, data) in &snap.ram_pages {
        let base = match page {
            8 => 0x4000,
            4 => 0x8000,
            5 => 0xC000,
            _ => continue, // ignorar otras
        };

        for (i, b) in data.iter().enumerate() {
            cpu.bus.write_byte(base + i as u16, *b);
        }
    }

    // registros (idéntico a v1)
    cpu.reg.set_af(snap.af);
    cpu.reg.set_bc(snap.bc);
    cpu.reg.set_de(snap.de);
    cpu.reg.set_hl(snap.hl);

    cpu.alt.set_af(snap.af_);
    cpu.alt.set_bc(snap.bc_);
    cpu.alt.set_de(snap.de_);
    cpu.alt.set_hl(snap.hl_);

    cpu.reg.ixh = (snap.ix >> 8) as u8;
    cpu.reg.ixl = (snap.ix & 0xFF) as u8;
    cpu.reg.iyh = (snap.iy >> 8) as u8;
    cpu.reg.iyl = (snap.iy & 0xFF) as u8;

    cpu.reg.sp = snap.sp;
    cpu.reg.pc = snap.pc;

    cpu.reg.i = snap.i;
    cpu.reg.r = snap.r;

    run_state.iff1 = snap.iff1;
    run_state.iff1_pending = false;
    run_state.im = snap.im;
    run_state.halted = false;
    run_state.allow_interrupts = true;
    run_state.t_states = 0;
}

