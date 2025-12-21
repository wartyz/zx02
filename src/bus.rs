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

    /*// -------------------------
    // ENTRADA DE PUERTOS (IN)
    // -------------------------
    pub fn in_port(&self, port: u16) -> u8 {
        // En el Spectrum, el puerto fundamental es el 0xFE (254)
        if (port & 0x00FF) == 0xFE {
            let high = (port >> 8) as u8;

            // IMPORTANTE: Si keyboard.read_port_fe devuelve 0, la ROM se cuelga.
            // Asegúrate de que devuelva 0xFF si no hay teclas pulsadas.
            dbg!(self.keyboard.read_port_fe(high));
            return self.keyboard.read_port_fe(high);
        }
        // Bus flotante: por defecto devolvemos 0xFF
        0xFF
    }*/

    // -------------------------
    // ENTRADA DE PUERTOS (IN)
    // -------------------------
    // pub fn in_port(&self, port: u16) -> u8 {
    //     let port_low = (port & 0x00FF) as u8;
    //
    //     // El Spectrum decodifica el teclado cuando el bit 0 de la dirección de puerto es 0.
    //     // Aunque la forma más común es comprobar si es 0xFE.
    //     if port_low == 0xFE {
    //         let high = (port >> 8) as u8;
    //
    //         // Obtenemos el estado de las semi-filas del teclado
    //         let mut row_data = self.keyboard.read_port_fe(high);
    //
    //         // --- AJUSTES DE COMPATIBILIDAD ---
    //         // 1. Forzamos los bits 5 y 7 a 1 (no se usan en el teclado estándar)
    //         // 2. El bit 6 es el puerto EAR (cassette). Lo ponemos en 1 para evitar ruido.
    //         // Si row_data es p.ej. 0x1F (ninguna tecla), esto lo convierte en 0xBF.
    //         row_data |= 0b10100000;
    //
    //         // Si el bit 6 debe estar alto por defecto (típico para que la ROM no detecte carga):
    //         row_data |= 0x40;
    //
    //         return row_data;
    //     }
    //
    //     // Bus flotante: por defecto devolvemos 0xFF (no hay nada en ese puerto)
    //     0xFF
    // }

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
    pub fn out_port(&mut self, _port: u16, _value: u8) {
        // Aquí llegarán las instrucciones de Borde (bits 0-2)
        // y Beeper (bit 4). Por ahora solo ignoramos para avanzar.
    }
}


