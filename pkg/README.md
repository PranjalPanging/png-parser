# PNG Parser (WASM)
A high-performance PNG steganography engine written in Rust and compiled to WebAssembly. Securely hide, read, and delete encrypted messages within PNG files using custom ancillary chunks directly in the browser or Node.js.

## 📦 Installation

```Bash
npm i @pranjalpanging/png-parser
```

## ✨ Features
- **Browser Native**: Works with Uint8Array, File, and Blob APIs.
- **AES-256-GCM Encryption**: Secure your messages with industrial-grade encryption.
- **Non-Destructive**: Messages are stored in a custom stEg chunk. The image pixels remain untouched, and the file stays a valid PNG.
- **Blazing Fast**: Near-native execution speed powered by the Rust core.

## 🚀 Usage Example (Browser)
Since WASM handles raw bytes, you need to convert your files to Uint8Array before processing.

1. Initialize & Hide a Message
```JavaScript
import init, { hide_js } from "@pranjalpanging/png-parser";

async function encryptImage() {
    await init(); // Initialize the WASM module

    const fileInput = document.getElementById('myFile');
    const file = fileInput.files[0];
    
    // Convert File to Uint8Array
    const buffer = await file.arrayBuffer();
    const imageBytes = new Uint8Array(buffer);

    try {
        const message = "My Secret Message";
        const password = "secure-password-123";

        // Returns a new Uint8Array with the hidden data
        const encryptedBytes = hide_js(imageBytes, message, password);

        // Convert back to a Blob to download/display
        const blob = new Blob([encryptedBytes], { type: "image/png" });
        const url = URL.createObjectURL(blob);
        window.open(url); 
    } catch (e) {
        console.error("WASM Error:", e);
    }
}
```
2. Read a Hidden Message
```JavaScript
import { read_js } from "@pranjalpanging/png-parser";

async function decryptImage(bytes) {
    try {
        // Automatically detects if encrypted or plain text
        const secret = read_js(bytes, "secure-password-123");
        console.log("Decoded Message:", secret);
    } catch (e) {
        console.error("Failed to read message:", e);
    }
}
```
3. Delete Hidden Data
```JavaScript

import { delete_js } from "@pranjalpanging/png-parser";

const cleanedBytes = delete_js(imageBytes);
// Returns PNG bytes with the 'stEg' chunk completely removed
```
## 🛠 API Reference

| Function | Parameters | Returns | Description |
| :--- | :--- | :--- | :--- |
| `hide_js` | `(bytes: Uint8Array, msg: string, pass: string \| null)` | `Uint8Array` | Appends a hidden message chunk to the PNG. |
| `read_js` | `(bytes: Uint8Array, pass: string \| null)` | `string` | Extracts and decodes the hidden message. |
| `delete_js` | `(bytes: Uint8Array)` | `Uint8Array` | Removes the `stEg` chunk from the PNG. |

---

## 🏗 Development

If you want to build the package yourself, follow these steps:

### 1. Install Rust & wasm-pack
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh

# Install wasm-pack
npm install -g wasm-pack
```

## 🔒 Security Protocol
- **Chunk Type**: Ancillary chunk stEg (safe to copy, safe to ignore by viewers).
- **KDF**: PBKDF2-HMAC-SHA256 with 100,000 iterations.
- **Encryption**: AES-256-GCM (12-byte nonce, 16-byte auth tag).
- **Entropy**: Secure random values generated via Browser Web Crypto API.

--
Author: **Pranjal Panging**

[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/pranjalpanging)

## License

This project is licensed under the [MIT License](LICENSE)