        DEVICE ZXSPECTRUM48

        ORG $8000

start:
 ; -------------------------
 ; PÍXELES
 ; -------------------------

        ; arriba izquierda
        LD A,%10000000
        LD ($4000),A

        ; arriba derecha
        LD A,%00000001
        LD ($401F),A

        ; abajo izquierda
        LD A,%10000000
        LD ($57E0),A

        ; abajo derecha
        LD A,%00000001
        LD ($57FF),A

 ; -------------------------
 ; ATRIBUTOS (rojo brillante)
 ; -------------------------

        LD A,$42        ; BRIGHT + INK rojo + PAPER negro

        ; atributo arriba izquierda
        LD ($5800),A

        ; atributo arriba derecha
        LD ($581F),A

        ; atributo abajo izquierda
        LD ($5AE0),A

        ; atributo abajo derecha
        LD ($5AFF),A

loop:
        HALT
        JR loop


; ----------------------------------------
; Salida binaria
; ----------------------------------------
        SAVEBIN "screen_corners.bin", $8000, $200
