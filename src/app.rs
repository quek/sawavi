use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::audio_process::AudioProcess;
use crate::device::Device;
use crate::plugin::Plugin;
use eframe::egui;

pub fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    let result = eframe::run_native(
        "Sawavi",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    );

    result
}

struct MyApp {
    device: Option<Device>,
    plugin: Option<Plugin>,
    audio_process: Arc<Mutex<AudioProcess>>,
}

pub enum Msg {
    Process,
    DidProcess(Vec<Vec<f32>>),
}

impl Default for MyApp {
    fn default() -> Self {
        let device = Some(Device::open_default().unwrap());
        let audio_process = AudioProcess::new();

        Self {
            device,
            plugin: None,
            audio_process: Arc::new(Mutex::new(audio_process)),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");

            // ui.image(egui::include_image!(
            //     "../../../crates/egui/assets/ferris.png"
            // ));

            if ui.button("device open").clicked() {
                self.device = Some(Device::open_default().unwrap());
            }
            if ui.button("device start").clicked() {
                self.device
                    .as_mut()
                    .unwrap()
                    .start(self.audio_process.clone())
                    .unwrap();
            }
            if ui.button("device stop").clicked() {
                self.device.as_mut().unwrap().stop().unwrap();
            }

            ui.separator();

            if ui.button("Surge XT load").clicked() {
                let mut plugin = Plugin::new();
                let path =
                    Path::new("c:/Program Files/Common Files/CLAP/Surge Synth Team/Surge XT.clap");
                plugin.load(path);
                let _ = plugin.gui_open();
                self.plugin = Some(plugin);
            }
            if ui.button("Surge XT edit").clicked() {
                self.plugin.as_mut().map(|x| x.gui_open());
            }
            if ui.button("Surge XT close").clicked() {
                self.plugin.as_mut().map(|x| x.gui_close());
            }
            if ui.button("Surge XT start").clicked() {
                self.plugin.as_mut().map(|x| x.start());
            }
            if ui.button("Surge XT stop").clicked() {
                self.plugin.as_mut().map(|x| x.stop());
            }
        });
    }
}
