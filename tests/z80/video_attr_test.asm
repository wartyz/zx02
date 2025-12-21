        DEVICE ZXSPECTRUM48

        ORG $0000

START:
        DI

; --------------------------------------------------
; 1) LIMPIAR PANTALLA (todo a 0)
; --------------------------------------------------
        LD HL,$4000
        LD BC,$1800        ; 6144 bytes
        XOR A
CLS_PIX:
        LD (HL),A
        INC HL
        DEC BC
        LD A,B
        OR C
        JR NZ,CLS_PIX

; --------------------------------------------------
; 2) RELLENAR ATRIBUTOS
;    PAPER = azul, INK = amarillo, BRIGHT = 1
; --------------------------------------------------
        LD HL,$5800
        LD BC,$0300        ; 768 bytes

        ; BRIGHT=1, PAPER=1 (azul), INK=6 (amarillo)
        LD A,%01001110     ; 0x4E

CLS_ATTR:
        LD (HL),A
        INC HL
        DEC BC
        LD A,B
        OR C
        LD A,%01001110
        JR NZ,CLS_ATTR

; --------------------------------------------------
; 3) DIBUJAR RAYAS VERTICALES
; --------------------------------------------------
        LD HL,$4000
DRAW:
        LD A,%10101010
        LD (HL),A
        INC HL
        LD A,H
        CP $58
        JR NZ,DRAW

HALT_LOOP:
        HALT
        JR HALT_LOOP

; ----------------------------------------
; Salida binaria
; ----------------------------------------
        SAVEBIN "video_attr_test.bin", 0, $
