# png-parser

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![WASM](https://img.shields.io/badge/wasm-compiled-blueviolet.svg)](https://webassembly.org/)
[![Python](https://img.shields.io/badge/python-3.8+-blue.svg)](https://www.python.org/downloads/)
[![npm](https://img.shields.io/badge/npm-%40pranjalpanging%2Fpng--parser-red.svg)](https://www.npmjs.com/package/@pranjalpanging/png-parser)

Hide any file inside an image — compress, encrypt, embed.  
A high-performance steganography engine written in Rust, available for both Python and JavaScript (WebAssembly).

## Packages

| Platform | Install | Status |
|---|---|---|
| **Python** | `pip install png-parser` | ✅ Published |
| **JS / WASM** | `npm i @pranjalpanging/png-parser` | ✅ Published |

---

## What's new in v0.3.0

| Feature | v0.2 | v0.3 |
|---|---|---|
| Payload | Text strings only | **Any file** (PDF, ZIP, video, …) |
| Key derivation | PBKDF2-SHA256 | **Argon2id** (memory-hard) |
| Compression | None | **zstd level 19** |
| Salt | 16 bytes | **32 bytes** |
| Image formats | PNG only | **PNG, BMP, TIFF, WebP** |
| Embed modes | Chunk only | **Chunk + Pixel (adaptive LSB)** |
| Expiry | Hours only | **Days + hours + minutes + seconds** |
| Delete | Unprotected | **Password-protected** |
| New functions | — | `info`, `verify`, `reencrypt`, `capacity`, `fingerprint`, `split`, `merge` |

---

## How it works
```
hide:   file → zstd compress → AES-256-GCM encrypt → embed into image
reveal: extract from image → decrypt → decompress → restore file
```

Two embedding modes:

| Mode | Method | Pixel change | Formats |
|---|---|---|---|
| `chunk` (default) | Custom `stEg` ancillary chunk | **None** | PNG, BMP, TIFF, WebP |
| `pixel` | Adaptive LSB (high-texture only) | LSB of R/G/B only | PNG, BMP, TIFF |

---

## Python

### Install
```bash
pip install png-parser>=0.3.0
```

### Hide a file
```python
import png_parser

# Plain — no password
png_parser.hide("photo.png", "out.png", "document.pdf")

# Encrypted
png_parser.hide("photo.png", "out.png", "document.pdf",
    password="my-password")

# Encrypted + expires in 1 day 6 hours 30 minutes
png_parser.hide("photo.png", "out.png", "document.pdf",
    password="my-password",
    expires_days=1, expires_hours=6, expires_minutes=30)

# Pixel mode (embeds in image pixels instead of chunk)
png_parser.hide("photo.png", "out.png", "document.pdf",
    password="my-password",
    mode="pixel")
```

### Reveal a file
```python
# output_path can be a directory — original filename is restored
result = png_parser.reveal("out.png", "./extracted/", password="my-password")
print(result)  # ./extracted/document.pdf
```

### Inspect without extracting
```python
info = png_parser.info("out.png", password="my-password")
# {
#   "has_payload": true,
#   "encrypted": true,
#   "filename": "document.pdf",
#   "file_size": 204800,
#   "mode": "chunk",
#   "expires_at": "unix:1742000000",
#   "fingerprint": "a3f9c1d2..."
# }
```

### Other functions
```python
# Check password without extracting
ok = png_parser.verify("out.png", "my-password")  # True / False

# Delete payload (password required if encrypted)
png_parser.delete("out.png", "clean.png", password="my-password")

# Change password without extracting file
png_parser.reencrypt("out.png", "new.png",
    old_password="old", new_password="new")

# Check capacity before hiding
bytes_available = png_parser.capacity("photo.png", mode="chunk")
print(f"{bytes_available:,} bytes available")

# Fingerprint — identify payload without password
fp = png_parser.fingerprint("out.png")

# Split a large file across multiple images
shards = png_parser.split(
    "bigfile.zip",
    ["photo1.png", "photo2.png", "photo3.png"],
    "./shards/",
    password="my-password",
    expires_days=7,
)

# Reassemble — shards can be in any order
png_parser.merge(shards, "./output/", password="my-password")
```

---

## JavaScript / WASM

### Install
```bash
npm i @pranjalpanging/png-parser
```

### Hide a file
```javascript
import init, { hide_js, reveal_js, info_js, verify_js,
               delete_js, reencrypt_js, fingerprint_js,
               capacity_js } from '@pranjalpanging/png-parser';

await init();

// imageBytes and fileBytes are Uint8Array
const stego = hide_js(
    imageBytes,   // carrier image bytes
    fileBytes,    // file to hide
    "secret.pdf", // original filename
    "password",   // encryption password (or null)
    "chunk",      // mode: "chunk" or "pixel"
    null,         // expires_days
    null,         // expires_hours
    null,         // expires_minutes
    null          // expires_seconds
);
// stego is Uint8Array — the output image bytes
```

### Reveal a file
```javascript
// Returns Uint8Array of the hidden file bytes
const fileBytes = reveal_js(stegoBytes, "password");

// Save it in the browser
const blob = new Blob([fileBytes]);
const url  = URL.createObjectURL(blob);
const a    = document.createElement("a");
a.href     = url;
a.download = "secret.pdf";
a.click();
```

### Inspect without extracting
```javascript
// Returns a JSON string
const meta = JSON.parse(info_js(stegoBytes, "password"));
console.log(meta.filename);   // "secret.pdf"
console.log(meta.file_size);  // 204800
console.log(meta.expires_at); // "unix:1742000000" or "permanent"
```

### Other functions
```javascript
// Verify password
const ok = verify_js(stegoBytes, "password"); // true / false

// Delete payload (password required if encrypted)
const cleanBytes = delete_js(stegoBytes, "password");

// Change password
const newBytes = reencrypt_js(stegoBytes, "old-password", "new-password");

// Fingerprint
const fp = fingerprint_js(stegoBytes);

// Capacity
const bytes = capacity_js(imageBytes, "chunk");
```

### Full browser example
```html
<!DOCTYPE html>
<html>
<body>
  <input type="file" id="carrier" accept="image/*">
  <input type="file" id="secret">
  <input type="password" id="password" placeholder="Password">
  <button onclick="hideFile()">Hide</button>

  <script type="module">
    import init, { hide_js } from './pkg/png_parser.js';
    await init();

    window.hideFile = async () => {
      const carrier  = document.getElementById("carrier").files[0];
      const secret   = document.getElementById("secret").files[0];
      const password = document.getElementById("password").value;

      const carrierBytes = new Uint8Array(await carrier.arrayBuffer());
      const secretBytes  = new Uint8Array(await secret.arrayBuffer());

      const result = hide_js(
          carrierBytes, secretBytes, secret.name,
          password || null, "chunk",
          null, null, null, null
      );

      const blob = new Blob([result], { type: "image/png" });
      const a    = document.createElement("a");
      a.href     = URL.createObjectURL(blob);
      a.download = "stego_" + carrier.name;
      a.click();
    };
  </script>
</body>
</html>
```

---

## Supported formats

| Format | Chunk mode | Pixel mode | Notes |
|---|---|---|---|
| PNG | ✅ | ✅ | Recommended |
| BMP | ✅ | ✅ | |
| TIFF | ✅ | ✅ | |
| WebP | ✅ | ❌ | Chunk mode only |
| JPEG | ❌ | ❌ | Lossy — rejected |

---

## Security

| Component | Detail |
|---|---|
| Cipher | AES-256-GCM (authenticated encryption) |
| KDF | Argon2id (memory-hard, GPU-resistant) |
| Salt | 32 bytes, random per operation |
| Nonce | 12 bytes, random per operation |
| Compression | zstd level 19 before encryption |
| Header | Filename + expiry inside ciphertext — tamper-proof |
| Delete | Payload zeroed before removal |

---

## Build from source

**Requirements:**
- Rust stable
- Python 3.8+ (for Python build)
- `pip install maturin`
- `npm install -g wasm-pack` (for WASM build)
```bash
# Python
maturin develop --release

# WASM
wasm-pack build --target web

# CLI
cargo build --release --no-default-features
```

---

## CLI
```bash
# Hide
png-parser-cli hide -i photo.png -f secret.pdf -o out.png -p password --days 1 --hours 6

# Reveal
png-parser-cli reveal -i out.png -o ./extracted/ -p password

# Info
png-parser-cli info -i out.png -p password

# Verify password
png-parser-cli verify -i out.png -p password

# Delete
png-parser-cli delete -i out.png -o clean.png -p password

# Change password
png-parser-cli reencrypt -i out.png -o new.png --old-password abc --new-password xyz

# Capacity
png-parser-cli capacity -i photo.png --mode pixel

# Split across multiple images
png-parser-cli split -f bigfile.zip -c photo1.png photo2.png photo3.png -o ./shards/ -p password

# Merge shards
png-parser-cli merge -i shards/shard_0_photo1.png shards/shard_1_photo2.png -o ./output/ -p password
```

---

## License

MIT — [Pranjal Panging](https://github.com/pranjalpanging)
```
