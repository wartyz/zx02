use sdl2::keyboard::Keycode;

pub struct Keyboard {
    // 8 filas (una por cada bit del byte alto de la dirección del puerto)
    // Cada fila tiene 5 bits (bits 0-4). Un bit a 0 significa tecla PULSADA.
    pub rows: [u8; 8],

}
impl Keyboard {
    pub fn new() -> Self {
        Self {
            // Inicializamos con 0x1F (00011111), que significa "ninguna tecla pulsada"
            rows: [0x1F; 8],

        }
    }

    /// Llamar cuando se pulse una tecla del PC
    pub fn key_down(&mut self, key: Keycode) {
        self.update_key_state(key, true);
    }

    /// Llamar cuando se suelte una tecla del PC
    pub fn key_up(&mut self, key: Keycode) {
        self.update_key_state(key, false);
    }

    /// Actualiza el bit correspondiente en la matriz del Spectrum
    fn update_key_state(&mut self, key: Keycode, pressed: bool) {
        // Obtenemos fila y bit. Si la tecla no está mapeada, no hacemos nada.
        if let Some((row, bit)) = self.get_matrix_coords(key) {
            if pressed {
                self.rows[row] &= !(1 << bit); // Ponemos el bit a 0 (Pulsada)
            } else {
                self.rows[row] |= 1 << bit;    // Ponemos el bit a 1 (Soltada)
            }
        }
    }

    /// Devuelve el estado del puerto 0xFE basado en las filas seleccionadas (bits a 0 en high_byte)
    pub fn read_port_fe(&self, high_byte: u8) -> u8 {
        let mut result = 0x1F; // 5 bits bajos en 1

        // El Spectrum activa una fila si el bit correspondiente en high_byte es 0
        if (high_byte & 0x01) == 0 { result &= self.rows[0]; } // Caps a V
        if (high_byte & 0x02) == 0 { result &= self.rows[1]; } // A a G
        if (high_byte & 0x04) == 0 { result &= self.rows[2]; } // Q a T
        if (high_byte & 0x08) == 0 { result &= self.rows[3]; } // 1 a 5
        if (high_byte & 0x10) == 0 { result &= self.rows[4]; } // 0 a 6
        if (high_byte & 0x20) == 0 { result &= self.rows[5]; } // P a Y
        if (high_byte & 0x40) == 0 { result &= self.rows[6]; } // Enter a H
        if (high_byte & 0x80) == 0 { result &= self.rows[7]; } // Space a B

        result
    }

    /// Mapeo de teclas PC -> Matriz ZX Spectrum (Fila, Bit)
    fn get_matrix_coords(&self, key: Keycode) -> Option<(usize, u8)> {
        match key {
            // Fila 0: (FEFE) -> Caps, Z, X, C, V
            Keycode::LShift => Some((0, 0)),
            Keycode::Z => Some((0, 1)),
            Keycode::X => Some((0, 2)),
            Keycode::C => Some((0, 3)),
            Keycode::V => Some((0, 4)),

            // Fila 1: (FDFE) -> A, S, D, F, G
            Keycode::A => Some((1, 0)),
            Keycode::S => Some((1, 1)),
            Keycode::D => Some((1, 2)),
            Keycode::F => Some((1, 3)),
            Keycode::G => Some((1, 4)),

            // Fila 2: (FBFE) -> Q, W, E, R, T
            Keycode::Q => Some((2, 0)),
            Keycode::W => Some((2, 1)),
            Keycode::E => Some((2, 2)),
            Keycode::R => Some((2, 3)),
            Keycode::T => Some((2, 4)),

            // Fila 3: (F7FE) -> 1, 2, 3, 4, 5
            Keycode::Num1 => Some((3, 0)),
            Keycode::Num2 => Some((3, 1)),
            Keycode::Num3 => Some((3, 2)),
            Keycode::Num4 => Some((3, 3)),
            Keycode::Num5 => Some((3, 4)),

            // Fila 4: (EFFE) -> 0, 9, 8, 7, 6
            Keycode::Num0 => Some((4, 0)),
            Keycode::Num9 => Some((4, 1)),
            Keycode::Num8 => Some((4, 2)),
            Keycode::Num7 => Some((4, 3)),
            Keycode::Num6 => Some((4, 4)),

            // Fila 5: (DFFE) -> P, O, I, U, Y
            Keycode::P => Some((5, 0)),
            Keycode::O => Some((5, 1)),
            Keycode::I => Some((5, 2)),
            Keycode::U => Some((5, 3)),
            Keycode::Y => Some((5, 4)),

            // Fila 6: (BFFE) -> Enter, L, K, J, H
            Keycode::Return => Some((6, 0)),
            Keycode::L => Some((6, 1)),
            Keycode::K => Some((6, 2)),
            Keycode::J => Some((6, 3)),
            Keycode::H => Some((6, 4)),

            // Fila 7: (7FFE) -> Space, Sym, M, N, B
            Keycode::Space => Some((7, 0)),
            Keycode::LCtrl => Some((7, 1)), // Symbol Shift
            Keycode::M => Some((7, 2)),
            Keycode::N => Some((7, 3)),
            Keycode::B => Some((7, 4)),

            _ => None,
        }
    }
}

// Mapeo mínimo PC → Spectrum (ampliable)
/*fn map_pc_to_spectrum(key: Keycode) -> Option<u8> {
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
}*/