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
mod machine;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

//use std::collections::HashMap;
use std::time::{Duration, Instant};

use botones::ButtonAction;

use crate::constantes::{ALTO_VENTANA, ANCHO_VENTANA, ESCALA_PANTALLA_ZX, ESCALA_VENTANA_ZX};
use crate::machine::zx_machine::ZxMachine;

#[derive(Copy, Clone, Debug)]
pub enum LoadState {
    None,
    Rom,
    Sna,
    Z80,
    Bin,
}

fn main() -> Result<(), String> {
    //let mut load_state = LoadState::None;
    //let mut debug_enabled = false;

    let mut machine = ZxMachine::new(ESCALA_VENTANA_ZX);

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
                    machine.bus.keyboard.key_down(k);
                }
                Event::KeyUp { keycode: Some(k), .. } => {
                    machine.bus.keyboard.key_up(k);
                }

                Event::MouseButtonDown { x, y, .. } => {
                    for b in botones::default_buttons() {
                        if b.contains(x, y) {
                            match b.action {
                                ButtonAction::Step => {
                                    // let snap = step(
                                    //     &mut machine.cpu,
                                    //     &mut machine.bus,
                                    //     &mut machine.run_state,
                                    //     machine.interrupt_pending,
                                    //     &mut machine.executed_instrs,
                                    //     &mut machine.unimpl_tracker,
                                    //     &mut machine.stack_tracker,
                                    //     true,
                                    // );
                                    // if machine.debug_enabled {
                                    //     machine.last_snapshot = Some(snap);
                                    // }

                                    machine.step_once();
                                }

                                ButtonAction::Run => machine.debugger.run(),
                                ButtonAction::RunFast => machine.debugger.run_fast(),
                                ButtonAction::Pause => machine.debugger.pause(),

                                ButtonAction::Load => {
                                    // match load::load_file_dialog(&mut machine.cpu, &mut machine.run_state) {
                                    //     Ok(kind) => {
                                    //         machine.load_file(kind);
                                    //     }
                                    //     Err(e) => {
                                    //         println!("Carga cancelada o error: {}", e);
                                    //     }
                                    // }

                                    if let Err(e) = machine.load_from_dialog() {
                                        println!("Carga cancelada o error: {}", e);
                                    }
                                }
                                ButtonAction::DebugToggle => {
                                    machine.debug_enabled = !machine.debug_enabled;

                                    if !machine.debug_enabled {
                                        machine.debugger.pause();
                                        machine.last_snapshot = None;
                                    }

                                    println!(
                                        "DEBUG {}",
                                        if machine.debug_enabled { "ON" } else { "OFF" });
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

        machine.run_frame();

        // ===================== RENDER =====================
        machine.update_video_from_bus();

        machine.draw_debug(&mut debug_canvas, &font)?;
        debug_canvas.present();

        machine.draw_zx_screen(&mut zx_canvas)?;
        zx_canvas.present();

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }

    Ok(())
}

