use can_crc_project::{parse_binary_input, parse_hex_input, compute_batch_crcs_optimized, CrcResult};
use clap::{Parser, ValueEnum};
use std::time::Instant;

#[derive(Debug, Clone, ValueEnum)]
enum InputFormat {
    #[value(name = "binarny")]
    Binary,
    #[value(name = "hex")]
    Hex,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Kalkulator CRC CAN - Interfejs Linii Poleceń", long_about = None)]
struct Args {
    #[arg(short, long, help = "Dane wejściowe (binarnie: \"101010...\" lub hex: \"AA BB CC\")")]
    data: String,

    #[arg(short = 'f', long, value_enum, default_value = "hex", help = "Format danych wejściowych")]
    format: InputFormat,

    #[arg(short, long, default_value = "1", help = "Liczba iteracji (1 do 1,000,000,000)")]
    iterations: u64,

    #[arg(short, long, help = "Szczegółowe informacje")]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    if args.iterations == 0 || args.iterations > 1_000_000_000 {
        eprintln!("❌ Błąd: Liczba iteracji musi być między 1 a 1,000,000,000");
        eprintln!("   Podano: {}", args.iterations);
        std::process::exit(1);
    }

    let bits = match args.format {
        InputFormat::Binary => {
            match parse_binary_input(&args.data) {
                Ok(bits) => bits,
                Err(e) => {
                    eprintln!("{}", e);
                    eprintln!("\n💡 Wskazówka: Użyj tylko znaków '0' i '1', np: -d \"10101010\"");
                    std::process::exit(1);
                }
            }
        }
        InputFormat::Hex => {
            match parse_hex_input(&args.data) {
                Ok(bits) => bits,
                Err(e) => {
                    eprintln!("{}", e);
                    eprintln!("\n💡 Wskazówka: Użyj tylko znaków 0-9 i A-F, np: -d \"AA BB CC\"");
                    std::process::exit(1);
                }
            }
        }
    };

    if bits.is_empty() {
        eprintln!("❌ Błąd: Brak prawidłowych danych wejściowych");
        std::process::exit(1);
    }

    if args.verbose {
        println!("\n╔══════════════════════════════════════╗");
        println!("║       Kalkulator CRC CAN             ║");
        println!("╚══════════════════════════════════════╝");
        println!("📋 Format wejściowy: {:?}", args.format);
        println!("📝 Dane wejściowe: {}", args.data);
        println!("🔢 Liczba bitów: {}", bits.len());
        println!("🔄 Liczba iteracji: {}", format_number(args.iterations));
        println!();
    }

    let start = Instant::now();
    let crc_value = compute_batch_crcs_optimized(&bits, args.iterations, args.verbose);
    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;

    let result = CrcResult::new(crc_value, duration_ms);

    println!("\n✅ Wyniki:");
    println!("═══════════════════════════════════════");
    println!("🎯 Wartość CRC (hex):    0x{}", result.crc_hex);
    println!("🔢 Wartość CRC (dec):    {}", result.crc_value);
    println!("🔢 Wartość CRC (bin):    {:015b}", result.crc_value);
    
    println!("\n⚡ Wydajność:");
    println!("═══════════════════════════════════════");
    println!("⏱️  Czas całkowity:      {:.3} ms", result.duration_ms);
    
    if args.iterations > 1 {
        let avg_time = result.duration_ms / args.iterations as f64;
        println!("⏱️  Średni czas na CRC:  {:.6} ms", avg_time);
        println!("⏱️  Średni czas na CRC:  {:.3} µs", avg_time * 1000.0);
        
        let ops_per_sec = (args.iterations as f64 / result.duration_ms) * 1000.0;
        println!("📊 Przepustowość:        {} CRC/s", format_number(ops_per_sec as u64));
    }

    if args.verbose && args.iterations >= 100_000 {
        println!("\n💡 Uwaga: Użyto przetwarzania równoległego dla optymalnej wydajności.");
    }
}

fn format_number(num: u64) -> String {
    let s = num.to_string();
    let mut result = String::new();
    let mut count = 0;
    
    for ch in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(' ');
        }
        result.push(ch);
        count += 1;
    }
    
    result.chars().rev().collect()
} 