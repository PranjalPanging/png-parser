# 🖼️ PNG-Parser

A powerful, command-line tool built in **Rust** for deep-diving into PNG file structures. This project allows you to inspect the binary "chunks" of an image, strip away metadata traces, and hide secret messages using steganography.

---

## 🚀 Features

* **🔍 Inspect**: List all chunks within a PNG (IHDR, IDAT, IEND, etc.) including their size and order.
* **🤫 Hide**: Inject a secret message into a custom `stEg` chunk that remains invisible to standard image viewers.
* **🔓 Read**: Extract and decode hidden messages from specific chunk types.
* **🧹 Strip**: Remove non-essential metadata chunks (tEXt, zTXt, iTXt) to clean the file and reduce its digital footprint.

---

## 💻 Usage

First, build the project using Cargo:
```powershell
cargo build --release
```
## 1. Hide a Secret Message
Hides your text inside a new PNG file. The original image remains unchanged.
```poweshell
cargo run -- hide --input test.png --message "The ghost is in the machine" --output secret.png
```
## 2. Read the Secret Message
Retrieves the message you hid by searching for the specific chunk type.
```poweshell
cargo run -- read --file secret.png --chunk-type stEg
```
## 3. Inspect Image Chunks
View the "DNA" of your PNG file:
```poweshell
cargo run -- strip --input secret.png --output clean.png
```

## 📂 Project Structure
- **src/png.rs** : Logic for the PNG signature and managing the collection of chunks.
- **src/chunk.rs**: Handles individual data blocks, including CRC (Cyclic Redundancy Check) calculation.
- **src/chunk_type.rs**: Validates chunk names according to the PNG specification.
- **src/commands/**: Contains the high-level implementation for the CLI actions.

## 🛡️ Requirements
- Rust (latest stable version)
- Cargo (included with Rust)

## License
This project is licensed under the MIT License - see the LICENSE file for details.
