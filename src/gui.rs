use eframe::egui;
use can_crc_project::{parse_binary_input, parse_hex_input, compute_batch_crcs_optimized, CrcResult};
use std::time::Instant;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 750.0])
            .with_title("🚗 Kalkulator CRC CAN"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Kalkulator CRC CAN",
        options,
        Box::new(|_cc| Ok(Box::new(CanCrcApp::default()))),
    )
}

#[derive(Default)]
struct CanCrcApp {
    input_format: InputFormat,
    binary_input: String,
    hex_input: String,
    iterations_input: String,
    result: Option<CrcResult>,
    error_message: String,
    is_calculating: bool,
    last_calculation_time: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum InputFormat {
    Binary,
    #[default]
    Hex,
}

impl eframe::App for CanCrcApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🚗 Kalkulator CRC CAN");
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(15.0);
            
            ui.horizontal(|ui| {
                ui.label("📋 Format wejściowy:");
                ui.radio_value(&mut self.input_format, InputFormat::Binary, "Binarny");
                ui.radio_value(&mut self.input_format, InputFormat::Hex, "Heksadecymalny");
            });
            
            ui.add_space(10.0);
            
            match self.input_format {
                InputFormat::Binary => {
                    ui.horizontal(|ui| {
                        ui.label("🔢 Sekwencja binarna:");
                        let response = ui.add(egui::TextEdit::singleline(&mut self.binary_input)
                            .desired_width(400.0)
                            .hint_text("101010111100..."));
                        
                        // Walidacja w czasie rzeczywistym
                        if response.changed() {
                            self.binary_input = self.binary_input.chars()
                                .filter(|c| c.is_whitespace() || *c == '0' || *c == '1')
                                .collect();
                        }
                    });
                    ui.small("Format: tylko 0 i 1, maksymalnie 96 bitów");
                    
                    // Pokaż liczbę bitów
                    let bit_count = self.binary_input.chars().filter(|c| *c == '0' || *c == '1').count();
                    if bit_count > 0 {
                        ui.small(format!("Wprowadzono: {} bitów", bit_count));
                    }
                }
                InputFormat::Hex => {
                    ui.horizontal(|ui| {
                        ui.label("📝 Sekwencja hex:");
                        let response = ui.add(egui::TextEdit::singleline(&mut self.hex_input)
                            .desired_width(400.0)
                            .hint_text("AA BB CC DD"));
                        
                        // Konwertuj na wielkie litery
                        if response.changed() {
                            self.hex_input = self.hex_input.to_uppercase();
                        }
                    });
                    ui.small("Format: AA BB CC DD (oddzielone spacjami, maks. 12 bajtów = 96 bitów)");
                    
                    // Pokaż liczbę bajtów
                    let hex_chars = self.hex_input.chars().filter(|c| c.is_ascii_hexdigit()).count();
                    if hex_chars > 0 && hex_chars % 2 == 0 {
                        ui.small(format!("Wprowadzono: {} bajtów = {} bitów", hex_chars / 2, hex_chars * 4));
                    }
                }
            }
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label("🔄 Liczba iteracji:");
                let response = ui.add(egui::TextEdit::singleline(&mut self.iterations_input)
                    .desired_width(150.0)
                    .hint_text("1000000"));
                
                // Filtruj tylko cyfry
                if response.changed() {
                    self.iterations_input = self.iterations_input.chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect();
                }
                
                ui.label("(1 do 1 000 000 000)");
            });
            
            ui.add_space(15.0);
                        
            let calc_button = egui::Button::new(if self.is_calculating { 
                "⏳ Obliczanie..." 
            } else { 
                "🚀 Oblicz CRC" 
            }).min_size(egui::vec2(120.0, 30.0));
            
            if ui.add_enabled(!self.is_calculating, calc_button).clicked() {
                self.calculate_crc();
            }
            
            if self.is_calculating {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Obliczanie CRC CAN...");
                });
            }
            
            ui.add_space(15.0);
            
            if !self.error_message.is_empty() {
                ui.group(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 100, 100), &self.error_message);
                });
                ui.add_space(10.0);
            }
            
            if let Some(result) = &self.result {
                ui.separator();
                ui.add_space(10.0);
                ui.heading("📊 Wyniki");
                ui.add_space(10.0);
                
                egui::Grid::new("results_grid")
                    .num_columns(2)
                    .spacing([20.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("🎯 CRC (hex):");
                        ui.code(format!("0x{}", result.crc_hex));
                        ui.end_row();
                        
                        ui.label("🔢 CRC (dziesiętnie):");
                        ui.code(format!("{}", result.crc_value));
                        ui.end_row();
                        
                        ui.label("🔢 CRC (binarnie):");
                        ui.code(format!("{:015b}", result.crc_value));
                        ui.end_row();
                        
                        ui.label("⏱️ Czas wykonania:");
                        ui.code(format!("{:.3} ms", result.duration_ms));
                        ui.end_row();
                        
                        if let Ok(iterations) = self.iterations_input.parse::<u64>() {
                            if iterations > 1 {
                                let avg_time = result.duration_ms / iterations as f64;
                                ui.label("⏱️ Średni czas na CRC:");
                                ui.code(format!("{:.6} ms ({:.3} µs)", avg_time, avg_time * 1000.0));
                                ui.end_row();
                                
                                let ops_per_sec = (iterations as f64 / result.duration_ms) * 1000.0;
                                ui.label("⚡ Wydajność:");
                                ui.code(format!("{} CRC/s", format_number(ops_per_sec as u64)));
                                ui.end_row();
                                
                                if iterations >= 100_000 {
                                    ui.label("🔥 Tryb:");
                                    ui.code("Przetwarzanie równoległe");
                                    ui.end_row();
                                }
                            }
                        }
                    });
            }
            
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            
            ui.heading("📋 Przykładowe dane");
            ui.add_space(10.0);
            
            ui.label("Przykłady binarne:");
            ui.horizontal(|ui| {
                if ui.button("10101010").clicked() {
                    self.binary_input = "10101010".to_string();
                    self.input_format = InputFormat::Binary;
                }
                if ui.button("11110000 11110000").clicked() {
                    self.binary_input = "11110000 11110000".to_string();
                    self.input_format = InputFormat::Binary;
                }
                if ui.button("10011001 10011001 10011001").clicked() {
                    self.binary_input = "10011001 10011001 10011001".to_string();
                    self.input_format = InputFormat::Binary;
                }
            });
            
            ui.add_space(5.0);
            ui.label("Przykłady hex:");
            ui.horizontal(|ui| {
                if ui.button("AA").clicked() {
                    self.hex_input = "AA".to_string();
                    self.input_format = InputFormat::Hex;
                }
                if ui.button("01 04 00 00").clicked() {
                    self.hex_input = "01 04 00 00".to_string();
                    self.input_format = InputFormat::Hex;
                }
                if ui.button("FF EE DD CC BB AA").clicked() {
                    self.hex_input = "FF EE DD CC BB AA".to_string();
                    self.input_format = InputFormat::Hex;
                }
            });
            
            ui.add_space(5.0);
            ui.label("Liczba iteracji:");
            ui.horizontal(|ui| {
                if ui.button("1 000").clicked() {
                    self.iterations_input = "1000".to_string();
                }
                if ui.button("100 000").clicked() {
                    self.iterations_input = "100000".to_string();
                }
                if ui.button("1 000 000").clicked() {
                    self.iterations_input = "1000000".to_string();
                }
                if ui.button("10 000 000").clicked() {
                    self.iterations_input = "10000000".to_string();
                }
            });
            
            ui.add_space(15.0);
            
            ui.separator();
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("💡");
                ui.label("CAN używa 15-bitowego CRC z wielomianem 0x4599");
            });
            ui.horizontal(|ui| {
                ui.label("⚡");
                ui.label("Automatyczne przetwarzanie równoległe dla >100k iteracji");
            });
            
            if let Some(calc_time) = self.last_calculation_time {
                ui.horizontal(|ui| {
                    ui.label("⏰");
                    ui.label(format!("Ostatnie obliczenie: {:.1}ms", calc_time));
                });
            }
        });
        
        if self.is_calculating {
            ctx.request_repaint();
        }
    }
}

impl CanCrcApp {
    fn calculate_crc(&mut self) {
        self.error_message.clear();
        self.is_calculating = true;
        
        let bits = match self.input_format {
            InputFormat::Binary => {
                match parse_binary_input(&self.binary_input) {
                    Ok(b) => b,
                    Err(e) => {
                        self.error_message = e;
                        self.is_calculating = false;
                        return;
                    }
                }
            }
            InputFormat::Hex => {
                match parse_hex_input(&self.hex_input) {
                    Ok(b) => b,
                    Err(e) => {
                        self.error_message = e;
                        self.is_calculating = false;
                        return;
                    }
                }
            }
        };
        
        if bits.is_empty() {
            self.error_message = "❌ Błąd: Proszę wprowadzić przynajmniej jeden bit danych.".to_string();
            self.is_calculating = false;
            return;
        }
        
        let iterations: u64 = match self.iterations_input.trim().parse() {
            Ok(num) => {
                if num == 0 {
                    self.error_message = "❌ Błąd: Liczba iteracji musi być większa od 0".to_string();
                    self.is_calculating = false;
                    return;
                }
                if num > 1_000_000_000 {
                    self.error_message = format!("❌ Błąd: Liczba iteracji za duża: {} (maks. 1 000 000 000)", format_number(num));
                    self.is_calculating = false;
                    return;
                }
                num
            }
            Err(_) => {
                if self.iterations_input.is_empty() {
                    self.error_message = "❌ Błąd: Proszę podać liczbę iteracji".to_string();
                } else {
                    self.error_message = "❌ Błąd: Nieprawidłowa liczba iteracji (użyj tylko cyfr)".to_string();
                }
                self.is_calculating = false;
                return;
            }
        };
        
        let start = Instant::now();
        let crc_val = compute_batch_crcs_optimized(&bits, iterations, false);
        let duration = start.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;
        
        self.result = Some(CrcResult::new(crc_val, duration_ms));
        self.last_calculation_time = Some(duration_ms);
        self.is_calculating = false;
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