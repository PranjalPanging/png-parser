# PNG Parser (WASM & Python)

[![npm version](https://img.shields.io/npm/v/@pranjalpanging/png-parser.svg)](https://www.npmjs.com/package/@pranjalpanging/png-parser)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance PNG steganography and parsing tool written in **Rust**. This project provides a unified core that is available for both **JavaScript (WebAssembly)** and **Python**.

## 📦 Available Packages

| Platform | Installation | Documentation |
| :--- | :--- | :--- |
| **JavaScript / WASM** | `npm install @pranjalpanging/png-parser` | [JS Guide](#-javascript-usage-webnode) |
| **Python** | `pip install png-parser` | [Python Guide](#-python-usage) |

---

## ✨ Features
- **Blazing Fast**: Core logic implemented in Rust for maximum speed.
- **Stealthy**: Uses custom PNG chunks (`stEg`) that are ignored by standard image viewers.
- **Cross-Platform**: Run the same logic in a web browser, Node.js, or a Python script.
- **Safe**: Includes functions to strip hidden data and restore original file integrity.

---

## 🚀 JavaScript Usage (Web/Node)
This package is compiled to WebAssembly. When using it in the browser, you must initialize the WASM module first.

```javascript
import init, { hide_message, read_message } from "@pranjalpanging/png-parser";

async function run() {
    // Initialize the WASM engine
    await init();

    // Example: Reading a message from a Uint8Array (file buffer)
    const secret = read_message(imageBuffer);
    console.log("Hidden message:", secret);
}

run();
```
Note: For more detailed JS examples, check the pkg/README.md.

## 🐍 Python Usage
Installation
Once the package is uploaded to PyPI, install it via pip:

```Bash
pip install png-parser
```
### 1. Hiding a Message
Embed a secret string into any PNG file. This adds the data safely without corrupting the image.
```Python
import png_parser

# Appends a hidden 'stEg' chunk to the image
status = png_parser.hide("my_image.png", "hello world")
print(status) # Output: Success: Message hidden!
```
### 2. Reading a Message
Extract the hidden data from the image.
```Python

import png_parser

secret = png_parser.read("my_image.png")
print(f"Secret message: {secret}") # Output: Secret message: hello world
```
### 3. Deleting the Secret
Remove the hidden chunks and restore the PNG to its original state.
```Python

import png_parser

status = png_parser.delete("my_image.png")
print(status) # Output: Success: Secret message deleted!
```
## 🛠 Technical Details
This tool manipulates the PNG Chunk Structure. Every PNG consists of a series of chunks. This library inserts an Ancillary Chunk (optional data) named stEg. Standard image viewers skip chunks they don't recognize. By placing our data before the IEND marker, the file remains a valid image while carrying your hidden payload.

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