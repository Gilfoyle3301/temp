use crate::pn532reader::constants::*;
use crate::pn532reader::device::PN532;
use embedded_hal::i2c::I2c;

impl PN532 {
    pub(crate) fn read_response(
        &mut self,
        max_length: usize,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if !self.wait_ready(1000)? {
            return Err("Timeout waiting for response".into());
        }
        // Читаем весь ответ целиком
        // +10 - резерв памяти
        let mut buffer = vec![0u8; max_length + 10];
        self.i2c.read(self.address, &mut buffer)?;
        // 20.min(buffer.len()) - страхующий вызов от ситуаций, когда пакет пришёл частично или пустой.
        // Если buffer.len() = 40, то срез будет buffer[0..20]
        // Если buffer.len() = 8, то срез будет buffer[0..8] - безопасно
        println!("RX raw data: {:02X?}", &buffer[..20.min(buffer.len())]);
        // Пропускаем статусный байт
        let buffer = &buffer[1..];
        // Проверяем заголовок
        if buffer[0] != 0x00 || buffer[1] != 0x00 || buffer[2] != 0xFF {
            return Err("Invalid response header".into());
        }
        let len = buffer[3] as usize;
        let lcs = buffer[4];
        // Проверяем LCS
        if ((len as u8).wrapping_add(lcs)) != 0x00 {
            return Err("LCS checksum error".into());
        }
        // Проверяем, что у нас достаточно данных
        // len = TFI + DATA
        // PRE+START+LEN+LCS = 5 байтов
        // DCS/POST = 2 байта
        if buffer.len() < 5 + len + 2 {
            return Err("Response too short".into());
        }
        // Проверяем TFI
        if buffer[5] != PN532_PN532TOHOST {
            return Err("Invalid TFI".into());
        }
        // Извлекаем данные (от позиции 6 до 6 + len - 1)
        // len включает TFI, поэтому данные это с позиции 6 до 5 + len
        let data_end = 5 + len;
        let data = buffer[6..data_end].to_vec();
        // Проверяем контрольную сумму
        let mut checksum = 0u8;
        for i in 5..data_end {
            checksum = checksum.wrapping_add(buffer[i]);
        }
        // (TFI + DATA + DCS) % 256 == 0
        checksum = (!checksum).wrapping_add(1);
        if buffer[data_end] != checksum {
            println!(
                "Checksum error: expected {:02X}, got {:02X}",
                checksum, buffer[data_end]
            );
        }

        println!("Response data: {:02X?}", data);

        Ok(data)
    }
}
