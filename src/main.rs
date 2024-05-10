use std::{error::Error, time::Duration};

use chrono::{DateTime, Local, Timelike};
use date_ext::DateExt;
use eframe::egui::{self, RichText, WindowLevel};
use google::GoogleCalendar;
use log::warn;

mod date_ext;
mod google;
mod oauth_browser_delegate;

static GOOGLE_REFRESH_INTERVAL: i64 = 60 * 60; // 1 hour
static MAX_NUM_EVENTS: usize = 10;
//
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    #[cfg(not(debug_assertions))]
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_fullscreen(true)
            .with_decorations(false),
        ..Default::default()
    };

    #[cfg(debug_assertions)]
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_inner_size([800.0, 400.0])
            .with_decorations(false),
        ..Default::default()
    };

    eframe::run_native(
        "Deskclock",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )?;
    Ok(())
}

struct MyApp {
    current_time: DateTime<Local>,
    last_event_update: DateTime<Local>,
    events: Vec<google::CalendarEvent>,
    calendar: GoogleCalendar,
    current_filter: EventFilter,
}

#[derive(PartialEq)]
enum EventFilter {
    Today,
    Later,
}

impl Default for MyApp {
    fn default() -> Self {
        let current_time = chrono::offset::Local::now();
        let mut calendar =
            google::GoogleCalendar::try_new().expect("Failed to create GoogleCalendar");
        calendar.authenticate();
        let events = calendar.get_events();

        MyApp {
            current_time,
            last_event_update: current_time,
            calendar,
            events,
            current_filter: EventFilter::Today,
        }
    }
}

impl MyApp {
    fn update_time(&mut self) {
        self.current_time = chrono::offset::Local::now();
    }

    fn update_events(&mut self) {
        // This will block the UI for a while, but it's fine for now
        // It's just a clock
        let now = chrono::offset::Local::now();
        if now
            .signed_duration_since(self.last_event_update)
            .num_seconds()
            > GOOGLE_REFRESH_INTERVAL
        {
            warn!("Updating events");
            self.events = self.calendar.get_events();
            self.last_event_update = now;
        }
    }

    fn render_time(&self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(format!(
                "{:02}:{:02}",
                self.current_time.hour(),
                self.current_time.minute(),
            ))
            .monospace()
            .size(50.0),
        );
    }

    fn render_events(&self, ui: &mut egui::Ui) {
        let mut events: Vec<&google::CalendarEvent> = self
            .events
            .iter()
            .filter(|event| match self.current_filter {
                EventFilter::Today => event.start.is_today(),
                EventFilter::Later => event.start.is_after_today(),
            })
            .collect();
        if events.len() > 5 {
            events.retain(|&event| {
                event
                    .end
                    .is_before(self.current_time - chrono::Duration::hours(1))
            });
        }
        let mut last_date = self.current_time;
        for event in events.iter().take(MAX_NUM_EVENTS) {
            ui.horizontal(|ui| {
                if EventFilter::Later == self.current_filter {
                    if !last_date.is_on_same_day_as(event.start) {
                        ui.label(
                            RichText::new(format!("{}", event.start.format("%d-%m"))).strong(),
                        );
                    } else {
                        ui.add_space(40.0);
                    }
                    last_date = event.start;
                }

                let text = self.format_event(event);

                ui.label(text);
            });
        }
    }

    fn format_event(&self, event: &google::CalendarEvent) -> RichText {
        let mut text = RichText::new(format!(
            "{} - {}:   {}",
            event.start.format("%H:%M"),
            event.end.format("%H:%M"),
            event.summary
        ));

        if event.end < self.current_time {
            text = text.strikethrough();
            return text;
        }

        if event
            .start
            .is_after(self.current_time - chrono::Duration::minutes(10))
        {
            text = text.color(egui::Color32::LIGHT_RED);
        }

        // Already started
        if event.start.is_after(self.current_time) {
            text = text.color(egui::Color32::LIGHT_GREEN);
        }

        if event == self.events.first().expect("No events") {
            text = text.strong();
        }

        text
    }

    fn render_next_event(&self, ui: &mut egui::Ui) {
        if let Some(event) = self.events.iter().find(|event| {
            event.end.is_on_same_day_as(self.current_time)
                && (event.end.is_before(self.current_time)
                    || event.start.is_before(self.current_time))
        }) {
            let text = self.format_event(event);

            ui.label(text);
        }
    }

    fn render_event_label(&mut self, ui: &mut egui::Ui, filter: EventFilter) {
        use EventFilter::*;

        let enabled = self.current_filter == filter;
        let text = RichText::new(match filter {
            Today => "today",
            Later => "later",
        })
        .size(20.0);
        if enabled {
            ui.label(text.strong());
        } else if ui.label(text).clicked() {
            self.current_filter = filter;
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_time();
        self.update_events();
        let my_frame = egui::containers::Frame::default().fill(egui::Color32::BLACK);
        egui::CentralPanel::default()
            .frame(my_frame)
            .show(ctx, |ui| {
                if ui.small_button("close").clicked() {
                    std::process::exit(0);
                };
                ui.allocate_ui(egui::Vec2::new(700.0, 400.0), |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.allocate_ui(egui::Vec2::new(250.0, 300.0), |ui| {
                            ui.vertical_centered_justified(|ui| {
                                self.render_time(ui);
                                self.render_next_event(ui);
                            });
                        });
                        ui.allocate_ui(egui::Vec2::new(275.0, 300.0), |ui| {
                            ui.horizontal_centered(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        self.render_event_label(ui, EventFilter::Today);
                                        ui.add_space(10.0);
                                        self.render_event_label(ui, EventFilter::Later);
                                    });
                                    ui.separator();
                                    ui.add_space(5.0);
                                    self.render_events(ui);
                                });
                            });
                        });
                    });
                });
            });

        ctx.request_repaint_after(Duration::new(60, 0));
    }
}
