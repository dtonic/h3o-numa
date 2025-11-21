use clap::Parser;
use h3o::{CellIndex, Resolution};
use std::io::{self, BufWriter, Result, Write};

/// Generate H3 cell indices at a given resolution
#[derive(Parser, Debug)]
#[command(name = "gen_index_stdout")]
#[command(about = "Generate H3 cell indices at a given resolution", long_about = None)]
struct Args {
    /// Resolution: 0 for base cells, 1-15 for child cells
    #[arg(value_parser = clap::value_parser!(u8).range(0..=15))]
    resolution: u8,

    /// Output as little-endian bytes instead of text
    #[arg(long)]
    raw: bool,

    /// Add newline after each output (text mode only)
    #[arg(long)]
    newline: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let stdout = io::stdout();
    let mut handle = BufWriter::with_capacity(8 * 1024 * 1024, stdout.lock());

    let res = if args.resolution == 0 {
        None
    } else {
        Some(Resolution::try_from(args.resolution).unwrap())
    };

    // Generate cells iterator
    let mut cells: Box<dyn Iterator<Item = CellIndex>> = Box::new(
        CellIndex::base_cells().map(CellIndex::from)
    );

    if let Some(target_res) = res {
        cells = Box::new(cells.flat_map(move |bc| bc.children(target_res)));
    }

    // Write cells based on output mode
    if args.raw {
        cells.try_for_each(|cell| handle.write_all(&u64::from(cell).to_le_bytes()))?;
    } else if args.newline {
        cells.try_for_each(|cell| writeln!(handle, "{cell}"))?;
    } else {
        cells.try_for_each(|cell| write!(handle, "{cell}"))?;
    }

    handle.flush()?;
    Ok(())
}
