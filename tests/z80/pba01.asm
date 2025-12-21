        DEVICE ZXSPECTRUM48

        ORG $8000

        DI
EI

loop:
        HALT
        JR loop

; ----------------------------------------
; Salida binaria
; ----------------------------------------
        SAVEBIN "pba01.bin", 0, $
