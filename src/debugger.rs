#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RunMode {
    Step,
    Run,        // Con render
    RunFast,    // Sin render
    Paused,
}

/* ==================================================
 * BREAKPOINT (POR AHORA FIJO)
 * ================================================== */

//pub const BREAKPOINT_ADDR: u16 = 0x0038;
pub const BREAKPOINT_ADDR: u16 = 0xFFFF; // sin Breakpoint
//pub const BREAKPOINT_ADDR: u16 = 0x8000;

/* ==================================================
 * DEBUGGER
 * ================================================== */

pub struct Debugger {
    pub mode: RunMode,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            mode: RunMode::Paused,
        }
    }

    /// Devuelve true si hay que parar antes de ejecutar la instrucción
    pub fn check_breakpoint(&mut self, pc: u16) -> bool {
        match self.mode {
            RunMode::Run | RunMode::RunFast => {
                if pc == BREAKPOINT_ADDR {
                    self.mode = RunMode::Paused;
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    pub fn run(&mut self) {
        self.mode = RunMode::Run;
    }

    pub fn step(&mut self) {
        self.mode = RunMode::Step;
    }

    pub fn pause(&mut self) {
        self.mode = RunMode::Paused;
    }

    pub fn run_fast(&mut self) {
        self.mode = RunMode::RunFast;
    }
}
