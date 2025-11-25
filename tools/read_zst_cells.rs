//! Read CellIndex values from a zstd-compressed binary file.
//!
//! This tool reads 8-byte little-endian u64 values from a zst file,
//! converts them to CellIndex, and prints statistics.

use h3on::CellIndex;
use std::{fs::File, io::Read, path::PathBuf};
use zstd::stream::read::Decoder;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-zst-file>", args[0]);
        eprintln!("Example: {} dataset/res9_cells.zst", args[0]);
        std::process::exit(1);
    }

    let path = PathBuf::from(&args[1]);

    println!("Reading from: {}", path.display());

    let file = File::open(&path)?;
    let mut decoder = Decoder::new(file)?;
    // Set window log max to handle large compressed files (2GB window size)
    decoder.window_log_max(31)?;

    let mut buf = [0u8; 8];
    let mut count = 0_u64;
    let mut valid_count = 0_u64;
    let mut invalid_count = 0_u64;

    println!("\nReading cells...");

    loop {
        match decoder.read_exact(&mut buf) {
            Ok(_) => {
                count += 1;
                let raw_value = u64::from_le_bytes(buf);

                match CellIndex::try_from(raw_value) {
                    Ok(cell) => {
                        valid_count += 1;

                        // Print first 10 cells (debug mode only)
                        #[cfg(debug_assertions)]
                        if valid_count <= 10 {
                            println!(
                                "  {}: 0x{:016x} -> {:?}",
                                valid_count, raw_value, cell
                            );
                        }
                    }
                    Err(_) => {
                        invalid_count += 1;
                        #[cfg(debug_assertions)]
                        if invalid_count <= 5 {
                            eprintln!(
                                "  Invalid cell at position {}: 0x{:016x}",
                                count, raw_value
                            );
                        }
                    }
                }

                // Print progress every 100k cells (debug mode only)
                #[cfg(debug_assertions)]
                if count % 100_000 == 0 {
                    println!(
                        "Processed {} cells... ({} valid, {} invalid)",
                        count, valid_count, invalid_count
                    );
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Total entries read: {}", count);
    println!("Valid CellIndex: {}", valid_count);
    println!("Invalid CellIndex: {}", invalid_count);
    println!(
        "Success rate: {:.2}%",
        (valid_count as f64 / count as f64) * 100.0
    );

    Ok(())
}
