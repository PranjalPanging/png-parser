# PNG Parser

A high-performance steganography tool built with **Rust** and **Python**. This library allows you to hide, read, and delete secret messages within PNG images using custom data chunks without affecting the visual quality of the image.

---

## Features
- **Blazing Fast**: Core logic implemented in Rust for maximum speed.
- **Stealthy**: Uses custom PNG chunks (`stEg`) that are ignored by standard image viewers.
- **Easy to Use**: Simple Pythonic API.
- **Safe**: Includes a `delete` function to strip hidden data and restore original file integrity.

---

## Installation

Once the package is uploaded to PyPI, you can install it using pip:

```bash
pip install png_parser
```

## Usage
### 1. Hiding a Message
Embed a secret string into any PNG file. This adds the data safely before the end of the file.
```Python
import png_parser

# This appends a hidden 'stEg' chunk to the image
status = png_parser.hide("my_image.png", "hello world")
print(status)
# Output: Success: Message hidden!
```
### 2. Reading a Message
Extract the hidden data from the image. If no message is found, it returns a helpful error string.
```Python
import png_parser

secret = png_parser.read("my_image.png")
print(f"Secret message: {secret}")
# Output: Secret message: hello world
```
### 3. Deleting the Secret
Remove the hidden chunks and restore the PNG to its original state (cleaning up the file size).
```Python
import png_parser

status = png_parser.delete("my_image.png")
print(status) 
# Output: Success: Secret message deleted!
```
## Technical Details
This tool manipulates the PNG Chunk Structure.
Every PNG consists of a series of chunks. This library inserts an "Ancillary Chunk" (optional data) named `stEg`. Standard image viewers are programmed to skip chunks they don't recognize. By placing our data before the `IEND` (End of Image) marker, the file remains a valid, viewable image while carrying your hidden payload.

## Development
If you want to build this project from source, you will need:
- Rust (Cargo)
- Python 3.7+
- Maturin (`pip install maturin`)

To build locally:
```Bash
maturin develop
```
## License
This project is licensed under the MIT License.

## Author
Pranjal Panging
