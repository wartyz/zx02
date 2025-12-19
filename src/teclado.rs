use sdl2::keyboard::Keycode;

pub struct Keyboard {
    rows: [u8; 8], // matriz 8×5 (bits activos en 0)
}
impl Keyboard {
    pub fn new() -> Self {
        Self {
            rows: [0xFF; 8],
        }
    }

    pub fn key_down(&mut self, row: usize, bit: u8) {
        self.rows[row] &= !(1 << bit);
    }

    pub fn key_up(&mut self, row: usize, bit: u8) {
        self.rows[row] |= 1 << bit;
    }

    pub fn read_port_fe(&self, port_high: u8) -> u8 {
        let mut value = 0xFF;

        for row in 0..8 {
            if (port_high & (1 << row)) == 0 {
                value &= self.rows[row];
            }
        }

        value
    }
}

pub fn map_key_down(kb: &mut Keyboard, key: Keycode) {
    match key {
        Keycode::A => kb.key_down(1, 0),
        Keycode::S => kb.key_down(1, 1),
        Keycode::D => kb.key_down(1, 2),
        Keycode::F => kb.key_down(1, 3),
        Keycode::G => kb.key_down(1, 4),

        Keycode::Q => kb.key_down(2, 0),
        Keycode::W => kb.key_down(2, 1),
        Keycode::E => kb.key_down(2, 2),
        Keycode::R => kb.key_down(2, 3),
        Keycode::T => kb.key_down(2, 4),

        Keycode::LShift | Keycode::RShift => kb.key_down(0, 0), // CAPS
        Keycode::LCtrl | Keycode::RCtrl => kb.key_down(7, 1), // SYM

        Keycode::Return => kb.key_down(6, 0),
        Keycode::Space => kb.key_down(7, 0),

        _ => {}
    }
}

pub fn map_key_up(kb: &mut Keyboard, key: Keycode) {
    match key {
        Keycode::A => kb.key_up(1, 0),
        Keycode::S => kb.key_up(1, 1),
        Keycode::D => kb.key_up(1, 2),
        Keycode::F => kb.key_up(1, 3),
        Keycode::G => kb.key_up(1, 4),

        Keycode::Q => kb.key_up(2, 0),
        Keycode::W => kb.key_up(2, 1),
        Keycode::E => kb.key_up(2, 2),
        Keycode::R => kb.key_up(2, 3),
        Keycode::T => kb.key_up(2, 4),

        Keycode::LShift | Keycode::RShift => kb.key_up(0, 0),
        Keycode::LCtrl | Keycode::RCtrl => kb.key_up(7, 1),

        Keycode::Return => kb.key_up(6, 0),
        Keycode::Space => kb.key_up(7, 0),

        _ => {}
    }
}
