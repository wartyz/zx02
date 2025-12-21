; =====================================================
; Test completo de usos del STACK (para el emulador)
; =====================================================
        DEVICE ZXSPECTRUM48
        ORG $0000

START:
        DI
        LD SP,$8000
; -------------------------
; 1) PUSH normales
; -------------------------
        LD   BC,$1111
        LD   DE,$2222
        LD   HL,$3333
        PUSH BC
        PUSH DE
        PUSH HL

; -------------------------
; 2) CALL / RET
; -------------------------
        CALL SUB1
        NOP

; -------------------------
; 3) RST (CALL implícito)
; -------------------------
        RST  $08

; -------------------------
; 4) Escritura MANUAL en stack
; -------------------------


; -------------------------
; 5) POP (no escribe)
; -------------------------
        POP  HL
        POP  DE
        POP  BC

; -------------------------
; 6) "Interrupción" simulada
; -------------------------
        CALL FAKE_INT

HALT_LOOP:
        HALT
        JR HALT_LOOP

; =====================================================
; Subrutinas
; =====================================================

SUB1:
        PUSH AF
        POP  AF
        RET

FAKE_INT:
        PUSH AF
        PUSH HL
        POP  HL
        POP  AF
        RET

; =====================================================
; Salida binaria
; =====================================================
        SAVEBIN "stack_colors.bin", 0, $
