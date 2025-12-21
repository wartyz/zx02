        DEVICE ZXSPECTRUM48

        ORG $0000

        DI

; --------------------------------------------------
; Limpiar pantalla de píxeles
; --------------------------------------------------
        LD HL,$4000
        LD BC,$1800
        XOR A
CLS_PIX:
        LD (HL),A
        INC HL
        DEC BC
        LD A,B
        OR C
        JR NZ,CLS_PIX

; --------------------------------------------------
; Rellenar atributos
; --------------------------------------------------
        LD HL,$5800
        LD B,24          ; filas
ROW:
        LD C,32          ; columnas
COL:
        PUSH BC

        ; INK = columna & 7
        LD A,C
        DEC A
        AND 7            ; 0..7
        LD D,A

        ; PAPER = fila & 7
        LD A,B
        DEC A
        AND 7
        SLA A
        SLA A
        SLA A            ; << 3
        OR D             ; PAPER + INK

        ; BRIGHT cada 8 filas
        LD A,B
        AND 8
        JR Z,NO_BRIGHT
        OR %01000000
NO_BRIGHT:

        ; FLASH cada 12 filas
        LD A,B
        CP 12
        JR C,NO_FLASH
        OR %10000000
NO_FLASH:

        LD (HL),A
        INC HL

        POP BC
        DEC C
        JR NZ,COL
        DEC B
        JR NZ,ROW

; --------------------------------------------------
; Espera infinita
; --------------------------------------------------
LOOP:
        HALT
        JR LOOP

; ----------------------------------------
; Salida binaria
; ----------------------------------------
        SAVEBIN "all_colors_flash.bin", 0, $

