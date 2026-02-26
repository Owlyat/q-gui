mod audio;
mod console;
mod dmx_output;
mod dmx_types;
mod osc;
mod ui;

use eframe::NativeOptions;
use egui::Vec2;
use rosc::OscPacket;
use std::sync::Mutex;

use ui::{ConsoleState, Tab, show_audio_tab, show_dmx_console, show_liveshow_tab};

use crate::{dmx_output::mix_executor_outputs, ui::show_midi_osc_tab};

pub struct AppState {
    state: Mutex<ConsoleState>,
}

impl AppState {
    fn new() -> Self {
        Self {
            state: Mutex::new(ConsoleState::default()),
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut state = self.state.lock().unwrap();
        if let Some(osc_manager) = &mut state.osc_manager.1 {
            crate::osc::handle_osc(osc_manager.get_osc(), &mut state);
        }
        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.selected_tab, Tab::DmxConsole, "DMX Console");
                ui.selectable_value(&mut state.selected_tab, Tab::Audio, "Audio");
                ui.selectable_value(&mut state.selected_tab, Tab::MidiOsc, "MIDI/OSC");
                ui.selectable_value(&mut state.selected_tab, Tab::Show, "Show");
                if ui.input(|i| i.key_pressed(egui::Key::Num1) && i.modifiers.command) {
                    state.selected_tab = Tab::DmxConsole;
                }
                if ui.input(|i| i.key_pressed(egui::Key::Num2) && i.modifiers.command) {
                    state.selected_tab = Tab::Audio;
                }
                if ui.input(|i| i.key_pressed(egui::Key::Num3) && i.modifiers.command) {
                    state.selected_tab = Tab::MidiOsc;
                }
                if ui.input(|i| i.key_pressed(egui::Key::Num4) && i.modifiers.command) {
                    state.selected_tab = Tab::Show;
                }
                if ui.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.ctrl) {
                    state.selected_tab = [Tab::DmxConsole, Tab::Audio, Tab::MidiOsc, Tab::Show]
                        [(state.selected_tab.clone() as usize + 1) % 4]
                        .clone();
                }
            });
        });

        match state.selected_tab {
            Tab::DmxConsole => show_dmx_console(ctx, &mut state),
            Tab::Audio => show_audio_tab(ctx, &mut state),
            Tab::MidiOsc => show_midi_osc_tab(ctx, &mut state),
            Tab::Show => show_liveshow_tab(ctx, &mut state),
        }
        // Send DMX Values
        mix_executor_outputs(&mut state);

        ctx.request_repaint();
    }
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Egui Live")
            .with_min_inner_size(Vec2::new(1280.0, 800.0)),
        ..Default::default()
    };

    eframe::run_native(
        "DMX Console",
        options,
        Box::new(|_cc| Ok(Box::new(AppState::new()))),
    )
}
