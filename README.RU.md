# 🧠 VM Escape Through NFC

> Аппаратно-программное CTF-задание: виртуальная машина, NFC, Rust, reverse engineering, флаг.

---

## 📦 Описание проекта

Этот проект — стенд CTF с реальным оборудованием, написанный на чистом Rust и Go, где участник должен:

- 📲 через NFC отправить зашифрованный payload на устройство,
- 🔐 пройти проверки: `XOR`, `CRC32`, `tag`,
- 🧠 передать команду в виртуальную машину (VM),
- 🚪 выполнить цепочку инструкций (`MOV`, `ADD`, `JEQ`, `CALL`),
- ✅ вызвать скрытую функцию `send_flag()`,
- 📡 отправить HTTP POST-запрос на сервер, чтобы получить флаг.

---

## 📁 Структура репозитория

```text
vm-escape-nfc/
│
├── builder/               # 🔧 build_payload.rs – генератор кода VM → байты
│
├── griph/         # 🧠 Бинарь на стенде: VM, проверка, флаг
│   └── src/
│       └── vm.rs          # Реализация виртуальной машины
│
├── nfc_reader/            # 📡 Чтение NFC (Rust-only, через SPI)
│
├── backend/               # 🌐 Go-сервер, принимает флаг
│
├── system/                # ⚙ Сервисы systemd, автозапуск
│
├── payloads/              # 📂 Примеры payload-ов и расшифровка
│
└── docs/                  # 📚 Аппаратная схема, Reverse writeup и описание уровней
```

---

## 🛠 Требования

| Компонент       | Версия или аналог |
|------------------|-------------------|
| ✅ Rust          | 1.70+             |
| ✅ Cargo         | да                |
| ✅ Orange Pi 5   | либо Pi 4         |
| ✅ Модуль NFC    | PN532 по SPI      |
| ✅ NFC метки     | NTAG213/215       |
| ✅ Linux         | Ubuntu / Armbian  |
| Go (бэкенд)      | 1.23+             |

---

## 🔧 Установка

### Rust окружение:

```bash
rustup default stable
```

### Установка зависимостей:

```bash
sudo apt update
sudo apt install libudev-dev spidev libssl-dev
```

---

## 🖥 Подключение PN532 к Orange Pi 5

| PN532        | Orange Pi PIN |
|--------------|----------------|
| VCC          | 5V (PIN 2)     |
| GND          | GND (PIN 6)    |
| SCK          | PIN 23 (SPI0_CLK)   |
| MISO         | PIN 21 (SPI0_MISO)  |
| MOSI         | PIN 19 (SPI0_MOSI)  |
| SS (CS)      | PIN 24 (SPI0_CS0)   |

✅ SPI включается через `armbian-config`

---

## 🚀 Запуск стенда

```bash
# Собрать и запустить NFC ридер
cd nfc_reader
cargo build --release
./target/release/nfc_reader
```

```bash
# Собрать и установить VM
cd griph
cargo build --release
./target/release/griph
```

Стартовый payload можно сгенерировать:

```bash
cd builder
cargo run --release -- \
  build-sequence \
  --sequence "MOV 0 41 ADD 0 1 JEQ 0 42 3 CALL 4195636"
```

---

## 🧠 Виртуальная машина

| Инструкция | Описание                                |
|------------|-----------------------------------------|
| `MOV`      | Записать значение в регистр             |
| `ADD`      | Прибавить к регистру                    |
| `JEQ`      | Условный переход                        |
| `CALL`     | Вызов функции по адресу (unsafe)        |
| `NOP`      | Ничего не делает                        |

Пример:
```
MOV R0, 41
ADD R0, 1
JEQ R0, 42, 3
CALL 0x401234
```

---

## 📄 Payload формат

```text
[ 0x03 ][ зашифрованная VM-программа ][ CRC32 ]
                  ^ XOR с ключом 0x5A
```

- Минимум длины: ~10 байт
- Проверяется внутри `griph`
- Записывается через Android (NFC Tools → MIME)

---

## 🧊 Защита

- ✅ tag check (0x03)
- ✅ CRC32 хеш
- ✅ XOR
- ✅ VM с opcode-машиной
- ✅ `#[no_mangle]` на `send_flag()` — открыта только по адресу
- ✅ Unsafe `fn()` вызов: как ROP-lite

---

## 🔐 Получение флага

Если вызов `send_flag()` происходит корректно, он отправляет:

```json
POST /flag
{ "token": "CTF{super_ctf_mastermind}" }
```

Ответ:
```json
{ "flag": "CTF{you_did_it}" }
```

---

## 📱 Android-поддержка

- NFC Tools (Android/iOS)
- → Запись : [Custom MIME-type : `application/x-ctf`]
- → Payload = ваш собранный `rfid_input.bin`

---

## 📂 Примеры

```bash
cat payloads/example_1_desc.txt
> MOV R0, 41 + ADD 1 + JEQ 42 + CALL send_flag

xxd payloads/example_1.bin
> 03 b4 9a 1e 4f ... (XOR + CRC)
```

---

## 📋 Полезные команды

```bash
nfc-list                  # проверки PN532
cat /tmp/rfid_input.bin   # проверка payload
nm target/release/vm      # поиск адреса send_flag
```

---

## 📐 Roadmap

- [x] Чистый Rust NFC reader через SPI
- [x] Payload builder на Rust (без Python)
- [x] Unsafe CALL → флаг
- [x] Payload: tag + xor + crc
- [ ] HTML-декодер + write-пример
- [ ] Web-интерфейс генерации payload
- [ ] Обратный дизассемблер на Rust
- [ ] Мультикартная VM (UID)

---

## 🧑‍💻 Авторы

**hRAZ**
Reverse инженер & Rust-разработчик уровня CTF-архитектора
Контакты: [скрыты для репозитория] 😉

**p1zza**

---

## 💡 Благодарности
- Вселенная, Rust и Coffee
