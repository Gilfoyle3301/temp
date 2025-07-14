use crate::pn532reader::constants::*;
use linux_embedded_hal::I2cdev;
use std::{thread, time::Duration};

pub struct PN532 {
    pub(crate) i2c: I2cdev,
    pub(crate) address: u8,
}

impl PN532 {
    pub fn new(device: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let i2c = I2cdev::new(device)?;
        Ok(PN532 {
            i2c,
            address: PN532_I2C_ADDRESS,
        })
    }
    pub fn get_firmware_version(&mut self) -> Result<u32, Box<dyn std::error::Error>> {
        self.write_command(&[PN532_COMMAND_GETFIRMWAREVERSION])?;
        if !self.read_ack()? {
            return Err("No ACK received".into());
        }
        let response = self.read_response(20)?;
        if response.len() >= 4 && response[0] == PN532_COMMAND_GETFIRMWAREVERSION + 1 {
            let version =
                ((response[1] as u32) << 16) | ((response[2] as u32) << 8) | (response[3] as u32);
            Ok(version)
        } else {
            println!("Response: {:02X?}", response);
            Err("Invalid firmware version response".into())
        }
    }
    pub fn sam_configuration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.write_command(&[PN532_COMMAND_SAMCONFIGURATION, 0x01, 0x14, 0x01])?;

        if !self.read_ack()? {
            return Err("No ACK received".into());
        }

        let response = self.read_response(10)?;
        if response.len() >= 1 && response[0] == PN532_COMMAND_SAMCONFIGURATION + 1 {
            Ok(())
        } else {
            Err("SAM configuration failed".into())
        }
    }
    pub fn read_passive_target(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.write_command(&[PN532_COMMAND_INLISTPASSIVETARGET, 0x01, 0x00])?;
        if !self.read_ack()? {
            return Err("No ACK received".into());
        }
        thread::sleep(Duration::from_millis(50));
        match self.read_response(30) {
            Ok(response) => {
                if response.len() < 3 || response[0] != PN532_COMMAND_INLISTPASSIVETARGET + 1 {
                    return Err("Invalid response".into());
                }

                let nb_targets = response[1]; // сколько карт найдено.
                if nb_targets == 0 {
                    return Err("No card found".into());
                }

                if response.len() < 7 {
                    return Err("Response too short".into());
                }

                // _tag_number — номер цели (например, 1)
                // sens_res — параметры карты (тип, скорость — для ISO14443A)
                // sel_res — selection response (вторичный ID)
                // uid_length — сколько байтов занимает UID

                let _tag_number = response[2];
                let _sens_res = ((response[3] as u16) << 8) | (response[4] as u16);
                let _sel_res = response[5];
                let uid_length = response[6] as usize;

                if response.len() < 7 + uid_length {
                    return Err("Response too short for UID".into());
                }

                Ok(response[7..7 + uid_length].to_vec())
            }
            Err(_) => Err("No card detected".into()),
        }
    }
    pub fn read_block(&mut self, block_number: u8) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let command = vec![PN532_COMMAND_INDATAEXCHANGE, 0x01, 0x30, block_number];

        self.write_command(&command)?;

        if !self.read_ack()? {
            return Err("No ACK received".into());
        }

        let response = self.read_response(30)?;

        if response.len() >= 18
            && response[0] == PN532_COMMAND_INDATAEXCHANGE + 1
            && response[1] == 0x00
        {
            Ok(response[2..18].to_vec())
        } else {
            Err("Failed to read block".into())
        }
    }

    // pub fn dump_available_data(
    //     &mut self,
    //     uid: &[u8],
    // ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    //     let keys = [[0xFF; 6], [0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5], [0x00; 6]];
    //     let mut result = Vec::new();

    //     for block in 0..64 {
    //         if block % 4 == 3 {
    //             continue; // skip trailer
    //         }

    //         let mut authed = false;
    //         for key in keys {
    //             if self.authenticate_block(block, uid, 0x60, &key)? {
    //                 authed = true;
    //                 break;
    //             }
    //             if self.authenticate_block(block, uid, 0x61, &key)? {
    //                 authed = true;
    //                 break;
    //             }
    //         }

    //         if !authed {
    //             result.extend_from_slice(&[b'?'; 16]); // или b'-'
    //             continue;
    //         }

    //         let block_data = self.read_block(block)?;
    //         result.extend_from_slice(&block_data);
    //     }

    //     Ok(result)
    // }

    /// Считывает все data-блоки (без трейлеров) и собирает строку из них
    pub fn read_full_data(
        &mut self,
        uid: &[u8],
        auth_key: u8,
        key: &[u8; 6],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut all_data = Vec::new();

        for sector in 0..16 {
            let block_start = sector * 4;

            // Аутентификация производится только один раз на сектор – по первому блоку (сюда входит весь сектор)
            let auth_result = self.authenticate_block(block_start, uid, auth_key, key)?;

            if auth_result {
                println!("Auth success on sector {}", sector);
                for i in 0..4 {
                    let block = block_start + i;
                    match self.read_block(block) {
                        Ok(data) => {
                            println!("  Block {:2}: {:?}", block, data);
                            all_data.extend_from_slice(&data);
                        }
                        Err(e) => {
                            eprintln!("  ❌ Failed to read block {}: {}", block, e);
                            all_data.extend_from_slice(&[0u8; 16]); // Записываем пустой блок, чтобы размер оставался
                        }
                    }
                }
            } else {
                println!("❌ Auth failed on sector {}", sector);
                all_data.extend_from_slice(&[0u8; 64]); // 4 блока по 16 байт
            }
        }

        Ok(all_data)
    }

    pub fn authenticate_block(
        &mut self,
        block_number: u8,
        uid: &[u8],
        key_type: u8,  // 0x60 = KEY_A, 0x61 = KEY_B
        key: &[u8; 6], // 6 байт ключа
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut command = vec![PN532_COMMAND_INDATAEXCHANGE, 0x01, key_type, block_number];
        command.extend_from_slice(key);
        command.extend_from_slice(uid);

        self.write_command(&command)?;

        if !self.read_ack()? {
            return Err("No ACK received after auth".into());
        }

        let response = self.read_response(10)?;
        if response.len() >= 2
            && response[0] == PN532_COMMAND_INDATAEXCHANGE + 1
            && response[1] == 0x00
        {
            println!("Auth success for block {}", block_number);
            Ok(true)
        } else {
            println!("Auth failed: {:?}", response);
            Ok(false)
        }
    }
}
