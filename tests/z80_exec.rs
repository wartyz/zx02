use zilog_z80::cpu::CPU;
const Z80_MEM_SIZE: u16 = 0xFFFF;
#[test]
fn test_ld_inc_a() {
    let mut cpu = CPU::new(Z80_MEM_SIZE);

    // Programa:
    // LD A,0x42
    // INC A
    // HALT
    cpu.bus.write_byte(0x0000, 0x3E);
    cpu.bus.write_byte(0x0001, 0x42);
    cpu.bus.write_byte(0x0002, 0x3C);
    cpu.bus.write_byte(0x0003, 0x76);

    cpu.reg.pc = 0x0000;

    // Ejecutar instrucciones
    cpu.execute(); // LD A,0x42
    assert_eq!(cpu.reg.a, 0x42);

    cpu.execute(); // INC A
    assert_eq!(cpu.reg.a, 0x43);
}

// ADD A,B
// 
// Comprueba suma básica entre registros.
#[test]
fn test_add_a_b() {
    let mut cpu = CPU::new(0xFFFF);

    // LD A,0x10
    // LD B,0x05
    // ADD A,B
    cpu.bus.write_byte(0x0000, 0x3E);
    cpu.bus.write_byte(0x0001, 0x10);
    cpu.bus.write_byte(0x0002, 0x06);
    cpu.bus.write_byte(0x0003, 0x05);
    cpu.bus.write_byte(0x0004, 0x80); // ADD A,B

    cpu.reg.pc = 0x0000;

    cpu.execute();
    cpu.execute();
    cpu.execute();

    assert_eq!(cpu.reg.a, 0x15);
}

// SUB C
// 
// Comprueba resta y orden de operandos.

#[test]
fn test_sub_c() {
    let mut cpu = CPU::new(0xFFFF);

    // LD A,0x20
    // LD C,0x08
    // SUB C
    cpu.bus.write_byte(0x0000, 0x3E);
    cpu.bus.write_byte(0x0001, 0x20);
    cpu.bus.write_byte(0x0002, 0x0E);
    cpu.bus.write_byte(0x0003, 0x08);
    cpu.bus.write_byte(0x0004, 0x91); // SUB C

    cpu.reg.pc = 0x0000;

    cpu.execute();
    cpu.execute();
    cpu.execute();

    assert_eq!(cpu.reg.a, 0x18);
}

// ADC A,B (con carry)
// 
// Aquí comprobamos uso del flag C.

#[test]
fn test_adc_a_b_with_carry() {
    let mut cpu = CPU::new(0xFFFF);

    // LD A,0xFF
    // LD B,0x01
    // ADD A,B  -> A = 0x00, C = 1
    // ADC A,B  -> A = 0x02
    cpu.bus.write_byte(0x0000, 0x3E);
    cpu.bus.write_byte(0x0001, 0xFF);
    cpu.bus.write_byte(0x0002, 0x06);
    cpu.bus.write_byte(0x0003, 0x01);
    cpu.bus.write_byte(0x0004, 0x80); // ADD A,B
    cpu.bus.write_byte(0x0005, 0x88); // ADC A,B

    cpu.reg.pc = 0x0000;

    for _ in 0..4 {
        cpu.execute();
    }

    assert_eq!(cpu.reg.a, 0x02);
}

// JR +2 (salto hacia delante)
// 
// Este test detecta errores de PC muy comunes.

#[test]
fn test_jr_forward() {
    let mut cpu = CPU::new(0xFFFF);

    // JR +2
    // NOP        (saltado)
    // LD A,0x42
    cpu.bus.write_byte(0x0000, 0x18); // JR
    cpu.bus.write_byte(0x0001, 0x02);
    cpu.bus.write_byte(0x0002, 0x00); // NOP
    cpu.bus.write_byte(0x0003, 0x3E); // LD A,0x42
    cpu.bus.write_byte(0x0004, 0x42);

    cpu.reg.pc = 0x0000;

    cpu.execute(); // JR
    cpu.execute(); // LD A,0x42

    assert_eq!(cpu.reg.a, 0x42);
}

// JR -2 (salto hacia atrás)
// 
// Este es el test más importante para saltos.
#[test]
fn test_jr_backward() {
    let mut cpu = CPU::new(0xFFFF);

    // LD A,0x01
    // JR -2 (vuelve al LD)
    cpu.bus.write_byte(0x0000, 0x3E);
    cpu.bus.write_byte(0x0001, 0x01);
    cpu.bus.write_byte(0x0002, 0x18); // JR
    cpu.bus.write_byte(0x0003, 0xFE); // -2

    cpu.reg.pc = 0x0000;

    cpu.execute(); // LD A,1
    cpu.execute(); // JR -2
    cpu.execute(); // LD A,1 otra vez

    assert_eq!(cpu.reg.a, 0x01);
    assert_eq!(cpu.reg.pc, 0x0002);
}
// HALT
// 
// Comprueba que el PC no avanza.
#[test]
fn test_halt() {
    let mut cpu = CPU::new(0xFFFF);

    cpu.bus.write_byte(0x0000, 0x76); // HALT
    cpu.bus.write_byte(0x0001, 0x3E); // LD A,0x99 (no debe ejecutarse)
    cpu.bus.write_byte(0x0002, 0x99);

    cpu.reg.pc = 0x0000;

    cpu.execute(); // HALT
    let pc_after = cpu.reg.pc;

    cpu.execute(); // no debería avanzar

    assert_eq!(cpu.reg.pc, pc_after);
    assert_eq!(cpu.reg.a, 0x00);
}

// BIT 7,A (CB 7F)
// 
// Comprueba:
// 
// lectura de bit
// 
// no modifica A
// 
// flag Z correcto
#[test]
fn test_cb_bit_7_a() {
    let mut cpu = CPU::new(0xFFFF);

    // LD A,0x80
    // CB 7F   -> BIT 7,A
    cpu.bus.write_byte(0x0000, 0x3E);
    cpu.bus.write_byte(0x0001, 0x80);
    cpu.bus.write_byte(0x0002, 0xCB);
    cpu.bus.write_byte(0x0003, 0x7F);

    cpu.reg.pc = 0x0000;

    cpu.execute(); // LD A,0x80
    cpu.execute(); // BIT 7,A

    assert_eq!(cpu.reg.a, 0x80);

    let f = (cpu.reg.get_af() & 0x00FF) as u8;
    // Z debe ser 0 porque el bit 7 está a 1
    assert_eq!(f & 0x40, 0x00);
}
// BIT 0,B con bit a 0 (CB 40)
// 
// Comprueba flag Z = 1.

#[test]
fn test_cb_bit_0_b_zero() {
    let mut cpu = CPU::new(0xFFFF);

    // LD B,0x00
    // CB 40   -> BIT 0,B
    cpu.bus.write_byte(0x0000, 0x06);
    cpu.bus.write_byte(0x0001, 0x00);
    cpu.bus.write_byte(0x0002, 0xCB);
    cpu.bus.write_byte(0x0003, 0x40);

    cpu.reg.pc = 0x0000;

    cpu.execute(); // LD B,0
    cpu.execute(); // BIT 0,B

    assert_eq!(cpu.reg.b, 0x00);
    let f = (cpu.reg.get_af() & 0x00FF) as u8;
    // Z debe ser 1
    assert_eq!(f & 0x40, 0x40);
}
// SET 0,C (CB C1)
// 
// Comprueba modificación del registro.
#[test]
fn test_cb_set_0_c() {
    let mut cpu = CPU::new(0xFFFF);

    // LD C,0x00
    // CB C1   -> SET 0,C
    cpu.bus.write_byte(0x0000, 0x0E);
    cpu.bus.write_byte(0x0001, 0x00);
    cpu.bus.write_byte(0x0002, 0xCB);
    cpu.bus.write_byte(0x0003, 0xC1);

    cpu.reg.pc = 0x0000;

    cpu.execute(); // LD C,0
    cpu.execute(); // SET 0,C

    assert_eq!(cpu.reg.c, 0x01);
}
// RES 1,D (CB 8A)
// 
// Comprueba borrado de bit.
#[test]
fn test_cb_res_1_d() {
    let mut cpu = CPU::new(0xFFFF);

    // LD D,0xFF
    // CB 8A   -> RES 1,D
    cpu.bus.write_byte(0x0000, 0x16);
    cpu.bus.write_byte(0x0001, 0xFF);
    cpu.bus.write_byte(0x0002, 0xCB);
    cpu.bus.write_byte(0x0003, 0x8A);

    cpu.reg.pc = 0x0000;

    cpu.execute(); // LD D,0xFF
    cpu.execute(); // RES 1,D

    assert_eq!(cpu.reg.d, 0xFD);
}
// RLC A (CB 07)
// 
// Comprueba:
// 
// rotación
// 
// bit 7 → carry
// 
// resultado correcto
#[test]
fn test_cb_rlc_a() {
    let mut cpu = CPU::new(0xFFFF);

    // LD A,0x81
    // CB 07   -> RLC A
    cpu.bus.write_byte(0x0000, 0x3E);
    cpu.bus.write_byte(0x0001, 0x81);
    cpu.bus.write_byte(0x0002, 0xCB);
    cpu.bus.write_byte(0x0003, 0x07);

    cpu.reg.pc = 0x0000;

    cpu.execute(); // LD A,0x81
    cpu.execute(); // RLC A
    let f = (cpu.reg.get_af() & 0x00FF) as u8;
    // 0x81 -> 0x03, carry = 1
    assert_eq!(cpu.reg.a, 0x03);
    assert_eq!(f & 0x01, 0x01); // C = 1
}

// LD IX,nn
// 
// Comprueba carga directa de IX.

#[test]
fn test_ld_ix_nn() {
    let mut cpu = CPU::new(0xFFFF);

    // LD IX,0x1234
    cpu.bus.write_byte(0x0000, 0xDD);
    cpu.bus.write_byte(0x0001, 0x21);
    cpu.bus.write_byte(0x0002, 0x34);
    cpu.bus.write_byte(0x0003, 0x12);

    cpu.reg.pc = 0x0000;

    cpu.execute();

    assert_eq!(cpu.reg.get_ix(), 0x1234);
}
// LD (IX+1),A
// 
// Escritura en memoria con desplazamiento positivo.
#[test]
fn test_ld_ix_plus_d_a() {
    let mut cpu = CPU::new(0xFFFF);

    cpu.reg.set_ix(0x2000);
    cpu.reg.a = 0x42;

    // LD (IX+1),A
    cpu.bus.write_byte(0x0000, 0xDD);
    cpu.bus.write_byte(0x0001, 0x77);
    cpu.bus.write_byte(0x0002, 0x01);

    cpu.reg.pc = 0x0000;

    cpu.execute();

    assert_eq!(cpu.bus.read_byte(0x2001), 0x42);
}
// LD A,(IX-1)
// 
// Lectura de memoria con desplazamiento negativo.
#[test]
fn test_ld_a_ix_minus_d() {
    let mut cpu = CPU::new(0xFFFF);

    cpu.reg.set_ix(0x3000);
    cpu.bus.write_byte(0x2FFF, 0x99);

    // LD A,(IX-1)
    cpu.bus.write_byte(0x0000, 0xDD);
    cpu.bus.write_byte(0x0001, 0x7E);
    cpu.bus.write_byte(0x0002, 0xFF); // -1

    cpu.reg.pc = 0x0000;

    cpu.execute();

    assert_eq!(cpu.reg.a, 0x99);
}

// BIT 0,(IX+0) (DD CB d xx)
// 
// Este es el más crítico: prefijo doble + CB.
#[test]
fn test_cb_bit_ix() {
    let mut cpu = CPU::new(0xFFFF);

    cpu.reg.set_ix(0x4000);
    cpu.bus.write_byte(0x4000, 0x01); // bit 0 = 1

    // BIT 0,(IX+0)
    cpu.bus.write_byte(0x0000, 0xDD);
    cpu.bus.write_byte(0x0001, 0xCB);
    cpu.bus.write_byte(0x0002, 0x00);
    cpu.bus.write_byte(0x0003, 0x46);

    cpu.reg.pc = 0x0000;

    cpu.execute();

    let f = (cpu.reg.get_af() & 0x00FF) as u8;
    // Z debe ser 0 porque el bit está a 1
    assert_eq!(f & 0x40, 0x00);
}

// SET 7,(IY+2)
// 
// Comprueba escritura con IY.
#[test]
fn test_set_iy() {
    let mut cpu = CPU::new(0xFFFF);

    cpu.reg.set_iy(0x5000);
    cpu.bus.write_byte(0x5002, 0x00);

    // SET 7,(IY+2)
    cpu.bus.write_byte(0x0000, 0xFD);
    cpu.bus.write_byte(0x0001, 0xCB);
    cpu.bus.write_byte(0x0002, 0x02);
    cpu.bus.write_byte(0x0003, 0xFE);

    cpu.reg.pc = 0x0000;

    cpu.execute();

    assert_eq!(cpu.bus.read_byte(0x5002), 0x80);
}

// CALL nn + RET
// 
// Este es el test más importante de todos.
#[test]
fn test_call_ret() {
    let mut cpu = CPU::new(0xFFFF);

    // Programa:
    // 0000: CALL 0005
    // 0003: HALT
    // 0005: LD A,0x42
    // 0007: RET

    cpu.bus.write_byte(0x0000, 0xCD); // CALL
    cpu.bus.write_byte(0x0001, 0x05);
    cpu.bus.write_byte(0x0002, 0x00);

    cpu.bus.write_byte(0x0003, 0x76); // HALT

    cpu.bus.write_byte(0x0005, 0x3E); // LD A,0x42
    cpu.bus.write_byte(0x0006, 0x42);
    cpu.bus.write_byte(0x0007, 0xC9); // RET

    cpu.reg.pc = 0x0000;
    cpu.reg.sp = 0xFFFE;

    cpu.execute(); // CALL 0005
    assert_eq!(cpu.reg.pc, 0x0005);

    cpu.execute(); // LD A,0x42
    assert_eq!(cpu.reg.a, 0x42);

    cpu.execute(); // RET
    assert_eq!(cpu.reg.pc, 0x0003);
}
// RST 38h
// 
// RST es como un CALL corto a una dirección fija.
#[test]
#[test]
fn test_rst_38() {
    let mut cpu = CPU::new(0xFFFF);

    // RST 38h
    cpu.bus.write_byte(0x0000, 0xFF);

    cpu.reg.pc = 0x0000;
    cpu.reg.sp = 0xFFFE;

    cpu.execute();

    // Salto correcto
    assert_eq!(cpu.reg.pc, 0x0038);

    // Dirección de retorno apilada (según implementación del core)
    let lo = cpu.bus.read_byte(0xFFFE);
    let hi = cpu.bus.read_byte(0xFFFD);
    let ret = ((hi as u16) << 8) | (lo as u16);

    // El core apila el PC actual (0x0000)
    assert_eq!(ret, 0x0000);
}

