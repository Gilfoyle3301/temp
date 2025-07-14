use reqwest;
use serde_json::json;

use super::consts::{REG_COUNT, XOR_KEY};

type Reg = usize;

#[derive(Debug)]
enum Instruction {
    Mov { reg: Reg, val: u64 },
    Add { reg: Reg, val: u64 },
    Jeq { reg: Reg, cmp: u64, jmp: usize },
    Call { addr: usize },
    Nop,
}
// payload: [tag][encrypted payload][CRC] -> [1b][1b][4b]
pub fn handle_payload(input: &[u8]) {
    if input.len() < 5 {
        println!("Payload too short");
        return;
    }

    let tag: u8 = input[0];
    //type of payload: 0x03 -> VM Payload
    if tag != 0x03 {
        println!("Invalid payload type: {:?}", tag);
        return;
    }
    // Get encrypted body
    let encrypted = &input[1..];
    if encrypted.len() < 5 {
        println!("Encrypted body too short");
        return;
    }
    // Get encrypted payload
    let e_payload: &[u8] = &encrypted[..encrypted.len() - 4];
    println!("Encrypted payload: {:?}", e_payload);
    // Get crc
    let crc_body: &[u8] = &encrypted[encrypted.len() - 4..];
    println!("CRC body: {:?}", crc_body);

    let d_payload: Vec<u8> = e_payload.iter().map(|b| b ^ XOR_KEY).collect();
    println!("Decrypted payload: {:?}", d_payload);
    let checksum_payload = crc32fast::hash(&d_payload);
    println!("Calculated checksum: {:?}", checksum_payload);
    let expected_checksum = u32::from_le_bytes(crc_body.try_into().unwrap());
    println!("Expected checksum: {:?}", expected_checksum);

    if checksum_payload != expected_checksum {
        println!("Checksum mismatch");
        return;
    }
    println!("Payload decrypted successfully");
    let program = parse_program(&d_payload);
    println!("Program length: {:?}; Program {:?}", program.len(), program);
    run_vm(program);
}

fn parse_program(bytes: &[u8]) -> Vec<Instruction> {
    let mut program_counter: usize = 0;
    let mut program = Vec::new();

    while program_counter < bytes.len() {
        match bytes[program_counter] {
            // [op code: 1B][reg: 1B][val: 8B] (10B)
            0x01 => {
                // MOV
                if program_counter + 9 > bytes.len() {
                    break;
                }
                let reg = bytes[program_counter + 1] as usize;
                let val = u64::from_le_bytes(
                    bytes[program_counter + 2..program_counter + 10]
                        .try_into()
                        .unwrap(),
                );
                program.push(Instruction::Mov { reg, val });
                program_counter += 10;
            }
            0x02 => {
                // ADD
                if program_counter + 9 > bytes.len() {
                    break;
                }
                let reg = bytes[program_counter + 1] as usize;
                let val = u64::from_le_bytes(
                    bytes[program_counter + 2..program_counter + 10]
                        .try_into()
                        .unwrap(),
                );
                program.push(Instruction::Add { reg, val });
                program_counter += 10;
            }

            0x03 => {
                //JEQ
                // [op code: 1B][reg: 1B][val: 8B][jmp: 1B] (11B)
                if program_counter + 11 > bytes.len() {
                    break;
                }
                let reg = bytes[program_counter + 1] as usize;
                let val = u64::from_le_bytes(
                    bytes[program_counter + 2..program_counter + 10]
                        .try_into()
                        .unwrap(),
                );
                let jmp = bytes[program_counter + 10] as usize;
                program.push(Instruction::Jeq { reg, cmp: val, jmp });
                program_counter += 11;
            }
            0x04 => {
                // CALL
                // [op code: 1B][addr: 8B]
                if program_counter + 8 > bytes.len() {
                    break;
                }
                let addr = u64::from_le_bytes(
                    bytes[program_counter + 1..program_counter + 9]
                        .try_into()
                        .unwrap(),
                ) as usize;
                program.push(Instruction::Call { addr });
                program_counter += 9;
            }

            0x00 => {
                // NOP
                program.push(Instruction::Nop);
                program_counter += 1;
            }

            _ => break,
        }
    }
    program
}

fn run_vm(instraction: Vec<Instruction>) {
    let mut registers: [u64; REG_COUNT] = [0; REG_COUNT];
    let mut program_count = 0;
    let mut vm_allowed_to_call = false;

    while program_count < instraction.len() {
        match &instraction[program_count] {
            Instruction::Mov { reg, val } => {
                if *reg < REG_COUNT {
                    registers[*reg] = *val;
                }
            }
            Instruction::Add { reg, val } => {
                if *reg < REG_COUNT {
                    registers[*reg] = registers[*reg].wrapping_add(*val);
                }
            }
            Instruction::Jeq { reg, cmp, jmp } => {
                if *reg < REG_COUNT && registers[*reg] == *cmp {
                    if *reg == 0 && *cmp == 3826 {
                        vm_allowed_to_call = true;
                    }
                    program_count = *jmp;
                    continue;
                }
            }
            Instruction::Call { addr } => {
                if vm_allowed_to_call {
                    unsafe {
                        let f: fn() = std::mem::transmute(*addr);
                        f();
                    }
                }
            }
            Instruction::Nop => {}
        }
        program_count += 1;
    }
}

#[unsafe(no_mangle)]
pub fn send_flag() {
    let client = reqwest::blocking::Client::new();
    let flag = client
        .post("http://localhost:8080/api/v1/secret/flags") // need env or args
        .json(&json!({"token": "CTF{super_ctf_mastermind}"}))
        .send();

    match flag {
        Ok(resp) => match resp.text() {
            Ok(text) => println!("YOU ARE CODE: {}", text),
            Err(e) => eprintln!("ERR READ INFO: {}", e),
        },
        Err(e) => eprintln!("ERR SEND TOKEN: {}", e),
    }
}
