use sdl2::{
    pixels::Color,
    rect::Rect,
    render::Canvas,
    ttf::Font,
    video::Window,
};

use crate::{botones, LoadState};
use crate::disasm::disassemble;
use crate::cpu_exec::CpuSnapshot;
use crate::botones::{Button, ButtonAction};
use crate::constantes::{MARGEN_NEGRO, ZX_BORDER, ZX_H, ZX_W};
use crate::stack_tracker::{StackTracker, StackWriteKind};
use crate::video::Video;

// const ZX_W: i32 = 256;
// const ZX_H: i32 = 192;
// const MARGEN_NEGRO: i32 = 20;
// const ZX_BORDER: i32 = 16;

/* ================================================== */
/* PANTALLA ZX SPECTRUM                               */
/* ================================================== */

pub fn draw_zx_screen(
    canvas: &mut Canvas<Window>,
    video: &Video,
) -> Result<(), String> {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    draw_screen(canvas, video, MARGEN_NEGRO, MARGEN_NEGRO)?;

    Ok(())
}

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

    // 1) MARCO NEGRO (BEZEL)
    let frame = Rect::new(
        x0 - border,
        y0 - border,
        (screen_w + 2 * border) as u32,
        (screen_h + 2 * border) as u32,
    );
    canvas.set_draw_color(Color::BLACK);
    canvas.fill_rect(frame)?;

    // 2) CLIP SOLO ÁREA ZX
    let clip = Rect::new(x0, y0, screen_w as u32, screen_h as u32);
    canvas.set_clip_rect(clip);

    // 3) DIBUJAR PANTALLA (USANDO EL FRAMEBUFFER PROCESADO)
    for y in 0..ZX_H {
        for x in 0..ZX_W {
            // Obtenemos el índice de color (0-15) del framebuffer consolidado
            let color_idx = video.framebuffer[(y * 256 + x) as usize];

            // Convertimos el índice a Color RGB usando nuestra paleta
            let color = zx_color_from_index(color_idx);

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

    canvas.set_clip_rect(None);
    Ok(())
}

/// Convierte el índice 0-15 del framebuffer al color RGB real del Spectrum
fn zx_color_from_index(index: u8) -> Color {
    let bright = index >= 8;
    let code = index & 0x07;

    match (code, bright) {
        (0, _) => Color::RGB(0, 0, 0),             // Negro
        (1, false) => Color::RGB(0, 0, 192),       // Azul
        (1, true) => Color::RGB(0, 0, 255),        // Azul Brillante
        (2, false) => Color::RGB(192, 0, 0),       // Rojo
        (2, true) => Color::RGB(255, 0, 0),        // Rojo Brillante
        (3, false) => Color::RGB(192, 0, 192),     // Magenta
        (3, true) => Color::RGB(255, 0, 255),      // Magenta Brillante
        (4, false) => Color::RGB(0, 192, 0),       // Verde
        (4, true) => Color::RGB(0, 255, 0),        // Verde Brillante
        (5, false) => Color::RGB(0, 192, 192),     // Cian
        (5, true) => Color::RGB(0, 255, 255),      // Cian Brillante
        (6, false) => Color::RGB(192, 192, 0),     // Amarillo
        (6, true) => Color::RGB(255, 255, 0),      // Amarillo Brillante
        (7, false) => Color::RGB(192, 192, 192),   // Blanco (Gris)
        (7, true) => Color::RGB(255, 255, 255),    // Blanco puro
        _ => Color::BLACK,
    }
}

/* ================================================== */
/* DEBUGGER Y OTROS (MANTENIDO IGUAL)                 */
/* ================================================== */

pub fn draw_debug(
    canvas: &mut Canvas<Window>,
    font: &Font,
    snapshot: Option<&CpuSnapshot>,
    stack_tracker: &StackTracker,
    load_state: LoadState,
    debug_enabled: bool,
) -> Result<(), String> {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    if debug_enabled {
        if let Some(s) = snapshot {
            draw_registers(canvas, font, s)?;
            draw_flags(canvas, font, s)?;
            draw_memory_dump(canvas, font, s)?;
            draw_instruction_window(canvas, font, s)?;
            draw_stack(canvas, font, s, stack_tracker, 560, 360)?;
        }
    }

    draw_buttons(canvas, font, &botones::default_buttons(), debug_enabled)?;
    draw_load_state(canvas, font, load_state)?;

    Ok(())
}

/* ================================================== */
/* DIBUJA EL ESTADO DE CARGA VACIO, ROM, SNA O BIN    */
/* ================================================== */
fn draw_load_state(
    canvas: &mut Canvas<Window>,
    font: &Font,
    state: LoadState,
) -> Result<(), String> {
    let (text, color) = match state {
        LoadState::None => ("NO CARGADO", Color::RGB(255, 0, 0)),   // Rojo
        LoadState::Rom => ("ROM CARGADA", Color::RGB(0, 255, 0)),   // Verde
        LoadState::Sna => ("SNA CARGADO", Color::RGB(255, 255, 0)), // Amarillo
        LoadState::Z80 => ("Z80 CARGADO", Color::RGB(255, 255, 0)), // Amarillo
        LoadState::Bin => ("BIN CARGADO", Color::RGB(255, 0, 255)), // Violeta
    };

    draw_text_color(canvas, font, &format!("ESTADO: {}", text), 280, 56, color)
}

/* ================================================== */
/* TEXT HELPERS (RESTAURADOS)                         */
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
    if text.is_empty() { return Ok(()); }

    let surface = font
        .render(text)
        .blended(color)
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    let target = Rect::new(x, y, surface.width(), surface.height());
    canvas.copy(&texture, None, Some(target))?;
    Ok(())
}
/* ================================================== */
/* DIBUJO DE REGISTROS Y ESTADO (DEBUGGER)            */
/* ================================================== */

fn draw_registers(
    canvas: &mut Canvas<Window>,
    font: &Font,
    s: &CpuSnapshot,
) -> Result<(), String> {
    let x1 = 20;
    let x2 = 180;
    let x3 = 360;
    let y0 = 100;
    let dy = 20;

    // Columna 1
    draw_text(canvas, font, &format!("PC: {:04X}", s.pc), x1, y0)?;
    draw_text(canvas, font, &format!("AF: {:04X}", s.af), x1, y0 + dy)?;
    draw_text(canvas, font, &format!("BC: {:04X}", s.bc), x1, y0 + 2 * dy)?;
    draw_text(canvas, font, &format!("DE: {:04X}", s.de), x1, y0 + 3 * dy)?;
    draw_text(canvas, font, &format!("HL: {:04X}", s.hl), x1, y0 + 4 * dy)?;

    // Columna 2 (Registros sombra/alternativos)
    draw_text(canvas, font, &format!("SP : {:04X}", s.sp), x2, y0)?;
    draw_text(canvas, font, &format!("AF': {:04X}", s.af_), x2, y0 + dy)?;
    draw_text(canvas, font, &format!("BC': {:04X}", s.bc_), x2, y0 + 2 * dy)?;
    draw_text(canvas, font, &format!("DE': {:04X}", s.de_), x2, y0 + 3 * dy)?;
    draw_text(canvas, font, &format!("HL': {:04X}", s.hl_), x2, y0 + 4 * dy)?;

    // Columna 3 (Interrupciones y ciclos)
    draw_text(canvas, font, &format!("I: {:02X}", s.i), x3, y0 + dy)?;
    draw_text(canvas, font, &format!("R: {:02X}", s.r), x3, y0 + 2 * dy)?;
    draw_text(canvas, font, &format!("CYC: {}", s.instr_cycles), x3, y0 + 3 * dy)?;

    Ok(())
}

/*fn draw_flags(
    canvas: &mut Canvas<Window>,
    font: &Font,
    s: &CpuSnapshot,
) -> Result<(), String> {
    let f = s.f;
    // Helper para extraer bits: Sign, Zero, Y (5), Half-Carry, X (3), Parity/V, N (Add/Sub), Carry
    let bit = |b: u8| if (f & (1u8 << b)) != 0 { '1' } else { '0' };

    draw_text(canvas, font, "FLAGS:", 20, 230)?;
    draw_text(canvas, font, "S  Z  5  H  3  P  N  C", 100, 230)?;
    draw_text(
        canvas,
        font,
        &format!(
            "{}  {}  {}  {}  {}  {}  {}  {}",
            bit(7), bit(6), bit(5), bit(4), bit(3), bit(2), bit(1), bit(0)
        ),
        100,
        255,
    )?;
    Ok(())
}
*/
fn draw_flags(
    canvas: &mut Canvas<Window>,
    font: &Font,
    s: &CpuSnapshot,
) -> Result<(), String> {
    // Flags reales del Z80 (incluyendo no documentados)
    let labels = ["S", "Z", "Y", "H", "X", "P", "N", "C"];
    let bits = [7, 6, 5, 4, 3, 2, 1, 0];

    //let x0 = 0;
    let y0 = 230;
    let dx = 35;

    // Título
    //draw_text(canvas, font, "FLAGS:", x0, y0)?;

    // Fila de etiquetas
    for (i, label) in labels.iter().enumerate() {
        draw_text(
            canvas,
            font,
            label,
            20 + (i as i32 * dx),
            y0,
        )?;
    }

    // Fila de valores (con color)
    for (i, &bit) in bits.iter().enumerate() {
        let before = (s.f_before >> bit) & 1;
        let after = (s.f >> bit) & 1;

        let color = if s.from_step {
            match (before, after) {
                (0, 1) => Color::RGB(0, 255, 0), // 0 → 1
                (1, 0) => Color::RGB(255, 0, 0), // 1 → 0
                _ => Color::WHITE,          // sin cambio
            }
        } else {
            Color::WHITE
        };

        draw_text_color(
            canvas,
            font,
            &after.to_string(),
            20 + (i as i32 * dx),
            y0 + 25,
            color,
        )?;
    }

    Ok(())
}

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
            if index >= s.mem_dump.len() { break; }

            let byte = s.mem_dump[index];
            let byte_addr = addr.wrapping_add(col as u16);

            // Resaltar en amarillo si es la dirección del PC
            let color = if byte_addr == s.pc { Color::RGB(255, 255, 0) } else { Color::WHITE };

            draw_text_color(
                canvas,
                font,
                &format!("{:02X}", byte),
                start_x + 65 + (col as i32 * 35),
                y,
                color,
            )?;
        }
    }
    Ok(())
}
/* ================================================== */
/* VENTANA DE INSTRUCCIONES (DESENSAMBLADO, 21 LÍNEAS, CENTRADA EN PC) */
/* ================================================== */
fn draw_instruction_window(
    canvas: &mut Canvas<Window>,
    font: &Font,
    s: &CpuSnapshot,
) -> Result<(), String> {
    let start_x = 20;
    let start_y = 360;
    let line_h = 22;

    // 1) Empezamos un poco antes del PC actual.
    // Usamos el PC del snapshot como referencia absoluta.
    let mut current_pc = s.pc.saturating_sub(15);
    let mut instrs = Vec::new();

    // 2) Intentamos llenar 40 líneas de instrucciones
    for _ in 0..40 {
        // Calculamos dónde cae este PC dentro de nuestro mem_dump
        // El dump en el snapshot empieza en s.mem_base
        let offset = current_pc.wrapping_sub(s.mem_base) as usize;

        // Si el offset se sale de los 512 bytes del dump, paramos
        if offset >= s.mem_dump.len() {
            instrs.push((current_pc, "??".to_string(), 1));
            current_pc = current_pc.wrapping_add(1);
            continue;
        }

        // --- EL TRUCO DEFINITIVO ---
        // Le pasamos el resto del dump desde el offset actual.
        // Y le decimos al desensamblador que la dirección de memoria de ese buffer
        // coincide exactamente con current_pc.
        let bytes_restantes = &s.mem_dump[offset..];

        // Importante: le pasamos current_pc como dirección actual
        // y también como base del buffer para que el offset interno sea 0.
        let (mnemonic, len) = disassemble(bytes_restantes, current_pc, current_pc);

        let safe_len = if len == 0 { 1 } else { len as u8 };

        instrs.push((current_pc, mnemonic, safe_len));

        // Avanzamos el PC para la siguiente instrucción
        current_pc = current_pc.wrapping_add(safe_len as u16);
    }

    // 3) Buscamos dónde quedó el PC real en nuestra lista generada para centrar la vista
    let pc_pos = instrs.iter().position(|(addr, _, _)| *addr == s.pc).unwrap_or(0);

    // Dibujamos 20 líneas a partir de un poco antes del PC encontrado
    let start_idx = pc_pos.saturating_sub(5);
    let mut y = start_y;

    for i in start_idx..(start_idx + 20) {
        if i >= instrs.len() { break; }

        let (pc, mnemonic, len) = &instrs[i];

        let color = if *pc == s.pc {
            Color::RGB(255, 255, 0) // Amarillo para el PC actual
        } else {
            Color::WHITE
        };

        // Re-extraemos los bytes para el texto HEX
        let mut hex_str = String::new();
        let off = pc.wrapping_sub(s.mem_base) as usize;
        for j in 0..(*len as usize) {
            if off + j < s.mem_dump.len() {
                hex_str.push_str(&format!("{:02X} ", s.mem_dump[off + j]));
            }
        }

        let text = format!("{:04X}: {:<12}  {}", pc, hex_str, mnemonic);
        draw_text_color(canvas, font, &text, start_x, y, color)?;
        y += line_h;
    }

    Ok(())
}
/* ================================================== */
/* DIBUJO DEL STACK (PILA)                            */
/* ================================================== */

fn draw_stack(
    canvas: &mut Canvas<Window>,
    font: &Font,
    s: &CpuSnapshot,
    stack_tracker: &StackTracker,
    x: i32,
    y: i32,
) -> Result<(), String> {
    const LINE_H: i32 = 18;

    draw_text(canvas, font, "STACK (Top 15)", x, y)?;

    // Dibujamos las primeras 15 entradas del dump del stack
    for (i, val) in s.stack_dump.iter().enumerate().take(15) {
        let addr = s.stack_base.wrapping_add(i as u16);

        // Marcador visual si el SP actual apunta a esta dirección
        let sp_marker = if addr == s.sp {
            "-> "
        } else {
            "   "
        };

        let text = format!("{}{:04X}: {:02X}", sp_marker, addr, val);

        // Usamos el stack_tracker para colorear el origen del dato
        let color = match stack_tracker.last_write_to(addr) {
            Some(StackWriteKind::Call) => Color::RGB(0, 255, 0),        // Verde: Direcciones de retorno
            Some(StackWriteKind::Push) => Color::RGB(0, 150, 255),      // Azul: Datos PUSH
            Some(StackWriteKind::Interrupt) => Color::RGB(255, 50, 50), // Rojo: Interrupciones
            Some(StackWriteKind::Manual) => Color::RGB(255, 165, 0),    // Naranja: Escritura manual
            _ => Color::RGB(150, 150, 150),                             // Gris: Desconocido
        };

        draw_text_color(
            canvas,
            font,
            &text,
            x,
            y + 25 + (i as i32) * LINE_H,
            color,
        )?;
    }

    Ok(())
}

/* ================================================== */
/* DIBUJO DE BOTONES DEL DEBUGGER                     */
/* ================================================== */

pub fn draw_buttons(
    canvas: &mut Canvas<Window>,
    font: &Font,
    buttons: &[Button],
    debug_enabled: bool,
) -> Result<(), String> {
    for b in buttons {
        // Rectángulo del cuerpo del botón
        let rect = Rect::new(b.x, b.y, b.w as u32, b.h as u32);

        // Poner color segun esté true o false DEBUG
        let bg_color = match b.action {
            ButtonAction::DebugToggle => {
                if debug_enabled {
                    Color::RGB(0, 120, 0)   // VERDE → DEBUG ON
                } else {
                    Color::RGB(160, 0, 0)   // ROJO → DEBUG OFF
                }
            }
            _ => Color::RGB(60, 60, 60), // botones normales
        };
        // Color de fondo (Gris oscuro)
        canvas.set_draw_color(bg_color);
        canvas.fill_rect(rect)?;

        // Borde del botón (Gris claro)
        canvas.set_draw_color(Color::RGB(200, 200, 200));
        canvas.draw_rect(rect)?;

        // Texto de la etiqueta (Label)
        let label = match b.action {
            ButtonAction::Step => "STEP",
            ButtonAction::Run => "RUN",
            ButtonAction::RunFast => "FAST",
            ButtonAction::Pause => "PAUSE",
            ButtonAction::Reset => "RESET",
            //ButtonAction::LoadRom => "LROM",
            //ButtonAction::LoadSna => "LSNA",
            ButtonAction::Load => "LOAD",
            ButtonAction::DebugToggle => "DBG",
        };

        let surface = font
            .render(label)
            .blended(Color::WHITE)
            .map_err(|e| e.to_string())?;

        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        // Centrar el texto dentro del botón
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