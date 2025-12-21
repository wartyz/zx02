        DEVICE ZXSPECTRUM48

        ORG $0000

; ----------------------------------------
; Programa mínimo de test de vídeo
; Escribe patrón en la RAM gráfica ($4000)
; ----------------------------------------

START:
        DI

        LD HL, $4000      ; inicio pantalla
        LD BC, $1800      ; 6144 bytes de bitmap

FILL_LOOP:
        LD (HL), $FF      ; 10101010 -> franjas claras
        INC HL
        DEC BC
        LD A, B
        OR C
        JR NZ, FILL_LOOP

; bucle infinito
HALT_LOOP:
        JR HALT_LOOP

; ----------------------------------------
; Salida binaria
; ----------------------------------------
        SAVEBIN "video_test.bin", 0, $
