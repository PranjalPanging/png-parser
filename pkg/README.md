# @pranjal/png-parser

A high-performance PNG steganography library powered by **Rust** and **WebAssembly**. Hide, read, and delete secret messages within PNG images directly in the browser or Node.js.

## Features
- **WASM Powered**: Near-native performance for byte manipulation.
- **Zero Dependencies**: Lightweight and secure.
- **Browser & Node Support**: Works anywhere JavaScript runs.

## Installation

```bash
npm install @pranjalpanging/png-parser
```
## Usage
### 1. Initialize and Hide a Message
To hide a message, pass the file bytes (as a `Uint8Array`) and your secret string.
JavaScript
```Javascript
import init, { hide_js } from "@pranjalpanging/png-parser";

async function run() {
    // Initialize the WASM module (required for web target)
    await init();
    
    // Get your file as a Uint8Array (example using file input)
    const file = document.getElementById('myFileInput').files[0];
    const fileBytes = new Uint8Array(await file.arrayBuffer());
    const secretMessage = "The eagle lands at midnight";

    try {
        const modifiedPng = hide_js(fileBytes, secretMessage);
        // modifiedPng is a new Uint8Array containing the hidden 'stEg' chunk
    } catch (e) {
        console.error("Failed to hide message:", e);
    }
}
```
### 2. Reading a Secret Message
The library scans the PNG structure for the specific stEg chunk and extracts the payload.
```Javascript
import { read_js } from "@pranjalpanging/png-parser";

try {
    const secret = read_js(fileBytes);
    console.log("Secret found:", secret);
} catch (e) {
    console.log("No hidden message discovered in this image.");
}
```
### 3. Deleting the Secret
Removes any stEg chunks entirely, restoring the PNG to its original state.

```JavaScript

import { delete_js } from "@pranjalpanging/png-parser";

try {
    const cleanedPng = delete_js(fileBytes);
    // Use cleanedPng as a normal PNG without the hidden metadata
} catch (e) {
    console.error("Error during deletion:", e);
}
```
## How it Works
This tool manipulates the PNG Chunk Structure. Every PNG file consists of a signature followed by a series of chunks.

By inserting data into a custom Ancillary Chunk, the hidden information exists outside the "critical" image data (IHDR, IDAT). Because the chunk type starts with a lowercase letter (s), image viewers are instructed by the PNG specification to ignore it if they don't recognize it.

## Technical Specifications
- **Core Engine**: Rust (PNG-Me implementation)
- **WASM Binding**: wasm-bindgen
- **Target**: wasm32-unknown-unknown

## Author

**Pranjal Panging**

[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/pranjalpanging)
[![npm](https://img.shields.io/badge/npm-CB3837?style=for-the-badge&logo=npm&logoColor=white)](https://www.npmjs.com/~pranjalpanging)

## License

This project is licensed under the [MIT License](LICENSE)