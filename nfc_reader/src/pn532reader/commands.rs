use super::constants::*;
use crate::pn532reader::device::PN532;
use embedded_hal::i2c::I2c;
use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::Duration;

impl PN532 {
    pub(crate) fn wait_ready(
        &mut self,
        timeout_ms: u32,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();

        loop {
            let mut buffer = [0u8; 1];
            match self.i2c.read(self.address, &mut buffer) {
                Ok(_) => {
                    // "Bit 0 of the status byte indicates if the PN532 is ready to be read (1: ready, 0: not ready)."
                    if buffer[0] & 0x01 == 0x01 {
                        return Ok(true);
                    }
                }
                Err(_) => {}
            }

            if start.elapsed().as_millis() > timeout_ms as u128 {
                return Ok(false);
            }
            thread::sleep(Duration::from_millis(5));
        }
    }

    pub(crate) fn write_command(
        &mut self,
        command: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // LEN — длина данных (включая TFI и command) LEN = длина(TFI + DATA)
        // LCS — контрольная сумма длины (Length Checksum)
        // TFI считается частью полезной нагрузки (Data Packet)
        let len = command.len() as u8 + 1;
        // LEN + LCS = 0 (по модулю 256)
        // !0x05 = 0xFA  // побитовая инверсия: 1111_1010
        //  +1     = 0xFB
        //  & 0xFF = 0xFB
        let lcs = (!len + 1) & 0xFF;
        let mut frame = vec![
            PN532_PREAMBLE,
            PN532_STARTCODE1,
            PN532_STARTCODE2,
            len,
            lcs,
            PN532_HOSTTOPN532,
        ];
        frame.extend_from_slice(command);
        let mut checksum = PN532_HOSTTOPN532;
        // вычисляем контрольную сумму данных (DCS)
        // PRE | START1 | START2 | LEN | LCS |  TFI |  DATA...   | DCS | POST
        for &byte in command {
            checksum = checksum.wrapping_add(byte);
        }
        checksum = (!checksum + 1) & 0xFF;

        frame.push(checksum);
        frame.push(PN532_POSTAMBLE);
        println!("TX: {:02X?}", frame);
        match self.i2c.write(self.address, &frame) {
            Ok(_) => {
                thread::sleep(Duration::from_millis(10));
                Ok(())
            }
            Err(e) => {
                println!("Write error: {:?}", e);
                Err(Box::new(e))
            }
        }
    }

    pub(crate) fn read_ack(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        if !self.wait_ready(50)? {
            return Err("Timeout waiting for ACK".into());
        }
        let mut ack_buf = [0u8; 7];
        match self.i2c.read(self.address, &mut ack_buf) {
            Ok(_) => {
                println!("ACK raw: {:02X?}", ack_buf);
                // Пропускаем статусный байт
                let ack = &ack_buf[1..];
                // Проверяем ACK: 00 00 FF 00 FF 00
                if ack[0] == 0x00
                    && ack[1] == 0x00
                    && ack[2] == 0xFF
                    && ack[3] == 0x00
                    && ack[4] == 0xFF
                    && ack[5] == 0x00
                {
                    println!("PN53x ACKed");
                    Ok(true)
                } else {
                    println!("Invalid ACK");
                    Ok(false)
                }
            }
            Err(e) => {
                println!("ACK read error: {:?}", e);
                Err(Box::new(e))
            }
        }
    }
    pub fn write_to_file(&self, filename: &str, data: &[u8]) -> std::io::Result<()> {
        // let text_data = data
        //     .iter()
        //     .map(|&byte| byte.to_string())
        //     .collect::<String>();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            // .append(true)
            // .truncate(true)
            .open(filename)?;

        // writeln!(file, "{}", text_data)?;
        file.write_all(data)?;
        Ok(())
    }
}
