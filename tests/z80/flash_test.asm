        DEVICE ZXSPECTRUM48

        ORG $0000

; -----------------------------------------------------
; Programa de prueba FLASH
; -----------------------------------------------------
; - Pantalla llena de bloques
; - Atributos con FLASH
; - INK y PAPER distintos
; -----------------------------------------------------

START:
        DI

; -----------------------------------------------------
; Limpiar bitmap ($4000-$57FF)
; -----------------------------------------------------
        LD HL, $4000
        LD DE, $4001
        LD BC, $17FF
        LD (HL), $FF          ; píxeles activos
        LDIR

; -----------------------------------------------------
; Escribir atributos con FLASH
; -----------------------------------------------------
; Formato atributo:
; bit 7 = FLASH
; bit 6 = BRIGHT
; bits 5-3 = PAPER
; bits 2-0 = INK
;
; Ejemplo:
; FLASH=1, BRIGHT=1, PAPER=1 (azul), INK=6 (amarillo)
; 1 1 001 110 = %1100_1110 = $CE
; -----------------------------------------------------

        LD HL, $5800
        LD BC, 32*24
        LD A, %11001110      ; FLASH + BRIGHT + colores visibles

ATTR_LOOP:
        LD (HL), A
        INC HL
        DEC BC
        LD A, B
        OR C
        JR NZ, ATTR_LOOP

; -----------------------------------------------------
; Bucle infinito (HALT sincroniza con IM 1)
; -----------------------------------------------------
MAIN_LOOP:
        HALT
        JR MAIN_LOOP

; ----------------------------------------
; Salida binaria
; ----------------------------------------
        SAVEBIN "flash_test.bin", 0, $
