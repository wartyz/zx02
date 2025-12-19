/*pub fn disassemble(mem: &[u8], pc: u16, base: u16) -> (String, u8) {
    let index = pc.wrapping_sub(base) as usize;
    if index >= mem.len() {
        return ("<fuera de memoria>".to_string(), 1);
    }

    let b0 = mem[index];

    let regs = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];
    let rp = ["BC", "DE", "HL", "SP"];
    let cond = ["NZ", "Z", "NC", "C"];

    match b0 {
        /* ==================================================
         * PREFIJO ED
         * ================================================== */

        0xED => {
            if index + 1 >= mem.len() {
                return ("ED <incompleto>".to_string(), 1);
            }

            let b1 = mem[index + 1];

            match b1 {
                /* ---------------------------------
                 * ADC HL,rr
                 * --------------------------------- */
                0x4A | 0x5A | 0x6A | 0x7A => {
                    let r = (b1 >> 4) & 3;
                    (format!("ADC HL,{}", rp[r as usize]), 2)
                }

                /* ---------------------------------
                 * SBC HL,rr
                 * --------------------------------- */
                0x42 | 0x52 | 0x62 | 0x72 => {
                    let r = (b1 >> 4) & 3;
                    (format!("SBC HL,{}", rp[r as usize]), 2)
                }

                /* ---------------------------------
                 * LD (nn),rr
                 * --------------------------------- */
                0x43 | 0x53 | 0x63 | 0x73 => {
                    if index + 3 >= mem.len() {
                        return ("LD (nn),rr <incompleto>".to_string(), 2);
                    }

                    let r = (b1 >> 4) & 3;
                    let lo = mem[index + 2] as u16;
                    let hi = mem[index + 3] as u16;
                    let addr = (hi << 8) | lo;

                    (
                        format!("LD (0x{:04X}),{}", addr, rp[r as usize]),
                        4,
                    )
                }

                /* ---------------------------------
                 * IM 0 / 1 / 2
                 * --------------------------------- */
                0x46 => ("IM 0".to_string(), 2),
                0x56 => ("IM 1".to_string(), 2),
                0x5E => ("IM 2".to_string(), 2),

                /* ---------------------------------
                 * RETN / RETI
                 * --------------------------------- */
                0x45 => ("RETN".to_string(), 2),
                0x4D => ("RETI".to_string(), 2),

                /* ---------------------------------
                 * NEG
                 * --------------------------------- */
                0x44 | 0x4C | 0x54 | 0x5C |
                0x64 | 0x6C | 0x74 | 0x7C => {
                    ("NEG".to_string(), 2)
                }

                /* ---------------------------------
                 * LD I,A / LD R,A / LD A,I / LD A,R
                 * --------------------------------- */
                0x47 => ("LD I,A".to_string(), 2),
                0x4F => ("LD R,A".to_string(), 2),
                0x57 => ("LD A,I".to_string(), 2),
                0x5F => ("LD A,R".to_string(), 2),

                /* ---------------------------------
                 * IN r,(C)
                 * --------------------------------- */
                b1 if (b1 & 0xC7) == 0x40 => {
                    let r = (b1 >> 3) & 7;
                    (format!("IN {},(C)", regs[r as usize]), 2)
                }

                /* ---------------------------------
                 * OUT (C),r
                 * --------------------------------- */
                b1 if (b1 & 0xC7) == 0x41 => {
                    let r = (b1 >> 3) & 7;
                    (format!("OUT (C),{}", regs[r as usize]), 2)
                }

                /* ---------------------------------
                 * Block transfer
                 * --------------------------------- */
                0xA0 => ("LDI".to_string(), 2),
                0xA8 => ("LDD".to_string(), 2),
                0xB0 => ("LDIR".to_string(), 2),
                0xB8 => ("LDDR".to_string(), 2),

                _ => (format!("DB 0xED,0x{:02X}", b1), 2),
            }
        }


        /* ==================================================
         * LD rr,nn
         * ================================================== */
        0x01 | 0x11 | 0x21 | 0x31 => {
            if index + 2 >= mem.len() {
                return ("LD rr,<incompleto>".to_string(), 1);
            }
            let r = (b0 >> 4) & 3;
            let lo = mem[index + 1] as u16;
            let hi = mem[index + 2] as u16;
            let nn = (hi << 8) | lo;
            (format!("LD {},0x{:04X}", rp[r as usize], nn), 3)
        }

        /* ==================================================
         * JP nn  <<< NUEVO >>>
         * ================================================== */
        0xC3 => {
            if index + 2 >= mem.len() {
                return ("JP <incompleto>".to_string(), 1);
            }
            let lo = mem[index + 1] as u16;
            let hi = mem[index + 2] as u16;
            let addr = (hi << 8) | lo;
            (format!("JP 0x{:04X}", addr), 3)
        }

        /* ==================================================
         * ADD HL,rr
         * ================================================== */
        0x09 | 0x19 | 0x29 | 0x39 => {
            let r = (b0 >> 4) & 3;
            (format!("ADD HL,{}", rp[r as usize]), 1)
        }

        /* ==================================================
         * INC r
         * ================================================== */
        0x04 | 0x0C | 0x14 | 0x1C |
        0x24 | 0x2C | 0x34 | 0x3C => {
            let r = (b0 >> 3) & 7;
            (format!("INC {}", regs[r as usize]), 1)
        }

        /* ==================================================
         * INC rr
         * ================================================== */
        0x03 | 0x13 | 0x23 | 0x33 => {
            let r = (b0 >> 4) & 3;
            (format!("INC {}", rp[r as usize]), 1)
        }

        /* ==================================================
         * DEC rr
         * ================================================== */
        0x0B | 0x1B | 0x2B | 0x3B => {
            let r = (b0 >> 4) & 3;
            (format!("DEC {}", rp[r as usize]), 1)
        }

        /* ==================================================
         * DEC r
         * ================================================== */
        0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
            let r = (b0 >> 3) & 7;
            (format!("DEC {}", regs[r as usize]), 1)
        }

        /* ==================================================
         * AND r
         * ================================================== */
        0xA0..=0xA7 => {
            let r = b0 & 7;
            (format!("AND {}", regs[r as usize]), 1)
        }

        /* ==================================================
         * XOR r
         * ================================================== */
        0xA8..=0xAF => {
            let r = b0 & 7;
            (format!("XOR {}", regs[r as usize]), 1)
        }

        /* ==================================================
         * CP r
         * ================================================== */
        0xB8..=0xBF => {
            let r = b0 & 7;
            (format!("CP {}", regs[r as usize]), 1)
        }

        /* ==================================================
         * JR condicional
         * ================================================== */
        0x20 | 0x28 | 0x30 | 0x38 => {
            let cc = (b0 >> 3) & 3;
            let disp = mem[index + 1] as i8;
            let target = pc.wrapping_add(2).wrapping_add(disp as u16);
            (format!("JR {},0x{:04X}", cond[cc as usize], target), 2)
        }

        /* ==================================================
         * JR
         * ================================================== */
        0x18 => {
            let disp = mem[index + 1] as i8;
            let target = pc.wrapping_add(2).wrapping_add(disp as u16);
            (format!("JR 0x{:04X}", target), 2)
        }

        /* ==================================================
         * DJNZ
         * ================================================== */
        0x10 => {
            if index + 1 >= mem.len() {
                return ("DJNZ <incompleto>".to_string(), 1);
            }

            let disp = mem[index + 1] as i8;
            let target = pc.wrapping_add(2).wrapping_add(disp as u16);

            (format!("DJNZ 0x{:04X}", target), 2)
        }

        /* ==================================================
         * EXX
         * ================================================== */
        0xD9 => ("EXX".to_string(), 1),

        /* ==================================================
         * DI / EI
         * ================================================== */
        0xF3 => ("DI".to_string(), 1),
        0xFB => ("EI".to_string(), 1),

        /* ==================================================
         * XOR n
         * ================================================== */
        0xEE => {
            let n = mem[index + 1];
            (format!("XOR 0x{:02X}", n), 2)
        }

        /* ==================================================
         * OR r
         * ================================================== */
        0xB0..=0xB7 => {
            let r = b0 & 7;
            (format!("OR {}", regs[r as usize]), 1)
        }

        /* ==================================================
         * SUB r
         * ================================================== */
        0x90..=0x97 => {
            let r = b0 & 7;
            (format!("SUB {}", regs[r as usize]), 1)
        }

        /* ==================================================
         * ADD A,r
         * ================================================== */
        0x80..=0x87 => {
            let r = b0 & 7;
            (format!("ADD A,{}", regs[r as usize]), 1)
        }

        /* ==================================================
         * ADC A,r
         * ================================================== */
        0x88..=0x8F => {
            let r = b0 & 7;
            (format!("ADC A,{}", regs[r as usize]), 1)
        }

        /* ==================================================
         * SBC A,r
         * ================================================== */
        0x98..=0x9F => {
            let r = b0 & 7;
            (format!("SBC A,{}", regs[r as usize]), 1)
        }




        /* ==================================================
         * OUT / IN
         * ================================================== */
        0xD3 => {
            let p = mem[index + 1];
            (format!("OUT (0x{:02X}),A", p), 2)
        }
        0xDB => {
            let p = mem[index + 1];
            (format!("IN A,(0x{:02X})", p), 2)
        }

        /* ==================================================
         * LD r,n
         * ================================================== */
        0x06 | 0x0E | 0x16 | 0x1E |
        0x26 | 0x2E | 0x36 | 0x3E => {
            if index + 1 >= mem.len() {
                return ("LD r,<incompleto>".to_string(), 1);
            }
            let r = (b0 >> 3) & 7;
            let n = mem[index + 1];
            (format!("LD {},0x{:02X}", regs[r as usize], n), 2)
        }

        /* ==================================================
         * LD r,r
         * ================================================== */
        0x40..=0x7F => {
            if b0 == 0x76 {
                ("HALT".to_string(), 1)
            } else {
                let dst = (b0 >> 3) & 7;
                let src = b0 & 7;
                (
                    format!("LD {},{}", regs[dst as usize], regs[src as usize]),
                    1,
                )
            }
        }

        /* ==================================================
         * LD (nn),HL  /  LD HL,(nn)
         * ================================================== */
        0x22 | 0x2A => {
            if index + 2 >= mem.len() {
                return ("LD (nn),HL <incompleto>".to_string(), 1);
            }

            let lo = mem[index + 1] as u16;
            let hi = mem[index + 2] as u16;
            let addr = (hi << 8) | lo;

            if b0 == 0x22 {
                (format!("LD (0x{:04X}),HL", addr), 3)
            } else {
                (format!("LD HL,(0x{:04X})", addr), 3)
            }
        }

        /* ==================================================
         * LD (nn),A  /  LD A,(nn)
         * ================================================== */
        0x32 | 0x3A => {
            if index + 2 >= mem.len() {
                return ("LD (nn),A <incompleto>".to_string(), 1);
            }

            let lo = mem[index + 1] as u16;
            let hi = mem[index + 2] as u16;
            let addr = (hi << 8) | lo;

            if b0 == 0x32 {
                (format!("LD (0x{:04X}),A", addr), 3)
            } else {
                (format!("LD A,(0x{:04X})", addr), 3)
            }
        }

        /* ==================================================
         * CALL nn
         * ================================================== */
        0xCD => {
            if index + 2 >= mem.len() {
                return ("CALL <incompleto>".to_string(), 1);
            }

            let lo = mem[index + 1] as u16;
            let hi = mem[index + 2] as u16;
            let addr = (hi << 8) | lo;

            (format!("CALL 0x{:04X}", addr), 3)
        }

        /* ==================================================
         * RET cc
         * ================================================== */
        0xC0 | 0xC8 | 0xD0 | 0xD8 |
        0xE0 | 0xE8 | 0xF0 | 0xF8 => {
            let cc = (b0 >> 3) & 7;
            let conds = ["NZ", "Z", "NC", "C", "PO", "PE", "P", "M"];
            (format!("RET {}", conds[cc as usize]), 1)
        }

        /* ==================================================
         * RET
         * ================================================== */
        0xC9 => ("RET".to_string(), 1),

        /* ==================================================
         * POP rr
         * ================================================== */
        0xC1 | 0xD1 | 0xE1 | 0xF1 => {
            let rp2 = ["BC", "DE", "HL", "AF"];
            let r = (b0 >> 4) & 3;
            (format!("POP {}", rp2[r as usize]), 1)
        }

        /* ==================================================
         * PUSH rr
         * ================================================== */
        0xC5 | 0xD5 | 0xE5 | 0xF5 => {
            let rp2 = ["BC", "DE", "HL", "AF"];
            let r = (b0 >> 4) & 3;
            (format!("PUSH {}", rp2[r as usize]), 1)
        }

        /* ==================================================
         * EX DE,HL
         * ================================================== */
        0xEB => ("EX DE,HL".to_string(), 1),

        /* ==================================================
         * LD SP,HL
         * ================================================== */
        0xF9 => ("LD SP,HL".to_string(), 1),

        /* ==================================================
         * PREFIJO FD  (IY)
         * ================================================== */
        0xFD => {
            if index + 1 >= mem.len() {
                return ("FD <incompleto>".to_string(), 1);
            }

            let b1 = mem[index + 1];

            match b1 {
                /* LD IY,nn */
                0x21 => {
                    if index + 3 >= mem.len() {
                        return ("LD IY,<incompleto>".to_string(), 2);
                    }

                    let lo = mem[index + 2] as u16;
                    let hi = mem[index + 3] as u16;
                    let nn = (hi << 8) | lo;

                    (format!("LD IY,0x{:04X}", nn), 4)
                }

                /* ---------------------------------
                 * DEC (IY+d)
                 * --------------------------------- */
                0x35 => {
                    if index + 2 >= mem.len() {
                        return ("DEC (IY+?) <incompleto>".to_string(), 2);
                    }

                    let d = mem[index + 2];
                    (format!("DEC (IY+0x{:02X})", d), 3)
                }

                /* ---------------------------------
                 * FD CB d xx : BIT / RES / SET (IY+d)
                 * --------------------------------- */
                0xCB => {
                    if index + 3 >= mem.len() {
                        return ("FD CB <incompleto>".to_string(), 2);
                    }

                    let d = mem[index + 2];
                    let op = mem[index + 3];

                    let bit = (op >> 3) & 7;
                    let group = op >> 6;

                    let mnemonic = match group {
                        0b01 => format!("BIT {},(IY+0x{:02X})", bit, d),
                        0b10 => format!("RES {},(IY+0x{:02X})", bit, d),
                        0b11 => format!("SET {},(IY+0x{:02X})", bit, d),
                        _ => format!("DB 0xFD,0xCB,0x{:02X},0x{:02X}", d, op),
                    };

                    (mnemonic, 4)
                }

                /* ---------------------------------
                 * LD (IY+d),r
                 * --------------------------------- */
                0x70..=0x77 => {
                    if index + 2 >= mem.len() {
                        return ("LD (IY+d),r <incompleto>".to_string(), 2);
                    }

                    let d = mem[index + 2];
                    let r = b1 & 7;
                    let regs = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];

                    (format!("LD (IY+0x{:02X}),{}", d, regs[r as usize]), 3)
                }

                _ => (format!("DB 0xFD,0x{:02X}", b1), 2),
            }
        }



        /* ==================================================
         * BÁSICAS
         * ================================================== */
        0x00 => ("NOP".to_string(), 1),

        _ => (format!("DB 0x{:02X}", b0), 1),
    }
}
*/
/* =========================================================
 * Z80 DISASSEMBLER
 * =========================================================
 *
 * Organización:
 *
 * 1) Prefijos
 *    - ED
 *    - FD (IY)
 *
 * 2) ALU 16-bit
 *    - ADD HL,rr
 *    - INC rr
 *    - DEC rr
 *
 * 3) ALU 8-bit
 *    - ADD / ADC / SUB / SBC
 *    - AND / OR / XOR / CP
 *
 * 4) Control de flujo
 *    - JR / JR cc / DJNZ
 *    - JP / CALL
 *    - RET / RET cc
 *
 * 5) Stack
 *    - PUSH / POP
 *
 * 6) Load
 *    - LD rr,nn
 *    - LD r,n
 *    - LD r,r
 *    - LD (nn),xx
 *
 * 7) Especiales
 *    - EX / EXX
 *    - DI / EI
 *
 * 8) Básicas / fallback
 * =========================================================
 */

pub fn disassemble(mem: &[u8], pc: u16, base: u16) -> (String, u8) {
    let index = pc.wrapping_sub(base) as usize;
    if index >= mem.len() {
        return ("<fuera de memoria>".to_string(), 1);
    }

    let b0 = mem[index];

    match b0 {
        /* ==================================================
         * PREFIJOS
         * ================================================== */
        0xED => decode_ed(mem, index),
        0xFD => decode_fd(mem, index),

        /* ==================================================
         * ALU 16-bit
         * ================================================== */
        0x09 | 0x19 | 0x29 | 0x39 => decode_add_hl_rr(b0),
        0x03 | 0x13 | 0x23 | 0x33 => decode_inc_rr(b0),
        0x0B | 0x1B | 0x2B | 0x3B => decode_dec_rr(b0),

        /* ==================================================
         * ALU 8-bit
         * ================================================== */
        0x80..=0x87 => decode_alu_r("ADD", b0),
        0x88..=0x8F => decode_alu_r("ADC", b0),
        0x90..=0x97 => decode_alu_r("SUB", b0),
        0x98..=0x9F => decode_alu_r("SBC", b0),
        0xA0..=0xA7 => decode_alu_r("AND", b0),
        0xA8..=0xAF => decode_alu_r("XOR", b0),
        0xB0..=0xB7 => decode_alu_r("OR", b0),
        0xB8..=0xBF => decode_alu_r("CP", b0),

        /* ==================================================
         * AND n
         * ================================================== */
        0xE6 => {
            let n = mem[index + 1];
            (format!("AND 0x{:02X}", n), 2)
        }


        /* ==================================================
         * CONTROL DE FLUJO
         * ================================================== */
        0x18 => decode_jr(mem, pc, index),
        0x20 | 0x28 | 0x30 | 0x38 => decode_jr_cc(mem, pc, index, b0),
        0x10 => decode_djnz(mem, pc, index),
        0xC3 => decode_jp(mem, index),
        0xCD => decode_call(mem, index),

        0xC9 => ("RET".to_string(), 1),
        0xC0 | 0xC8 | 0xD0 | 0xD8 |
        0xE0 | 0xE8 | 0xF0 | 0xF8 => decode_ret_cc(b0),

        /* ==================================================
         * RST p
         * ================================================== */
        0xC7 | 0xCF | 0xD7 | 0xDF |
        0xE7 | 0xEF | 0xF7 | 0xFF => {
            let addr = (b0 & 0x38) as u16;
            (format!("RST 0x{:04X}", addr), 1)
        }


        /* ==================================================
         * STACK
         * ================================================== */
        0xC5 | 0xD5 | 0xE5 | 0xF5 => decode_push(b0),
        0xC1 | 0xD1 | 0xE1 | 0xF1 => decode_pop(b0),

        /* ==================================================
         * LOAD
         * ================================================== */
        0x01 | 0x11 | 0x21 | 0x31 => decode_ld_rr_nn(mem, index, b0),
        0x06 | 0x0E | 0x16 | 0x1E |
        0x26 | 0x2E | 0x36 | 0x3E => decode_ld_r_n(mem, index, b0),

        0x40..=0x7F => decode_ld_r_r(b0),
        0x22 | 0x2A => decode_ld_nn_hl(mem, index, b0),
        0x32 | 0x3A => decode_ld_nn_a(mem, index, b0),

        /* ==================================================
         * ESPECIALES
         * ================================================== */
        0xEB => ("EX DE,HL".to_string(), 1),
        0xD9 => ("EXX".to_string(), 1),
        0xF3 => ("DI".to_string(), 1),
        0xFB => ("EI".to_string(), 1),

        /* ==================================================
         * BÁSICAS
         * ================================================== */
        0x00 => ("NOP".to_string(), 1),
        //0x76 => ("HALT".to_string(), 1),

        _ => (format!("DB 0x{:02X}", b0), 1),
    }
}

/* =========================================================
 * TABLAS
 * ========================================================= */

fn regs() -> [&'static str; 8] {
    ["B", "C", "D", "E", "H", "L", "(HL)", "A"]
}

fn rp() -> [&'static str; 4] {
    ["BC", "DE", "HL", "SP"]
}

/* =========================================================
 * DECODERS
 * ========================================================= */

fn decode_add_hl_rr(b0: u8) -> (String, u8) {
    let r = (b0 >> 4) & 3;
    (format!("ADD HL,{}", rp()[r as usize]), 1)
}

fn decode_inc_rr(b0: u8) -> (String, u8) {
    let r = (b0 >> 4) & 3;
    (format!("INC {}", rp()[r as usize]), 1)
}

fn decode_dec_rr(b0: u8) -> (String, u8) {
    let r = (b0 >> 4) & 3;
    (format!("DEC {}", rp()[r as usize]), 1)
}

fn decode_alu_r(op: &str, b0: u8) -> (String, u8) {
    let r = b0 & 7;
    (format!("{} {}", op, regs()[r as usize]), 1)
}

/* ---- flujo ---- */

fn decode_jr(mem: &[u8], pc: u16, index: usize) -> (String, u8) {
    let d = mem[index + 1] as i8;
    let target = pc.wrapping_add(2).wrapping_add(d as u16);
    (format!("JR 0x{:04X}", target), 2)
}

fn decode_jr_cc(mem: &[u8], pc: u16, index: usize, b0: u8) -> (String, u8) {
    let conds = ["NZ", "Z", "NC", "C"];
    let cc = (b0 >> 3) & 3;
    let d = mem[index + 1] as i8;
    let target = pc.wrapping_add(2).wrapping_add(d as u16);
    (format!("JR {},0x{:04X}", conds[cc as usize], target), 2)
}

fn decode_djnz(mem: &[u8], pc: u16, index: usize) -> (String, u8) {
    let d = mem[index + 1] as i8;
    let target = pc.wrapping_add(2).wrapping_add(d as u16);
    (format!("DJNZ 0x{:04X}", target), 2)
}

fn decode_jp(mem: &[u8], index: usize) -> (String, u8) {
    let lo = mem[index + 1] as u16;
    let hi = mem[index + 2] as u16;
    (format!("JP 0x{:04X}", (hi << 8) | lo), 3)
}

fn decode_call(mem: &[u8], index: usize) -> (String, u8) {
    let lo = mem[index + 1] as u16;
    let hi = mem[index + 2] as u16;
    (format!("CALL 0x{:04X}", (hi << 8) | lo), 3)
}

fn decode_ret_cc(b0: u8) -> (String, u8) {
    let conds = ["NZ", "Z", "NC", "C", "PO", "PE", "P", "M"];
    let cc = (b0 >> 3) & 7;
    (format!("RET {}", conds[cc as usize]), 1)
}

/* ---- stack ---- */

fn decode_push(b0: u8) -> (String, u8) {
    let rp2 = ["BC", "DE", "HL", "AF"];
    let r = (b0 >> 4) & 3;
    (format!("PUSH {}", rp2[r as usize]), 1)
}

fn decode_pop(b0: u8) -> (String, u8) {
    let rp2 = ["BC", "DE", "HL", "AF"];
    let r = (b0 >> 4) & 3;
    (format!("POP {}", rp2[r as usize]), 1)
}

/* ---- load ---- */

fn decode_ld_rr_nn(mem: &[u8], index: usize, b0: u8) -> (String, u8) {
    let r = (b0 >> 4) & 3;
    let lo = mem[index + 1] as u16;
    let hi = mem[index + 2] as u16;
    (format!("LD {},0x{:04X}", rp()[r as usize], (hi << 8) | lo), 3)
}

fn decode_ld_r_n(mem: &[u8], index: usize, b0: u8) -> (String, u8) {
    let r = (b0 >> 3) & 7;
    let n = mem[index + 1];
    (format!("LD {},0x{:02X}", regs()[r as usize], n), 2)
}

fn decode_ld_r_r(b0: u8) -> (String, u8) {
    if b0 == 0x76 {
        ("HALT".to_string(), 1)
    } else {
        let d = (b0 >> 3) & 7;
        let s = b0 & 7;
        (format!("LD {},{}", regs()[d as usize], regs()[s as usize]), 1)
    }
}

fn decode_ld_nn_hl(mem: &[u8], index: usize, b0: u8) -> (String, u8) {
    let lo = mem[index + 1] as u16;
    let hi = mem[index + 2] as u16;
    let addr = (hi << 8) | lo;

    if b0 == 0x22 {
        (format!("LD (0x{:04X}),HL", addr), 3)
    } else {
        (format!("LD HL,(0x{:04X})", addr), 3)
    }
}

fn decode_ld_nn_a(mem: &[u8], index: usize, b0: u8) -> (String, u8) {
    let lo = mem[index + 1] as u16;
    let hi = mem[index + 2] as u16;
    let addr = (hi << 8) | lo;

    if b0 == 0x32 {
        (format!("LD (0x{:04X}),A", addr), 3)
    } else {
        (format!("LD A,(0x{:04X})", addr), 3)
    }
}

fn decode_ed(mem: &[u8], index: usize) -> (String, u8) {
    if index + 1 >= mem.len() {
        return ("ED <incompleto>".to_string(), 1);
    }

    let b1 = mem[index + 1];
    let rp = ["BC", "DE", "HL", "SP"];
    let regs = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];

    match b1 {
        /* ==================================================
         * ADC / SBC HL,rr
         * ================================================== */
        0x4A | 0x5A | 0x6A | 0x7A => {
            let r = (b1 >> 4) & 3;
            (format!("ADC HL,{}", rp[r as usize]), 2)
        }
        0x42 | 0x52 | 0x62 | 0x72 => {
            let r = (b1 >> 4) & 3;
            (format!("SBC HL,{}", rp[r as usize]), 2)
        }

        /* ==================================================
         * LD (nn),rr
         * ================================================== */
        0x43 | 0x53 | 0x63 | 0x73 => {
            if index + 3 >= mem.len() {
                return ("LD (nn),rr <incompleto>".to_string(), 2);
            }
            let r = (b1 >> 4) & 3;
            let lo = mem[index + 2] as u16;
            let hi = mem[index + 3] as u16;
            let addr = (hi << 8) | lo;
            (format!("LD (0x{:04X}),{}", addr, rp[r as usize]), 4)
        }

        /* ==================================================
         * LD rr,(nn)
         * ================================================== */
        0x4B | 0x5B | 0x6B | 0x7B => {
            if index + 3 >= mem.len() {
                return ("LD rr,(nn) <incompleto>".to_string(), 2);
            }
            let r = (b1 >> 4) & 3;
            let lo = mem[index + 2] as u16;
            let hi = mem[index + 3] as u16;
            let addr = (hi << 8) | lo;
            (format!("LD {},(0x{:04X})", rp[r as usize], addr), 4)
        }

        /* ==================================================
         * IM
         * ================================================== */
        0x46 => ("IM 0".to_string(), 2),
        0x56 => ("IM 1".to_string(), 2),
        0x5E => ("IM 2".to_string(), 2),

        /* ==================================================
         * RETN / RETI
         * ================================================== */
        0x45 => ("RETN".to_string(), 2),
        0x4D => ("RETI".to_string(), 2),

        /* ==================================================
         * NEG
         * ================================================== */
        0x44 | 0x4C | 0x54 | 0x5C |
        0x64 | 0x6C | 0x74 | 0x7C => ("NEG".to_string(), 2),

        /* ==================================================
         * LD I/R
         * ================================================== */
        0x47 => ("LD I,A".to_string(), 2),
        0x4F => ("LD R,A".to_string(), 2),
        0x57 => ("LD A,I".to_string(), 2),
        0x5F => ("LD A,R".to_string(), 2),

        /* ==================================================
         * IN / OUT (C)
         * ================================================== */
        b if (b & 0xC7) == 0x40 => {
            let r = (b >> 3) & 7;
            (format!("IN {},(C)", regs[r as usize]), 2)
        }
        b if (b & 0xC7) == 0x41 => {
            let r = (b >> 3) & 7;
            (format!("OUT (C),{}", regs[r as usize]), 2)
        }

        /* ==================================================
         * RRD / RLD
         * ================================================== */
        0x67 => ("RRD".to_string(), 2),
        0x6F => ("RLD".to_string(), 2),

        /* ==================================================
         * Block I/O
         * ================================================== */
        0xA2 => ("INI".to_string(), 2),
        0xAA => ("IND".to_string(), 2),
        0xB2 => ("INIR".to_string(), 2),
        0xBA => ("INDR".to_string(), 2),

        0xA3 => ("OUTI".to_string(), 2),
        0xAB => ("OUTD".to_string(), 2),
        0xB3 => ("OTIR".to_string(), 2),
        0xBB => ("OTDR".to_string(), 2),

        /* ==================================================
         * ED NOPs documentados
         * ================================================== */
        0x00 | 0xFF => ("NOP".to_string(), 2),

        _ => (format!("DB 0xED,0x{:02X}", b1), 2),
    }
}

fn decode_fd(mem: &[u8], index: usize) -> (String, u8) {
    if index + 1 >= mem.len() {
        return ("FD <incompleto>".to_string(), 1);
    }

    let b1 = mem[index + 1];
    let regs = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];

    match b1 {
        /* LD IY,nn */
        0x21 => {
            if index + 3 >= mem.len() {
                return ("LD IY,<incompleto>".to_string(), 2);
            }
            let lo = mem[index + 2] as u16;
            let hi = mem[index + 3] as u16;
            (format!("LD IY,0x{:04X}", (hi << 8) | lo), 4)
        }

        /* DEC (IY+d) */
        0x35 => {
            if index + 2 >= mem.len() {
                return ("DEC (IY+d) <incompleto>".to_string(), 2);
            }
            let d = mem[index + 2];
            (format!("DEC (IY+0x{:02X})", d), 3)
        }

        /* FD CB d xx : BIT / RES / SET */
        0xCB => {
            if index + 3 >= mem.len() {
                return ("FD CB <incompleto>".to_string(), 2);
            }

            let d = mem[index + 2];
            let op = mem[index + 3];
            let bit = (op >> 3) & 7;

            match op >> 6 {
                0b01 => (format!("BIT {},(IY+0x{:02X})", bit, d), 4),
                0b10 => (format!("RES {},(IY+0x{:02X})", bit, d), 4),
                0b11 => (format!("SET {},(IY+0x{:02X})", bit, d), 4),
                _ => (format!("DB 0xFD,0xCB,0x{:02X},0x{:02X}", d, op), 4),
            }
        }

        /* LD (IY+d),r */
        0x70..=0x77 => {
            if index + 2 >= mem.len() {
                return ("LD (IY+d),r <incompleto>".to_string(), 2);
            }
            let d = mem[index + 2];
            let r = b1 & 7;
            (format!("LD (IY+0x{:02X}),{}", d, regs[r as usize]), 3)
        }

        _ => (format!("DB 0xFD,0x{:02X}", b1), 2),
    }
}

