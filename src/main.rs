#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

extern crate serial;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;
use serial::prelude::*;

use csv::Writer;

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
    data: Vec<[f64; 2]>,
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
        ctx.request_repaint_after(Duration::from_millis(16));
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

                let len = serial.read(&mut buf[..]).unwrap_or_default();
                //println!("Buf: {:?}", &buf[0..len]);

                let s = std::str::from_utf8(&buf[0..len]).unwrap_or_else(|e| {
                    println!("Invalid UTF-8 sequence: {}", e);
                    ""
                });

                if s != "" {
                    let mut data: Vec<&str> = s.split("\n").collect();


                    for (x, datum) in data.iter().enumerate() {
                        if x == 0 || x >= data.len() - 1 {
                            continue;
                        }
                        let mut values: Vec<&str> = datum.split(",").collect();
                        if values.len() != 2 {
                            continue;
                        }
                        println!("Values: {:?}", values);
                        //self.data.push([vaLues.nth(0).unwrap().parse().unwrap(), values.nth(1).unwrap().parse().unwrap()]);
                        let pos = values[0].trim().parse();
                        let val = values[1].trim().parse();

                        if (pos.is_ok() && val.is_ok()) {
                            self.data.push([pos.unwrap(), val.unwrap()]);
                        }

                    }
                }
            }


            if ui.button("Save CSV").clicked() {
                let mut wtr = Writer::from_path("C:\\Users\\a0488091\\Documents\\data.csv").unwrap();

                for data in &self.data {
                    wtr.write_record(&[data[0].to_string(), data[1].to_string()]);
                }

            }

            let my_plot = Plot::new("My Plot").legend(Legend::default());

            // let's create a dummy line in the plot
            //let graph: Vec<[f64; 2]> = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0]];
            let inner = my_plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new(PlotPoints::from(self.data.clone())).name("curve"));
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
        //ctx.request_repaint();
    }
}
