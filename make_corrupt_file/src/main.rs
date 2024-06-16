use std::env;
use std::fs::File;
use std::io::{Read, Write};
use rand::Rng;

fn corrupt_file(input_path: &str, output_path: &str) -> std::io::Result<()> {
    // Otevřít vstupní soubor pro čtení
    let mut input_file = File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    // Poškodit některé bajty
    let mut rng = rand::thread_rng();
    let num_bytes_to_corrupt = buffer.len() / 10; // Poškodí 10% souboru
    for _ in 0..num_bytes_to_corrupt {
        let index = rng.gen_range(0..buffer.len());
        buffer[index] = rng.gen();
    }

    // Zapsat poškozený obsah do nového souboru
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&buffer)?;

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        return;
    }
    
    let input_path = &args[1];
    let output_path = "corrupted_file";

    if let Err(e) = corrupt_file(input_path, output_path) {
        eprintln!("Error corrupting file: {}", e);
    } else {
        println!("File successfully corrupted and saved as {}", output_path);
    }
}
