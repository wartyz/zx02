use zilog_z80::cpu::CPU;
use std::collections::{HashMap, HashSet};
use crate::stack_tracker::{StackTracker, StackWriteKind};

pub static mut IM1_COUNT: u64 = 0;

/* ==================================================
 * SNAPSHOT DE CPU
 * ================================================== */

#[derive(Clone, Debug)]
pub struct CpuSnapshot {
    // Registros principales
    pub pc: u16,
    pub sp: u16,

    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,

    // Registros alternativos
    pub af_: u16,
    pub bc_: u16,
    pub de_: u16,
    pub hl_: u16,

    // Registros de interrupción
    pub i: u8,
    pub r: u8,

    // Registro F separado
    pub f: u8,

    // Byte actual
    pub mem_addr: u16,
    pub mem_value: u8,

    // Dump de memoria
    pub mem_base: u16,
    pub mem_dump: Vec<u8>,

    // Stack
    pub stack_base: u16,
    pub stack_dump: Vec<u8>,

    // Longitud de la instrucción actual (1–4 bytes)
    pub instr_len: u8,
    // ciclos gastados por la instrucción
    pub instr_cycles: u32,

}
/* ==================================================
 * ESTADO DE LA CPU
 * ================================================== */
pub struct CpuRunState {
    pub halted: bool,       // Interrupciones
    pub iff1: bool,         // Interrupciones habilitadas
    pub iff1_pending: bool, // para ejecutar en la siguiente instrucción
    pub iff1_delay: u8,     // para modificar iff1  mas tarde
    pub im: u8,             // Modo de interrupción (0,1,2)
    pub t_states: u64,
}
impl CpuRunState {
    pub fn new() -> Self {
        Self {
            halted: false,
            iff1: false,   // en Spectrum tras ROM: habilitadas
            iff1_pending: false,
            iff1_delay: 0,
            im: 1,        // Spectrum usa IM 1
            t_states: 0,
        }
    }
}

/* ==================================================
 * CPU INIT
 * ================================================== */

pub fn init_cpu(rom_path: &str) -> CPU {
    let mut cpu = CPU::new(0xFFFF);
    cpu.bus.load_bin(rom_path, 0).unwrap();
    cpu
}

/* ==================================================
 * TRACKER DE INSTRUCCIONES NO IMPLEMENTADAS
 * ================================================== */

pub struct UnimplTracker {
    seen: HashSet<u16>,
}

impl UnimplTracker {
    pub fn new() -> Self {
        Self {
            seen: HashSet::new(),
        }
    }

    pub fn report(&mut self, pc: u16, bytes: &[u8], mnemonic: &str) {
        if self.seen.insert(pc) {
            print!("⚠️ UNIMPL PC={:04X}: ", pc);
            for b in bytes {
                print!("{:02X} ", b);
            }
            println!("=> {}", mnemonic);
        }
    }
}

/* ==================================================
 * STEP (UNA INSTRUCCIÓN)
 * ================================================== */

pub fn step(
    cpu: &mut CPU,
    run_state: &mut CpuRunState,
    interrupt_pending: bool,
    executed: &mut HashMap<u16, (u8, String)>,
    unimpl: &mut UnimplTracker,
    stack_tracker: &mut StackTracker,
) -> CpuSnapshot {
    //dbg!(cpu.reg.pc);

    // -----------------------------------------------
    // Estado previo
    // -----------------------------------------------
    let pc_before = cpu.reg.pc;
    let sp_before = cpu.reg.sp;

    /*if interrupt_pending {
        println!(
            "INT pendiente: PC={:04X} iff1={}",
            cpu.reg.pc,
            run_state.iff1
        );
    }*/

    if interrupt_pending && run_state.iff1 {
        if cpu.reg.pc == 0x0038 {
            dbg!("EJECUTANDO 0038");
        }
    }

    // -----------------------------------------------
    // Activación diferida de EI (Z80 REAL)
    // -----------------------------------------------
    if run_state.iff1_pending {
        run_state.iff1 = true;
        run_state.iff1_pending = false;
    }

    // ------------------------------------------------
    // INTERRUPCIÓN (IM 1)    Tiene PRIORIDAD ABSOLUTA
    // ------------------------------------------------
    /*if interrupt_pending && run_state.iff1 {
        //dbg!(("INT", cpu.reg.pc));
        // Si estaba en HALT, despertamos
        run_state.halted = false;

        // En IM 1 el Z80 deshabilita interrupciones
        run_state.iff1 = false;

        match run_state.im {
            1 => {
                // IM 1 = RST 38h
                let pc = cpu.reg.pc;

                // PUSH PC (little endian)
                let sp = cpu.reg.sp.wrapping_sub(2);
                cpu.reg.sp = sp;
                cpu.bus.write_byte(sp, (pc & 0x00FF) as u8);
                cpu.bus.write_byte(sp.wrapping_add(1), (pc >> 8) as u8);

                println!("IM1 -> ANTES de entrar en 0038 desde {:04X}", pc);

                // Saltar a 0038h
                cpu.reg.pc = 0x0038;

                println!("IM1 -> DESPUES de entrar en 0038 desde {:04X}", pc);

                // Coste real: ~13 T-states
                return snapshot(cpu, pc, 0, 13);
            }
            _ => {
                // Otros modos todavía no
            }
        }
    }

    // -----------------------------------------------
    // SI LA CPU ESTÁ EN HALT
    // -----------------------------------------------
    if run_state.halted {
        dbg!("CPU EN HALT");
        // El Z80 sigue consumiendo tiempo en HALT
        run_state.t_states += 4;

        return snapshot(cpu, pc_before, 0, 4);
    }*/

    // -----------------------------------------------
    // Leer hasta 4 bytes de instrucción
    // -----------------------------------------------
    let mut instr_bytes = [0u8; 4];
    for i in 0..4 {
        instr_bytes[i] = cpu.bus.read_byte(pc_before.wrapping_add(i as u16));
    }
    let opcode = instr_bytes[0];

    // -----------------------------------------------
    // Desensamblado
    // -----------------------------------------------
    let (mnemonic, instr_len) =
        crate::disasm::disassemble(&instr_bytes, pc_before, pc_before);

    // Detectar instrucciones no implementadas
    if mnemonic.starts_with("UNIMPL") {
        let len = instr_len as usize;
        unimpl.report(pc_before, &instr_bytes[..len.min(4)], &mnemonic);
    }

    // -----------------------------------------------
    // Ejecutar instrucción
    // -----------------------------------------------
    let instr_cycles = cpu.execute();
    //let cursor = cpu.bus.read_byte(0x5C5C);
    //dbg!(cursor);
    /// -----------------------------------------------
    // RETI / RETN restauran interrupciones (Z80 REAL)
    // -----------------------------------------------
    if opcode == 0xED {
        let next = cpu.bus.read_byte(pc_before.wrapping_add(1));
        if next == 0x4D || next == 0x45 {
            run_state.iff1 = true;
        }
    }

    // -----------------------------------------------
    // EI / DI (Z80 REAL)
    // -----------------------------------------------
    match opcode {
        0xFB => {
            // EI → se activa DESPUÉS de la siguiente instrucción
            run_state.iff1_pending = true;
            run_state.iff1_delay = 1;
        }
        0xF3 => {
            // DI → inmediato
            run_state.iff1 = false;
            run_state.iff1_pending = false;
        }
        _ => {}
    }

    // -----------------------------------------------
    // HALT: entrar en estado detenido
    // -----------------------------------------------
    if opcode == 0x76 {
        // El Z80 queda detenido hasta una interrupción
        run_state.halted = true;
    }

    // -----------------------------------------------
    // Tracking de escrituras en el stack
    // -----------------------------------------------
    let sp_after = cpu.reg.sp;

    if sp_after < sp_before {
        // Número de bytes escritos
        let written = sp_before.wrapping_sub(sp_after);
        let kind = classify_stack_write(&mnemonic);

        for i in 0..written {
            let addr = sp_after.wrapping_add(i);
            stack_tracker.record(addr, kind, pc_before);
        }
    }

    // Actualiza los t_states
    run_state.t_states += instr_cycles as u64;

    // -----------------------------------------------
    // Registrar instrucción ejecutada
    // -----------------------------------------------
    executed.insert(pc_before, (instr_len, mnemonic));

    // -----------------------------------------------
    // Snapshot final
    // -----------------------------------------------
    snapshot(cpu, pc_before, instr_len, instr_cycles)
}

fn classify_stack_write(mnemonic: &str) -> StackWriteKind {
    if mnemonic.starts_with("CALL") || mnemonic.starts_with("RST") {
        StackWriteKind::Call
    } else if mnemonic.starts_with("PUSH") {
        StackWriteKind::Push
    } else {
        StackWriteKind::Manual
    }
}

/* ==================================================
 * SNAPSHOT (USADO TRAS EJECUTAR O EN BREAK)
 * ================================================== */

pub(crate) fn snapshot(
    cpu: &CPU,
    pc: u16,
    instr_len: u8,
    instr_cycles: u32,
) -> CpuSnapshot {
    let mem_value = cpu.bus.read_byte(pc);

    // Dump de 256 bytes alrededor del PC
    let mem_base = pc.wrapping_sub(0x80);
    let mut mem_dump = Vec::with_capacity(256);
    for i in 0..256 {
        mem_dump.push(cpu.bus.read_byte(mem_base.wrapping_add(i)));
    }

    // Stack
    const STACK_BYTES: usize = 30;

    let sp = cpu.reg.sp;
    let stack_base = sp;

    let mut stack_dump = Vec::with_capacity(STACK_BYTES);
    for i in 0..STACK_BYTES {
        stack_dump.push(cpu.bus.read_byte(sp.wrapping_add(i as u16)));
    }

    CpuSnapshot {
        pc,
        sp: cpu.reg.sp,

        af: cpu.reg.get_af(),
        bc: cpu.reg.get_bc(),
        de: cpu.reg.get_de(),
        hl: cpu.reg.get_hl(),

        // Registros alternativos
        af_: cpu.alt.get_af(),
        bc_: cpu.alt.get_bc(),
        de_: cpu.alt.get_de(),
        hl_: cpu.alt.get_hl(),

        // Interrupciones
        i: cpu.reg.i,
        r: cpu.reg.r,

        mem_base,
        mem_dump,
        f: (cpu.reg.get_af() & 0x00FF) as u8,

        mem_addr: pc,
        mem_value,

        stack_base,
        stack_dump,

        instr_len,
        instr_cycles,

    }
}

