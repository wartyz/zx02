use crate::teclado::Keyboard;
use zilog_z80::bus::Bus;

pub struct ZxBus {
    pub mem: Vec<u8>,
    pub keyboard: Keyboard,
}

impl ZxBus {
    pub fn new() -> Self {
        Self {
            mem: vec![0; 65536],
            keyboard: Keyboard::new(),
        }
    }

    // -------------------------
    // MEMORIA
    // -------------------------
    pub fn read_byte(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        self.mem[addr as usize] = value;
    }

    // -------------------------
    // PUERTOS
    // -------------------------
    pub fn in_port(&self, port: u16) -> u8 {
        // ZX Spectrum: solo importa FEh
        if (port & 0x00FF) == 0xFE {
            let high = (port >> 8) as u8;
            return self.keyboard.read_port_fe(high);
        }
        0xFF
    }

    pub fn out_port(&mut self, _port: u16, _value: u8) {
        // De momento ignoramos (border, beeper)
    }
}


