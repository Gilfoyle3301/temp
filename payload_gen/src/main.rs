use clap::Parser;
use crc32fast::Hasher;
use std::fs::File;
use std::io::Write;

const MOV: u8 = 0x01;
const ADD: u8 = 0x02;
const JEQ: u8 = 0x03;
const CALL: u8 = 0x04;
const NOP: u8 = 0x00;

const TAG: u8 = 0x03;

#[derive(Parser)]
struct Args {
    #[arg(short = 'i', long)]
    instruction: String,
    #[arg(short = 'a', long, default_value = "4195636")]
    call_addr: usize,
    #[arg(short = 'k', long, default_value = "0x5B")]
    xkey: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let tokens = args.instruction.split_whitespace().collect::<Vec<_>>();
    let xkey = u8::from_str_radix(&args.xkey.trim_start_matches("0x"), 16).unwrap();
    let mut program: Vec<u8> = Vec::new();
    let mut i = 0;
    println!("{:?}", tokens);
    //MOV 0 41 ADD 0 1 JEQ 0 42 3 CALL 4195636
    while i < tokens.len() {
        match tokens[i].to_uppercase().as_str() {
            "MOV" => {
                let register: u8 = tokens[i + 1].parse().unwrap();
                let value: u64 = tokens[i + 2].parse().unwrap();
                program.push(MOV);
                program.push(register);
                program.extend(&value.to_le_bytes());
                i += 3;
            }
            "ADD" => {
                let register: u8 = tokens[i + 1].parse().unwrap();
                let value: u64 = tokens[i + 2].parse().unwrap();
                program.push(ADD);
                program.push(register);
                program.extend(&value.to_le_bytes());
                i += 3;
            }
            "JEQ" => {
                let register: u8 = tokens[i + 1].parse().unwrap();
                let cmp: u64 = tokens[i + 2].parse().unwrap();
                let jmp: u8 = tokens[i + 3].parse().unwrap();
                program.push(JEQ);
                program.push(register);
                program.extend(&cmp.to_le_bytes());
                program.push(jmp);
                i += 4;
            }
            "CALL" => {
                let addr: usize = if tokens.len() > i - 1 {
                    let addr_str = tokens[i + 1];
                    if addr_str.starts_with("0x") || addr_str.starts_with("0X") {
                        usize::from_str_radix(
                            addr_str.trim_start_matches("0x").trim_start_matches("0X"),
                            16,
                        )
                        .unwrap()
                    } else {
                        addr_str.parse().unwrap()
                    }
                } else {
                    args.call_addr
                };
                program.push(CALL);
                program.extend(&(addr as u64).to_le_bytes());
                i += if tokens.len() > i + 1 { 2 } else { 1 };
            }
            "NOP" => {
                program.push(NOP);
            }
            _ => {
                eprintln!("Unknown instruction: {}", tokens[i]);
                return Ok(());
            }
        }
    }
    let encrypted: Vec<u8> = program.iter().map(|b| b ^ xkey).collect();

    let mut hasher = Hasher::new();
    hasher.update(&program);
    let crc = hasher.finalize();
    let crc_byte = crc.to_le_bytes();
    let mut f_payload = vec![TAG];
    f_payload.extend(&encrypted);
    f_payload.extend(&crc_byte);
    let mut file = File::create("/tmp/rfid_input.bin")?;
    file.write_all(&f_payload)?;

    Ok(())
}
