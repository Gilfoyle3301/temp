pub const PN532_I2C_ADDRESS: u8 = 0x24;

pub const PN532_PREAMBLE: u8 = 0x00;
pub const PN532_STARTCODE1: u8 = 0x00;
pub const PN532_STARTCODE2: u8 = 0xFF;
pub const PN532_POSTAMBLE: u8 = 0x00;

pub const PN532_HOSTTOPN532: u8 = 0xD4;
pub const PN532_PN532TOHOST: u8 = 0xD5;

// Команды PN532
pub const PN532_COMMAND_GETFIRMWAREVERSION: u8 = 0x02;
pub const PN532_COMMAND_SAMCONFIGURATION: u8 = 0x14;
pub const PN532_COMMAND_INLISTPASSIVETARGET: u8 = 0x4A;
pub const PN532_COMMAND_INDATAEXCHANGE: u8 = 0x40;

/*
| PRE | SC1 | SC2 | LEN | LCS | TFI | DATA... | DCS | POST |
Поле	Размер	Пример	Назначение
 PRE	    1 байт	0x00	Преамбула – начало фрейма
 SC1	    1 байт	0x00	Start Code 1 (всегда 0x00)
 SC2	    1 байт	0xFF	Start Code 2 (всегда 0xFF)
 LEN	    1 байт	0x03	Длина поля TFI + DATA
 LCS	    1 байт	0xFD	Компенсация длины: LEN + LCS == 0x00
 TFI	    1 байт	0xD4	Frame Identifier: 0xD4 от Host
 DATA	N байт	0x02	Команда + аргументы
 DCS	    1 байт		    Контрольная сумма (TFI + DATA)
 POST	1 байт	0x00	Постамбула
*/
