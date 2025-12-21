        DEVICE ZXSPECTRUM48

; --------------------------------------------------
; Programa de test de STACK con CALL / RET
; --------------------------------------------------
; Cargar en $8000
; Observar:
;   - En cada CALL se deben escribir 2 bytes en SP
;   - Dirección guardada = PC de retorno
; --------------------------------------------------

        ORG $8000

START:
        DI                  ; evitar interrupciones
        LD SP, $9000        ; stack en zona limpia

        CALL FUNC1          ; CALL nivel 1
        CALL FUNC2          ; CALL nivel 1 (otro)

END_LOOP:
        JR END_LOOP         ; bucle infinito (para debug)

; --------------------------------------------------
; FUNCIONES
; --------------------------------------------------

FUNC1:
        CALL FUNC1_A        ; CALL nivel 2
        RET

FUNC1_A:
        CALL FUNC1_B        ; CALL nivel 3
        RET

FUNC1_B:
        RET

FUNC2:
        CALL FUNC2_A
        RET

FUNC2_A:
        RET

; --------------------------------------------------
; BINARIO
; --------------------------------------------------
        SAVEBIN "stack_call_test.bin", $8000, $
