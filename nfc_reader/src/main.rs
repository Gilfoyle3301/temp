extern crate nfc_reader;
use clap::Parser;
use hex;
use nfc_reader::pn532reader::device::PN532;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
struct Args {
    /// I2C device path
    #[arg(short, long, default_value = "/dev/i2c-0")]
    device: String,
    /// I2C address of PN532 (default 0x24)
    #[arg(short, long, default_value = "0x24")]
    address: String,
    /// Key auth of card (default 0x24)
    #[arg(short, long, default_value = "0x60")]
    key: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("PN532 NFC Reader\n");
    let args = Args::parse();
    let auth_key = u8::from_str_radix(&args.key.trim_start_matches("0x"), 16)?;
    let mut pn532 = PN532::new(&args.device)?;
    println!("Getting firmware version...");
    match pn532.get_firmware_version() {
        Ok(version) => {
            println!("Firmware version: 0x{:06X}", version);
            let ic = (version >> 16) & 0xFF;
            let ver = (version >> 8) & 0xFF;
            let rev = version & 0xFF;
            println!("IC: PN5{:02X}, Version: {}.{}", ic, ver, rev);
        }
        Err(e) => {
            println!("Failed to get firmware version: {}", e);
            return Err(e);
        }
    }

    println!("\nConfiguring SAM...");
    pn532.sam_configuration()?;
    println!("SAM configured");
    println!("\nReady to read cards. Place a card near the reader...\n");

    let mut last_uid = Vec::new();

    loop {
        match pn532.read_passive_target() {
            Ok(uid) => {
                if uid != last_uid {
                    println!("Card detected! UID: ");
                    for byte in &uid {
                        print!("{:02X} ", byte);
                    }
                    if uid.len() == 4 {
                        let key = [0xFF; 6];
                        if pn532.authenticate_block(4, &uid, auth_key, &key)? {
                            match pn532.read_full_data(&uid, auth_key, &key) {
                                Ok(data) => {
                                    println!("Read block size: {}", data.len());
                                    println!("Read Data: {:?}", data);
                                    let bin = hex::decode(data).unwrap();
                                    pn532.write_to_file("/tmp/rfid_input.bin", &bin)?;
                                }
                                Err(e) => {
                                    println!("Cannot read block 4: {}", e);
                                }
                            }
                        }
                    } else {
                        println!("The length of the yuid is different: {}", uid.len())
                    }
                    last_uid = uid;
                    println!();
                }
            }
            Err(_) => {
                if !last_uid.is_empty() {
                    last_uid.clear();
                }
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
}
