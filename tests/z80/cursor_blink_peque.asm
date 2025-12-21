    DEVICE ZXSPECTRUM48

; test_cursor_small.asm - Menos de 32KB
    org $8000  ; 32768 decimal

    ; Código pequeño
    di
    ld sp, $FFFF
    im 1
    ei

    call $0DAF  ; CLS

    ; Configurar cursor
    ld hl, $5C3C
    ld (hl), 20
    inc hl
    ld (hl), 10

    call $0A4F  ; CURSOR

loop:
    halt
    jr loop

    ; Rellenar solo hasta 0x8100 (256 bytes)
    org $8100

; ----------------------------------------
; Salida binaria
; ----------------------------------------
    SAVEBIN "cursor_blink_peque.bin", 0, $
