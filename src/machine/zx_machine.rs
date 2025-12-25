use std::collections::HashMap;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;
use zilog_z80::cpu::CPU;

use crate::bus::ZxBus;
use crate::constantes::TSTATES_PER_FRAME;
use crate::cpu_exec::{step, CpuRunState, UnimplTracker};
use crate::interrupt::InterruptController;
use crate::stack_tracker::StackTracker;
use crate::video::Video;
use crate::debugger::{Debugger, RunMode};
use crate::formatos::load;
use crate::formatos::load::LoadResult;
use crate::LoadState;

/// Estado completo de la máquina ZX Spectrum
pub struct ZxMachine {
    // CPU y BUS
    pub cpu: CPU,
    pub bus: ZxBus,

    // Estado de ejecución
    pub run_state: CpuRunState,
    pub interrupt_ctrl: InterruptController,
    pub interrupt_pending: bool,

    // Debug / tracking
    pub debugger: Debugger,

    pub executed_instrs: HashMap<u16, (u8, String)>,
    pub unimpl_tracker: UnimplTracker,
    pub last_snapshot: Option<crate::cpu_exec::CpuSnapshot>,
    pub stack_tracker: StackTracker,

    // Video
    pub video: Video,

    pub debug_enabled: bool,
    pub load_state: LoadState,
}

impl ZxMachine {
    /// Crea una máquina ZX exactamente igual a la inicialización actual del main
    pub fn new(video_scale: u32) -> Self {
        Self {
            cpu: CPU::new(0xFFFF),
            bus: ZxBus::new(),

            run_state: CpuRunState::new(),
            interrupt_ctrl: InterruptController::new(),
            interrupt_pending: false,

            debugger: Debugger::new(),
            executed_instrs: HashMap::new(),
            unimpl_tracker: UnimplTracker::new(),
            last_snapshot: None,
            stack_tracker: StackTracker::new(512),

            video: Video::new(video_scale),

            debug_enabled: false,
            load_state: LoadState::None,

        }
    }

    /// Ejecuta CPU según el modo actual (Run / RunFast)
    pub fn run_frame(&mut self) {
        match self.debugger.mode {
            /* ===========================
             * RUN: 1 frame (50 Hz)
             * =========================== */
            RunMode::Run => {
                let mut states: u64 = 0;

                while states < TSTATES_PER_FRAME {
                    if self.debugger.check_breakpoint(self.cpu.reg.pc) {
                        self.debugger.pause();
                        break;
                    }

                    let snap = step(
                        &mut self.cpu,
                        &mut self.bus,
                        &mut self.run_state,
                        self.interrupt_pending,
                        &mut self.executed_instrs,
                        &mut self.unimpl_tracker,
                        &mut self.stack_tracker,
                        false,
                    );

                    if self.interrupt_ctrl.add_cycles(snap.instr_cycles) {
                        self.interrupt_pending = true;
                    }

                    if self.interrupt_pending && self.cpu.reg.pc == 0x0038 {
                        self.interrupt_pending = false;
                    }

                    states += snap.instr_cycles as u64;

                    if self.debug_enabled {
                        self.last_snapshot = Some(snap);
                    }
                }

                self.video.on_vsync();
            }

            /* ===========================
             * RUNFAST: 10 frames seguidos
             * =========================== */
            RunMode::RunFast => {
                for _ in 0..10 {
                    let mut states: u64 = 0;

                    while states < TSTATES_PER_FRAME {
                        if self.debugger.check_breakpoint(self.cpu.reg.pc) {
                            self.debugger.pause();
                            break;
                        }

                        let snap = step(
                            &mut self.cpu,
                            &mut self.bus,
                            &mut self.run_state,
                            self.interrupt_pending,
                            &mut self.executed_instrs,
                            &mut self.unimpl_tracker,
                            &mut self.stack_tracker,
                            false,
                        );

                        if self.interrupt_ctrl.add_cycles(snap.instr_cycles) {
                            self.interrupt_pending = true;
                        }

                        if self.interrupt_pending && self.cpu.reg.pc == 0x0038 {
                            self.interrupt_pending = false;
                        }

                        states += snap.instr_cycles as u64;

                        if self.debug_enabled {
                            self.last_snapshot = Some(snap);
                        }
                    }

                    self.video.on_vsync();
                }
            }

            /* ===========================
             * PAUSE
             * =========================== */
            RunMode::Paused => {
                // No ejecutar CPU
            }

            _ => {}
        }
    }

    pub fn draw_debug(
        &mut self,
        canvas: &mut Canvas<Window>,
        font: &Font,
    ) -> Result<(), String> {
        crate::gui::draw_debug(
            canvas,
            font,
            if self.debug_enabled {
                self.last_snapshot.as_ref()
            } else {
                None
            },
            &self.stack_tracker,
            self.load_state,
            self.debug_enabled,
        )
    }

    /// Actualiza el framebuffer de vídeo a partir del bus
    pub fn update_video_from_bus(&mut self) {
        self.video.update_from_bus(&self.cpu.bus);
    }

    pub fn draw_zx_screen(
        &mut self,
        canvas: &mut Canvas<Window>,
    ) -> Result<(), String> {
        crate::gui::draw_zx_screen(canvas, &self.video)
    }

    pub fn step_once(&mut self) {
        let snap = step(
            &mut self.cpu,
            &mut self.bus,
            &mut self.run_state,
            self.interrupt_pending,
            &mut self.executed_instrs,
            &mut self.unimpl_tracker,
            &mut self.stack_tracker,
            true,
        );

        if self.debug_enabled {
            self.last_snapshot = Some(snap);
        }
    }

    pub fn load_file(&mut self, kind: LoadResult) {
        // Estado común tras cualquier carga
        self.interrupt_pending = false;
        self.interrupt_ctrl = InterruptController::new();
        self.last_snapshot = None;
        self.run_state.halted = false;

        // Decidir estado según lo cargado
        self.load_state = match kind {
            LoadResult::Rom => LoadState::Rom,
            LoadResult::Sna => LoadState::Sna,
            LoadResult::Z80 => LoadState::Z80,
            LoadResult::Bin => LoadState::Bin,
        };

        // Política de debug (una sola para todos, como pedías)
        // 🔹 aquí puedes cambiar el criterio cuando quieras
        // por ahora: NO tocar debug_enabled
        // self.debug_enabled = self.debug_enabled;

        println!("ZxMachine: carga completada -> {:?}", self.load_state);
    }

    pub fn load_from_dialog(&mut self) -> Result<(), String> {
        let kind = load::load_file_dialog(&mut self.cpu, &mut self.run_state)?;

        self.on_file_loaded(kind);

        Ok(())
    }

    fn on_file_loaded(&mut self, kind: LoadResult) {
        // Estado común tras cualquier carga
        self.interrupt_pending = false;
        self.interrupt_ctrl = InterruptController::new();
        self.last_snapshot = None;
        self.run_state.halted = false;

        // Estado visual / lógico
        self.load_state = match kind {
            LoadResult::Rom => LoadState::Rom,
            LoadResult::Sna => LoadState::Sna,
            LoadResult::Z80 => LoadState::Z80,
            LoadResult::Bin => LoadState::Bin,
        };

        println!("ZxMachine: cargado {:?}", self.load_state);
    }
}


