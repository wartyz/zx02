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
    pub halted: bool,           // Interrupciones
    pub iff1: bool,             // Interrupciones habilitadas
    pub iff1_pending: bool,     // para ejecutar en la siguiente instrucción
    pub iff1_delay: u8,         // para modificar iff1  mas tarde
    pub im: u8,                 // Modo de interrupción (0,1,2)
    pub t_states: u64,
    pub allow_interrupts: bool, // se permiten interrupciones
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
            allow_interrupts: true,
        }
    }
}

/* ==================================================
 * CPU INIT
 * ================================================== */
const DIR_CARGA_ROM: u16 = 0x0000;
const DIR_CARGA_TEST: u16 = 0x8000;
pub fn init_cpu(rom_path: &str) -> CPU {
    let mut cpu = CPU::new(0xFFFF);
    cpu.bus.load_bin(rom_path, DIR_CARGA_ROM).unwrap();

    cpu.reg.sp = 0xFFFF;
    cpu.reg.pc = DIR_CARGA_ROM;

    // ⭐⭐ INICIALIZAR VARIABLES DEL SISTEMA ⭐⭐
    // FLAGS
    cpu.bus.write_byte(0x5C08, 0x00);

    // CURCHL - Dirección del cursor (EDIT/INPUT)
    cpu.bus.write_byte(0x5C5C, 0x00);  // LOW
    cpu.bus.write_byte(0x5C5D, 0x40);  // HIGH (apunta a 0x4000)

    // COORDS - Coordenadas del cursor (0,0)
    cpu.bus.write_byte(0x5C3C, 0x00);  // X
    cpu.bus.write_byte(0x5C3D, 0x00);  // Y

    // S_POSN - Posición en pantalla (1,1)
    cpu.bus.write_byte(0x5C3E, 0x01);  // linea
    cpu.bus.write_byte(0x5C3F, 0x01);  // columna

    // MODE - Modo de entrada (K/L/C/E/etc)
    cpu.bus.write_byte(0x5C3A, 0x00);

    // ERR_SP - Stack de error (apilado)
    let sp = 0xFFFE;
    cpu.reg.sp = sp;
    cpu.bus.write_byte(sp, 0x00);
    cpu.bus.write_byte(sp + 1, 0x00);

    cpu.reg.i = 0x00;
    cpu.reg.r = 0x00;
    cpu
}

// para poder cargar la ROM y un programa de pruebas a la vez
pub fn init_cpu_with_test(rom_path: &str, test_path: &str) -> CPU {
    let mut cpu = CPU::new(0xFFFF);

    // 1. Cargar ROM oficial del Spectrum (0x0000-0x3FFF)
    cpu.bus.load_bin(rom_path, DIR_CARGA_ROM).unwrap();

    // 2. Cargar el programa de test (0x8000-0x8FFF aprox)
    cpu.bus.load_bin(test_path, DIR_CARGA_TEST).unwrap();

    // 3. Configurar PC para ejecutar el programa
    cpu.reg.pc = 0x8000;  // Ejecutar desde el test

    // 4. Inicializar registros
    cpu.reg.sp = 0xFFFF;
    cpu.reg.i = 0x00;
    cpu.reg.r = 0x00;

    // 5. Inicializar variables del sistema
    cpu.bus.write_byte(0x5C08, 0x00);    // FLAGS
    cpu.bus.write_byte(0x5C3C, 0x00);    // COORDS X
    cpu.bus.write_byte(0x5C3D, 0x00);    // COORDS Y
    cpu.bus.write_byte(0x5C3E, 0x01);    // S_POSN línea
    cpu.bus.write_byte(0x5C3F, 0x01);    // S_POSN columna

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
    // -----------------------------------------------
    // Estado previo
    // -----------------------------------------------
    let pc_before = cpu.reg.pc;
    let sp_before = cpu.reg.sp;

    // -----------------------------------------------
    // HALT: no ejecuta opcode, pero consume tiempo
    // -----------------------------------------------
    if run_state.halted {
        // Si hay interrupción y están habilitadas → despertar
        if interrupt_pending && run_state.iff1 {
            run_state.halted = false;
        } else {
            run_state.t_states += 4;
            return snapshot(cpu, pc_before, 0, 4);
        }
    }

    // -----------------------------------------------
    // Leer instrucción
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

    if mnemonic.starts_with("UNIMPL") {
        unimpl.report(pc_before, &instr_bytes[..instr_len as usize], &mnemonic);
    }

    // --------------------------------------------------
    // INTERCEPTAR IN A,(n)  (teclado ZX)
    // --------------------------------------------------
    /*if opcode == 0xDB {
        let port = instr_bytes[1];
        if port == 0xFE {
            // leer teclado PC → ZX
            let val = crate::teclado::read_fe_port();
            cpu.reg.a = val;
            run_state.t_states += 11; // timing real
            cpu.reg.pc = pc_before.wrapping_add(2);
            return snapshot(cpu, pc_before, 2, 11);
        }
    }*/

    // -----------------------------------------------
    // Ejecutar instrucción
    // -----------------------------------------------
    let instr_cycles = cpu.execute();

    // DBG -------------------------------------------
    let iy_full = ((cpu.reg.iyh as u16) << 8) | (cpu.reg.iyl as u16);
    let ix_full = ((cpu.reg.ixh as u16) << 8) | (cpu.reg.ixl as u16);

    if cpu.reg.pc >= 0x1219 && cpu.reg.pc <= 0x12A2 {
        println!("PC: {:04X} | IY: {:04X} | SP: {:04X}", cpu.reg.pc, iy_full, cpu.reg.sp);
    }
    if cpu.reg.pc == 0x0000 {
        println!("¡RESET DETECTADO! La CPU ha vuelto al inicio.");
    }
    // if cpu.reg.pc == 0x1238 {
    //     println!("¡La CPU ha llegado a la rutina de impresión del mensaje inicial!");
    // }

    // let frames = cpu.bus.read_byte(0x5C78);
    // println!("  FRAMES: ({})", frames);

    if cpu.reg.pc == 0x0038 {
        println!("[ROM] Llamada a MASKABLE INTERRUPT");
    }
    // if cpu.reg.pc == 0x0052 {
    //     println!("[ROM] En 0x0052 ret de la interrupción MASKABLE INTERRUPT");
    // }
    if cpu.reg.pc == 0x0C0A {
        println!("[ROM] Llamada a PO-MSG");
    }
    // if cpu.reg.pc == 0x02BF {
    //     println!("[ROM] Llamada a KEYBOARD");
    // }
    // if cpu.reg.pc == 0x0048 {
    //     println!("[ROM] En KEY-INT");
    // }
    // if cpu.reg.pc == 0x028E {
    //     println!("[ROM] Llamada a KEY-SCAN");
    // }
    // if cpu.reg.pc == 0x02AB {
    //     println!("[ROM] Llamada a KEY-DONE");
    // }
    if cpu.reg.pc == 0x0D6B {
        println!("[ROM] Llamada a CLS");
    }
    if cpu.reg.pc == 0x0EDF {
        println!("[ROM] Llamada a CLEAR-PRB");
    }
    if cpu.reg.pc == 0x16B0 {
        println!("[ROM] Llamada a SET-MIN");
    }
    if cpu.reg.pc == 0x12A9 {
        println!("[ROM] En MAIN-1");
    }
    if cpu.reg.pc == 0x12AC {
        println!("[ROM] En MAIN-2");
    }
    if cpu.reg.pc == 0x1219 {
        println!("[ROM] En RAM-SET");
    }
    if cpu.reg.pc == 0x11CB {
        println!("[ROM] Llamada a START");
    }
    if cpu.reg.pc == 0x11EF {
        println!("[ROM] En RAM-DONE");
    }

    if cpu.reg.pc == 0x0A4F {
        println!("[ROM] Llamada a PO-ENTER (carriage return)");
    }
    if cpu.reg.pc == 0x18E1 {
        dbg!("ROM: rutina de cursor");
    }
    if cpu.reg.pc == 0x15D4 {
        dbg!("ROM: WAIT-KEY");
    }
    if cpu.reg.pc == 0x0A4F {
        dbg!("[CURSOR] Entrando en rutina de cursor");
    }

    if cpu.reg.pc == 0x0B7B {
        println!("[ROM] Llamada a PRINT-AT - debería mostrar cursor");

        // Debug: mostrar coordenadas
        let coords_x = cpu.bus.read_byte(0x5C3C);
        let coords_y = cpu.bus.read_byte(0x5C3D);
        let flags = cpu.bus.read_byte(0x5C08);
        println!("  COORDS: ({},{}) FLAGS: {:08b}", coords_x, coords_y, flags);
    }
    if cpu.reg.pc == 0x0DD9 {
        println!("[ROM] Llamada a CL-SET - establece coordenadas");
    }

    // -----------------------------------------------
    // EI diferido (Z80 REAL)
    // -----------------------------------------------
    if run_state.iff1_pending {
        run_state.iff1_delay -= 1;
        if run_state.iff1_delay == 0 {
            run_state.iff1 = true;
            run_state.iff1_pending = false;
        }
    }

    // -----------------------------------------------
    // EI / DI
    // -----------------------------------------------
    match opcode {
        0xFB => {
            // EI → se activa tras la siguiente instrucción
            run_state.iff1_pending = true;
            run_state.iff1_delay = 1;
        }
        0xF3 => {
            // DI inmediato
            run_state.iff1 = false;
            run_state.iff1_pending = false;
        }
        _ => {}
    }

    // -----------------------------------------------
    // RETI / RETN restauran interrupciones
    // -----------------------------------------------
    if opcode == 0xED {
        let next = cpu.bus.read_byte(pc_before.wrapping_add(1));
        if next == 0x4D || next == 0x45 {
            run_state.iff1 = true;
        }
    }

    // -----------------------------------------------
    // HALT
    // -----------------------------------------------
    if opcode == 0x76 {
        run_state.halted = true;
    }

    // -----------------------------------------------
    // INTERRUPCIÓN (IM 1)
    // -----------------------------------------------
    //if interrupt_pending && run_state.iff1 {
    if interrupt_pending && run_state.iff1 && run_state.allow_interrupts {
        run_state.halted = false;
        run_state.iff1 = false;

        match run_state.im {
            1 => {
                let pc = cpu.reg.pc;

                let sp = cpu.reg.sp.wrapping_sub(2);
                cpu.reg.sp = sp;
                cpu.bus.write_byte(sp, (pc & 0x00FF) as u8);
                cpu.bus.write_byte(sp.wrapping_add(1), (pc >> 8) as u8);

                cpu.reg.pc = 0x0038;

                // DEBUG opcional
                // dbg!("INT → RST 38");

                run_state.t_states += 13;
                return snapshot(cpu, pc, 0, 13);
            }
            _ => {}
        }
    }

    // -----------------------------------------------
    // Tracking stack
    // -----------------------------------------------
    let sp_after = cpu.reg.sp;
    if sp_after < sp_before {
        let written = sp_before.wrapping_sub(sp_after);
        let kind = classify_stack_write(&mnemonic);
        for i in 0..written {
            stack_tracker.record(sp_after.wrapping_add(i), kind, pc_before);
        }
    }

    // -----------------------------------------------
    // T-states
    // -----------------------------------------------
    run_state.t_states += instr_cycles as u64;

    // -----------------------------------------------
    // Registrar ejecución
    // -----------------------------------------------
    executed.insert(pc_before, (instr_len, mnemonic));

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

