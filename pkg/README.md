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

## ✨ Features
- **Invisible Storage**: Messages are stored in a custom stEg ancillary chunk. Image pixels remain untouched, and the file remains a valid PNG.
- **AES-256-GCM Encryption**: Secure your messages with industrial-grade encryption using PBKDF2 key derivation (100,000 iterations).
- **Self-Destruction**: Optional expiry timer. Once the time limit is reached, the message becomes unreadable even with the correct password.
- **Browser Native**: Blazing fast execution powered by Rust, working directly with `Uint8Array`, `File`, and `Blob` APIs.

---

## 🚀Implementation Guide

### 1. Hide: Injecting Secret Data
The `hide_js` function takes your original PNG bytes and appends a custom stEg chunk containing your metadata.

Code Implementation:
```javascript
import init, { hide_js } from "png-parser";

// 1. Initialize once at the top level
const wasmPromise = init();

async function handleHide() {
    // Ensure WASM is ready
    await wasmPromise; 
    
    const fileInput = document.getElementById('input');
    if (!fileInput.files.length) return alert("Please select a file");

    const file = fileInput.files[0];
    const bytes = new Uint8Array(await file.arrayBuffer());

    const message = "Top Secret Data";
    
    // FIX: Ensure empty inputs are passed as null
    const password = "user-password-123" || null; 
    const expiryHours = 48 || null;

    try {
        // 3. Execute the WASM function
        const resultBytes = hide_js(bytes, message, password, expiryHours);

        // 4. Create and trigger download (Native Browser Way)
        const blob = new Blob([resultBytes], { type: "image/png" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = "secret_image.png";
        a.click();
        URL.revokeObjectURL(url); // Clean up memory
        
    } catch (err) {
        console.error("Encryption failed:", err);
    }
}

```

### 2. Read: Extracting and Decrypting
The `read_js` function scans the PNG for the `stEg` chunk. It automatically detects if the data is encrypted or expired.

Code Implementation:
```javascript
import init, { read_js } from "png-parser";

// Initialize WASM globally so it's ready for any function
const wasmReady = init();

async function handleRead() {
    // 1. Wait for the engine to be loaded
    await wasmReady; 

    const fileInput = document.getElementById('input');
    if (!fileInput.files.length) return alert("Please select a PNG file.");

    const file = fileInput.files[0];
    const bytes = new Uint8Array(await file.arrayBuffer());
    
    // 2. Clean the password input (remove spaces, handle empty as null)
    const passInput = document.getElementById('pass').value.trim();
    const password = passInput === "" ? null : passInput;

    try {
        // 3. Attempt to read the 'stEg' chunk
        const decodedMessage = read_js(bytes, password);
        
        console.log("Secret Message:", decodedMessage);
        alert("Success! Message: " + decodedMessage);
    } catch (err) {
        // 4. Handle specific Rust-thrown errors
        // Common: "Incorrect password or corrupted data" 
        // OR "This message has expired and self-destructed"
        alert("Read Failed: " + err);
    }
}
```
### 3. Delete: Sanitizing the Image
The `delete_js` function is used for privacy. It locates the `stEg` chunk and removes it entirely, returning a "clean" PNG.

Code Implementation:
```javascript

import init, { delete_js } from "png-parser";

// Initialize the WASM engine once per page load
const wasmReady = init();

async function handleCleanup() {
    // 1. Ensure the engine is loaded before calling delete_js
    await wasmReady;

    const fileInput = document.getElementById('input');
    if (!fileInput.files.length) return alert("Please select a file.");

    const file = fileInput.files[0];
    const bytes = new Uint8Array(await file.arrayBuffer());

    try {
        // 2. Remove the 'stEg' chunk
        const cleanBytes = delete_js(bytes);
        
        // 3. Trigger download using standard browser API
        const blob = new Blob([cleanBytes], { type: "image/png" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `cleaned_${file.name}`;
        a.click();
        
        // Clean up the URL object to free memory
        URL.revokeObjectURL(url);

        alert("Privacy Scan Complete: Hidden data has been stripped.");
    } catch (err) {
        // 4. Handle errors (e.g., "No hidden message found in this image")
        alert("Cleanup failed: " + err);
    }
}
```
## 🔒 Security Protocol
- **Chunk Type**: Uses the `stEg` ancillary chunk (Ancillary, Private, Reserved, Safe-to-copy).
- **KDF**: `PBKDF2-HMAC-SHA256` with 100,000 iterations to protect against brute-force attacks.
- **Encryption**: `AES-256-GCM` (12-byte nonce, 16-byte auth tag).
- **Structure**:
1. **Flag Byte**: `0x01` (Encrypted) or `0x00` (Plain).
2. **Payload**: 8-byte BigEndian Timestamp (expiry) + the message bytes.

## 🏗 Development
To build the package yourself:
Build Python:

1. Install Rust & wasm-pack:
```Bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install wasm-pack
```
Build WASM:
```Bash
wasm-pack build --target web
```

**Author**:**Pranjal Panging**

[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/pranjalpanging)

## License

This project is licensed under the [MIT License](LICENSE)