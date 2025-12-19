mod cpu_exec;
mod gui;
mod disasm;
mod debugger;
mod teclado;
mod botones;
mod stack_tracker;
mod video;
mod interrupt;

use cpu_exec::{init_cpu, step, UnimplTracker};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use debugger::{Debugger, RunMode};
use teclado::Keyboard;
use botones::ButtonAction;
use stack_tracker::StackTracker;
use video::Video;
use std::collections::HashMap;
use crate::cpu_exec::CpuRunState;

const RUNFAST_REPORT_EVERY: usize = 10_000;
const ESCALA_VENTANA_ZX: u32 = 4;
const ESCALA_PANTALLA_ZX: u32 = 12;
const ANCHO_VENTANA: u32 = 3800;
const ALTO_VENTANA: u32 = 2800;
const TSTATES_PER_FRAME: u64 = 69888;
fn main() -> Result<(), String> {
    // --------------------------------------------------
    // CPU + ESTADO
    // --------------------------------------------------
    let mut cpu = init_cpu("ROMS/DMG_ROMdeGameBoy.bin");
    //let mut cpu = init_cpu("tests/z80/video_attr_test.bin");
    //let mut cpu = init_cpu("tests/z80/all_colors_flash.bin");
    //let mut cpu = init_cpu("tests/z80/flash_test.bin");

    let mut run_state = CpuRunState::new(); // Estado de la CPU
    let mut interrupt_pending = false;
    let mut next_interrupt = TSTATES_PER_FRAME;

    let mut executed_instrs = HashMap::new();
    let mut unimpl_tracker = UnimplTracker::new();
    let mut last_snapshot = None;

    let mut debugger = Debugger::new();
    let mut tstates: u64 = 0;
    let mut stack_tracker = StackTracker::new(512);

    let mut next_interrupt: u64 = TSTATES_PER_FRAME;

    // PANTALLA ZX (buffer lógico)
    let mut pantalla = Video::new(ESCALA_VENTANA_ZX);

    // --------------------------------------------------
    // SDL
    // --------------------------------------------------
    let sdl = sdl2::init()?;
    let video_sub = sdl.video()?;
    let ttf = sdl2::ttf::init().map_err(|e| e.to_string())?;

    // ---------------- DEBUG WINDOW ----------------
    let debug_window = video_sub
        .window("ZX Debugger", ANCHO_VENTANA, ALTO_VENTANA)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut debug_canvas = debug_window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    /* // Tamaño lógico fijo
     debug_canvas
         .set_logical_size(ANCHO_VENTANA, ALTO_VENTANA)
         .map_err(|e| e.to_string())?;*/

    let font = ttf.load_font("FONTS/DejaVuSansMono.ttf", 16)?;

    // ---------------- ZX SCREEN WINDOW ----------------
    let zx_window = video_sub
        .window(
            "ZX Spectrum",
            //256 * ESCALA_PANTALLA_ZX,
            //192 * ESCALA_PANTALLA_ZX,
            256 * ESCALA_PANTALLA_ZX,
            192 * ESCALA_PANTALLA_ZX,
        )
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut zx_canvas = zx_window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl.event_pump()?;

    // --------------------------------------------------
    // BUCLE PRINCIPAL
    // --------------------------------------------------
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,

                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::MouseButtonDown { x, y, .. } => {
                    for b in botones::default_buttons() {
                        if b.contains(x, y) {
                            match b.action {
                                ButtonAction::Step => {
                                    debugger.step();
                                    let snap = step(
                                        &mut cpu,
                                        &mut run_state,
                                        interrupt_pending,
                                        &mut executed_instrs,
                                        &mut unimpl_tracker,
                                        &mut stack_tracker,
                                    );
                                    tstates += snap.instr_cycles as u64;
                                    last_snapshot = Some(snap);
                                }
                                ButtonAction::Run => debugger.run(),
                                ButtonAction::RunFast => debugger.run_fast(),
                                ButtonAction::Pause => debugger.pause(),
                                ButtonAction::Reset => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // ---------------- CPU RUN ----------------
        match debugger.mode {
            RunMode::Run => {
                let snap = step(
                    &mut cpu,
                    &mut run_state,
                    interrupt_pending,
                    &mut executed_instrs,
                    &mut unimpl_tracker,
                    &mut stack_tracker,
                );
                tstates += snap.instr_cycles as u64;
                if tstates >= next_interrupt {
                    //cpu.interrupt();      // IM 1 → vector $0038
                    cpu.int_request(0xFF); // IM 1 → vector $0038
                    next_interrupt += TSTATES_PER_FRAME;
                    pantalla.on_vsync();     // ← lo usaremos para FLASH
                }

                last_snapshot = Some(snap);

                // ----------------------------------------
                // IM 1: una interrupción cada ~20 ms (50 Hz)
                // ----------------------------------------
                unsafe {
                    cpu_exec::IM1_COUNT += 1;

                    if cpu_exec::IM1_COUNT % 32 == 0 {
                        pantalla.flash_phase = !pantalla.flash_phase;
                    }
                }
            }
            RunMode::RunFast => {
                for _ in 0..50_000 {
                    let snap = step(
                        &mut cpu,
                        &mut run_state,
                        interrupt_pending,
                        &mut executed_instrs,
                        &mut unimpl_tracker,
                        &mut stack_tracker,
                    );
                    tstates += snap.instr_cycles as u64;
                    if tstates >= next_interrupt {
                        cpu.int_request(0xFF); // Interrupcción IM 1 → vector $0038

                        next_interrupt += TSTATES_PER_FRAME;
                        pantalla.on_vsync();     // ← FLASH
                    }

                    last_snapshot = Some(snap);
                    if debugger.mode != RunMode::RunFast {
                        break;
                    }
                }
            }
            _ => {}
        }

        // ---------------- INTERRUPCION ----------------
        if run_state.t_states >= next_interrupt {
            interrupt_pending = true;
            next_interrupt += TSTATES_PER_FRAME;
        }

        // ---------------- VIDEO UPDATE ----------------
        pantalla.update_from_bus(&cpu.bus);

        // ---------------- RENDER ----------------
        gui::draw_debug(
            &mut debug_canvas,
            &font,
            last_snapshot.as_ref(),
            &stack_tracker,
        )?;

        gui::draw_zx_screen(
            &mut zx_canvas,
            &pantalla,
        )?;
    }

    Ok(())
}

