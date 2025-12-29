use std::path::PathBuf;
use std::fs;
use std::str::FromStr;
use crate::png::Png;
use crate::chunk::Chunk;
use crate::chunk_type::ChunkType;
use std::convert::TryFrom;



pub fn inspect(file: PathBuf) {
    let bytes = fs::read(&file).expect("Could not read file");
    let png = Png::try_from(&bytes[..]).expect("Invalid PNG");
    println!("Total Chunks: {}", png.chunks().len());
    for chunk in png.chunks() {
        println!("  Chunk: {} ({} bytes)", chunk.chunk_type(), chunk.length());
    }
}

pub fn strip(input: PathBuf, output: PathBuf) {
    let bytes = fs::read(&input).expect("Could not read input");
    let mut png = Png::try_from(&bytes[..]).expect("Invalid PNG");
    
    let metadata_types = ["tEXt", "zTXt", "iTXt", "eXIf"];
    for name in metadata_types {
        png.remove_chunk(name);
    }
    
    fs::write(&output, png.as_bytes()).expect("Could not write output");
    println!("Metadata stripped successfully.");
}

pub fn hide(input: PathBuf, message: String, output: PathBuf) {
    let bytes = fs::read(&input).expect("Could not read input");
    let mut png = Png::try_from(&bytes[..]).expect("Invalid PNG");
    
    let chunk_type = ChunkType::from_str("stEg").expect("Invalid chunk type");
    let chunk = Chunk::new(chunk_type, message.as_bytes().to_vec());
    
    png.append_chunk(chunk);
    
    fs::write(&output, png.as_bytes()).expect("Could not write output");
    println!("Message hidden successfully.");
}

pub fn read(file: PathBuf, chunk_type_str: String) {
    let bytes = fs::read(&file).expect("Could not read file");
    let png = Png::try_from(&bytes[..]).expect("Invalid PNG");
    
    if let Some(chunk) = png.chunks().iter().find(|c| c.chunk_type().to_string() == chunk_type_str) {
        let message = String::from_utf8_lossy(chunk.data());
        println!("Secret message found: {}", message);
    } else {
        println!("No chunk of type '{}' found in this file.", chunk_type_str);
    }
}