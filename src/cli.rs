use can_crc_project::{parse_binary_input, parse_hex_input, compute_batch_crcs_optimized, CrcResult};
use clap::{Parser, ValueEnum};
use std::io;
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
    #[arg(short, long, help = "Szczegółowe informacje")]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    loop {
        println!("\nWybierz format ('hex', 'bin') lub wpisz 'exit' aby zakończyć:");
        let mut format_input = String::new();
        if io::stdin().read_line(&mut format_input).is_err() {
            eprintln!("❌ Błąd: Nie udało się odczytać formatu.");
            continue;
        }

        let format = match format_input.trim().to_lowercase().as_str() {
            "hex" => InputFormat::Hex,
            "bin" => InputFormat::Binary,
            "exit" => break,
            _ => {
                eprintln!("❌ Błąd: Nieprawidłowy format. Wybierz 'hex' lub 'bin'.");
                continue;
            }
        };

        println!("Podaj dane wejściowe:");
        let mut data_input = String::new();
        if io::stdin().read_line(&mut data_input).is_err() {
            eprintln!("❌ Błąd: Nie udało się odczytać danych.");
            continue;
        }
        let data_input = data_input.trim();

        println!("Podaj liczbę iteracji (1 do 1,000,000,000):");
        let mut iterations_input = String::new();
        if io::stdin().read_line(&mut iterations_input).is_err() {
            eprintln!("❌ Błąd: Nie udało się odczytać liczby iteracji.");
            continue;
        }
        let iterations: u64 = match iterations_input.trim().parse() {
            Ok(n) if (1..=1_000_000_000).contains(&n) => n,
            _ => {
                eprintln!("❌ Błąd: Liczba iteracji musi być między 1 a 1,000,000,000.");
                continue;
            }
        };

        let bits = match format {
            InputFormat::Binary => match parse_binary_input(data_input) {
                Ok(bits) => bits,
                Err(e) => {
                    eprintln!("{}", e);
                    eprintln!("\n💡 Wskazówka: Użyj tylko znaków '0' i '1'.");
                    continue;
                }
            },
            InputFormat::Hex => match parse_hex_input(data_input) {
                Ok(bits) => bits,
                Err(e) => {
                    eprintln!("{}", e);
                    eprintln!("\n💡 Wskazówka: Użyj tylko znaków 0-9 i A-F.");
                    continue;
                }
            },
        };

        if bits.is_empty() {
            eprintln!("❌ Błąd: Brak prawidłowych danych wejściowych");
            continue;
        }

        if args.verbose {
            println!("\n╔══════════════════════════════════════╗");
            println!("║       Kalkulator CRC CAN             ║");
            println!("╚══════════════════════════════════════╝");
            println!("📋 Format wejściowy: {:?}", format);
            println!("📝 Dane wejściowe: {}", data_input);
            println!("🔢 Liczba bitów: {}", bits.len());
            println!("🔄 Liczba iteracji: {}", format_number(iterations));
            println!();
        }

        let start = Instant::now();
        let crc_value = compute_batch_crcs_optimized(&bits, iterations, args.verbose);
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

        if iterations > 1 {
            let avg_time = result.duration_ms / iterations as f64;
            println!("⏱️  Średni czas na CRC:  {:.6} ms", avg_time);
            println!("⏱️  Średni czas na CRC:  {:.3} µs", avg_time * 1000.0);

            let ops_per_sec = (iterations as f64 / result.duration_ms) * 1000.0;
            println!("📊 Przepustowość:        {} CRC/s", format_number(ops_per_sec as u64));
        }

        if args.verbose && iterations >= 100_000 {
            println!("\n💡 Uwaga: Użyto przetwarzania równoległego dla optymalnej wydajności.");
        }
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