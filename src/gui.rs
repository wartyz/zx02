use sdl2::{
    pixels::Color,
    rect::Rect,
    render::Canvas,
    ttf::Font,
    video::Window,
};

use crate::botones;
use crate::disasm::disassemble;
use crate::cpu_exec::CpuSnapshot;
use crate::botones::{Button, ButtonAction};
use crate::stack_tracker::{StackTracker, StackWriteKind};
use crate::video::Video;
/* ================================================== */
/* ENTRY POINT GUI                                    */
/* ================================================== */

const ZX_W: i32 = 256; // Ancho pantalla ZX
const ZX_H: i32 = 192;  // Alto pantall ZX
//const ZX_SCALE: i32 = 4; //Escala multiplocxadora tamaño pixeles de ZX
const ZX_BORDER: i32 = 16; // marco negro alrededor en pantalla ZX
//const X_PANTALLA: i32 = 520;
//const Y_PANTALLA: i32 = 360;
const MARGEN_NEGRO: i32 = 20;

// Solo para presentar datos de DEBUG en pantalla de DEBUG
pub fn draw_debug(
    canvas: &mut Canvas<Window>,
    font: &Font,
    snapshot: Option<&CpuSnapshot>,
    stack_tracker: &StackTracker,
) -> Result<(), String> {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    if let Some(s) = snapshot {
        draw_registers(canvas, font, s)?;
        draw_flags(canvas, font, s)?;
        draw_memory_dump(canvas, font, s)?;
        draw_instruction_window(canvas, font, s)?;
        draw_stack(canvas, font, s, stack_tracker, 360, 260)?;
    }

    draw_buttons(canvas, font, &botones::default_buttons())?;
    canvas.present();
    Ok(())
}

// Solo para dibujar la pantalla de ZX Spectrum
pub fn draw_zx_screen(
    canvas: &mut Canvas<Window>,
    video: &Video,
) -> Result<(), String> {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    // La pantalla ZX siempre empieza en (0,0)
    draw_screen(canvas, video, MARGEN_NEGRO, MARGEN_NEGRO)?;

    canvas.present();
    Ok(())
}

fn draw_registers(
    canvas: &mut Canvas<Window>,
    font: &Font,
    s: &CpuSnapshot,
) -> Result<(), String> {
    let x1 = 20;
    let x2 = 180;
    let x3 = 360;
    let y0 = 80;
    let dy = 20;

    // --------------------------------------------------
    // REGISTROS PRINCIPALES
    // --------------------------------------------------
    draw_text(canvas, font, &format!("PC: {:04X}", s.pc), x1, y0)?;
    draw_text(canvas, font, &format!("SP : {:04X}", s.sp), x2, y0)?;

    draw_text(canvas, font, &format!("AF: {:04X}", s.af), x1, y0 + dy)?;
    draw_text(canvas, font, &format!("AF': {:04X}", s.af_), x2, y0 + dy)?;

    draw_text(canvas, font, &format!("BC: {:04X}", s.bc), x1, y0 + 2 * dy)?;
    draw_text(canvas, font, &format!("BC': {:04X}", s.bc_), x2, y0 + 2 * dy)?;

    draw_text(canvas, font, &format!("DE: {:04X}", s.de), x1, y0 + 3 * dy)?;
    draw_text(canvas, font, &format!("DE': {:04X}", s.de_), x2, y0 + 3 * dy)?;

    draw_text(canvas, font, &format!("HL: {:04X}", s.hl), x1, y0 + 4 * dy)?;
    draw_text(canvas, font, &format!("HL': {:04X}", s.hl_), x2, y0 + 4 * dy)?;

    // --------------------------------------------------
    // REGISTROS DE INTERRUPCIÓN
    // --------------------------------------------------
    draw_text(canvas, font, &format!("I: {:02X}", s.i), x3, y0 + dy)?;
    draw_text(canvas, font, &format!("R: {:02X}", s.r), x3, y0 + 2 * dy)?;

    // (opcional, si luego lo guardas)
    // draw_text(canvas, font, &format!("IM: {}", s.im), x3, y0 + 3 * dy)?;

    Ok(())
}

/* ================================================== */
/* FLAGS (DEBAJO DE F:)                               */
/* ================================================== */

fn draw_flags(
    canvas: &mut Canvas<Window>,
    font: &Font,
    s: &CpuSnapshot,
) -> Result<(), String> {
    let f = s.f;
    let bit = |b: u8| if (f & (1u8 << b)) != 0 { '1' } else { '0' };

    draw_text(canvas, font, "F:", 20, 230)?;
    draw_text(canvas, font, "S  Z  H  P  N  C", 60, 230)?;
    draw_text(
        canvas,
        font,
        &format!(
            "{}  {}  {}  {}  {}  {}",
            bit(7), bit(6), bit(4), bit(2), bit(1), bit(0)
        ),
        60,
        260,
    )?;
    Ok(())
}

/* ================================================== */
/* MEMORY DUMP (AJUSTADO A PANTALLA)                  */
/* ================================================== */

fn draw_memory_dump(
    canvas: &mut Canvas<Window>,
    font: &Font,
    s: &CpuSnapshot,
) -> Result<(), String> {
    let start_x = 560;
    let start_y = 20;
    let line_h = 20;
    let bytes_per_line = 16;

    for row in 0..16 {
        let addr = s.mem_base.wrapping_add((row * bytes_per_line) as u16);
        let y = start_y + row * line_h;

        draw_text(canvas, font, &format!("{:04X}:", addr), start_x, y)?;

        for col in 0..bytes_per_line {
            let index = (row * bytes_per_line + col) as usize;
            if index >= s.mem_dump.len() {
                continue;
            }

            let byte_addr = addr.wrapping_add(col as u16);
            let byte = s.mem_dump[index];

            let color = if byte_addr == s.pc {
                Color::RGB(255, 255, 0)
            } else {
                Color::WHITE
            };

            draw_text_color(
                canvas,
                font,
                &format!("{:02X}", byte),
                start_x + 68 + col * 36,
                y,
                color,
            )?;
        }
    }

    Ok(())
}

/* ================================================== */
/* INSTRUCTION WINDOW (21 LÍNEAS, CENTRADA EN PC)     */
/* ================================================== */
fn draw_instruction_window(
    canvas: &mut Canvas<Window>,
    font: &Font,
    s: &CpuSnapshot,
) -> Result<(), String> {
    let start_x = 20;
    let start_y = 360;
    let line_h = 22;

    // 1) Elegimos un punto seguro ANTERIOR
    let mut pc = s.pc.saturating_sub(40);

    // 2) Desensamblamos hacia delante
    let mut instrs = Vec::new();

    while instrs.len() < 40 {
        let (mnemonic, len) = disassemble(&s.mem_dump, pc, s.mem_base);
        instrs.push((pc, mnemonic, len));
        pc = pc.wrapping_add(len as u16);

        if pc == 0 || pc == 0xFFFF {
            break;
        }
    }

    // 3) Encontrar la instrucción actual
    let current_index = instrs
        .iter()
        .position(|(addr, _, _)| *addr == s.pc)
        .unwrap_or(0);

    // 4) Calcular ventana [10 antes, actual, 10 después]
    let start = current_index.saturating_sub(10);
    let end = (start + 21).min(instrs.len());

    let mut y = start_y;

    for i in start..end {
        let (pc, mnemonic, len) = &instrs[i];

        // Bytes de la instrucción
        let mut bytes = String::new();
        for j in 0..*len {
            let addr = pc.wrapping_add(j as u16);
            let index = addr.wrapping_sub(s.mem_base) as usize;
            if index < s.mem_dump.len() {
                bytes.push_str(&format!("{:02X} ", s.mem_dump[index]));
            }
        }

        let color = if *pc == s.pc {
            Color::RGB(255, 255, 0)
        } else {
            Color::WHITE
        };

        //let text = format!("{:04X}: {:<18}    {}", pc, bytes, mnemonic);

        let text = format!("{:04X}: {:<10}    {}", pc, bytes, mnemonic);

        draw_text_color(canvas, font, &text, start_x, y, color)?;
        y += line_h;
    }

    Ok(())
}

/* ================================================== */
/* UNA LÍNEA DE INSTRUCCIÓN                           */
/* ================================================== */

fn draw_instruction_line(
    canvas: &mut Canvas<Window>,
    font: &Font,
    s: &CpuSnapshot,
    pc: u16,
    y: i32,
    color: Color,
) -> Result<(), String> {
    // Desensamblar SIEMPRE
    let (mnemonic, len) = disassemble(&s.mem_dump, pc, s.mem_base);

    // Bytes de la instrucción
    let mut bytes = String::new();
    for i in 0..len {
        let addr = pc.wrapping_add(i as u16);
        let index = addr.wrapping_sub(s.mem_base) as usize;
        if index < s.mem_dump.len() {
            bytes.push_str(&format!("{:02X} ", s.mem_dump[index]));
        }
    }

    let text = format!(
        "{:04X}: {:<18}    {}",
        pc,
        bytes,
        mnemonic
    );

    draw_text_color(canvas, font, &text, 20, y, color)?;
    Ok(())
}

/* ================================================== */
/* TEXT HELPERS                                       */
/* ================================================== */

fn draw_text(
    canvas: &mut Canvas<Window>,
    font: &Font,
    text: &str,
    x: i32,
    y: i32,
) -> Result<(), String> {
    draw_text_color(canvas, font, text, x, y, Color::WHITE)
}

fn draw_text_color(
    canvas: &mut Canvas<Window>,
    font: &Font,
    text: &str,
    x: i32,
    y: i32,
    color: Color,
) -> Result<(), String> {
    let surface = font
        .render(text)
        .blended(color)
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    let rect = surface.rect();
    let target = Rect::new(x, y, rect.width(), rect.height());

    canvas.copy(&texture, None, Some(target))?;
    Ok(())
}

// fn instr_len_from_dump(s: &CpuSnapshot, pc: u16) -> u8 {
//     let index = pc.wrapping_sub(s.mem_base) as usize;
//     if index >= s.mem_dump.len() {
//         return 1;
//     }
// 
//     let b0 = s.mem_dump[index];
// 
//     match b0 {
//         // Prefijos
//         0xCB | 0xED => 2,
// 
//         0xDD | 0xFD => {
//             if index + 1 < s.mem_dump.len() && s.mem_dump[index + 1] == 0xCB {
//                 4
//             } else {
//                 2
//             }
//         }
// 
//         // Instrucciones de 2 bytes (LD r,n, ALU inmediatas, I/O)
//         0x06 | 0x0E | 0x16 | 0x1E |
//         0x26 | 0x2E | 0x36 | 0x3E |   // ✅ 0x36 AÑADIDO AQUÍ
//         0xC6 | 0xD6 | 0xE6 | 0xF6 |
//         0xCE | 0xDE | 0xEE | 0xFE |
//         0xD3 | 0xDB => 2,
// 
//         // Instrucciones de 3 bytes
//         0x01 | 0x11 | 0x21 | 0x31 |
//         0xC3 | 0xCD |
//         0x22 | 0x2A | 0x32 | 0x3A => 3,
// 
//         _ => 1,
//     }
// }

// Dibuja los botones para debugger
pub fn draw_buttons(
    canvas: &mut Canvas<Window>,
    font: &Font,
    buttons: &[Button],
) -> Result<(), String> {
    for b in buttons {
        // Rectángulo del botón
        let rect = Rect::new(b.x, b.y, b.w as u32, b.h as u32);

        // Color fondo
        canvas.set_draw_color(Color::RGB(60, 60, 60));
        canvas.fill_rect(rect)?;

        // Borde
        canvas.set_draw_color(Color::RGB(200, 200, 200));
        canvas.draw_rect(rect)?;

        // Texto del botón
        let label = match b.action {
            ButtonAction::Step => "STEP",
            ButtonAction::Run => "RUN",
            ButtonAction::RunFast => "FAST",
            ButtonAction::Pause => "PAUSE",
            ButtonAction::Reset => "RESET",
        };

        let surface = font
            .render(label)
            .blended(sdl2::pixels::Color::RGB(255, 255, 255))
            .map_err(|e| e.to_string())?;

        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        let text_rect = Rect::new(
            b.x + (b.w - surface.width() as i32) / 2,
            b.y + (b.h - surface.height() as i32) / 2,
            surface.width(),
            surface.height(),
        );

        canvas.copy(&texture, None, Some(text_rect))?;
    }

    Ok(())
}

//    Color	          Significado
//    -------------------------------------------
// 🟢 Verde fuerte	  SP actual
// 🟢 Verde normal	  escrito por CALL
// 🔵 Azul	          PUSH
// 🔴 Rojo	          interrupción
// 🟠 Naranja	      escritura manual
//    Gris            desconocido / nunca escrito
fn draw_stack(
    canvas: &mut Canvas<Window>,
    font: &Font,
    s: &CpuSnapshot,
    stack_tracker: &StackTracker,
    x: i32,
    y: i32,
) -> Result<(), String> {
    const LINE_H: i32 = 18;

    draw_text(canvas, font, "STACK", x, y)?;

    for (i, val) in s.stack_dump.iter().enumerate() {
        let addr = s.stack_base.wrapping_add(i as u16);

        // ─────────────────────────────────────────────
        // Prefijo visual del SP
        // ─────────────────────────────────────────────
        let sp_marker = if addr == s.sp {
            "-> "
        } else {
            "   "
        };

        let text = format!("{}{:04X}: {:02X}", sp_marker, addr, val);

        // ─────────────────────────────────────────────
        // Color según origen de la escritura
        // ─────────────────────────────────────────────
        let color = match stack_tracker.last_write_to(addr) {
            Some(StackWriteKind::Call) => Color::RGB(0, 200, 0),     // verde
            Some(StackWriteKind::Push) => Color::RGB(0, 120, 255),  // azul
            Some(StackWriteKind::Interrupt) => Color::RGB(255, 140, 0), // naranja
            Some(StackWriteKind::Manual) => Color::RGB(180, 180, 180),  // gris claro
            Some(StackWriteKind::Unknown) => Color::RGB(120, 120, 120), // gris oscuro
            None => Color::RGB(200, 200, 200), // sin info
        };

        draw_text_colored(
            canvas,
            font,
            &text,
            x,
            y + 20 + (i as i32) * LINE_H,
            color,
        )?;
    }

    Ok(())
}

fn draw_text_colored(
    canvas: &mut Canvas<Window>,
    font: &Font,
    text: &str,
    x: i32,
    y: i32,
    color: Color,
) -> Result<(), String> {
    let surface = font
        .render(text)
        .blended(color)
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    let target = Rect::new(x, y, surface.width(), surface.height());
    canvas.copy(&texture, None, target)?;

    Ok(())
}

// Controla los colores del stack segun que instrucción ha metido el dato
fn color_for_stack_write(kind: StackWriteKind) -> Color {
    match kind {
        StackWriteKind::Call => Color::RGB(0, 220, 0),      // verde
        StackWriteKind::Push => Color::RGB(0, 120, 255),    // azul
        StackWriteKind::Interrupt => Color::RGB(255, 0, 0), // rojo
        StackWriteKind::Manual => Color::RGB(255, 165, 0),  // naranja
        StackWriteKind::Unknown => Color::RGB(180, 180, 180),
    }
}

// Tamaños resultantes (para decidir escala)
// Escala	Resolución final
// 1	    256×192 (muy pequeño)
// 2	    512×384
// 3	    768×576
// 4	    1024×768 ✅
// 5	    1280×960

/*pub fn draw_screen(
    canvas: &mut Canvas<Window>,
    video: &Video,
    x0: i32,
    y0: i32,
) -> Result<(), String> {
    let scale = video.scale as i32;

    let screen_w = ZX_W * scale;
    let screen_h = ZX_H * scale;
    let border = ZX_BORDER * scale;

    // --------------------------------------------------
    // 1) DIBUJAR MARCO NEGRO (BEZEL)
    // --------------------------------------------------
    let frame = Rect::new(
        x0 - border,
        y0 - border,
        (screen_w + 2 * border) as u32,
        (screen_h + 2 * border) as u32,
    );

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.fill_rect(frame)?;

    // --------------------------------------------------
    // 2) CLIP: SOLO DENTRO DE LA PANTALLA ZX
    // --------------------------------------------------
    let clip = Rect::new(x0, y0, screen_w as u32, screen_h as u32);
    canvas.set_clip_rect(clip);

    // --------------------------------------------------
    // 3) DIBUJAR PÍXELES ZX
    // --------------------------------------------------
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    /*let scale = 4;
    for y in 0..ZX_H {
        for x in 0..ZX_W {
            if video.pixels[(y * 256 + x) as usize] != 0 {
                let r = Rect::new(
                    x0 + x * scale,
                    y0 + y * scale,
                    scale as u32,
                    scale as u32,
                );
                canvas.fill_rect(r)?;
            }
        }
    }*/
    //let scale = 4;
    for y in 0..ZX_H {
        for x in 0..ZX_W {
            let pixel = video.pixels[(y * 256 + x) as usize];

            // ---- atributo correspondiente ----
            let attr_x = (x / 8) as usize;
            let attr_y = (y / 8) as usize;
            let attr = video.attrs[attr_y * 32 + attr_x];

            let ink = attr & 0b0000_0111;
            let paper = (attr >> 3) & 0b0000_0111;
            let bright = (attr & 0b0100_0000) != 0;

            let color = if pixel != 0 {
                zx_color(ink, bright)
            } else {
                zx_color(paper, bright)
            };

            canvas.set_draw_color(color);

            let r = Rect::new(
                x0 + x * scale,
                y0 + y * scale,
                scale as u32,
                scale as u32,
            );
            let attr_index = (y as usize / 8) * 32 + (x as usize / 8);
            let attr = video.attrs[attr_index];

            let bright = (attr & 0b0100_0000) != 0;

            let ink = attr & 0b0000_0111;
            let paper = (attr >> 3) & 0b0000_0111;

            let color = if pixel != 0 {
                zx_color(ink, bright)
            } else {
                zx_color(paper, bright)
            };

            canvas.set_draw_color(color);

            canvas.fill_rect(r)?;
        }
    }*/
// -----------------------------
pub fn draw_screen(
    canvas: &mut Canvas<Window>,
    video: &Video,
    x0: i32,
    y0: i32,
) -> Result<(), String> {
    let scale = video.scale as i32;

    let screen_w = ZX_W * scale;
    let screen_h = ZX_H * scale;
    let border = ZX_BORDER * scale;

    // --------------------------------------------------
    // 1) MARCO NEGRO (BEZEL)
    // --------------------------------------------------
    let frame = Rect::new(
        x0 - border,
        y0 - border,
        (screen_w + 2 * border) as u32,
        (screen_h + 2 * border) as u32,
    );

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.fill_rect(frame)?;

    // --------------------------------------------------
    // 2) CLIP SOLO ÁREA ZX
    // --------------------------------------------------
    let clip = Rect::new(x0, y0, screen_w as u32, screen_h as u32);
    canvas.set_clip_rect(clip);

    // --------------------------------------------------
    // 3) DIBUJAR PANTALLA ZX (PIXELS + ATTRS)
    // --------------------------------------------------
    /*for y in 0..ZX_H {
        for x in 0..ZX_W {
            let pixel_on = video.pixels[(y * 256 + x) as usize] != 0;

            // atributo 8x8
            let attr_index = (y / 8) * 32 + (x / 8);
            let attr = video.attrs[attr_index as usize];

            let flash = (attr & 0b1000_0000) != 0;
            let bright = (attr & 0b0100_0000) != 0;

            let mut ink = attr & 0b0000_0111;
            let mut paper = (attr >> 3) & 0b0000_0111;

            // FLASH: intercambiar INK/PAPER
            if flash && video.flash_phase {
                std::mem::swap(&mut ink, &mut paper);
            }

            let color = if pixel_on {
                zx_color(ink, bright)
            } else {
                zx_color(paper, bright)
            };

            canvas.set_draw_color(color);

            let r = Rect::new(
                x0 + x * scale,
                y0 + y * scale,
                scale as u32,
                scale as u32,
            );

            canvas.fill_rect(r)?;
        }
    }*/
    for y in 0..ZX_H {
        for x in 0..ZX_W {
            let pixel = video.pixels[(y * 256 + x) as usize];

            let attr_x = (x / 8) as usize;
            let attr_y = (y / 8) as usize;
            let attr = video.attrs[attr_y * 32 + attr_x];

            let mut ink = attr & 0b0000_0111;
            let mut paper = (attr >> 3) & 0b0000_0111;
            let bright = (attr & 0b0100_0000) != 0;
            let flash = (attr & 0b1000_0000) != 0;

            // --- FLASH REAL ---
            if flash && video.flash_phase {
                std::mem::swap(&mut ink, &mut paper);
            }

            let color = if pixel != 0 {
                zx_color(ink, bright)
            } else {
                zx_color(paper, bright)
            };

            canvas.set_draw_color(color);

            let r = Rect::new(
                x0 + x * scale,
                y0 + y * scale,
                scale as u32,
                scale as u32,
            );
            canvas.fill_rect(r)?;
        }
    }

    // --------------------------------------------------
    // 4) DESACTIVAR CLIP
    // --------------------------------------------------
    canvas.set_clip_rect(None);

    Ok(())
}

/*// --------------------------------------------------
// 4) FLASH
// --------------------------------------------------
let attr = video.attrs[((y / 8) * 32 + (x / 8)) as usize];

let flash = (attr & 0b1000_0000) != 0;
let mut ink = attr & 0b0000_0111;
let mut paper = (attr >> 3) & 0b0000_0111;

if flash && video.flash_phase {
    std::mem::swap(&mut ink, &mut paper);
}

// --------------------------------------------------
// 5) DESACTIVAR CLIP
// --------------------------------------------------
canvas.set_clip_rect(None);

Ok(())
}*/

// Auxiliar para los atributos (colores)
/*fn zx_color(code: u8, bright: bool) -> Color {
    let base = match code & 7 {
        0 => (0, 0, 0),         // negro
        1 => (0, 0, 192),       // azul
        2 => (192, 0, 0),       // rojo
        3 => (192, 0, 192),     // magenta
        4 => (0, 192, 0),       // verde
        5 => (0, 192, 192),     // cian
        6 => (192, 192, 0),     // amarillo
        7 => (192, 192, 192),   // blanco
        _ => (0, 0, 0),
    };

    if bright {
        Color::RGB(
            (base.0 + 63).min(255),
            (base.1 + 63).min(255),
            (base.2 + 63).min(255),
        )
    } else {
        Color::RGB(base.0, base.1, base.2)
    }
}*/

fn zx_color(code: u8, bright: bool) -> Color {
    match (code & 0x07, bright) {
        (0, _) => Color::RGB(0, 0, 0),

        (1, false) => Color::RGB(0, 0, 192),
        (1, true) => Color::RGB(0, 0, 255),

        (2, false) => Color::RGB(192, 0, 0),
        (2, true) => Color::RGB(255, 0, 0),

        (3, false) => Color::RGB(192, 0, 192),
        (3, true) => Color::RGB(255, 0, 255),

        (4, false) => Color::RGB(0, 192, 0),
        (4, true) => Color::RGB(0, 255, 0),

        (5, false) => Color::RGB(0, 192, 192),
        (5, true) => Color::RGB(0, 255, 255),

        (6, false) => Color::RGB(192, 192, 0),
        (6, true) => Color::RGB(255, 255, 0),

        (7, false) => Color::RGB(192, 192, 192),
        (7, true) => Color::RGB(255, 255, 255),

        _ => Color::RGB(0, 0, 0),
    }
}




