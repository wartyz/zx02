        DEVICE ZXSPECTRUM48

        ORG $0000

        DI
        IM 1

        LD SP, $FFFE

        LD HL, contador
        LD (HL), 0

        EI
        NOP              ; ← instrucción clave para EI real

main:
        HALT
        JR main

; --------------------------------------
; Rutina de interrupción IM 1 ($0038)
; --------------------------------------
        ORG $0038
int_handler:
        PUSH AF
        LD A, (contador)
        INC A
        LD (contador), A
        POP AF
        RETN

; --------------------------------------
contador:
        DB 0

; ----------------------------------------
; Salida binaria
; ----------------------------------------
        SAVEBIN "pba00.bin", 0, $
