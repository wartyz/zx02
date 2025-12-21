use crate::teclado::Keyboard;

pub struct ZxBus {
    pub mem: Vec<u8>,
    pub keyboard: Keyboard,
    border: u8,
}

impl ZxBus {
    pub fn new() -> Self {
        Self {
            mem: vec![0; 65536],
            keyboard: Keyboard::new(),
            border: 0,
        }
    }

    // -------------------------
    // LECTURA DE MEMORIA
    // -------------------------
    pub fn read_byte(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    // -------------------------
    // ESCRITURA DE MEMORIA
    // -------------------------
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        // PROTECCIÓN DE ROM:
        // Las direcciones 0x0000 a 0x3FFF son ROM. No se debe escribir en ellas.
        if addr >= 0x4000 {
            self.mem[addr as usize] = value;
        }
    }

    pub fn in_port(&mut self, port: u16) -> u8 {
        // En el Spectrum, el teclado se lee cuando el bit 0 del puerto es 0 (puerto 0xFE).
        if (port & 0x0001) == 0 {
            let high = (port >> 8) as u8;
            //let mut data = self.keyboard.read_port_fe(high);
            let keys = self.keyboard.read_port_fe(high);

            // Obligatorio para que la ROM no se confunda:
            // Bits 5 y 7 siempre a 1. Bit 6 (EAR) a 1.
            // data |= 0xE0;
            // return data;

            // Bits 5 y 7 a 1, Bit 6 (EAR) a 1 para evitar ruido de carga
            return (keys & 0x1F) | 0xE0;
        }

        // Bus flotante: por defecto devolvemos 0xFF (no hay nada conectado en otros puertos)
        0xFF
    }

    // -------------------------
    // SALIDA DE PUERTOS (OUT)
    // -------------------------
    pub fn out_port(&mut self, port: u16, value: u8) {
        // Si el bit 0 del puerto es 0, es una escritura a la ULA (Borde, Mic, Beeper)
        if (port & 0x0001) == 0 {
            // Los bits 0, 1 y 2 definen el color del borde (0-7)
            self.border = value & 0x07;

            // Debug para confirmar que funciona
            println!("Cambio de borde a color: {}", self.border);
        }
    }
}


