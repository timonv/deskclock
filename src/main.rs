use std::{error::Error, time::Duration};

use chrono::Timelike;
use eframe::egui::{self, RichText, WindowLevel};
use google::GoogleCalendar;

mod google;
mod oauth_browser_delegate;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut calendar = google::GoogleCalendar::try_new()?;
    calendar.authenticate();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_fullscreen(true)
            .with_decorations(false),
        ..Default::default()
    };

    eframe::run_native(
        "Deskclock",
        options,
        Box::new(|_cc| Box::new(MyApp::default().with_updated_events(calendar))),
    )?;
    Ok(())
}

struct MyApp {
    hour: u32,
    minute: u32,
    second: u32,
    events: Vec<google::CalendarEvent>,
}

impl Default for MyApp {
    fn default() -> Self {
        let time = chrono::offset::Local::now();

        MyApp {
            hour: time.hour(),
            minute: time.minute(),
            second: time.second(),
            events: Vec::new(),
        }
    }
}

impl MyApp {
    pub fn with_updated_events(mut self, calendar: GoogleCalendar) -> Self {
        let events = calendar.get_events();

        self.events = events;
        self
    }

    fn update_time(&mut self) {
        let time = chrono::offset::Local::now();
        self.hour = time.hour();
        self.minute = time.minute();
        self.second = time.second();
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_time();

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.small_button("close").clicked() {
                std::process::exit(0);
            };
            ui.vertical_centered_justified(|ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{:02}:{:02}:{:02}",
                            self.hour, self.minute, self.second
                        ))
                        .monospace()
                        .size(50.0),
                    );
                    ui.vertical(|ui| {
                        for event in &self.events {
                            ui.label(format!(
                                "{} - {}:   {}",
                                event.start.format("%d/%m %H:%M"),
                                event.end.format("%H:%M"),
                                event.summary
                            ));
                        }
                    });
                });
            });

            // ui.heading("My egui Application");
            // ui.horizontal(|ui| {
            //     let name_label = ui.label("Your name: ");
            //     ui.text_edit_singleline(&mut self.name)
            //         .labelled_by(name_label.id);
            // });
            // ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            // if ui.button("Increment").clicked() {
            //     self.age += 1;
            // }
            //
            // ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });

        ctx.request_repaint_after(Duration::new(1, 0));
    }
}
