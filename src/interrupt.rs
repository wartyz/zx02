pub struct InterruptController {
    pub tstates_accum: u64,
    pub next_int: u64,
}

impl InterruptController {
    pub fn new() -> Self {
        Self {
            tstates_accum: 0,
            next_int: 69888, // 50 Hz reales del Spectrum
        }
    }

    /// Devuelve true cuando hay que generar una INT
    pub fn add_cycles(&mut self, cycles: u32) -> bool {
        self.tstates_accum += cycles as u64;

        if self.tstates_accum >= self.next_int {
            self.tstates_accum -= self.next_int;
            true // generar INT
        } else {
            false
        }
    }
}
