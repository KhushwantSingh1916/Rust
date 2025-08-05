use eframe::egui;
use rand::Rng;
use std::cmp::Ordering;

struct GuessingGame {
    secret_number: u32,
    guess: String,
    message: String,
    color: egui::Color32,
}

impl Default for GuessingGame {
    fn default() -> Self {
        Self {
            secret_number: rand::thread_rng().gen_range(1..=100),
            guess: String::new(),
            message: String::from("Guess a number between 1 and 100"),
            color: egui::Color32::WHITE,
        }
    }
}

impl eframe::App for GuessingGame {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎯 Guess the Number!");
            ui.label(egui::RichText::new(&self.message).color(self.color));

            ui.horizontal(|ui| {
                ui.label("Your Guess:");
                ui.text_edit_singleline(&mut self.guess);
            });

            if ui.button("Submit").clicked() {
                if let Ok(num) = self.guess.trim().parse::<u32>() {
                    match num.cmp(&self.secret_number) {
                        Ordering::Less => {
                            self.message = "Too Small!".to_string();
                            self.color = egui::Color32::RED;
                        }
                        Ordering::Greater => {
                            self.message = "Too Big!".to_string();
                            self.color = egui::Color32::RED;
                        }
                        Ordering::Equal => {
                            self.message = "🎉 You Win! Starting a new game...".to_string();
                            self.color = egui::Color32::GREEN;
                            self.secret_number = rand::thread_rng().gen_range(1..=100);
                            self.guess.clear();
                        }
                    }
                } else {
                    self.message = "❌ Please enter a valid number".to_string();
                    self.color = egui::Color32::YELLOW;
                }
            }
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Guessing Game",
        options,
        Box::new(|_cc| Ok(Box::new(GuessingGame::default()))),
    )
    .expect("Failed to start app");
}
