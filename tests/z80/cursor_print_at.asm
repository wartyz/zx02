    ; cursor_print_at.asm
    ; Ensamblar: sjasmplus cursor_print_at.asm
    ; Definir el dispositivo
    DEVICE ZXSPECTRUM48

    OUTPUT "cursor_print_at.bin"

    ORG $C000

start:
    di
    ld sp, $FF00

    ; Configurar IM1
    im 1
    ld a, $3F
    ld i, a
    ei

    ; Limpiar pantalla
    call $0DAF      ; CLS

    ; Configurar MODE para entrada
    ld a, 'K'       ; Modo K (keyword)
    ld ($5C3A), a   ; MODE

    ; Configurar FLAGS para mostrar cursor
    ld hl, $5C08    ; FLAGS
    set 5, (hl)     ; Bit 5 = mostrar cursor

    ; Usar PRINT-AT para posicionar y mostrar cursor
    ld a, 5         ; Línea 5 (0-21)
    ld b, 10        ; Columna 10 (0-31)
    call $0B7B      ; PRINT-AT

    ; También configurar COORDS manualmente
    ld hl, $5C3C    ; COORDS
    ld (hl), 80     ; X = 80 (10*8)
    inc hl
    ld (hl), 40     ; Y = 40 (5*8)

    ; Imprimir mensaje para ver dónde estamos
    ld hl, message
print_loop:
    ld a, (hl)
    or a
    jr z, wait
    rst $10         ; PRINT-A
    inc hl
    jr print_loop

wait:
    ; Bucle infinito - las interrupciones harán parpadear el cursor
main_loop:
    halt
    jr main_loop

message:
    db "Cursor en (5,10)", 13, 0

end:
