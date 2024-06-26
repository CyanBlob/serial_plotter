#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

extern crate serial;
use std::io::prelude::*;
use std::time::Duration;
use serial::prelude::*;

use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use serial::SystemPort;

fn main() -> eframe::Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([350.0, 200.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App with a plot",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

#[derive(Default)]
struct MyApp {
    port: String,
    serial: Option<SystemPort>,
}

const SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate: serial::Baud115200,
    char_size: serial::Bits8,
    parity: serial::ParityNone,
    stop_bits: serial::Stop1,
    flow_control: serial::FlowNone,
};

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut plot_rect = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Serial port");
            ui.text_edit_singleline(&mut self.port);

            if ui.button("Connect").clicked() {
                self.serial = match serial::open(&self.port) {
                    Ok(mut TTYPort) => {
                        TTYPort.configure(&SETTINGS).expect("Failed to configure serial port");
                        TTYPort.set_timeout(Duration::from_secs(1)).expect("Failed to configure serial timeout");
                        Some(TTYPort)
                    }
                    Err(e) => {
                        println!("Failed to open serial port: {:?}", e);
                        None
                    }
                };
            }

            if let Some(serial) = &mut self.serial {
                let mut buf: Vec<u8> = (0..255).collect();

                println!("reading bytes");
                serial.read(&mut buf[..]).expect("Failed to read data");
                println!("Buf: {:?}", buf);
            }


            if ui.button("Save Plot").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
            }

            let my_plot = Plot::new("My Plot").legend(Legend::default());

            // let's create a dummy line in the plot
            let graph: Vec<[f64; 2]> = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0]];
            let inner = my_plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new(PlotPoints::from(graph)).name("curve"));
            });
            // Remember the position of the plot
            plot_rect = Some(inner.response.rect);
        });

        // Check for returned screenshot:
        let screenshot = ctx.input(|i| {
            for event in &i.raw.events {
                if let egui::Event::Screenshot { image, .. } = event {
                    return Some(image.clone());
                }
            }
            None
        });

        if let (Some(screenshot), Some(plot_location)) = (screenshot, plot_rect) {
            if let Some(mut path) = rfd::FileDialog::new().save_file() {
                path.set_extension("png");

                // for a full size application, we should put this in a different thread,
                // so that the GUI doesn't lag during saving

                let pixels_per_point = ctx.pixels_per_point();
                let plot = screenshot.region(&plot_location, Some(pixels_per_point));
                // save the plot to png
                image::save_buffer(
                    &path,
                    plot.as_raw(),
                    plot.width() as u32,
                    plot.height() as u32,
                    image::ColorType::Rgba8,
                )
                    .unwrap();
                eprintln!("Image saved to {path:?}.");
            }
        }
    }
}
