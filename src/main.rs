/*mod cpu_exec;
mod gui;
mod disasm;
mod debugger;
mod teclado;
mod botones;
mod stack_tracker;
mod video;
mod interrupt;
mod bus;
mod formatos;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::collections::HashMap;
use std::time::{Duration, Instant};
use zilog_z80::cpu::CPU;
use debugger::{Debugger, RunMode};
use botones::ButtonAction;
use stack_tracker::StackTracker;
use video::Video;
use cpu_exec::{CpuRunState, init_cpu, step, UnimplTracker};
use bus::ZxBus;
use interrupt::InterruptController;

const TSTATES_PER_FRAME: u64 = 69888;
const ANCHO_VENTANA: u32 = 3800;
const ALTO_VENTANA: u32 = 2800;
const ESCALA_VENTANA_ZX: u32 = 4;
const ESCALA_PANTALLA_ZX: u32 = 12;

// Indica el estado segun presionemos unos botones u otros
#[derive(Copy, Clone)]
pub enum LoadState {
    None,
    Rom,
    Sna,
}

fn main() -> Result<(), String> {
    // 1. Creamos el Bus y el Teclado
    let mut zx_bus = ZxBus::new();

    // 2. Inicializamos la CPU con la ROM
    //let mut cpu = init_cpu("ROMS/ZXSpectrum48.rom");
    //let mut cpu = init_cpu("ROMS/ZX48_v2EN.rom");
    let mut cpu = CPU::new(0xFFFF);
    //load_rom(&mut cpu, "ROMS/ZXSpectrum48.rom");
    let mut load_state = LoadState::None; // Estado inicial VACIO

    let mut int_ctrl = InterruptController::new();
    let mut run_state = CpuRunState::new();
    let mut interrupt_pending = false;

    let mut executed_instrs = HashMap::new();
    let mut unimpl_tracker = UnimplTracker::new();
    let mut last_snapshot = None;

    let mut debugger = Debugger::new();
    let mut stack_tracker = StackTracker::new(512);
    let mut pantalla = Video::new(ESCALA_VENTANA_ZX);

    // SDL Setup
    let sdl = sdl2::init()?;
    let video_sub = sdl.video()?;
    let ttf = sdl2::ttf::init().map_err(|e| e.to_string())?;

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

    let font = ttf.load_font("FONTS/DejaVuSansMono.ttf", 16)?;

    let zx_window = video_sub
        .window("ZX Spectrum", 256 * ESCALA_PANTALLA_ZX, 192 * ESCALA_PANTALLA_ZX)
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
    let frame_duration = Duration::from_micros(20000); // 20ms = 50Hz

    // BUCLE PRINCIPAL
    'running: loop {
        let frame_start = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { keycode: Some(k), repeat: false, .. } => {
                    // IMPORTANTE: Usar el teclado que está dentro del bus
                    zx_bus.keyboard.key_down(k);
                }
                Event::KeyUp { keycode: Some(k), .. } => {
                    // Ahora le pasamos 'k' (el Keycode) para que sepa qué bit liberar
                    zx_bus.keyboard.key_up(k);
                }
                Event::MouseButtonDown { x, y, .. } => {
                    for b in botones::default_buttons() {
                        if b.contains(x, y) {
                            match b.action {
                                ButtonAction::Step => {
                                    let snap = step(
                                        &mut cpu,
                                        &mut zx_bus,
                                        &mut run_state,
                                        interrupt_pending,
                                        &mut executed_instrs,
                                        &mut unimpl_tracker,
                                        &mut stack_tracker,
                                        true,
                                    );
                                    last_snapshot = Some(snap);
                                }
                                ButtonAction::Run => debugger.run(),
                                ButtonAction::RunFast => debugger.run_fast(),
                                ButtonAction::Pause => debugger.pause(),
                                /*ButtonAction::LoadRom => {
                                    println!("Cargando ROM...");
                                    load_rom(&mut cpu, "ROMS/ZXSpectrum48.rom");
                                    run_state = CpuRunState::new();
                                    interrupt_pending = false;
                                    int_ctrl = InterruptController::new();
                                    last_snapshot = None;
                                    load_state = LoadState::Rom;
                                    println!("ROM cargada correctamente");
                                }*/

                                /*ButtonAction::LoadSna => {
                                    println!("Cargando snapshot SNA...");

                                    let sna = SnaSnapshot::load("SNAPS/manic.sna")
                                        .map_err(|e| e.to_string())?;
                                    apply_sna(&mut cpu, &mut run_state, &sna);

                                    interrupt_pending = false;
                                    int_ctrl = InterruptController::new();
                                    last_snapshot = None;
                                    load_state = LoadState::Sna;

                                    println!("SNA cargado correctamente");
                                }*/
                                ButtonAction::Load => {
                                    println!("Abriendo selector de fichero...");

                                    match formatos::load::load_file_dialog(&mut cpu, &mut run_state) {
                                        Ok(kind) => {
                                            // Estado limpio tras cualquier carga
                                            interrupt_pending = false;
                                            int_ctrl = InterruptController::new();
                                            last_snapshot = None;

                                            // Actualizar estado visual según lo cargado
                                            load_state = match kind {
                                                formatos::load::LoadResult::Rom => LoadState::Rom,
                                                formatos::load::LoadResult::Sna => LoadState::Sna,
                                                formatos::load::LoadResult::Z80 => LoadState::Sna, // mismo tratamiento visual
                                            };

                                            println!("Cargado correctamente: {:?}", kind);
                                        }
                                        Err(e) => {
                                            println!("Carga cancelada o error: {}", e);
                                        }
                                    }
                                }

                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Ejecución de la CPU
        match debugger.mode {
            RunMode::Run => {
                let mut states_en_este_frame: u64 = 0;

                // Ejecutamos instrucciones hasta alcanzar los 69888 ciclos (1 frame)
                while states_en_este_frame < TSTATES_PER_FRAME {
                    // Si el PC coincide con un breakpoint, pausamos y salimos del bucle
                    if debugger.check_breakpoint(cpu.reg.pc) {
                        debugger.pause();
                        break;
                    }

                    let snap = step(
                        &mut cpu,
                        &mut zx_bus,
                        &mut run_state,
                        interrupt_pending,
                        &mut executed_instrs,
                        &mut unimpl_tracker,
                        &mut stack_tracker,
                        false,
                    );

                    // ⬅️ ACUMULAMOS CICLOS REALES
                    if int_ctrl.add_cycles(snap.instr_cycles) {
                        interrupt_pending = true;
                    }

                    // Si la CPU aceptó la interrupción (saltó a 0x0038), bajamos la señal
                    if interrupt_pending && cpu.reg.pc == 0x0038 {
                        interrupt_pending = false;
                    }

                    states_en_este_frame += snap.instr_cycles as u64;
                    last_snapshot = Some(snap);
                }

                // Al finalizar el frame, generamos la señal de interrupción para el próximo
                // Actualizamos la estructura de video con la RAM actual
                pantalla.update_from_bus(&cpu.bus);
                pantalla.on_vsync();
            }

            RunMode::RunFast => {
                // En modo rápido, ejecutamos 10 veces más ciclos por cada ciclo de refresco de la UI
                for _ in 0..10 {
                    let mut states_sub_frame: u64 = 0;
                    while states_sub_frame < TSTATES_PER_FRAME {
                        let snap = step(
                            &mut cpu,
                            &mut zx_bus,
                            &mut run_state,
                            interrupt_pending,
                            &mut executed_instrs,
                            &mut unimpl_tracker,
                            &mut stack_tracker,
                            false,
                        );
                        if int_ctrl.add_cycles(snap.instr_cycles) {
                            interrupt_pending = true;
                        }
                        if interrupt_pending && cpu.reg.pc == 0x0038 {
                            interrupt_pending = false;
                        }

                        states_sub_frame += snap.instr_cycles as u64;
                        last_snapshot = Some(snap);
                    }
                    pantalla.on_vsync();
                }
                pantalla.update_from_bus(&cpu.bus);
            }

            RunMode::Paused => {
                // En pausa no hacemos nada, la CPU se queda donde está
            }

            _ => {}
        }

        // Sincronizar Video y Render
        pantalla.update_from_bus(&cpu.bus);
        gui::draw_debug(
            &mut debug_canvas,
            &font,
            last_snapshot.as_ref(),
            &stack_tracker,
            load_state,
        )?;
        gui::draw_zx_screen(&mut zx_canvas, &pantalla)?;

        // Presentar los cambios!
        debug_canvas.present();
        zx_canvas.present();

        // ESPERAR para clavar los 50 FPS
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
    Ok(())
}
*/

mod cpu_exec;
mod gui;
mod disasm;
mod debugger;
mod teclado;
mod botones;
mod stack_tracker;
mod video;
mod interrupt;
mod bus;
mod formatos;
mod constantes;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::collections::HashMap;
use std::time::{Duration, Instant};
use zilog_z80::cpu::CPU;
use debugger::{Debugger, RunMode};
use botones::ButtonAction;
use stack_tracker::StackTracker;
use video::Video;
use cpu_exec::{step, CpuRunState, UnimplTracker};
use bus::ZxBus;
use interrupt::InterruptController;
use formatos::load::{self, LoadResult};
use crate::constantes::{ALTO_VENTANA, ANCHO_VENTANA, ESCALA_PANTALLA_ZX, ESCALA_VENTANA_ZX, TSTATES_PER_FRAME};
// const TSTATES_PER_FRAME: u64 = 69888;
// const ANCHO_VENTANA: u32 = 3800;
// const ALTO_VENTANA: u32 = 2800;
// const ESCALA_VENTANA_ZX: u32 = 4;
// const ESCALA_PANTALLA_ZX: u32 = 12;

#[derive(Copy, Clone)]
pub enum LoadState {
    None,
    Rom,
    Sna,
    Z80,
    Bin,
}

fn main() -> Result<(), String> {
    let mut zx_bus = ZxBus::new();
    let mut cpu = CPU::new(0xFFFF);

    let mut load_state = LoadState::None;
    let mut debug_enabled = false;

    let mut int_ctrl = InterruptController::new();
    let mut run_state = CpuRunState::new();
    let mut interrupt_pending = false;

    let mut executed_instrs = HashMap::new();
    let mut unimpl_tracker = UnimplTracker::new();
    let mut last_snapshot = None;

    let mut debugger = Debugger::new();
    let mut stack_tracker = StackTracker::new(512);
    let mut pantalla = Video::new(ESCALA_VENTANA_ZX);

    // SDL
    let sdl = sdl2::init()?;
    let video_sub = sdl.video()?;
    let ttf = sdl2::ttf::init().map_err(|e| e.to_string())?;

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

    let font = ttf.load_font("FONTS/DejaVuSansMono.ttf", 16)?;

    let zx_window = video_sub
        .window("ZX Spectrum", 256 * ESCALA_PANTALLA_ZX, 192 * ESCALA_PANTALLA_ZX)
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
    let frame_duration = Duration::from_micros(20000);

    'running: loop {
        let frame_start = Instant::now();

        // ===================== EVENTOS =====================
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,

                Event::KeyDown { keycode: Some(k), repeat: false, .. } => {
                    zx_bus.keyboard.key_down(k);
                }
                Event::KeyUp { keycode: Some(k), .. } => {
                    zx_bus.keyboard.key_up(k);
                }

                Event::MouseButtonDown { x, y, .. } => {
                    for b in botones::default_buttons() {
                        if b.contains(x, y) {
                            match b.action {
                                ButtonAction::Step => {
                                    let snap = step(
                                        &mut cpu,
                                        &mut zx_bus,
                                        &mut run_state,
                                        interrupt_pending,
                                        &mut executed_instrs,
                                        &mut unimpl_tracker,
                                        &mut stack_tracker,
                                        true,
                                    );
                                    if debug_enabled {
                                        last_snapshot = Some(snap);
                                    }
                                }

                                ButtonAction::Run => debugger.run(),
                                ButtonAction::RunFast => debugger.run_fast(),
                                ButtonAction::Pause => debugger.pause(),

                                ButtonAction::Load => {
                                    match load::load_file_dialog(&mut cpu, &mut run_state) {
                                        Ok(kind) => {
                                            interrupt_pending = false;
                                            int_ctrl = InterruptController::new();
                                            last_snapshot = None;

                                            load_state = match kind {
                                                LoadResult::Rom => LoadState::Rom,
                                                LoadResult::Sna => LoadState::Sna,
                                                LoadResult::Z80 => LoadState::Z80,
                                                LoadResult::Bin => LoadState::Bin,
                                            };

                                            println!("Cargado correctamente: {:?}", kind);
                                        }
                                        Err(e) => {
                                            println!("Carga cancelada o error: {}", e);
                                        }
                                    }
                                }

                                ButtonAction::DebugToggle => {
                                    debug_enabled = !debug_enabled;

                                    if !debug_enabled {
                                        debugger.pause();
                                        last_snapshot = None;
                                    }

                                    println!("DEBUG {}", if debug_enabled { "ON" } else { "OFF" });
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // ===================== CPU =====================
        match debugger.mode {
            RunMode::Run => {
                let mut states = 0;

                // Ejecutamos instrucciones hasta alcanzar los 69888 ciclos (1 frame)
                while states < TSTATES_PER_FRAME {
                    // Si el PC coincide con un breakpoint, pausamos y salimos del bucle
                    if debugger.check_breakpoint(cpu.reg.pc) {
                        debugger.pause();
                        break;
                    }

                    let snap = step(
                        &mut cpu,
                        &mut zx_bus,
                        &mut run_state,
                        interrupt_pending,
                        &mut executed_instrs,
                        &mut unimpl_tracker,
                        &mut stack_tracker,
                        false,
                    );

                    // ⬅️ ACUMULAMOS CICLOS REALES
                    if int_ctrl.add_cycles(snap.instr_cycles) {
                        interrupt_pending = true;
                    }
                    // Si la CPU aceptó la interrupción (saltó a 0x0038), bajamos la señal
                    if interrupt_pending && cpu.reg.pc == 0x0038 {
                        interrupt_pending = false;
                    }

                    states += snap.instr_cycles as u64;
                    if debug_enabled {
                        last_snapshot = Some(snap);
                    }
                }
                pantalla.on_vsync();
            }

            RunMode::RunFast => {
                for _ in 0..10 {
                    let mut states = 0;
                    while states < TSTATES_PER_FRAME {
                        // Si el PC coincide con un breakpoint, pausamos y salimos del bucle
                        if debugger.check_breakpoint(cpu.reg.pc) {
                            debugger.pause();
                            break;
                        }
                        
                        let snap = step(
                            &mut cpu,
                            &mut zx_bus,
                            &mut run_state,
                            interrupt_pending,
                            &mut executed_instrs,
                            &mut unimpl_tracker,
                            &mut stack_tracker,
                            false,
                        );

                        if int_ctrl.add_cycles(snap.instr_cycles) {
                            interrupt_pending = true;
                        }
                        if interrupt_pending && cpu.reg.pc == 0x0038 {
                            interrupt_pending = false;
                        }

                        states += snap.instr_cycles as u64;
                        if debug_enabled {
                            last_snapshot = Some(snap);
                        }
                    }
                    pantalla.on_vsync();
                }
            }

            RunMode::Paused => {}
            _ => {}
        }

        // ===================== RENDER =====================
        pantalla.update_from_bus(&cpu.bus);

        gui::draw_debug(
            &mut debug_canvas,
            &font,
            if debug_enabled { last_snapshot.as_ref() } else { None },
            &stack_tracker,
            load_state,
            debug_enabled,
        )?;
        debug_canvas.present();

        gui::draw_zx_screen(&mut zx_canvas, &pantalla)?;
        zx_canvas.present();

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }

    Ok(())
}

