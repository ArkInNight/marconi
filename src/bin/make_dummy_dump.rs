use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

const NOISE: u8 = 0xAB;
const CHUNK: usize = 1024 * 1024;

const DUMMY_URL: &str = "https://game-re-en-service.sl916.com/query/summon?userId=12345&token=123e4567-e89b-12d3-a456-426614174000&sign=demoSign";

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(path) = args.first() else {
        eprintln!("usage: make-dummy-dump <output.dmp>");
        std::process::exit(2);
    };
    let path = PathBuf::from(path);

    let url_utf8 = DUMMY_URL.as_bytes().to_vec();
    let url_utf16: Vec<u8> = DUMMY_URL
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();

    let size = CHUNK * 4;
    let mut dump = vec![NOISE; size];
    // Straddle the 2nd->3rd chunk boundary so the chunked scanner's carry
    // (OVERLAP) path is exercised, not just single-chunk finds.
    let utf8_at = 2 * CHUNK - 40;
    dump[utf8_at..utf8_at + url_utf8.len()].copy_from_slice(&url_utf8);
    let utf16_at = 2 * CHUNK + 128 * 1024;
    dump[utf16_at..utf16_at + url_utf16.len()].copy_from_slice(&url_utf16);

    let mut file = File::create(&path)?;
    file.write_all(&dump)?;

    println!("Wrote dummy dump: {}", path.display());
    println!(
        "  size:  {:.1} MiB ({} chunks of {})",
        size as f64 / (1024.0 * 1024.0),
        size / CHUNK,
        CHUNK
    );
    println!("  URL:   {DUMMY_URL}");
    println!();
    println!("Test it with:");
    println!("  marconi --dump-file {}", path.display());
    println!("Expected output:");
    println!("  https://game-re-en-service.sl916.com/query/summon?userId=12345&token=123e4567-e89b-12d3-a456-426614174000");
    Ok(())
}
