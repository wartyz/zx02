use sdl2::keyboard::Keycode;
use zilog_z80::bus::Bus;

/// Variables del sistema ZX Spectrum
const LAST_K: u16 = 0x5C3A;
const FLAGS: u16 = 0x5C08;

/// Bit 5 de FLAGS = "new key"
const FLAG_NEW_KEY: u8 = 1 << 5;

pub struct Keyboard {
    //rows: [u8; 8], // 8 filas, 5 bits usados (activos en 0)
    last_key: u8,     // código Spectrum
    key_pressed: bool,
}
impl Keyboard {
    pub fn new() -> Self {
        Self {
            //rows: [0b0001_1111; 8], // ninguna tecla pulsada
            last_key: 0xFF, // no key
            key_pressed: false,
        }
    }

    /// Llamar cuando se pulse una tecla del PC
    pub fn key_down(&mut self, key: Keycode) {
        if let Some(code) = map_pc_to_spectrum(key) {
            self.last_key = code;
            self.key_pressed = true;
        }
    }

    /// Llamar cuando se suelte una tecla del PC
    pub fn key_up(&mut self) {
        self.last_key = 0xFF;
        self.key_pressed = false;
    }

    /// ⬅️ ESTA ES LA CLAVE
    /// Escribe el estado del teclado en la RAM del Z80
    pub fn apply_to_bus(&self, bus: &mut Bus) {
        if self.key_pressed {
            // SOLO escribir si la ROM aún no ha consumido la tecla
            if bus.read_byte(0x5C3A) == 0xFF {
                bus.write_byte(0x5C3A, self.last_key);

                let mut flags = bus.read_byte(0x5C08);
                flags |= 1 << 5; // new key
                bus.write_byte(0x5C08, flags);
            }
        } else {
            bus.write_byte(0x5C3A, 0xFF);

            let mut flags = bus.read_byte(0x5C08);
            flags &= !(1 << 5);
            bus.write_byte(0x5C08, flags);
        }
    }

    // filas activas en 0
    pub fn read_port_fe(&self, row_mask: u8) -> u8 {
        let mut result = 0b1111_1111;

        if !self.key_pressed {
            return result;
        }

        // ejemplo mínimo: fila 0 (Q A SPACE)
        if row_mask & 0b0000_0001 == 0 {
            match self.last_key {
                b'A' => result &= !(1 << 0),
                b'Q' => result &= !(1 << 1),
                b' ' => result &= !(1 << 4),
                _ => {}
            }
        }

        result
    }
}

/// Mapeo mínimo PC → Spectrum (ampliable)
fn map_pc_to_spectrum(key: Keycode) -> Option<u8> {
    Some(match key {
        Keycode::A => b'A',
        Keycode::B => b'B',
        Keycode::C => b'C',
        Keycode::D => b'D',
        Keycode::E => b'E',
        Keycode::F => b'F',
        Keycode::G => b'G',
        Keycode::H => b'H',
        Keycode::I => b'I',
        Keycode::J => b'J',
        Keycode::K => b'K',
        Keycode::L => b'L',
        Keycode::M => b'M',
        Keycode::N => b'N',
        Keycode::O => b'O',
        Keycode::P => b'P',
        Keycode::Q => b'Q',
        Keycode::R => b'R',
        Keycode::S => b'S',
        Keycode::T => b'T',
        Keycode::U => b'U',
        Keycode::V => b'V',
        Keycode::W => b'W',
        Keycode::X => b'X',
        Keycode::Y => b'Y',
        Keycode::Z => b'Z',

        Keycode::Space => b' ',
        Keycode::Return => 0x0D, // ENTER

        _ => return None,
    })
}