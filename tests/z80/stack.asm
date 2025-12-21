        DEVICE ZXSPECTRUM48
        ORG $0000

; ==========================================
; TEST DE PILA Z80
; ==========================================

START:
        LD SP,$5C00          ; Inicializar pila (OBLIGATORIO)

        ; -------- PUSH / POP --------
        LD BC,$1234
        LD DE,$ABCD
        LD HL,$0F0F

        PUSH BC
        PUSH DE
        PUSH HL

        LD BC,$0000
        LD DE,$0000
        LD HL,$0000

        POP HL               ; HL = $0F0F
        POP DE               ; DE = $ABCD
        POP BC               ; BC = $1234

        ; -------- CALL / RET --------
        CALL SUB1
        CALL SUB2

        ; -------- RST --------
        RST $08              ; Salta a handler
        RST $10

END_LOOP:
        JR END_LOOP          ; Bucle infinito seguro

; ==========================================
; SUBRUTINAS
; ==========================================

SUB1:
        PUSH AF
        LD A,$11
        POP AF
        RET

SUB2:
        PUSH HL
        LD HL,$2222
        POP HL
        RET

; ==========================================
; RST HANDLERS
; ==========================================

        ORG $0008
RST08:
        RET

        ORG $0010
RST10:
        RET



; -----------------------------------------------------
; Salida binaria
; -----------------------------------------------------
        SAVEBIN "stack.bin", 0, $
