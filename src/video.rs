use zilog_z80::bus::Bus;

pub struct Video {
    /// Buffer lógico ZX: 256x192, 1 byte = 1 pixel (0 o 1)
    pub pixels: Vec<u8>,
    /// Buffer de colores (atributos)
    pub attrs: Vec<u8>,       // 32×24
    /// Escalado: cuántos píxeles de ventana por píxel ZX
    pub scale: u32,

    // FLASH (color que hace flash cada ~0,32 s (≈ 16 frames a 50 Hz))
    pub flash_counter: u32,
    pub flash_phase: bool, // false = normal, true = invertido
}
impl Video {
    pub fn new(scale: u32) -> Self {
        Self {
            pixels: vec![0; 256 * 192],
            attrs: vec![0; 32 * 24],
            scale,
            flash_counter: 0,
            flash_phase: false,
        }
    }

    // controla el flash de colores
    pub fn tick_flash(&mut self) {
        self.flash_counter += 1;

        if self.flash_counter >= 16 {
            self.flash_counter = 0;
            self.flash_phase = !self.flash_phase;
        }
    }

    /// Actualiza el buffer de vídeo leyendo la RAM del ZX Spectrum
    /// - Bitmap:  $4000–$57FF (256×192 píxeles)
    /// - Atributos:$5800–$5AFF (32×24 celdas)
    pub fn update_from_bus(&mut self, bus: &Bus) {
        // --------------------------------------------------
        // 1) BITMAP (PIXELS)
        // --------------------------------------------------
        // 192 líneas, 32 bytes por línea (256 píxeles)
        for y in 0..192 {
            for x_byte in 0..32 {
                let addr = zx_screen_addr(x_byte, y);
                let byte = bus.read_byte(addr);
                self.store_pixel_byte(x_byte, y, byte);
            }
        }

        // --------------------------------------------------
        // 2) ATRIBUTOS (COLORES)
        // --------------------------------------------------
        // 24 filas × 32 columnas = 768 bytes
        for ay in 0..24 {
            for ax in 0..32 {
                let addr = 0x5800u16 + (ay * 32 + ax) as u16;
                self.attrs[ay * 32 + ax] = bus.read_byte(addr);
            }
        }
    }
    fn store_pixel_byte(&mut self, x_byte: usize, y: usize, byte: u8) {
        // Cada byte representa 8 píxeles
        for bit in 0..8 {
            let pixel_x = x_byte * 8 + (7 - bit); // MSB primero
            let pixel_y = y;

            if pixel_x >= 256 || pixel_y >= 192 {
                continue;
            }

            let idx = pixel_y * 256 + pixel_x;

            let pixel_on = (byte >> bit) & 1;
            self.pixels[idx] = pixel_on;
        }
    }

    // Usado en flash
    pub fn on_vsync(&mut self) {
        self.flash_counter += 1;

        if self.flash_counter == 16 {
            self.flash_counter = 0;
            self.flash_phase = !self.flash_phase;
        }
    }
}
/// Convierte coordenadas ZX (x_byte: 0..31, y: 0..191)
/// en la dirección real de memoria de pantalla ($4000-$57FF)
fn zx_screen_addr(x_byte: usize, y: usize) -> u16 {
    let y = y as u16;
    let x = x_byte as u16;

    let row = (y & 0b0000_0111) << 8;      // bits 0-2 → A8-A10
    let block = (y & 0b0011_1000) << 2;   // bits 3-5 → A5-A7
    let band = (y & 0b1100_0000) << 5;    // bits 6-7 → A11-A12

    0x4000 | band | row | block | x
}
