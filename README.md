
![Rust](https://img.shields.io/badge/Rust-stable-orange?style=for-the-badge&logo=rust)
![CTF](https://img.shields.io/badge/CTF-hardware-blueviolet?style=for-the-badge)
![NFC](https://img.shields.io/badge/NFC-ready-blue?style=for-the-badge)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)

# 🧠 VM Escape Through NFC
📖 [Русская версия доступна здесь](./README.RU.md)

> A hardware-based CTF challenge using NFC, a Rust-powered virtual machine, and deep binary logic. Build payloads, reverse the memory layout, decode encryption & cause unsafe behavior to leak the flag.

---

## 📦 Project Description

This repository contains the implementation of a CTF (Capture The Flag) challenge — built for real-world use on embedded systems. The player must:

- 🟢 Write a **custom instruction program** (payload),
- 🔐 Pass built-in protections: tag, XOR encryption, CRC32 checksum,
- 🎯 Execute a **virtual machine** with register-based logic (`MOV`, `ADD`, `JEQ`, `CALL`, etc),
- 🚨 Trigger a hidden function `send_flag` by providing its raw memory address,
- 📡 Send a POST request to the backend and receive the flag.

Fully written in Rust, safe by default — except when you want it dangerous 😈

---

## 📁 Repository Structure

```text
vm-escape-nfc/
├── builder/          # 🧪 Rust-based Payload generator CLI
├── target_device/    # 🧠 NFC challenge runtime with VM
├── nfc_reader/       # 📡 SPI-based NFC reader (no C)
├── backend/          # 🌐 Optional: Go server to process flag submissions
├── system/           # ⚙️ Systemd services, auto-start
├── payloads/         # 📂 Sample payloads and solutions
└── docs/             # 📚 Docs, diagrams, writeups
```

---

## 🧰 Requirements

| Component         | Version / Platform            |
|------------------|--------------------------------|
| Rust             | 1.70+                          |
| Go (backend)     | Optional, version 1.20+        |
| NFC Module       | **PN532**, SPI interface       |
| NFC Tags         | **NTAG213/215/216**, NDEF-ready|
| Orange Pi / Pi 4 | Recommended: Orange Pi 5       |
| OS               | Ubuntu/Debian/Armbian (64-bit) |

---

## 💡 Hardware Overview

- 📟 **Device**: Raspberry Pi 4 or **Orange Pi 5**
- 📡 **NFC chip**: PN532 via SPI
- 💾 NFC cards/tags: NTAG215 (Android compatible)
- 📤 Output: triggers `send_flag()` via raw pointer execution

**Wiring to Orange Pi 5 (SPI):**

| PN532      | Orange Pi PIN |
|------------|----------------|
| VCC        | 5V (PIN 2)     |
| GND        | GND (PIN 6)    |
| MISO       | PIN 21         |
| MOSI       | PIN 19         |
| SCK        | PIN 23         |
| SS         | PIN 24         |

Enable SPI via `armbian-config`.

---

## 🚀 Quick Start

### NFC Reader

```bash
cd nfc_reader
cargo build --release
./target/release/nfc_reader
```

✅ Waits for NFC card and writes UID or payload to `/tmp/rfid_input.bin`

---

### Virtual Machine Runtime

```bash
cd target_device
cargo build --release
./target/release/target_device
```

It parses the input file, decrypts it, validates CRC, decodes the instruction stream, and executes it.

---

### Generate Payload

```bash
cd builder
cargo run --release -- \
  build-sequence \
  --sequence "MOV 0 41 ADD 0 1 JEQ 0 42 3 CALL 4195636"
```

It creates `rfid_input.bin` with tag, XOR-encrypted instruction data and valid CRC32.

---

## 🧠 Virtual Machine Instructions

| Opcode   | Description                           |
|----------|----------------------------------------|
| `MOV`    | Write value to register (R0–R3)        |
| `ADD`    | Add value to register                  |
| `JEQ`    | If equal → jump to instruction idx     |
| `CALL`   | Call a function by memory address 🧨   |
| `NOP`    | Do nothing                             |

---

## 🔐 Protection layers

- ✅ `tag` validation (first byte == 0x03)
- 🔐 Payload is encrypted with XOR (0x5A)
- ✅ CRC32 hash check
- ✅ Instruction deserialization check
- ⚠️ Call to hidden `#[no_mangle] fn send_flag()`

> You must RE the binary to extract the address of the function
> and pass it into `CALL` inside the VM program.

---

## 📄 Payload Format

```text
[ 0x03 ][ encrypted bytecode ][ CRC32 checksum ]
         ^ ~XOR key 0x5A
```

Encrypted content is built from instruction sequence.
Final 4 bytes = CRC32 of raw (decrypted) bytes.

---

## 🌐 Flag Delivery Mechanism

When executed correctly, the payload will call `send_flag()`:

```rust
let client = reqwest::blocking::Client::new();
let _ = client.post("http://localhost:8080/api/flag")
    .json(&serde_json::json!({
        "token": "CTF{super_ctf_mastermind}"
    }))
    .send();
```

---

## 📱 Android Support

- Use [**NFC Tools**](https://play.google.com/store/apps/details?id=com.wakdev.wdnfc) app
- Use "Write" → "Custom MIME type"
  - Type: `application/x-ctf`
  - Payload: byte contents of `rfid_input.bin`

✅ You can use real NFC tags or simulate via phone.

---

## 📂 Examples

Generated payload (after builder):

```
03 2A 7B 99 ... <crc32>
```

Sample instructions encoded:

```asm
MOV R0, 41
ADD R0, 1
JEQ R0, 42, 3
CALL 0x401234
```

---

## 🧪 Tools for Debugging

```bash
nfc-list                 # Scan NFC devices
cat /tmp/rfid_input.bin  # Confirm written payload
nm target/release/vm     # Lookup function address
```

---

## 🧭 Development Status

- [x] NFC Reader (Rust-only, SPI)
- [x] Payload generator CLI (Rust)
- [x] Custom bytecode VM
- [x] Memory CALL pointer (unsafe)
- [x] XOR + CRC32 verification
- [ ] Web-based builder (coming)
- [ ] CTFd integration demo
- [ ] Instruction decompiler

---

## 👨‍💻 Author

> built by **hRAZ**
> reverse engineering enjoyer | hardware hacker | rust evangelist

> P1ZZA
> hardware built

## 🙏 Acknowledgements

- [pn532 crate](https://crates.io/crates/pn532)
- [linux-embedded-hal](https://crates.io/crates/linux-embedded-hal)
- NFC Tools app
- The Rust ecosystem 🙏
