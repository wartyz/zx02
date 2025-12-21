; =====================================================
; start.asm - programa Z80 mínimo de prueba
; =====================================================

        DEVICE ZXSPECTRUM48
        ORG 0000h

START:
        LD A,01h
        LD B,02h
        ADD A,B
        AND A
        XOR B

        LD HL,1000h
        LD (HL),A
        INC HL
        DEC HL

        JR START       ; bucle infinito

        HALT

; -----------------------------------------------------
; SALIDA BINARIA
; -----------------------------------------------------
        SAVEBIN "start.bin", 0, $
