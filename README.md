# PNG Parser (WASM & Python)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![WASM](https://img.shields.io/badge/wasm-compiled-blueviolet.svg)](https://webassembly.org/)
[![Python](https://img.shields.io/badge/python-3.7+-blue.svg)](https://www.python.org/downloads/)

A high-performance PNG steganography and parsing engine written in **Rust**. Securely hide, read, and manage encrypted messages within PNG files using a unified core available for both **JavaScript (WebAssembly)** and **Python**.

## 📦 Available Packages

| Platform | Installation | Status |
| :--- | :--- | :--- |
| **JS / WASM** | `npm i @pranjalpanging/png-parser` | ✅ Published |
| **Python** | `pip install png-parser` | ✅ Published |

---

## 📦 Installation

To access the latest security features including **AES-256-GCM encryption** and **message expiry**, ensure you are using version `0.2.0` or later.

### Python
```bash
pip install png-parser>=0.2.0
```

## ✨ Features
- **Zero-Overwrite Steganography**: Uses custom `stEg` ancillary chunks that don't affect image pixels or quality.
- **Optional AES-256-GCM Encryption**: Secure your messages with industrial-grade encryption (PBKDF2 key derivation).
- **Automatic Detection**: Smart logic automatically detects if a message is plain text or encrypted during reading.
- **Time-Based Expiration (Self-Destruct)**: Set a TTL (Time-To-Live) in hours for your messages. The parser will automatically treat data as "expired" once the time limit is reached.
- **Blazing Fast**: Core logic implemented in Rust for maximum speed and memory safety.
- **Valid PNG Structure**: Files remain 100% compliant with PNG standards and open in any standard viewer.

---

## 🐍 Python Usage

### 1. Hiding a Message
You can hide a message as plain text or encrypt it by simply providing a password.

```python
import png_parser

# Option A: Simple (Plain Text)
# Best for metadata or simple labels.
print(png_parser.hide("input.png", "Hello World"))

# Option B: Secure (AES-256-GCM Encryption)
# Encrypts the message. Only readable with the correct password.
print(png_parser.hide("input.png", "Secret Data", password="my_password"))

# Option C: Timed (Auto-Expiry)
# Message remains in the image but will be ignored by the reader after X hours.
print(png_parser.hide("input.png", "Self-destructing text", expires_in_hours=2))

# Option D: Maximum Security (Encryption + Expiry)
# Encrypted and timed. This is the most secure way to share data.
print(png_parser.hide(
    "input.png", 
    "Top Secret Mission", 
    password="secure_pass_123", 
    expires_in_hours=24
))
```

### 2. Reading a Message
The parser automatically detects if a message is encrypted or expired.
```Python
import png_parser

# To read a simple or timed message:
print(data = png_parser.read("input.png"))

# To read an encrypted message (Required for Options B & D):
# If the password is wrong or the data has expired, it returns an error/None.
print(png_parser.read("input.png", password="secure_pass_123"))
```
### 3. Deleting the Secret
Remove the hidden chunks and restore the PNG to its original state.
```Python

import png_parser

sprint(png_parser.delete("my_image.png"))
print(status)
```
## 🛠 Technical Details
The tool manipulates the **PNG Chunk Layer**. Every PNG starts with an 8-byte signature, followed by chunks like `IHDR`, `IDAT`, and `IEND`.
### Security Protocol:
1. **Ancillary Chunk**: We insert a non-critical chunk (`stEg`). Per PNG spec, viewers skip chunks they don't recognize.
2. **Security Flag**: The first byte of the chunk payload is a flag:
- `0x00`: Plain-text UTF-8 data.
- `0x01`: AES-GCM Payload (16-byte Salt + 12-byte Nonce + Ciphertext).
3. **Key Derivation**: We use PBKDF2-HMAC-SHA256 with 100,000 iterations to derive keys from passwords, providing strong resistance against brute-force attacks.

## 🏗 Development
To build this project from source:
- Rust (Cargo)
- Maturin (for Python: pip install maturin)
- wasm-pack (for JS: npm install -g wasm-pack)

Build Python:
```Bash
maturin develop
```
Build WASM:
```Bash
wasm-pack build --target web
```

**Pranjal Panging**

[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/pranjalpanging)

## License

This project is licensed under the [MIT License](LICENSE)