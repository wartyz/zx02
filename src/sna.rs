use std::fs::File;
use std::io::{Read, Result};
use zilog_z80::cpu::CPU;
use crate::cpu_exec::CpuRunState;

/// Snapshot SNA de ZX Spectrum 48K
///
/// Formato:
/// - 27 bytes de cabecera
/// - 48 KB de RAM (0x4000–0xFFFF)
pub struct SnaSnapshot {
    // Registros principales
    pub i: u8,
    pub hl_: u16,
    pub de_: u16,
    pub bc_: u16,
    pub af_: u16,
    pub hl: u16,
    pub de: u16,
    pub bc: u16,
    pub iy: u16,
    pub ix: u16,
    pub iff2: bool,
    pub r: u8,
    pub af: u16,
    pub sp: u16,
    pub im: u8,

    // Memoria RAM completa (48 KB)
    pub ram: Vec<u8>,
}

impl SnaSnapshot {
    /// Carga un fichero .sna (48K)
    pub fn load(path: &str) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if data.len() != 49179 {
            panic!("Tamaño inválido de .sna (esperado 49179 bytes)");
        }

        // --- Cabecera ---
        let i = data[0];

        let hl_ = u16::from_le_bytes([data[1], data[2]]);
        let de_ = u16::from_le_bytes([data[3], data[4]]);
        let bc_ = u16::from_le_bytes([data[5], data[6]]);
        let af_ = u16::from_le_bytes([data[7], data[8]]);

        let hl = u16::from_le_bytes([data[9], data[10]]);
        let de = u16::from_le_bytes([data[11], data[12]]);
        let bc = u16::from_le_bytes([data[13], data[14]]);

        let iy = u16::from_le_bytes([data[15], data[16]]);
        let ix = u16::from_le_bytes([data[17], data[18]]);

        let iff2 = data[19] != 0;
        let r = data[20];

        let af = u16::from_le_bytes([data[21], data[22]]);
        let sp = u16::from_le_bytes([data[23], data[24]]);

        let im = data[25] & 0x03;

        // data[26] = border color (opcional, no crítico ahora)

        // --- RAM ---
        let ram = data[27..].to_vec();

        Ok(Self {
            i,
            hl_,
            de_,
            bc_,
            af_,
            hl,
            de,
            bc,
            iy,
            ix,
            iff2,
            r,
            af,
            sp,
            im,
            ram,
        })
    }
}

pub fn apply_sna(cpu: &mut CPU, run_state: &mut CpuRunState, sna: &SnaSnapshot) {
    // -------------------------
    // RAM (0x4000–0xFFFF)
    // -------------------------
    for (i, b) in sna.ram.iter().enumerate() {
        cpu.bus.write_byte(0x4000 + i as u16, *b);
    }

    // -------------------------
    // Registros principales
    // -------------------------
    cpu.reg.i = sna.i;
    cpu.reg.r = sna.r;

    cpu.reg.set_af(sna.af);
    cpu.reg.set_bc(sna.bc);
    cpu.reg.set_de(sna.de);
    cpu.reg.set_hl(sna.hl);

    cpu.alt.set_af(sna.af_);
    cpu.alt.set_bc(sna.bc_);
    cpu.alt.set_de(sna.de_);
    cpu.alt.set_hl(sna.hl_);

    cpu.reg.ixh = (sna.ix >> 8) as u8;
    cpu.reg.ixl = (sna.ix & 0xFF) as u8;
    cpu.reg.iyh = (sna.iy >> 8) as u8;
    cpu.reg.iyl = (sna.iy & 0xFF) as u8;

    // -------------------------
    // Stack Pointer
    // -------------------------
    cpu.reg.sp = sna.sp;

    // -------------------------
    // PC: se extrae del stack
    // (formato SNA real)
    // -------------------------
    let pcl = cpu.bus.read_byte(cpu.reg.sp);
    let pch = cpu.bus.read_byte(cpu.reg.sp.wrapping_add(1));
    cpu.reg.pc = ((pch as u16) << 8) | pcl as u16;
    cpu.reg.sp = cpu.reg.sp.wrapping_add(2);

    // -------------------------
    // Interrupciones
    // -------------------------
    run_state.iff1 = sna.iff2;
    run_state.iff1_pending = false;
    run_state.im = sna.im;
    run_state.halted = false;
    run_state.allow_interrupts = true;
    
    // -------------------------
    // Tiempo
    // -------------------------
    run_state.t_states = 0;
}
