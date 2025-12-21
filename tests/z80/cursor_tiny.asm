; cursor_tiny.asm - Solo el código necesario
; Ensamblar: sjasmplus cursor_tiny.asm --lst --raw=cursor_tiny.bin
    DEVICE ZXSPECTRUM48

    ; DISP en lugar de ORG - solo genera el código, no rellena
    disp $8000

start:
    di
    ld sp, $FF00

    ; Configurar IM1
    im 1
    ld a, $3F
    ld i, a
    ei

    ; Limpiar pantalla
    call $0DAF

    ; Configurar cursor
    ld hl, $5C3C      ; COORDS
    ld (hl), 20       ; X
    inc hl
    ld (hl), 10       ; Y

    ; Posición pantalla
    ld hl, $5C3E      ; S_POSN
    ld (hl), 3        ; línea
    inc hl
    ld (hl), 10       ; columna

    ; Imprimir mensaje
    ld hl, message
print_loop:
    ld a, (hl)
    or a
    jr z, show_cursor
    rst $10
    inc hl
    jr print_loop

show_cursor:
    ; Dibujar cursor
    call $0A4F

    ; Bucle principal
main_loop:
    halt
    jr main_loop

message:
    db "Cursor test OK", 13, 0

    ; Terminar DISP
    end start

; ----------------------------------------
; Salida binaria
; ----------------------------------------
    SAVEBIN "cursor_tiny.bin", $8000, $ - $8000
