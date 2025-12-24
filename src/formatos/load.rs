use std::path::Path;
use zilog_z80::cpu::CPU;
use rfd::FileDialog;
use crate::cpu_exec::CpuRunState;
use crate::formatos::{bin, sna, z80};

/// Resultado de la carga (para la UI)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadResult {
    Rom,
    Sna,
    Z80,
    Bin,
}

/// Ventana de selección y carga automática
pub fn load_file_dialog(
    cpu: &mut CPU,
    run_state: &mut CpuRunState,
) -> Result<LoadResult, String> {
    let file = FileDialog::new()
        .set_title("Cargar ROM / SNA / Z80")
        .add_filter("ZX Spectrum", &["rom", "sna", "z80", "bin"])
        .pick_file()
        .ok_or("Carga cancelada")?;

    load_file(cpu, run_state, &file)
}

/// Carga según extensión
pub fn load_file(
    cpu: &mut CPU,
    run_state: &mut CpuRunState,
    path: &Path,
) -> Result<LoadResult, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .ok_or("Fichero sin extensión")?;

    match ext.as_str() {
        // -----------------------------
        // ROM 16K (0x0000)
        // -----------------------------
        "rom" => {
            load_rom(cpu, path)?;
            Ok(LoadResult::Rom)
        }
        // -----------------------------
        // Snapshot SNA
        // -----------------------------
        "sna" => {
            let snap = sna::SnaSnapshot::load(path)
                .map_err(|e| e.to_string())?;

            sna::apply_sna(cpu, run_state, &snap);
            Ok(LoadResult::Sna)
        }
        // -----------------------------
        // Snapshot Z80
        // -----------------------------
        "z80" => {
            let snap = z80::Z80Snapshot::load(path)
                .map_err(|e| e.to_string())?;

            z80::apply_z80(cpu, run_state, &snap);
            Ok(LoadResult::Z80)
        }
        // -----------------------------
        // BIN crudo (ORG fijo)
        // -----------------------------
        "bin" => {
            // Convención: cargar en 0x8000
            bin::load_bin(cpu, run_state, path)?;
            Ok(LoadResult::Bin)
        }

        _ => Err("Formato no soportado".into()),
    }
}

/// Carga de ROM pura (16K en 0x0000)
fn load_rom(cpu: &mut CPU, path: &Path) -> Result<(), String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("ROM: {}", e))?;

    if data.len() != 16 * 1024 {
        return Err("La ROM debe ser de 16 KB".into());
    }

    for (i, b) in data.iter().enumerate() {
        cpu.bus.write_byte(i as u16, *b);
    }

    cpu.reg.pc = 0x0000;
    Ok(())
}
