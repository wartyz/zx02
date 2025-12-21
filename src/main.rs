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

use cpu_exec::{init_cpu, step, UnimplTracker};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use debugger::{Debugger, RunMode};

use botones::ButtonAction;
use stack_tracker::StackTracker;
use video::Video;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::bus::ZxBus;
use crate::cpu_exec::CpuRunState;
use crate::interrupt::InterruptController;

const TSTATES_PER_FRAME: u64 = 69888;
const ANCHO_VENTANA: u32 = 3800;
const ALTO_VENTANA: u32 = 2800;
const ESCALA_VENTANA_ZX: u32 = 4;
const ESCALA_PANTALLA_ZX: u32 = 12;

fn main() -> Result<(), String> {
    // 1. Creamos el Bus y el Teclado
    let mut zx_bus = ZxBus::new();

    // 2. Inicializamos la CPU con la ROM
    let mut cpu = init_cpu("ROMS/ZXSpectrum48.rom");

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

    let mut debug_canvas = debug_window.into_canvas().accelerated().present_vsync().build().map_err(|e| e.to_string())?;
    let font = ttf.load_font("FONTS/DejaVuSansMono.ttf", 16)?;

    let zx_window = video_sub
        .window("ZX Spectrum", 256 * ESCALA_PANTALLA_ZX, 192 * ESCALA_PANTALLA_ZX)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut zx_canvas = zx_window.into_canvas().accelerated().present_vsync().build().map_err(|e| e.to_string())?;
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
                                    );
                                    last_snapshot = Some(snap);
                                }
                                ButtonAction::Run => debugger.run(),
                                ButtonAction::RunFast => debugger.run_fast(),
                                ButtonAction::Pause => debugger.pause(),
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
                    // FORZAR TECLA J (Fila 6, Bit 3) para probar:
                    //zx_bus.keyboard.rows[6] &= !(1 << 3);
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
                //interrupt_pending = true;
                //run_state.t_states += states_en_este_frame;

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
                //pantalla.on_vsync();
                pantalla.update_from_bus(&cpu.bus);
            }

            RunMode::Paused => {
                // En pausa no hacemos nada, la CPU se queda donde está
            }

            _ => {}
        }

        // Sincronizar Video y Render
        pantalla.update_from_bus(&cpu.bus);
        gui::draw_debug(&mut debug_canvas, &font, last_snapshot.as_ref(), &stack_tracker)?;
        gui::draw_zx_screen(&mut zx_canvas, &pantalla)?;

        // Presentar los cambios!
        debug_canvas.present();
        zx_canvas.present();

        // Pequeño delay para no consumir el 100% de la CPU del PC
        //std::thread::sleep(std::time::Duration::from_millis(16));

        // ESPERAR para clavar los 50 FPS
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
    Ok(())
}
