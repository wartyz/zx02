        DEVICE ZXSPECTRUM48

; cursor_blink.asm
; Muestra cursor que parpadea con interrupciones

    org $8000

start:
    ; --- INICIALIZAR SISTEMA ---
    di
    ld sp, $FFFF

    ; Configurar modo IM1
    ld a, $3F
    ld i, a        ; I = $3F
    im 1
    ei

    ; --- LIMPIAR PANTALLA ---
    call $0DAF     ; CLS

    ; --- CONFIGURAR CURSOR ---
    ld hl, $5C3C   ; COORDS
    ld (hl), 20    ; X = 20
    inc hl
    ld (hl), 10    ; Y = 10

    ; Posición en pantalla (para PRINT-AT)
    ld hl, $5C3E   ; S_POSN
    ld (hl), 3     ; línea = 3
    inc hl
    ld (hl), 10    ; columna = 10

    ; --- BUCLE PRINCIPAL ---
    ; La ROM maneja el parpadeo via interrupciones
main_loop:
    halt           ; Esperar interrupción
    jr main_loop

    ; --- RUTINA DE INTERRUPCIÓN (IM1) ---
    org $0038
    di
    push af
    push bc
    push de
    push hl

    ; Incrementar contador de flash
    ld hl, flash_counter
    inc (hl)
    ld a, (hl)
    cp 16          ; 16 frames = ~0.32s
    jr nz, skip_flash_toggle

    ; Alternar fase de flash
    ld (hl), 0
    ld hl, $5C08   ; FLAGS
    ld a, (hl)
    xor $20        ; Alternar bit 5 (NEW-KEY/FLASH)
    ld (hl), a

skip_flash_toggle:
    ; Llamar a rutina de cursor si es necesario
    call $0A4F     ; Esto hará parpadear el cursor

    pop hl
    pop de
    pop bc
    pop af
    ei
    reti

flash_counter:
    db 0

    ; Datos adicionales
    org $8100
message:
    db "Cursor test - Parpadeando...", 0

; ----------------------------------------
; Salida binaria
; ----------------------------------------
        SAVEBIN "cursor_blink.bin", 0, $
