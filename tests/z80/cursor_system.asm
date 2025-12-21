    ; cursor_system.asm
    ; Configura todo el sistema como lo haría la ROM

    ; Definir el dispositivo
    DEVICE ZXSPECTRUM48
    OUTPUT "cursor_system.bin"

    ORG $C000

start:
    di
    ld sp, $FF00

    ; 1. Configurar interrupciones
    im 1
    ld a, $3F
    ld i, a
    ei

    ; 2. Limpiar pantalla
    call $0DAF      ; CLS

    ; 3. Inicializar variables del sistema IMPORTANTES

    ; FLAGS - Bit 5 = mostrar cursor
    ld hl, $5C08    ; FLAGS
    ld (hl), %00100000  ; Bit 5 = 1

    ; MODE - Modo de entrada
    ld hl, $5C3A    ; MODE
    ld (hl), 'K'    ; Modo K (keyword)

    ; COORDS - Coordenadas del cursor (en píxeles)
    ld hl, $5C3C    ; COORDS
    ld (hl), 64     ; X = 64 (columna 8 * 8)
    inc hl
    ld (hl), 40     ; Y = 40 (línea 5 * 8)

    ; S_POSN - Posición en pantalla (en caracteres)
    ld hl, $5C3E    ; S_POSN
    ld (hl), 6      ; Línea 6 (1-based)
    inc hl
    ld (hl), 9      ; Columna 9 (1-based)

    ; CURCHL - Dirección del cursor (EDIT/INPUT)
    ld hl, $4000    ; Inicio de pantalla + offset
    ld ($5C5C), hl  ; CURCHL

    ; 4. Llamar a rutina que realmente dibuja el cursor
    ; La rutina $0B76 actualiza y posiblemente dibuja
    call $0B76      ; Actualizar posición cursor

    ; O usar $0DD9 - CL-SET que establece y dibuja
    ; call $0DD9     ; CL-SET

    ; 5. Imprimir algo para referencia
    ld hl, msg1
    call print_str
    ld hl, msg2
    call print_str

    ; 6. Bucle principal
main:
    halt
    jr main

; --- Subrutina para imprimir cadena ---
print_str:
    ld a, (hl)
    or a
    ret z
    rst $10
    inc hl
    jr print_str

msg1:
    db "Test cursor Spectrum", 13, 0
msg2:
    db "Linea 6, col 9 ->", 0

end:
