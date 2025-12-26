use zilog_z80::cpu::CPU;
use std::collections::{HashMap, HashSet};
use crate::bus::ZxBus;
use crate::stack_tracker::{StackTracker, StackWriteKind};

/* ==================================================
 * SNAPSHOT DE CPU
 * ================================================== */

/*
Struct Bus

Summary
pub struct Bus { /* private fields */ }
The Bus struct is hosting the Z80 memory map.

Implementations
Source
impl Bus
Source
pub fn new(size: u16) -> Bus
Creates a new bus instance. ‘Size’ will be its top address.

Source
pub fn set_romspace(&mut self, start: u16, end: u16)
Sets a ROM space. Write operations will be ineffective in this address range.

use zilog_z80::cpu::CPU;
let mut c = CPU::new(0xFFFF);
c.bus.set_romspace(0xF000, 0xFFFF);
Source
pub fn read_mem_slice(&self, start: usize, end: usize) -> Vec<u8> ⓘ
Reads a slice of bytes from memory

Source
pub fn clear_mem_slice(&mut self, start: usize, end: usize)
Clears a slice of bytes in memory

Source
pub fn read_byte(&self, address: u16) -> u8
Reads a byte from memory

Examples found in repository?
examples/cpmrun.rs (line 47)
fn bdos_call(c: &CPU) {
    if c.reg.c == 0x09 {
        let mut a = c.reg.get_de();
        loop {
            let c = c.bus.read_byte(a);
            if c as char == '$' {
                break;
            } else {
                a += 1;
            }
            print!("{}", c as char);
        }
    }
    if c.reg.c == 0x02 {
        print!("{}", c.reg.e as char);
    }
}
Source
pub fn write_byte(&mut self, address: u16, data: u8)
Writes a byte to memory

Source
pub fn read_word(&self, address: u16) -> u16
Reads a word stored in memory in little endian byte order, returns this word in BE byte order

Source
pub fn read_le_word(&self, address: u16) -> u16
Reads a word stored in memory in little endian byte order, returns this word in LE byte order

Source
pub fn read_le_dword(&self, address: u16) -> u32
Reads a dword stored in memory in little endian byte order, returns this dword in LE byte order

Source
pub fn write_word(&mut self, address: u16, data: u16)
Writes a word to memory in little endian byte order

Examples found in repository?
examples/cpmrun.rs (line 18)
fn load_execute() -> Result<(), Box<dyn Error>> {
    let a: Vec<String> = env::args().collect();
    let mut c = CPU::new(0xFFFF);
    // Loads assembled program into memory
    c.bus.load_bin(&a[1], 0x100)?;

    // RET at 0x05 for mocking of CP/M BDOS system calls
    c.bus.write_word(0x0005, 0xc9);

    // Setting PC to 0x0100 (CP/M Binaries are loaded with a 256 byte offset)
    c.reg.pc = 0x0100;

    /* Setting up stack : by disassembling CP/M software, it seems
    that the $0006 address is read to set the stack by some programs */
    c.bus.write_word(0x0006, 0xFF00);

    /* Setting up stack in case of the program does not read the $0006 address
    and does not set any stack. */
    c.reg.sp = 0xFF00;

    loop {
        c.execute();
        if c.reg.pc == 0x0005 {
            bdos_call(&c)
        }
        if c.reg.pc == 0x0000 {
            break;
        } //  if CP/M warm boot -> we exit
    }
    Ok(())
}
Source
pub fn load_bin(&mut self, file: &str, org: u16) -> Result<usize>

 */

#[derive(Clone, Debug)]
pub struct CpuSnapshot {
    pub pc: u16,
    pub sp: u16,
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub af_: u16,
    pub bc_: u16,
    pub de_: u16,
    pub hl_: u16,
    pub i: u8,
    pub r: u8,

    pub f: u8,
    pub f_before: u8,    // para saber el estado de f anterior y gestionar colores
    pub from_step: bool, // para saber que estoy usando el boton STEP

    pub mem_addr: u16,
    pub mem_value: u8,
    pub mem_base: u16,
    pub mem_dump: Vec<u8>,
    pub stack_base: u16,
    pub stack_dump: Vec<u8>,
    pub instr_len: u8,
    pub instr_cycles: u32,
}

/* ==================================================
 * ESTADO DE EJECUCIÓN
 * ================================================== */
pub struct CpuRunState {
    pub halted: bool,
    pub iff1: bool,
    pub iff1_pending: bool,
    pub iff1_delay: u8,
    pub im: u8,
    pub t_states: u64,
    pub allow_interrupts: bool,
}

impl CpuRunState {
    pub fn new() -> Self {
        Self {
            halted: false,
            iff1: true, // ⬅️ ANTES estaba en false (esto mataba el cursor)
            iff1_pending: false,
            iff1_delay: 0,
            im: 1,
            t_states: 0,
            allow_interrupts: true,
        }
    }
}

/* ==================================================
 * TRACKER DE INSTRUCCIONES (UNIMPL)
 * ================================================== */
pub struct UnimplTracker {
    seen: HashSet<u16>,
}

impl UnimplTracker {
    pub fn new() -> Self {
        Self { seen: HashSet::new() }
    }

    pub fn clear(&mut self) {
        self.seen.clear();
    }

    pub fn report(&mut self, pc: u16, bytes: &[u8], mnemonic: &str) {
        if self.seen.insert(pc) {
            print!("⚠️ UNIMPL PC={:04X}: ", pc);
            for b in bytes { print!("{:02X} ", b); }
            println!("=> {}", mnemonic);
        }
    }
}

// Carga una ROM en la memoria de la CPU
pub fn load_rom(cpu: &mut CPU, path: &str) {
    // Limpiar memoria
    for i in 0x0000..=0xFFFF {
        cpu.bus.write_byte(i, 0);
    }

    // Cargar ROM
    cpu.bus.load_bin(path, 0x0000).expect("Error cargando ROM");

    // Estado inicial Spectrum 48K
    cpu.reg.pc = 0x0000;
    cpu.reg.sp = 0xFFFF;

    cpu.reg.i = 0;
    cpu.reg.r = 0;

    // Variables del sistema
    cpu.bus.write_byte(0x5C08, 0x00); // FLAGS
    cpu.bus.write_byte(0x5C5C, 0x00);
    cpu.bus.write_byte(0x5C5D, 0x40);
}

/* ==================================================
 * INICIALIZACIÓN DE CPU
 * ================================================== */
const DIR_CARGA_ROM: u16 = 0x0000;

pub fn init_cpu(rom_path: &str) -> CPU {
    let mut cpu = CPU::new(0xFFFF);
    cpu.bus.load_bin(rom_path, DIR_CARGA_ROM).expect("Error cargando ROM");

    cpu.reg.sp = 0xFFFF;
    cpu.reg.pc = DIR_CARGA_ROM;

    // Inicializar variables del sistema para evitar basura en pantalla
    cpu.bus.write_byte(0x5C08, 0x00); // FLAGS
    cpu.bus.write_byte(0x5C5C, 0x00); // CURCHL low
    cpu.bus.write_byte(0x5C5D, 0x40); // CURCHL high

    cpu.reg.i = 0x00;
    cpu.reg.r = 0x00;
    cpu
}

pub fn init_cpu_with_test(rom_path: &str, test_path: &str) -> CPU {
    let mut cpu = init_cpu(rom_path);
    cpu.bus.load_bin(test_path, 0x8000).expect("Error cargando TEST");
    cpu.reg.pc = 0x8000;
    cpu
}

/* ==================================================
 * STEP (EJECUCIÓN)
 * ================================================== */

pub fn step(
    cpu: &mut CPU,
    zx_bus: &mut ZxBus,
    run_state: &mut CpuRunState,
    interrupt_pending: bool,
    executed: &mut HashMap<u16, (u8, String)>,
    unimpl: &mut UnimplTracker,
    stack_tracker: &mut StackTracker,
    from_step: bool,
) -> CpuSnapshot {
    let pc_before = cpu.reg.pc;
    let sp_before = cpu.reg.sp;
    // averiguamos antes de ejecutar el valor de F para poner colores
    let f_before = (cpu.reg.get_af() & 0x00FF) as u8;

    if run_state.halted {
        if interrupt_pending && run_state.iff1 {
            run_state.halted = false;
        } else {
            run_state.t_states += 4;
            return snapshot(cpu, pc_before, false, f_before, 0, 4);
        }
    }

    // Leemos bytes para el desensamblador
    let mut instr_bytes = [0u8; 4];
    for i in 0..4 {
        instr_bytes[i] = cpu.bus.read_byte(pc_before.wrapping_add(i as u16));
    }

    let (mnemonic, instr_len) = crate::disasm::disassemble(&instr_bytes, pc_before, pc_before);

    if mnemonic.starts_with("UNIMPL") {
        unimpl.report(pc_before, &instr_bytes[..instr_len as usize], &mnemonic);
    }

    // 1. Intercepción universal de entrada de puertos ANTES de ejecutar
    // El opcode 0xDB es "IN A, (n)"
    if instr_bytes[0] == 0xDB {
        let n = instr_bytes[1];
        let port = ((cpu.reg.a as u16) << 8) | (n as u16);

        // Si es el puerto de teclado (bit 0 = 0)
        if (port & 0x01) == 0 {
            let val = zx_bus.in_port(port);

            cpu.reg.a = val;
            // Nota: IN A, (n) NO afecta a los flags en un Z80 real.
            cpu.reg.pc = pc_before.wrapping_add(2);
            run_state.t_states += 11;

            executed.insert(pc_before, (2, mnemonic));
            return snapshot(cpu, pc_before, false, f_before, 2, 11);
        }
    }

    /*// Intercepción para IN A, (C) -> Opcode ED 78
    if instr_bytes[0] == 0xED && instr_bytes[1] == 0x78 {
        let port = cpu.reg.get_bc(); // Usa el registro BC completo como puerto
        if (port & 0x01) == 0 {
            cpu.reg.a = zx_bus.in_port(port);
            cpu.reg.pc = pc_before.wrapping_add(2);
            run_state.t_states += 12;
            return snapshot(cpu, pc_before, false, f_before, 2, 12);
        }
    }*/

    // 2. Intercepción universal para IN r, (C) -> Opcodes ED 40 a ED 78
    if instr_bytes[0] == 0xED && (instr_bytes[1] & 0xC7 == 0x40) {
        let port = cpu.reg.get_bc();
        if (port & 0x01) == 0 {
            let val = zx_bus.in_port(port);

            // 1. Guardar el valor en el registro correspondiente
            match (instr_bytes[1] >> 3) & 0x07 {
                0 => cpu.reg.b = val,
                1 => cpu.reg.c = val,
                2 => cpu.reg.d = val,
                3 => cpu.reg.e = val,
                4 => cpu.reg.h = val,
                5 => cpu.reg.l = val,
                7 => cpu.reg.a = val,
                _ => {} // Caso 6 es IN (C) que solo afecta a flags
            }

            // 2. Actualizar FLAGS
            // En zilog_z80, el registro F es el byte bajo de AF.
            let mut f = (cpu.reg.get_af() & 0xFF00); // Limpiamos los flags viejos
            let mut new_f: u8 = 0;

            if val & 0x80 != 0 { new_f |= 0x80; } // Sign
            if val == 0 { new_f |= 0x40; }        // Zero
            if val.count_ones() % 2 == 0 { new_f |= 0x04; } // Parity

            // El bit 1 (N) se pone a 0 en instrucciones IN
            // El bit 4 (H) se pone a 0 en instrucciones IN

            cpu.reg.set_af(f | (new_f as u16));

            cpu.reg.pc = pc_before.wrapping_add(2);
            run_state.t_states += 12;
            return snapshot(cpu, pc_before, false, f_before, 2, 12);
        }
    }

    let instr_cycles = cpu.execute();

    // -------------- BLOQUE DE DEBUG PRINTLNS ------------------
    // let iy_full = ((cpu.reg.iyh as u16) << 8) | (cpu.reg.iyl as u16);
    //
    // if cpu.reg.pc >= 0x1219 && cpu.reg.pc <= 0x12A2 {
    //     println!("PC: {:04X} | IY: {:04X} | SP: {:04X}", cpu.reg.pc, iy_full, cpu.reg.sp);
    // }
    // if cpu.reg.pc == 0x0000 {
    //     println!("¡RESET DETECTADO! La CPU ha vuelto al inicio.");
    // }
    //
    // if cpu.reg.pc == 0x0038 { println!("[ROM] Llamada a MASKABLE INTERRUPT"); }
    // //if cpu.reg.pc == 0x0039 { println!("[ROM] Llamada a MASKABLE INTERRUPT + 1 (0x39)"); }
    // if cpu.reg.pc == 0x0C0A { println!("[ROM] Llamada a PO-MSG"); }
    // if cpu.reg.pc == 0x0D6B { println!("[ROM] Llamada a CLS"); }
    // if cpu.reg.pc == 0x0EDF { println!("[ROM] Llamada a CLEAR-PRB"); }
    // if cpu.reg.pc == 0x16B0 { println!("[ROM] Llamada a SET-MIN"); }
    // if cpu.reg.pc == 0x12A9 { println!("[ROM] En MAIN-1"); }
    // if cpu.reg.pc == 0x12AC { println!("[ROM] En MAIN-2"); }
    // if cpu.reg.pc == 0x1219 { println!("[ROM] En RAM-SET"); }
    // if cpu.reg.pc == 0x11CB { println!("[ROM] Llamada a START"); }
    // if cpu.reg.pc == 0x11EF { println!("[ROM] En RAM-DONE"); }
    // if cpu.reg.pc == 0x15D4 { dbg!("ROM: WAIT-KEY"); }
    // println!("  TV_COUNT: {}", cpu.bus.read_byte(0x5C3C));
    // if cpu.reg.pc == 0x18E1 { dbg!("ROM: CURSOR ROUTINE"); }

    // if cpu.reg.pc == 0x0B7B {
    //     println!("[ROM] Llamada a PRINT-AT - debería mostrar cursor");
    //     let coords_x = cpu.bus.read_byte(0x5C3C);
    //     let coords_y = cpu.bus.read_byte(0x5C3D);
    //     let flags = cpu.bus.read_byte(0x5C08);
    //     println!("  COORDS: ({},{}) FLAGS: {:08b}", coords_x, coords_y, flags);
    // }

    // Fin DEBUG ----------------------------------------------------------

    // Lógica de interrupciones
    if run_state.iff1_pending {
        run_state.iff1_delay -= 1;
        if run_state.iff1_delay == 0 {
            run_state.iff1 = true;
            run_state.iff1_pending = false;
        }
    }

    match instr_bytes[0] {
        0xFB => {
            run_state.iff1_pending = true;
            run_state.iff1_delay = 1;
        }
        0xF3 => {
            run_state.iff1 = false;
            run_state.iff1_pending = false;
        }
        0x76 => { run_state.halted = true; }
        _ => {}
    }

    if interrupt_pending && run_state.iff1 && run_state.allow_interrupts {
        run_state.halted = false;
        run_state.iff1 = false;

        let pc_at_int = cpu.reg.pc;

        let sp = cpu.reg.sp.wrapping_sub(2);
        cpu.reg.sp = sp;
        cpu.bus.write_byte(sp, (pc_at_int & 0x00FF) as u8);
        cpu.bus.write_byte(sp.wrapping_add(1), (pc_at_int >> 8) as u8);

        cpu.reg.pc = 0x0038;
        run_state.t_states += 13;

        return snapshot(cpu, pc_at_int, false, f_before, 0, 13);
    }

    // Tracking stack
    let sp_after = cpu.reg.sp;
    if sp_after < sp_before {
        let kind = if mnemonic.starts_with("CALL") || mnemonic.starts_with("RST") {
            StackWriteKind::Call
        } else if mnemonic.starts_with("PUSH") {
            StackWriteKind::Push
        } else {
            StackWriteKind::Manual
        };
        for i in 0..sp_before.wrapping_sub(sp_after) {
            stack_tracker.record(sp_after.wrapping_add(i), kind, pc_before);
        }
    }

    run_state.t_states += instr_cycles as u64;
    executed.insert(pc_before, (instr_len, mnemonic));

    snapshot(cpu, pc_before, from_step, f_before, instr_len, instr_cycles)
}

/* ==================================================
 * SNAPSHOT (BUFFER AMPLIADO PARA GUI)
 * ================================================== */

pub fn snapshot(cpu: &CPU, pc: u16, from_step: bool, f_before: u8, instr_len: u8, instr_cycles: u32) -> CpuSnapshot {
    // 512 bytes alrededor del PC para que el desensamblador del GUI tenga margen
    let mem_base = pc.saturating_sub(128);
    let mut mem_dump = Vec::with_capacity(512);
    for i in 0..512 {
        mem_dump.push(cpu.bus.read_byte(mem_base.wrapping_add(i as u16)));
    }

    let sp = cpu.reg.sp;
    let mut stack_dump = Vec::with_capacity(32);
    for i in 0..32 {
        stack_dump.push(cpu.bus.read_byte(sp.wrapping_add(i as u16)));
    }

    CpuSnapshot {
        pc,
        sp,
        af: cpu.reg.get_af(),
        bc: cpu.reg.get_bc(),
        de: cpu.reg.get_de(),
        hl: cpu.reg.get_hl(),
        af_: cpu.alt.get_af(),
        bc_: cpu.alt.get_bc(),
        de_: cpu.alt.get_de(),
        hl_: cpu.alt.get_hl(),
        i: cpu.reg.i,
        r: cpu.reg.r,
        f: (cpu.reg.get_af() & 0x00FF) as u8,
        f_before,
        from_step,
        mem_addr: pc,
        mem_value: cpu.bus.read_byte(pc),
        mem_base,
        mem_dump,
        stack_base: sp,
        stack_dump,
        instr_len,
        instr_cycles,
    }
}