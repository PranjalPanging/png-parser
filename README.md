# png-parser

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![WASM](https://img.shields.io/badge/wasm-compiled-blueviolet.svg)](https://webassembly.org/)
[![Python](https://img.shields.io/badge/python-3.8+-blue.svg)](https://www.python.org/downloads/)
[![npm](https://img.shields.io/badge/npm-%40pranjalpanging%2Fpng--parser-red.svg)](https://www.npmjs.com/package/@pranjalpanging/png-parser)
[![PyPI](https://img.shields.io/pypi/v/png-parser.svg)](https://pypi.org/project/png-parser/)

> Hide any file inside an image — compress, encrypt, embed.

A high-performance steganography engine written in **Rust**, available for **Python** and **JavaScript (WebAssembly)**. Hide any file — PDF, video, ZIP, binary — inside a PNG, BMP, TIFF, or WebP image. The carrier image looks identical to the original. No pixels are changed in chunk mode.

---

## Table of Contents

- [How it works](#how-it-works)
- [Packages](#packages)
- [What's new in v0.3.0](#whats-new-in-v030)
- [Python](#python)
- [JavaScript / WASM](#javascript--wasm)
- [Supported formats](#supported-formats)
- [Security](#security)
- [Embedding modes](#embedding-modes)
- [Expiry system](#expiry-system)
- [Sharding](#sharding)
- [CLI](#cli)
- [Build from source](#build-from-source)
- [Architecture](#architecture)
- [Versioning](#versioning)
- [License](#license)

---

## How it works
```
hide:
  secret file
    → flate2 compress      (rust_backend, pure Rust, WASM-safe)
    → AES-256-GCM encrypt
    → embed into image
    → save image

reveal:
  stego image
    → extract payload
    → AES-256-GCM decrypt
    → flate2 decompress
    → restore original file
```

The original filename, file size, expiry, and embedding mode are stored inside the ciphertext — not in plaintext. An observer with the stego image but without the password sees none of this metadata.

---

## Packages

| Platform | Install | Status |
|---|---|---|
| **Python** | `pip install png-parser` | ✅ Published |
| **JS / WASM** | `npm i @pranjalpanging/png-parser` | ✅ Published |

---

## What's new in v0.3.0

| Feature | v0.2 | v0.3 |
|---|---|---|
| Payload type | Text strings only | **Any file** (PDF, ZIP, video, …) |
| Key derivation | PBKDF2-SHA256 (100k iter) | **Argon2id** (memory-hard, GPU-resistant) |
| Salt size | 16 bytes | **32 bytes** |
| Compression | None | **flate2 rust_backend (pure Rust, WASM-safe)** |
| Image formats | PNG only | **PNG, BMP, TIFF, WebP** |
| Embed modes | Chunk only | **Chunk + Pixel (adaptive LSB)** |
| Expiry granularity | Hours only | **Days + hours + minutes + seconds** |
| Delete protection | Unprotected | **Password-required for encrypted payloads** |
| Secure erase | No | **Payload zeroed before removal** |
| New functions | — | `info`, `verify`, `reencrypt`, `capacity`, `fingerprint`, `split`, `merge` |
| Filename preservation | No | **Original filename stored in header** |
| Sharding | No | **Split file across multiple images** |

---

## Python

### Install
```bash
pip install png-parser>=0.3.0
```

### Hide a file
```python
import png_parser

# No password — plain compression only
png_parser.hide("photo.png", "out.png", "document.pdf")

# With encryption
png_parser.hide("photo.png", "out.png", "document.pdf",
    password="my-password")

# With encryption + flexible expiry (all units are additive)
png_parser.hide("photo.png", "out.png", "document.pdf",
    password="my-password",
    expires_days=1,
    expires_hours=6,
    expires_minutes=30,
    expires_seconds=0)

# Pixel mode — embeds in image pixels instead of a chunk
png_parser.hide("photo.png", "out.png", "document.pdf",
    password="my-password",
    mode="pixel")
```

### Reveal a file
```python
# output_path can be a directory — original filename is restored automatically
result = png_parser.reveal("out.png", "./extracted/",
    password="my-password")
print(result)
# ./extracted/document.pdf
```

### Inspect without extracting
```python
import json

raw  = png_parser.info("out.png", password="my-password")
meta = json.loads(raw)

print(meta["filename"])    # document.pdf
print(meta["file_size"])   # 204800
print(meta["encrypted"])   # True
print(meta["expires_at"])  # unix:1742000000 or permanent
print(meta["fingerprint"]) # a3f9c1d2...
print(meta["mode"])        # chunk or pixel

# Without password — only fingerprint and encrypted flag visible
raw = png_parser.info("out.png")
# {"encrypted": true, "fingerprint": "a3f9c1d2...", ...}
```

### Verify a password
```python
# Check without extracting anything
ok = png_parser.verify("out.png", "my-password")
print(ok)  # True or False
```

### Delete a payload
```python
# Password required if payload is encrypted
# Data is zeroed in place before removal — not just stripped
png_parser.delete("out.png", "clean.png", password="my-password")
```

### Change password
```python
# Re-encrypts without writing plaintext to disk at any point
png_parser.reencrypt("out.png", "new.png",
    old_password="old-password",
    new_password="new-password")
```

### Check capacity
```python
# How many bytes can this image hold?
bytes_available = png_parser.capacity("photo.png", mode="chunk")
print(f"{bytes_available:,} bytes available")

bytes_available = png_parser.capacity("photo.png", mode="pixel")
print(f"{bytes_available:,} bytes available in pixel mode")
```

### Fingerprint
```python
# SHA-256 of the raw envelope — no password needed
# Useful to verify two images carry the same payload
fp = png_parser.fingerprint("out.png")
print(fp)  # a3f9c1d2ee4b...
```

### Split a large file across multiple images
```python
# Each carrier image gets one shard
shards = png_parser.split(
    "bigfile.zip",
    ["photo1.png", "photo2.png", "photo3.png"],
    "./shards/",
    password="my-password",
    expires_days=7,
)
print(shards)
# ['./shards/shard_0_photo1.png',
#  './shards/shard_1_photo2.png',
#  './shards/shard_2_photo3.png']
```

### Merge shards
```python
# Shards can be passed in any order — sorted internally by index
# SHA-256 of reassembled file is verified against stored hash
output = png_parser.merge(
    ["./shards/shard_2_photo3.png",
     "./shards/shard_0_photo1.png",
     "./shards/shard_1_photo2.png"],
    "./output/",
    password="my-password",
)
print(output)  # ./output/bigfile.zip
```

---

## JavaScript / WASM

### Install
```bash
npm i @pranjalpanging/png-parser
```

### Initialise
```javascript
import init, {
    hide_js,
    reveal_js,
    info_js,
    verify_js,
    delete_js,
    reencrypt_js,
    fingerprint_js,
    capacity_js,
} from '@pranjalpanging/png-parser';

await init();
```

### Hide a file
```javascript
// imageBytes and fileBytes are Uint8Array
const stego = hide_js(
    imageBytes,    // carrier image bytes
    fileBytes,     // file to hide
    "secret.pdf",  // original filename (stored in header)
    "password",    // encryption password — pass null for no encryption
    "chunk",       // mode: "chunk" or "pixel"
    null,          // expires_days
    null,          // expires_hours
    null,          // expires_minutes
    null           // expires_seconds
);
// stego is Uint8Array — the modified image bytes
```

### Reveal a file
```javascript
// Returns Uint8Array of the hidden file bytes
const fileBytes = reveal_js(stegoBytes, "password");

// Download in browser
const blob = new Blob([fileBytes]);
const url  = URL.createObjectURL(blob);
const a    = document.createElement("a");
a.href     = url;
a.download = "secret.pdf";
a.click();
URL.revokeObjectURL(url);
```

### Inspect without extracting
```javascript
// Returns JSON string
const meta = JSON.parse(info_js(stegoBytes, "password"));
console.log(meta.filename);    // "secret.pdf"
console.log(meta.file_size);   // 204800
console.log(meta.encrypted);   // true
console.log(meta.expires_at);  // "unix:1742000000" or "permanent"
console.log(meta.fingerprint); // "a3f9c1d2..."
```

### Other functions
```javascript
// Verify password
const ok = verify_js(stegoBytes, "password"); // true / false

// Delete payload (password required if encrypted)
const cleanBytes = delete_js(stegoBytes, "password");

// Change password
const newBytes = reencrypt_js(stegoBytes, "old-pass", "new-pass");

// Fingerprint (no password needed)
const fp = fingerprint_js(stegoBytes);

// Capacity
const bytes = capacity_js(imageBytes, "chunk");
```

### Full browser example
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>png-parser demo</title>
</head>
<body>
  <h2>Hide a file inside an image</h2>

  <label>Carrier image (PNG/BMP/TIFF/WebP)</label><br>
  <input type="file" id="carrier" accept="image/*"><br><br>

  <label>Secret file (any format)</label><br>
  <input type="file" id="secret"><br><br>

  <label>Password (optional)</label><br>
  <input type="password" id="password" placeholder="Leave blank for no encryption"><br><br>

  <label>Expiry (optional)</label><br>
  <input type="number" id="days"    placeholder="Days"   style="width:80px">
  <input type="number" id="hours"   placeholder="Hours"  style="width:80px">
  <input type="number" id="minutes" placeholder="Minutes" style="width:80px"><br><br>

  <button onclick="hideFile()">Hide and Download</button>
  <button onclick="revealFile()">Reveal from image</button>

  <p id="status"></p>

  <script type="module">
    import init, { hide_js, reveal_js } from './pkg/png_parser.js';
    await init();

    function status(msg) {
      document.getElementById("status").textContent = msg;
    }

    function download(bytes, name, type = "image/png") {
      const blob = new Blob([bytes], { type });
      const a    = document.createElement("a");
      a.href     = URL.createObjectURL(blob);
      a.download = name;
      a.click();
      URL.revokeObjectURL(a.href);
    }

    window.hideFile = async () => {
      const carrier  = document.getElementById("carrier").files[0];
      const secret   = document.getElementById("secret").files[0];
      const password = document.getElementById("password").value || null;
      const days     = parseInt(document.getElementById("days").value)    || null;
      const hours    = parseInt(document.getElementById("hours").value)   || null;
      const minutes  = parseInt(document.getElementById("minutes").value) || null;

      if (!carrier || !secret) {
        return status("Please select both files.");
      }

      const carrierBytes = new Uint8Array(await carrier.arrayBuffer());
      const secretBytes  = new Uint8Array(await secret.arrayBuffer());

      try {
        const result = hide_js(
            carrierBytes, secretBytes, secret.name,
            password, "chunk",
            days, hours, minutes, null
        );
        download(result, "stego_" + carrier.name);
        status("Done — file hidden successfully.");
      } catch (e) {
        status("Error: " + e);
      }
    };

    window.revealFile = async () => {
      const carrier  = document.getElementById("carrier").files[0];
      const password = document.getElementById("password").value || null;

      if (!carrier) return status("Please select a stego image.");

      const carrierBytes = new Uint8Array(await carrier.arrayBuffer());

      try {
        const fileBytes = reveal_js(carrierBytes, password);
        download(fileBytes, "revealed_file", "application/octet-stream");
        status("Done — file revealed successfully.");
      } catch (e) {
        status("Error: " + e);
      }
    };
  </script>
</body>
</html>
```

---

## Supported formats

| Format | Chunk mode | Pixel mode | Method |
|---|---|---|---|
| PNG | ✅ | ✅ | Custom `stEg` ancillary chunk / adaptive LSB |
| BMP | ✅ | ✅ | Append after EOF / adaptive LSB |
| TIFF | ✅ | ✅ | Append after EOF / adaptive LSB |
| WebP | ✅ | ❌ | Unknown RIFF chunk |
| JPEG | ❌ | ❌ | Rejected — lossy format destroys data |

---

## Security

| Component | Detail |
|---|---|
| Cipher | AES-256-GCM — authenticated encryption |
| KDF | Argon2id — memory-hard, resistant to GPU/ASIC attacks |
| Salt | 32 bytes, randomly generated per operation |
| Nonce | 12 bytes, randomly generated per operation |
| Compression | flate2 rust_backend — pure Rust, zero C dependencies |
| Metadata | Filename, expiry, mode stored inside ciphertext — tamper-proof |
| Integrity | GCM authentication tag detects any modification |
| Secure delete | Payload bytes zeroed in place before chunk removal |
| Password verify | Decrypt attempted — plaintext never returned on failure |

Two encryptions of the same file with the same password produce different ciphertext because salt and nonce are randomised every time.

---

## Embedding modes

### Chunk mode (default)

Inserts a custom ancillary PNG chunk named `stEg` just before `IEND`. Per the PNG specification, unknown ancillary chunks are silently skipped by all standard image viewers. The image pixels are never modified — the file is visually and statistically identical to the original.

For BMP and TIFF, data is appended after the image EOF with a magic marker. For WebP, a proper RIFF chunk is inserted.

### Pixel mode

Embeds data into the least-significant bit of the R, G, B channels of high-texture pixels only. Flat, uniform regions are skipped entirely. This defeats simple statistical detectors such as chi-square analysis and RS analysis which look for LSB changes in smooth areas.

Use `capacity()` to check how many bytes are available before embedding.

---

## Expiry system

Expiry is stored as a Unix timestamp inside the ciphertext. It is tamper-proof — an attacker cannot extend or remove the expiry without the password.

All time units are optional and additive:
```python
# Expires in exactly 1 day, 6 hours, 30 minutes from now
png_parser.hide("photo.png", "out.png", "file.pdf",
    password="x",
    expires_days=1,
    expires_hours=6,
    expires_minutes=30)
```

When `reveal()` is called after expiry, a `PermissionError` is raised. The file bytes are never written to disk.

---

## Sharding

Split a file too large for a single carrier across multiple images:
```
file.zip (300 MB)
  → shard 0 → photo1.png  (100 MB payload)
  → shard 1 → photo2.png  (100 MB payload)
  → shard 2 → photo3.png  (100 MB payload)
```

Each shard header stores the shard index, total count, and SHA-256 of the full original file. `merge()` sorts shards by index, assembles them, and verifies the SHA-256 before writing output. Any missing or mismatched shard causes an error.

---

## CLI
```bash
# Hide a file
png-parser-cli hide -i photo.png -f secret.pdf -o out.png \
    -p password --days 1 --hours 6 --minutes 30 --mode chunk

# Reveal
png-parser-cli reveal -i out.png -o ./extracted/ -p password

# Inspect metadata
png-parser-cli info -i out.png -p password

# Verify password
png-parser-cli verify -i out.png -p password

# Delete payload (password required if encrypted)
png-parser-cli delete -i out.png -o clean.png -p password

# Change password
png-parser-cli reencrypt -i out.png -o new.png \
    --old-password old --new-password new

# Check capacity
png-parser-cli capacity -i photo.png --mode chunk
png-parser-cli capacity -i photo.png --mode pixel

# Fingerprint
png-parser-cli fingerprint -i out.png

# Inspect PNG chunk structure
png-parser-cli inspect -i photo.png

# Strip metadata chunks (tEXt, zTXt, iTXt, eXIf)
png-parser-cli strip -i photo.png -o clean.png

# Split a file across multiple carriers
png-parser-cli split -f bigfile.zip \
    -c photo1.png photo2.png photo3.png \
    -o ./shards/ -p password --days 7

# Merge shards (any order)
png-parser-cli merge \
    -i shards/shard_0_photo1.png \
       shards/shard_1_photo2.png \
       shards/shard_2_photo3.png \
    -o ./output/ -p password
```

---

## Build from source

**Requirements:**
- Rust stable (`rustup install stable`)
- Python 3.8+ with `pip install maturin` (for Python build)
- `npm install -g wasm-pack` (for WASM build)
```bash
# Clone
git clone https://github.com/PranjalPanging/png-parser
cd png-parser

# Python — installs into current virtualenv
maturin develop --release

# WASM — outputs to pkg/
wasm-pack build --target web --release \
    --no-default-features --features js

# CLI binary
cargo build --release --no-default-features
```

---

## Architecture
```
src/
├── lib.rs          Python + WASM API surface
├── main.rs         CLI entry point
├── error.rs        All error variants + Python exception mapping
├── chunk_type.rs   4-byte PNG chunk name + spec rules
├── chunk.rs        PNG chunk: parse, CRC verify, secure erase
├── png.rs          Full PNG file: parse, build, insert, remove
├── header.rs       PayloadHeader — inside ciphertext, tamper-proof
├── crypto.rs       Argon2id KDF + AES-256-GCM + zlib
├── format.rs       Format detection + routing (PNG/BMP/TIFF/WebP)
├── pixel.rs        Adaptive LSB engine
└── commands/
    ├── mod.rs
    └── mode.rs     All operations: hide, reveal, info, verify,
                    delete, reencrypt, capacity, fingerprint,
                    split, merge, inspect, strip
```

---

## Versioning

This project follows [Semantic Versioning](https://semver.org).

| Package | Current | Registry |
|---|---|---|
| Python | `0.3.0` | [PyPI](https://pypi.org/project/png-parser/) |
| JavaScript | `0.3.0` | [npm](https://www.npmjs.com/package/@pranjalpanging/png-parser) |

Python and JavaScript versions are kept in sync from v0.3.0 onwards.

| Version | Notes |
|---|---|
| `0.3.0` | Full rewrite — any file type, Argon2id, multi-format, WASM, sharding |
| `0.2.0` | AES-256-GCM encryption, expiry system |
| `0.1.0` | Initial release |

Versions below `1.0.0` may have breaking changes between minor releases.

---

## License

MIT — see [LICENSE](LICENSE)

---

<div align="center">

Built with ❤️ by **[Pranjal Panging](https://github.com/pranjalpanging)**

[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/pranjalpanging)

</div>
```
