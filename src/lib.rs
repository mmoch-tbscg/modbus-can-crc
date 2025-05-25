use rayon::prelude::*;
use std::sync::atomic::{AtomicU16, Ordering};

/// CAN CRC polynomial: 0x4599
const CAN_POLY: u16 = 0x4599;

/// Result structure for CRC calculation
#[derive(Debug, Clone)]
pub struct CrcResult {
    pub crc_value: u16,
    pub crc_hex: String,
    pub duration_ms: f64,
}

impl CrcResult {
    pub fn new(crc_value: u16, duration_ms: f64) -> Self {
        Self {
            crc_value,
            crc_hex: format!("{:04X}", crc_value),
            duration_ms,
        }
    }
}

/// Parse binary string input (e.g., "101010111100...")
pub fn parse_binary_input(input: &str) -> Result<Vec<bool>, String> {
    if input.trim().is_empty() {
        return Err("❌ Błąd: Dane wejściowe są puste".to_string());
    }
    
    // Najpierw sprawdź czy są nieprawidłowe znaki
    let invalid_chars: Vec<char> = input.chars()
        .filter(|c| !c.is_whitespace() && *c != '0' && *c != '1')
        .collect();
    
    if !invalid_chars.is_empty() {
        let invalid_str: String = invalid_chars.iter().take(5).collect();
        return Err(format!(
            "❌ Błąd: Znaleziono nieprawidłowe znaki: '{}' (dozwolone tylko: 0, 1, spacje)",
            invalid_str
        ));
    }
    
    let cleaned = input.chars()
        .filter(|c| *c == '0' || *c == '1')
        .collect::<String>();
    
    if cleaned.is_empty() {
        return Err("❌ Błąd: Brak prawidłowych danych binarnych (tylko 0 i 1)".to_string());
    }
    
    if cleaned.len() > 96 {
        return Err(format!(
            "❌ Błąd: Dane za długie: {} bitów (maksymalnie dozwolone: 96 bitów)",
            cleaned.len()
        ));
    }
    
    Ok(cleaned.chars()
        .map(|c| c == '1')
        .collect())
}

/// Parse hex string input (e.g., "AA BB CC")
pub fn parse_hex_input(input: &str) -> Result<Vec<bool>, String> {
    if input.trim().is_empty() {
        return Err("❌ Błąd: Dane wejściowe są puste".to_string());
    }
    
    let cleaned = input.trim().to_uppercase();
    
    // Sprawdź nieprawidłowe znaki
    let invalid_chars: Vec<char> = cleaned.chars()
        .filter(|c| !c.is_ascii_hexdigit() && !c.is_whitespace())
        .collect();
    
    if !invalid_chars.is_empty() {
        let invalid_str: String = invalid_chars.iter().take(5).collect();
        return Err(format!(
            "❌ Błąd: Znaleziono nieprawidłowe znaki: '{}' (dozwolone tylko: 0-9, A-F, spacje)",
            invalid_str
        ));
    }
    
    let hex_string: String = cleaned.chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect();
    
    if hex_string.is_empty() {
        return Err("❌ Błąd: Brak prawidłowych danych hex".to_string());
    }
    
    if hex_string.len() % 2 != 0 {
        return Err(format!(
            "❌ Błąd: Nieparzysta liczba znaków hex: {} (wymagana parzysta liczba)",
            hex_string.len()
        ));
    }
    
    let bytes: Result<Vec<u8>, _> = (0..hex_string.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_string[i..i+2], 16))
        .collect();
    
    match bytes {
        Ok(byte_vec) => {
            if byte_vec.len() > 12 {
                return Err(format!(
                    "❌ Błąd: Dane za długie: {} bajtów = {} bitów (maksymalnie: 12 bajtów = 96 bitów)",
                    byte_vec.len(),
                    byte_vec.len() * 8
                ));
            }
            Ok(bytes_to_bits(&byte_vec))
        },
        Err(_) => Err("❌ Błąd: Nieprawidłowy format hex".to_string()),
    }
}

/// Convert bytes to bits (MSB first)
fn bytes_to_bits(bytes: &[u8]) -> Vec<bool> {
    let mut bits = Vec::with_capacity(bytes.len() * 8);
    for byte in bytes {
        for i in (0..8).rev() {
            bits.push((byte >> i) & 1 == 1);
        }
    }
    bits
}

/// Basic CAN CRC calculation (following the algorithm specification)
pub fn calculate_can_crc(bits: &[bool]) -> u16 {
    let mut crc_rg: u16 = 0;
    
    for &nxtbit in bits {
        // CRCNXT = NXTBIT EXOR CRC_RG(14)
        let crcnxt = nxtbit ^ ((crc_rg >> 14) & 1 == 1);
        
        // CRC_RG(14:1) = CRC_RG(13:0)
        crc_rg = (crc_rg << 1) & 0x7FFF;
        
        // IF CRCNXT THEN CRC_RG(14:0) = CRC_RG(14:0) EXOR (4599hex)
        if crcnxt {
            crc_rg ^= CAN_POLY;
        }
    }
    
    crc_rg
}

/// Optimized CAN CRC calculation using lookup table
pub fn calculate_can_crc_optimized(bits: &[bool]) -> u16 {
    // Pre-calculate CRC for each possible byte
    static CRC_TABLE: [u16; 256] = generate_crc_table();
    
    let mut crc_rg: u16 = 0;
    
    // Process complete bytes first
    let full_bytes = bits.len() / 8;
    for i in 0..full_bytes {
        let mut byte = 0u8;
        for j in 0..8 {
            if bits[i * 8 + j] {
                byte |= 1 << (7 - j);
            }
        }
        
        // Process byte using lookup table
        let tbl_idx = ((crc_rg >> 7) ^ (byte as u16)) as u8;
        crc_rg = ((crc_rg << 8) ^ CRC_TABLE[tbl_idx as usize]) & 0x7FFF;
    }
    
    // Process remaining bits
    for i in (full_bytes * 8)..bits.len() {
        let nxtbit = bits[i];
        let crcnxt = nxtbit ^ ((crc_rg >> 14) & 1 == 1);
        crc_rg = (crc_rg << 1) & 0x7FFF;
        if crcnxt {
            crc_rg ^= CAN_POLY;
        }
    }
    
    crc_rg
}

/// Generate CRC lookup table
const fn generate_crc_table() -> [u16; 256] {
    let mut table = [0u16; 256];
    let mut i = 0;
    
    while i < 256 {
        let mut crc = (i as u16) << 7;
        let mut j = 0;
        
        while j < 8 {
            if (crc & 0x4000) != 0 {
                crc = ((crc << 1) ^ CAN_POLY) & 0x7FFF;
            } else {
                crc = (crc << 1) & 0x7FFF;
            }
            j += 1;
        }
        
        table[i] = crc;
        i += 1;
    }
    
    table
}

/// Compute CRC multiple times with optimization
pub fn compute_batch_crcs_optimized(bits: &[bool], iterations: u64, verbose: bool) -> u16 {
    if iterations == 1 {
        return calculate_can_crc_optimized(bits);
    }
    
    // For large iteration counts, use parallel processing
    if iterations >= 100_000 {
        if verbose {
            println!("ℹ️  Używanie przetwarzania równoległego dla {} iteracji", iterations);
        }
        
        let result = AtomicU16::new(0);
        let num_threads = rayon::current_num_threads();
        let chunk_size = (iterations as usize / num_threads).max(1);
        
        (0..num_threads)
            .into_par_iter()
            .for_each(|thread_idx| {
                let start = thread_idx * chunk_size;
                let end = if thread_idx == num_threads - 1 {
                    iterations as usize
                } else {
                    (thread_idx + 1) * chunk_size
                };
                
                let mut local_crc = 0u16;
                for _ in start..end {
                    local_crc = calculate_can_crc_optimized(bits);
                }
                result.store(local_crc, Ordering::Relaxed);
            });
        
        result.load(Ordering::Relaxed)
    } else {
        // For smaller counts, sequential is more efficient
        let mut crc = 0u16;
        for _ in 0..iterations {
            crc = calculate_can_crc_optimized(bits);
        }
        crc
    }
} 