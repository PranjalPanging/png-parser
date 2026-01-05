# PNG Parser (WASM & Python)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![WASM](https://img.shields.io/badge/wasm-compiled-blueviolet.svg)](https://webassembly.org/)
[![Python](https://img.shields.io/badge/python-3.7+-blue.svg)](https://www.python.org/downloads/)

A high-performance PNG steganography and parsing engine written in **Rust**. Securely hide, read, and manage encrypted messages within PNG files using a unified core available for both **JavaScript (WebAssembly)** and **Python**.

## 📦 Available Packages

| Platform | Installation | Status |
| :--- | :--- | :--- |
| **JS / WASM** | `npm i png_parser` | ✅ Published |
| **Python** | `pip install png-parser` | ✅ Published |

---

## ✨ Features
- **Zero-Overwrite Steganography**: Uses custom `stEg` ancillary chunks that don't affect image pixels or quality.
- **Optional AES-256-GCM Encryption**: Secure your messages with industrial-grade encryption (PBKDF2 key derivation).
- **Automatic Detection**: Smart logic automatically detects if a message is plain text or encrypted during reading.
- **Blazing Fast**: Core logic implemented in Rust for maximum speed and memory safety.
- **Valid PNG Structure**: Files remain 100% compliant with PNG standards and open in any standard viewer.

---

## 🐍 Python Usage

### 1. Hiding a Message
You can hide a message as plain text or encrypt it by simply providing a password.

```python
import png_parser

# Option A: Simple hiding (Plain Text)
png_parser.hide("input.png", "Hello World")

# Option B: Secure hiding (AES-256-GCM Encryption)
png_parser.hide("input.png", "Top Secret Data", password="my_secure_password")

```

### 2. Reading a Message
Extract the hidden data from the image.The parser detects the encryption flag. If you try to read an encrypted message without a password, it will return an error.
```Python
import png_parser

# Decrypting an encrypted message
secret = png_parser.read("input.png", password="my_secure_password")
print(f"Decoded: {secret}")

# Reading a plain text message
plain = png_parser.read("input.png")
print(f"Decoded: {plain}")
```
### 3. Deleting the Secret
Remove the hidden chunks and restore the PNG to its original state.
```Python

import png_parser

status = png_parser.delete("my_image.png")
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