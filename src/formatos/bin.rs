use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::cpu_exec::CpuRunState;
use zilog_z80::cpu::CPU;
use crate::constantes::DIR_BIN_DEFAULT;
// pub fn load_bin(
//     cpu: &mut CPU,
//     run_state: &mut CpuRunState,
//     path: &Path,
//     org: u16,
// ) -> Result<(), String> {
//     let data = std::fs::read(path)
//         .map_err(|e| format!("BIN: {}", e))?;
//
//     for (i, b) in data.iter().enumerate() {
//         cpu.bus.write_byte(org.wrapping_add(i as u16), *b);
//     }
//
//     cpu.reg.pc = org;
//
//     // Estado limpio
//     run_state.halted = false;
//     run_state.allow_interrupts = true;
//
//     Ok(())
// }

pub fn load_bin(
    cpu: &mut CPU,
    run_state: &mut CpuRunState,
    path: &Path,
) -> Result<(), String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("BIN: {}", e))?;

    if data.len() < 4 {
        return Err("BIN demasiado pequeño".into());
    }

    // ==========================================
    // BIN CON CABECERA ZX
    // ==========================================
    if data[0] == b'Z' && data[1] == b'X' { // Si tiene la firma ZX
        if data.len() < 10 {
            return Err("BIN ZX: cabecera incompleta".into());
        }

        let org = u16::from_le_bytes([data[2], data[3]]);
        let pc = u16::from_le_bytes([data[4], data[5]]);
        let size = u16::from_le_bytes([data[6], data[7]]) as usize;

        if data.len() < 10 + size {
            return Err("BIN ZX: tamaño inválido".into());
        }

        let code = &data[10..10 + size];

        for (i, b) in code.iter().enumerate() {
            cpu.bus.write_byte(org.wrapping_add(i as u16), *b);
        }

        cpu.reg.pc = pc;
    }

    // ==========================================
    // BIN PLANO (modo antiguo)
    // ==========================================
    else {
        let org = DIR_BIN_DEFAULT;
        for (i, b) in data.iter().enumerate() {
            cpu.bus.write_byte(org.wrapping_add(i as u16), *b);
        }
        cpu.reg.pc = org;
    }

    // Estado limpio
    run_state.halted = false;
    run_state.allow_interrupts = false;
    //run_state.allow_interrupts = true;
    run_state.t_states = 0;

    run_state.iff1 = false;
    run_state.iff1_pending = false;

    run_state.im = 1;

    Ok(())
}




