use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;
use zilog_z80::cpu::CPU;
use crate::constantes::RAM_LEN_MAX;
use crate::cpu_exec::CpuRunState;

/// Snapshot Z80 versión 1 (ZX Spectrum 48K)
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

    // RAM 48K
    pub ram: Vec<u8>,
}

impl Z80Snapshot {
    /// Carga un fichero .z80 versión 1 (48K)
    pub fn load(path: &Path) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if data.len() < 30 {
            panic!("Fichero .z80 demasiado pequeño");
        }

        // -------------------------
        // HEADER (30 bytes)
        // -------------------------
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
        let compressed = (flags & 0x20) != 0;

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

        // Solo versión 1
        if pc == 0 {
            panic!("Formato .z80 v2/v3 no soportado todavía");
        }

        // -------------------------
        // RAM
        // -------------------------
        let mut ram = Vec::with_capacity(48 * 1024);
        let mut pos = 30;

        if !compressed {
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
