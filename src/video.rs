use zilog_z80::bus::Bus;

pub struct Video {
    /// Buffer final de imagen (256x192). Almacenamos el color real (0-15) de cada píxel.
    /// Esto facilita mucho el dibujo en SDL después.
    pub framebuffer: Vec<u8>,
    pub scale: u32,

    // FLASH (color que hace flash cada ~0,32 s)
    pub flash_counter: u32,
    pub flash_phase: bool,
}

impl Video {
    pub fn new(scale: u32) -> Self {
        Self {
            framebuffer: vec![0; 256 * 192],
            scale,
            flash_counter: 0,
            flash_phase: false,
        }
    }

    /// 🔁 Reset del estado temporal del vídeo (FLASH, etc.)
    pub fn reset_timing(&mut self) {
        self.flash_counter = 0;
        self.flash_phase = false;
    }
    
    /// Actualiza el framebuffer combinando píxeles y atributos
    pub fn update_from_bus(&mut self, bus: &Bus) {
        for y in 0..192 {
            for x_byte in 0..32 {
                // 1. Leer el byte de píxeles (8 píxeles horizontales)
                let pixel_addr = zx_screen_addr(x_byte, y);
                let pixel_byte = bus.read_byte(pixel_addr);

                // 2. Leer el byte de atributo correspondiente a esta celda de 8x8
                // La dirección de atributo es 0x5800 + (y/8 * 32) + x_byte
                let attr_addr = 0x5800 + ((y / 8) * 32 + x_byte) as u16;
                let attr = bus.read_byte(attr_addr);

                // 3. Extraer componentes del atributo
                let ink = attr & 0x07;            // Bits 0-2
                let paper = (attr >> 3) & 0x07;   // Bits 3-5
                let bright = (attr & 0x40) != 0; // Bit 6
                let flash = (attr & 0x80) != 0;  // Bit 7

                // 4. Aplicar brillo (colores 8-15)
                let mut ink_color = ink;
                let mut paper_color = paper;
                if bright {
                    ink_color += 8;
                    paper_color += 8;
                }

                // 5. Lógica de FLASH (invertir si toca)
                if flash && self.flash_phase {
                    std::mem::swap(&mut ink_color, &mut paper_color);
                }

                // 6. Dibujar los 8 píxeles en el framebuffer
                for bit in 0..8 {
                    let pixel_on = (pixel_byte & (0x80 >> bit)) != 0;
                    let final_color = if pixel_on { ink_color } else { paper_color };

                    let pixel_x = x_byte * 8 + bit;
                    self.framebuffer[y * 256 + pixel_x] = final_color;
                }
            }
        }
    }

    pub fn on_vsync(&mut self) {
        self.flash_counter += 1;
        // El Spectrum real cambia el flash cada 16 o 32 interrupciones
        if self.flash_counter >= 16 {
            self.flash_counter = 0;
            self.flash_phase = !self.flash_phase;
        }
    }
}

/// Direccionamiento entrelazado del Spectrum
fn zx_screen_addr(x_byte: usize, y: usize) -> u16 {
    let y = y as u16;
    let x = x_byte as u16;

    // Formato dirección: 010 (banda) (row) (block) (x)
    // banda: bits 7,6 de y
    // row: bits 2,1,0 de y
    // block: bits 5,4,3 de y
    let band = (y & 0b1100_0000) << 5;
    let row = (y & 0b0000_0111) << 8;
    let block = (y & 0b0011_1000) << 2;

    0x4000 | band | row | block | x
}
