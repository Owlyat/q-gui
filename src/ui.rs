use crate::console::{ConsoleCommand, execute_console_command};
use crate::dmx_types::{
    AudioAction, AudioTrack, ChannelType, Cue, DMX_CHANNELS, DMXBufferValue, Executor, Fixture,
    FixtureGroup, FixtureTemplateLibrary,
};
use egui::epaint::ColorMode;
use egui::{Color32, DragValue, Key, RichText, ScrollArea, TextEdit, Vec2};
use open_dmx::check_valid_channel;
#[derive(PartialEq, Default, Clone)]
pub enum Tab {
    #[default]
    DmxConsole = 0,
    Audio = 1,
    MidiOsc = 2,
    Show = 3,
}

#[derive(PartialEq, Default)]
pub enum DmxSubTab {
    #[default]
    Executor,
    Fixtures,
}

#[derive(PartialEq, Default)]
pub enum FixturesTab {
    #[default]
    Creation,
    Grouping,
    Editing,
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum EditingState {
    #[default]
    None,
    Store,
    Edit,
    Delete,
    Copy,
    Move,
    Label,
    OSCLearn,
}

impl EditingState {
    pub fn reset_if_set(&mut self, to: Self) {
        if self.if_any() {
            *self = Self::None
        } else {
            *self = to;
        }
    }
    pub fn set(&mut self, state: Self) {
        *self = state;
    }
    pub fn reset(&mut self) {
        *self = Self::None;
    }
    pub fn if_any(&self) -> bool {
        *self != Self::None
    }
    pub fn is_edit(&self) -> bool {
        *self == Self::Edit
    }
    pub fn is_osc_learn(&self) -> bool {
        *self == Self::OSCLearn
    }
    pub fn is_none(&self) -> bool {
        *self == Self::None
    }
    pub fn is_delete(&self) -> bool {
        *self == Self::Delete
    }
    pub fn is_label(&self) -> bool {
        *self == Self::Label
    }
    pub fn is_copy(&self) -> bool {
        *self == Self::Copy
    }
    pub fn is_move(&self) -> bool {
        *self == Self::Move
    }
    pub fn is_store(&self) -> bool {
        *self == Self::Store
    }
}

/// Main application state containing all DMX console data
pub struct ConsoleState {
    pub edit_state: EditingState,
    /// Current text in the command input field
    pub command_input: String,
    /// Error message to display from last command (if any)
    pub command_error: Option<String>,
    /// History of successfully executed commands
    pub command_history: Vec<ConsoleCommand>,
    /// Final mixed DMX output channels (512 channels) sent to hardware
    pub channels: Vec<u8>,
    /// Buffer containing pending DMX values before storing to a cue
    pub buffer: Vec<DMXBufferValue>,
    /// Index of cue being labeled (if any)
    pub labeling_cue: Option<usize>,
    /// Temporary name buffer for labeling
    pub label_buffer: String,
    /// Index of executor currently being edited (if any)
    pub editing_executor: Option<usize>,
    /// Index of executor pending delete confirmation (if any)
    pub delete_confirm_executor: Option<usize>,
    /// Whether the buffer popup window is visible
    pub show_buffer: bool,
    /// List of executors (playback faders with cue lists)
    pub executors: Vec<Executor>,
    /// Currently selected main tab
    pub selected_tab: Tab,
    /// Currently selected sub-tab within DMX Console
    pub dmx_sub_tab: DmxSubTab,
    /// Currently selected tab within Fixtures menu
    pub fixtures_tab: FixturesTab,
    /// List of defined fixtures
    pub fixtures: Vec<Fixture>,
    /// List of fixture groups
    pub fixture_groups: Vec<FixtureGroup>,
    /// Fixture template library
    pub template_library: FixtureTemplateLibrary,
    /// Selected template ID for new fixture
    pub selected_template_id: Option<u32>,
    /// Selected mode index for new fixture
    pub selected_mode_index: usize,
    /// IDs of fixtures currently selected for grouping
    pub selected_fixture_ids: Vec<usize>,
    /// ID of currently selected fixture group (if any)
    pub selected_group_id: Option<u32>,
    /// Input field for new fixture name
    pub new_fixture_name: String,
    /// Input field for new fixture start channel
    pub new_fixture_start_channel: String,
    /// Input field for new group name
    pub new_group_name: String,
    /// Grid index for new group
    pub new_group_grid_index: Option<usize>,
    /// Error message for fixture operations
    pub fixture_error: Option<String>,
    /// Master dimmer fader (0.0 to 1.0)
    pub master_dimmer: f32,
    /// Audio tracks
    pub audio_tracks: Vec<AudioTrack>,
    /// Master volume for audio (0.0 to 1.0)
    pub master_volume: f32,
    /// Currently selected audio track ID
    pub selected_audio_track_id: Option<u32>,
    /// Audio index for playback (0-based)
    pub audio_index: usize,
    /// Audio engine for playback
    pub audio_engine: Option<crate::audio::AudioEngine>,
    /// Serial connection to Open DMX hardware
    pub dmx_serial: Option<open_dmx::DMXSerial>,
    /// Whether DMX hardware is currently connected and responding
    pub dmx_connected: bool,
    /// Last error message from DMX serial operations
    pub dmx_serial_error: String,
    /// The OSC Manager
    pub osc_manager: (String, Option<crate::osc::OSCManager>),
    /// Binding osc address to application actions
    pub osc_address_manager: crate::osc::OSCNaming,
}

impl Default for ConsoleState {
    fn default() -> Self {
        let port = if cfg!(target_os = "windows") {
            "COM3"
        } else {
            "/dev/ttyUSB0"
        };
        Self {
            command_input: Default::default(),
            command_error: Default::default(),
            command_history: Default::default(),
            channels: vec![0; DMX_CHANNELS],
            buffer: Default::default(),
            labeling_cue: Default::default(),
            label_buffer: Default::default(),
            editing_executor: Default::default(),
            delete_confirm_executor: Default::default(),
            show_buffer: Default::default(),
            executors: (0..10).map(Executor::new).collect(),
            selected_tab: Default::default(),
            dmx_sub_tab: Default::default(),
            fixtures_tab: Default::default(),
            fixtures: Default::default(),
            fixture_groups: Default::default(),
            template_library: FixtureTemplateLibrary::new(),
            selected_template_id: Default::default(),
            selected_mode_index: Default::default(),
            selected_fixture_ids: Default::default(),
            selected_group_id: None,
            new_fixture_name: Default::default(),
            new_fixture_start_channel: Default::default(),
            new_group_name: Default::default(),
            new_group_grid_index: None,
            fixture_error: Default::default(),
            master_dimmer: 1.0,
            audio_tracks: Default::default(),
            master_volume: 1.0,
            selected_audio_track_id: Default::default(),
            audio_index: Default::default(),
            audio_engine: crate::audio::AudioEngine::new().ok(),
            dmx_serial: open_dmx::DMXSerial::open(port).ok(),
            dmx_connected: Default::default(),
            dmx_serial_error: Default::default(),
            edit_state: Default::default(),
            osc_manager: (Default::default(), Default::default()),
            osc_address_manager: Default::default(),
        }
    }
}

pub fn show_executor_panel_content(ui: &mut egui::Ui, state: &mut ConsoleState) {
    ui.heading("Executors");
    ui.separator();

    let executor_count = state.executors.len();
    let fader_width = 60.0;
    let spacing = 10.0;
    let available_width = ui.available_width();
    let executors_per_row =
        ((available_width + spacing) / (fader_width + spacing)).floor() as usize;
    let executors_per_row = executors_per_row.max(1);

    for row in 0..((executor_count + executors_per_row - 1) / executors_per_row) {
        ui.horizontal(|ui| {
            for col in 0..executors_per_row {
                let exec_idx = row * executors_per_row + col;
                if exec_idx >= executor_count {
                    break;
                }

                let exec = &mut state.executors[exec_idx];
                let has_cues = !exec.cue_list.is_empty();

                ui.vertical(|ui| {
                    ui.label(RichText::new(format!("Exec {}", exec_idx + 1)).strong());

                    let _slider_response = ui.add_enabled(
                        has_cues,
                        egui::Slider::new(&mut exec.fader_level, 0.0..=1.0)
                            .vertical()
                            .text(""),
                    );

                    if !has_cues {
                        ui.label(RichText::new("(No cues)").weak().small());
                    }

                    if ui
                        .add_enabled(
                            has_cues,
                            DragValue::new(&mut exec.fader_level)
                                .range(0.0..=1.0)
                                .speed(0.01)
                                .suffix("%"),
                        )
                        .clicked()
                    {
                        exec.current_cue = if let Some(id) = exec.current_cue {
                            Some(id)
                        } else {
                            Some(1)
                        };
                    }

                    let button_size = Vec2::new(fader_width, 30.0);
                    let go_button = egui::Button::new("GO").fill(Color32::DARK_GREEN);
                    if ui.add_sized(button_size, go_button).clicked() {
                        if state.edit_state.is_store() {
                            let mut levels = vec![0; DMX_CHANNELS];
                            for val in &state.buffer {
                                if check_valid_channel(val.chan).is_ok() {
                                    levels[val.chan] = val.dmx;
                                }
                            }
                            let mut new_cue = Cue::new(exec.cue_list.len().saturating_add(1));
                            new_cue.levels = levels;
                            exec.cue_list.push(new_cue);
                            state.edit_state.reset();
                        } else if state.edit_state.is_edit() {
                            state.editing_executor = Some(exec_idx);
                        } else if state.edit_state.is_delete() {
                            state.delete_confirm_executor = Some(exec_idx);
                        } else if state.edit_state.is_label() {
                            state.editing_executor = Some(exec_idx);
                        } else {
                            exec.go();
                        }
                    }

                    let go_back_button = egui::Button::new("BACK").fill(Color32::DARK_BLUE);
                    if ui.add_sized(button_size, go_back_button).clicked() {
                        if state.edit_state.is_delete() {
                            state.delete_confirm_executor = Some(exec_idx);
                        } else {
                            exec.go_back();
                        }
                    }

                    if exec.fader_level > 0.0 && exec.current_cue_index < exec.cue_list.len() {
                        let current_cue = &exec.cue_list[exec.current_cue_index];
                        ui.label(
                            RichText::new(format!("Cue {} - {}", current_cue.id, current_cue.name))
                                .small()
                                .color(Color32::GREEN),
                        );

                        if exec.current_cue_index > 0 {
                            let prev_cue = &exec.cue_list[exec.current_cue_index - 1];
                            ui.label(
                                RichText::new(format!("Prev: {} - {}", prev_cue.id, prev_cue.name))
                                    .small()
                                    .weak(),
                            );
                        }

                        if exec.current_cue_index + 1 < exec.cue_list.len() {
                            let next_cue = &exec.cue_list[exec.current_cue_index + 1];
                            ui.label(
                                RichText::new(format!("Next: {} - {}", next_cue.id, next_cue.name))
                                    .small()
                                    .weak(),
                            );
                        }
                    } else if has_cues {
                        let current_cue = &exec.cue_list[exec.current_cue_index];
                        ui.label(
                            RichText::new(format!(
                                "Off: {} - {}",
                                current_cue.id, current_cue.name
                            ))
                            .small()
                            .color(Color32::GRAY),
                        );
                    }
                });

                if col < executors_per_row - 1 && exec_idx < executor_count - 1 {
                    ui.add_space(spacing);
                }
            }
        });
        ui.add_space(spacing);
    }
}

pub fn show_dmx_console<'a>(ctx: &egui::Context, state: &mut ConsoleState) {
    if let Some(exec_idx) = state.editing_executor {
        show_edit_executor_panel(ctx, state, exec_idx);
    } else if let Some(exec_idx) = &state.delete_confirm_executor {
        show_confirm_prompt_panel(ctx, state, *exec_idx);
    }
    show_sidebar_master_fader(ctx, state);

    egui::CentralPanel::default().show(ctx, |ui| {
        show_dmx_status(state, ui);
        ui.separator();
        ui.horizontal(|ui| {
            ui.selectable_value(&mut state.dmx_sub_tab, DmxSubTab::Executor, "Executor");
            ui.selectable_value(&mut state.dmx_sub_tab, DmxSubTab::Fixtures, "Fixtures");
        });
        ui.separator();
        show_command_button(state, ui);
        ui.separator();

        show_buffer_list(ctx, state);

        ui.separator();

        if let Some(ref error) = state.command_error {
            ui.label(RichText::new(error).color(Color32::RED));
            ui.separator();
        }

        ui.heading("Console Command");
        ui.separator();
        ui.label(RichText::new("Usage: chan [1-512] at [0-255]").small());

        show_console_input(state, ui);

        ui.separator();
        ui.heading("Console Buttons");
        ui.separator();

        show_command_palette_button(state, ui);

        show_command_history(state, ui);

        egui::SidePanel::right("executor_panel")
            .min_width(400.0)
            .max_width(500.0)
            .show(ctx, |ui| match state.dmx_sub_tab {
                DmxSubTab::Executor => {
                    show_executor_panel_content(ui, state);
                }
                DmxSubTab::Fixtures => {
                    show_fixtures_tab_content(ui, state);
                }
            });
    });
}

fn show_command_history(state: &mut ConsoleState, ui: &mut egui::Ui) {
    if !state.command_history.is_empty() {
        ui.separator();
        ui.heading("Command History");
        ScrollArea::vertical()
            .id_salt("command_history")
            .max_height(150.0)
            .show(ui, |ui| {
                for cmd in state.command_history.iter().rev().take(10) {
                    ui.label(RichText::new(format!("> {}", cmd)).monospace());
                }
            });
    }
}

fn show_command_palette_button(state: &mut ConsoleState, ui: &mut egui::Ui) {
    let button_size = Vec2::new(50.0, 35.0);
    ui.horizontal(|ui| {
        if ui
            .add_sized(
                button_size,
                egui::Button::new(RichText::new("Chan").color(Color32::GREEN)),
            )
            .clicked()
        {
            state.command_input.push_str("Chan ");
        }
        if ui
            .add_sized(
                button_size,
                egui::Button::new(RichText::new("Fixture").color(Color32::ORANGE)),
            )
            .clicked()
        {
            state.command_input.push_str("Fix ");
        }
        if ui.add_sized(button_size, egui::Button::new("0")).clicked() {
            state.command_input.push_str("0");
        }
        if ui
            .add_sized(
                button_size,
                egui::Button::new(RichText::new("B/O").color(Color32::RED)),
            )
            .clicked()
        {
            state.command_input = "b/o".to_string();
            execute_console_command(state);
            state.command_input.clear();
        }
    });

    ui.horizontal(|ui| {
        for num in &["1", "2", "3"] {
            if ui.add_sized(button_size, egui::Button::new(*num)).clicked() {
                state.command_input.push_str(num);
            }
        }
    });
    ui.horizontal(|ui| {
        for num in &["4", "5", "6"] {
            if ui.add_sized(button_size, egui::Button::new(*num)).clicked() {
                state.command_input.push_str(num);
            }
        }
    });
    ui.horizontal(|ui| {
        for num in &["7", "8", "9"] {
            if ui.add_sized(button_size, egui::Button::new(*num)).clicked() {
                state.command_input.push_str(num);
            }
        }
    });
    ui.horizontal(|ui| {
        if ui
            .add_sized(
                button_size,
                egui::Button::new(RichText::new("At").color(Color32::YELLOW)),
            )
            .clicked()
        {
            let input = state.command_input.trim_end();
            if input.ends_with("at") || input.ends_with("at ") {
                if let Some(pos) = input.rfind("at") {
                    let new_input = format!("{} at 255", &input[..pos]);
                    state.command_input = new_input;
                    execute_console_command(state);
                    state.command_input.clear();
                }
            } else {
                state.command_input.push_str(" at ");
            }
        }
        if ui
            .add_sized(
                button_size,
                egui::Button::new(RichText::new(".").color(Color32::BLUE)),
            )
            .clicked()
        {
            if state.command_input.trim_end().ends_with("at") {
                state.command_input = format!("{} 0", state.command_input.trim_end());
            }
            state.command_input.push_str(" at 0");
            execute_console_command(state);
            state.command_input.clear();
        }
        if ui
            .add_sized(button_size, egui::Button::new("please"))
            .clicked()
        {
            if !state.command_input.is_empty() {
                execute_console_command(state);
                state.command_input.clear();
            }
        }
    });
}

fn show_console_input(state: &mut ConsoleState, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(">");
        let response = ui.add(TextEdit::singleline(&mut state.command_input).desired_width(300.0));
        if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
            if !state.command_input.is_empty() {
                execute_console_command(state);
                state.command_input.clear();
            }
        }
        if ui.button("Send").clicked() {
            if !state.command_input.is_empty() {
                execute_console_command(state);
                state.command_input.clear();
            }
        }
    });
}

fn show_buffer_list(ctx: &egui::Context, state: &mut ConsoleState) {
    if state.show_buffer {
        egui::Window::new("Buffer")
            .collapsible(true)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Buffer Values");
                ui.separator();

                ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    if state.buffer.is_empty() {
                        ui.label(RichText::new("(empty)").weak());
                    } else {
                        for val in &state.buffer {
                            ui.label(
                                RichText::new(format!("Ch {}: {}", val.chan, val.dmx)).monospace(),
                            );
                        }
                    }
                });
            });
    }
}

fn show_dmx_status(state: &mut ConsoleState, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("DMX Status:");
        if state.dmx_connected {
            ui.label(RichText::new("Connected").color(Color32::GREEN).strong());
        } else if state.dmx_serial.is_some() {
            ui.label(RichText::new("Disconnected").color(Color32::RED).strong());
            if !state.dmx_serial_error.is_empty() {
                ui.label(RichText::new(&state.dmx_serial_error).color(Color32::ORANGE));
            }
        } else {
            ui.label(
                RichText::new("Not Initialized")
                    .color(Color32::YELLOW)
                    .weak(),
            );
            if !state.dmx_serial_error.is_empty() {
                ui.label(RichText::new(&state.dmx_serial_error).color(Color32::ORANGE));
            }
        }
    });
}

fn show_sidebar_master_fader(ctx: &egui::Context, state: &mut ConsoleState) {
    egui::SidePanel::left("master_panel")
        .resizable(true)
        .min_width(40.0)
        .max_width(80.0)
        .show(ctx, |ui| {
            ui.heading("Master");
            ui.separator();
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.label(format!("{}%", (state.master_dimmer * 100.0) as u32));
                let available_height = ui.available_height();
                ui.add_sized(
                    egui::vec2(30.0, available_height),
                    egui::Slider::new(&mut state.master_dimmer, 0.0..=1.0).vertical(),
                );
            });
        });
}

fn show_command_button(state: &mut ConsoleState, ui: &mut egui::Ui) {
    let active_size = Vec2::new(120.0, 35.0);
    let normal_size = Vec2::new(80.0, 35.0);
    let clear_button =
        egui::Button::new("Clear").fill(if !state.buffer.is_empty() | state.edit_state.if_any() {
            Color32::DARK_RED
        } else {
            Color32::GRAY
        });

    let store_button = egui::Button::new("Store").fill(Color32::from_rgb(0, 100, 200));
    let store_active = egui::Button::new("Store (ACTIVE)").fill(Color32::from_rgb(0, 150, 255));
    let store_disabled = egui::Button::new("Store").fill(Color32::GRAY);

    let edit_button = egui::Button::new("Edit").fill(Color32::from_rgb(200, 100, 0));
    let edit_active = egui::Button::new("Edit (ACTIVE)").fill(Color32::from_rgb(255, 165, 0));

    let delete_button = egui::Button::new("Delete").fill(Color32::from_rgb(150, 50, 50));
    let delete_active = egui::Button::new("Delete (ACTIVE)").fill(Color32::from_rgb(255, 100, 100));

    let label_button = egui::Button::new("Label").fill(Color32::from_rgb(50, 100, 200));
    let label_active = egui::Button::new("Label (ACTIVE)").fill(Color32::from_rgb(100, 150, 255));

    let copy_button = egui::Button::new("Copy").fill(Color32::from_rgb(200, 100, 0));
    let copy_active = egui::Button::new("Copy (ACTIVE)").fill(Color32::from_rgb(200, 100, 0));

    let move_button = egui::Button::new("Move").fill(Color32::from_rgb(200, 100, 0));
    let move_active = egui::Button::new("Move (ACTIVE)").fill(Color32::from_rgb(200, 100, 0));

    let buffer_button =
        egui::Button::new(RichText::new("Buffer").color(if state.buffer.is_empty() {
            Color32::BLACK
        } else {
            Color32::WHITE
        }))
        .fill(if state.buffer.is_empty() {
            Color32::GRAY
        } else {
            Color32::DARK_BLUE
        });

    ui.horizontal(|ui| {
        let size = if state.edit_state.is_store() {
            active_size
        } else {
            normal_size
        };
        let btn = if !state.buffer.is_empty() {
            if state.edit_state.is_store() {
                store_active
            } else {
                store_button
            }
        } else {
            store_disabled
        };
        if ui.add_sized(size, btn).clicked() {
            if !state.buffer.is_empty() {
                state.edit_state.reset_if_set(EditingState::Store);
            }
        }

        // Edit Button
        if ui
            .add_sized(
                if state.edit_state.is_edit() {
                    active_size
                } else {
                    normal_size
                },
                if state.edit_state.is_edit() {
                    edit_active
                } else {
                    edit_button
                },
            )
            .clicked()
        {
            if state.edit_state.is_edit() | state.edit_state.is_none() {
                state.edit_state.reset_if_set(EditingState::Edit);
            } else {
                state.edit_state.set(EditingState::Edit);
            }
        }

        // Delete Button
        if ui
            .add_sized(
                if state.edit_state.is_delete() {
                    active_size
                } else {
                    normal_size
                },
                if state.edit_state.is_delete() {
                    delete_active
                } else {
                    delete_button
                },
            )
            .clicked()
        {
            if state.edit_state.is_delete() | state.edit_state.is_none() {
                state.edit_state.reset_if_set(EditingState::Delete);
            } else {
                state.edit_state.set(EditingState::Delete);
            }
        }
        // Label Button
        if ui
            .add_sized(
                if state.edit_state.is_label() {
                    active_size
                } else {
                    normal_size
                },
                if state.edit_state.is_label() {
                    label_active
                } else {
                    label_button
                },
            )
            .clicked()
        {
            if state.edit_state.is_label() | state.edit_state.is_none() {
                state.edit_state.reset_if_set(EditingState::Label);
            } else {
                state.edit_state.set(EditingState::Label);
            }
        }
        // Copy Button
        if ui
            .add_sized(
                if state.edit_state.is_copy() {
                    active_size
                } else {
                    normal_size
                },
                if state.edit_state.is_copy() {
                    copy_active
                } else {
                    copy_button
                },
            )
            .clicked()
        {
            if state.edit_state.is_copy() | state.edit_state.is_none() {
                state.edit_state.reset_if_set(EditingState::Copy);
            } else {
                state.edit_state.set(EditingState::Copy);
            }
        }
        // Move Button
        if ui
            .add_sized(
                if state.edit_state.is_move() {
                    active_size
                } else {
                    normal_size
                },
                if state.edit_state.is_move() {
                    move_active
                } else {
                    move_button
                },
            )
            .clicked()
        {
            if state.edit_state.is_move() | state.edit_state.is_none() {
                state.edit_state.reset_if_set(EditingState::Move);
            } else {
                state.edit_state.set(EditingState::Move);
            }
        }

        if ui.add_sized(active_size, buffer_button).clicked() {
            state.show_buffer = !state.show_buffer;
        }

        if ui.add_sized(normal_size, clear_button).clicked() {
            if !state.buffer.is_empty() {
                state.command_history.push(ConsoleCommand::Clear);
                state.buffer.clear();
            } else {
                state.edit_state.reset();
            }
        }
    });
    match state.dmx_sub_tab {
        DmxSubTab::Executor => match state.edit_state {
            EditingState::None => {}
            EditingState::Store => {
                ui.label(
                    RichText::new("Click an executor to store the buffer to a new cue")
                        .small()
                        .color(Color32::GOLD),
                );
            }
            EditingState::Edit => {
                ui.label(
                    RichText::new("Click an executor to edit its cues")
                        .small()
                        .color(Color32::GOLD),
                );
            }
            EditingState::Delete => {
                ui.label(
                    RichText::new("Click an executor to delete all its cues")
                        .small()
                        .color(Color32::LIGHT_RED),
                );
            }
            EditingState::Label => {
                ui.label(
                    RichText::new("Click an executor to rename its cues")
                        .small()
                        .color(Color32::from_rgb(100, 150, 255)),
                );
            }
            EditingState::Copy => {}
            EditingState::Move => {}
            EditingState::OSCLearn => {}
        },
        DmxSubTab::Fixtures => match state.edit_state {
            EditingState::None => {}
            EditingState::Store => {}
            EditingState::Edit => {}
            EditingState::Delete => {}
            EditingState::Copy => {}
            EditingState::Move => {}
            EditingState::Label => {}
            EditingState::OSCLearn => {}
        },
    }
}

fn show_confirm_prompt_panel(ctx: &egui::Context, state: &mut ConsoleState, exec_idx: usize) {
    egui::Window::new("Confirm Delete")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading(format!("Delete all cues from Executor {}?", exec_idx + 1));
            ui.separator();
            ui.label("This action cannot be undone.");

            ui.horizontal(|ui| {
                if ui.button("Yes, Delete All").clicked() {
                    state.executors[exec_idx].cue_list.clear();
                    state.executors[exec_idx].current_cue = None;
                    state.executors[exec_idx].current_cue_index = 0;
                    state.executors[exec_idx].stored_channels = vec![0; DMX_CHANNELS];
                    state.delete_confirm_executor = None;
                    state.edit_state.set(EditingState::None);
                }
                if ui.button("Cancel").clicked() {
                    state.delete_confirm_executor = None;
                }
            });
        });
}

fn show_edit_executor_panel(ctx: &egui::Context, state: &mut ConsoleState, exec_idx: usize) {
    let mut exec_command = false;
    egui::Window::new("Cue List")
        .collapsible(true)
        .resizable(true)
        .movable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading(format!("Executor {} - Cue List", exec_idx + 1));
            ui.separator();

            // V2
            if let Some(executor) = state.executors.get_mut(exec_idx) {
                if executor.cue_list.is_empty() {
                    ui.label("No cues in this executor.");
                } else {
                    ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        ui.vertical(|ui| {
                            executor.cue_list.iter_mut().for_each(|cue| {
                                // CUE ITERATION
                                ui.horizontal(|ui| {
                                    if ui
                                        .add_sized(
                                            Vec2::new(120.0, 35.0),
                                            egui::Button::new(
                                                RichText::new(format!(
                                                    "[Executor {}] {} ID: {}",
                                                    executor.id.saturating_add(1), // Base 1 instead of base 0
                                                    cue.name,
                                                    cue.id,
                                                ))
                                                .color(Color32::GRAY),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        match state.edit_state {
                                            EditingState::Move => {
                                                if state
                                                    .command_input
                                                    .to_lowercase()
                                                    .trim_end()
                                                    .ends_with("to exec")
                                                {
                                                    state.command_input = format!(
                                                        "{} {} Cue {}",
                                                        state.command_input.trim_end(),
                                                        exec_idx.saturating_add(1),
                                                        cue.id
                                                    );
                                                    state
                                                        .edit_state
                                                        .reset_if_set(EditingState::Move);
                                                    exec_command = true;
                                                } else {
                                                    state.command_input = format!(
                                                        "Move Exec {} Cue {} To Exec ",
                                                        exec_idx.saturating_add(1),
                                                        cue.id
                                                    );
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    if let Some(cue_idx) = state.labeling_cue
                                        && state.edit_state.is_label()
                                        && cue_idx == cue.id
                                    {
                                        ui.add_sized(
                                            Vec2::new(120.0, 35.0),
                                            egui::TextEdit::singleline(&mut state.label_buffer),
                                        );
                                        if ui
                                            .add_sized(
                                                Vec2::new(120.0, 35.0),
                                                egui::Button::new("Save Cue Name"),
                                            )
                                            .clicked()
                                        {
                                            cue.name = state.label_buffer.clone();
                                            state.label_buffer.clear();
                                            state.edit_state.reset();
                                            state.labeling_cue = None;
                                        }
                                    }
                                    if state.edit_state.is_label()
                                        && state.labeling_cue != Some(cue.id)
                                    {
                                        if ui
                                            .add_sized(
                                                Vec2::new(120.0, 35.0),
                                                egui::Button::new("Rename"),
                                            )
                                            .clicked()
                                        {
                                            state.labeling_cue = Some(cue.id);
                                        }
                                    }
                                });
                            });
                        });
                    });
                }
            }
            // V1
            /* let cue_count = state.executors[exec_idx].cue_list.len();
            if cue_count == 0 {
                ui.label("No cues in this executor.");
            } else {
                ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    // Display all cues from the selected executor
                    for i in 0..=cue_count - 1 {
                        let is_labeling = state.edit_state.is_label()
                            && state.labeling_cue == Some((exec_idx, i));

                        if is_labeling {
                            ui.horizontal(|ui| {
                                ui.label("Rename:");
                                let _response = ui.add(
                                    TextEdit::singleline(&mut state.label_buffer)
                                        .desired_width(150.0),
                                );
                                if ui.button("Save").clicked() {
                                    if !state.label_buffer.is_empty() {
                                        state.executors[exec_idx].cue_list[i].name =
                                            state.label_buffer.clone();
                                    }
                                    state.labeling_cue = None;
                                    state.edit_state.reset();
                                    state.label_buffer.clear();
                                }
                                if ui.button("Cancel").clicked() {
                                    state.labeling_cue = None;
                                    state.edit_state.reset();
                                    state.label_buffer.clear();
                                }
                            });
                        } else {
                            if ui
                                .button(
                                    RichText::new(format!(
                                        "[Exec{}] [Cue{}] - {}",
                                        exec_idx + 1,
                                        state.executors[exec_idx].cue_list[i].id,
                                        state.executors[exec_idx].cue_list[i].name
                                    ))
                                    .strong(),
                                )
                                .clicked()
                            {
                                if state.edit_state.is_move() {
                                    if state
                                        .command_input
                                        .to_lowercase()
                                        .trim_end()
                                        .ends_with("to exec")
                                    {
                                        state.command_input = format!(
                                            "{} {} Cue {}",
                                            state.command_input.trim_end(),
                                            exec_idx + 1,
                                            i + 1
                                        );
                                        execute_console_command(state);
                                        state.command_input.clear();
                                        state.edit_state.reset_if_set(EditingState::Move);
                                    } else {
                                        state.command_input = format!(
                                            "Move Exec {} Cue {} To Exec ",
                                            exec_idx + 1,
                                            i + 1
                                        );
                                    }
                                }
                            }

                            if state.edit_state.is_label() && ui.button("Rename").clicked() {
                                state.labeling_cue = Some((exec_idx, i));
                                state.label_buffer =
                                    state.executors[exec_idx].cue_list[i].name.clone();
                            }
                        }

                        ui.horizontal(|ui| {
                            ui.label("Fade:");
                            ui.add(
                                DragValue::new(
                                    &mut state.executors[exec_idx].cue_list[i].fade_time,
                                )
                                .range(0.0..=f32::MAX)
                                .suffix("s")
                                .speed(0.1),
                            );
                        });
                        if ui.button("X").clicked() {
                            state.executors[exec_idx].cue_list.remove(i);
                        }
                        ui.separator();
                    }
                });
            } */
            if ui.button("Close").clicked() {
                state.editing_executor = None;
                state.edit_state.set(EditingState::None);
            }
        });
    if exec_command {
        execute_console_command(state);
        state.command_input.clear();
    }
}

pub fn show_fixtures_tab_content(ui: &mut egui::Ui, state: &mut ConsoleState) {
    ui.heading("Fixtures");
    ui.separator();

    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.fixtures_tab, FixturesTab::Creation, "Creation");
        ui.selectable_value(&mut state.fixtures_tab, FixturesTab::Grouping, "Grouping");
        ui.selectable_value(&mut state.fixtures_tab, FixturesTab::Editing, "Editing");
    });
    ui.separator();

    match state.fixtures_tab {
        FixturesTab::Creation => {
            ui.heading("Create Fixture");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.add(TextEdit::singleline(&mut state.new_fixture_name).desired_width(150.0));
            });

            ui.horizontal(|ui| {
                ui.label("Start Channel:");
                ui.add(
                    TextEdit::singleline(&mut state.new_fixture_start_channel).desired_width(80.0),
                );
            });

            ui.separator();
            ui.heading("Select Template");

            // Template selection
            egui::ComboBox::from_id_salt("template_select")
                .selected_text("Select Template...")
                .show_ui(ui, |ui| {
                    for template in &state.template_library.templates {
                        let label = format!("{} ({})", template.name, template.manufacturer);
                        if ui
                            .selectable_value(
                                &mut state.selected_template_id,
                                Some(template.id),
                                label,
                            )
                            .clicked()
                        {
                            state.selected_mode_index = 0;
                        }
                    }
                });

            // Show selected template details
            if let Some(template_id) = state.selected_template_id {
                if let Some(template) = state.template_library.get_template(template_id) {
                    ui.separator();
                    ui.label(RichText::new(format!("Template: {}", template.name)).strong());
                    ui.label(format!("Manufacturer: {}", template.manufacturer));

                    if template.modes.len() > 1 {
                        ui.separator();
                        ui.label("Mode:");
                        egui::ComboBox::from_id_salt("mode_select")
                            .selected_text(format!("Mode {}", state.selected_mode_index + 1))
                            .show_ui(ui, |ui| {
                                for (idx, mode) in template.modes.iter().enumerate() {
                                    let label =
                                        format!("{} ({}ch)", mode.name, mode.channels.len());
                                    if ui
                                        .selectable_value(
                                            &mut state.selected_mode_index,
                                            idx,
                                            label,
                                        )
                                        .clicked()
                                    {
                                        state.selected_mode_index = idx;
                                    }
                                }
                            });
                    }

                    // Show channel list
                    if let Some(mode) = template.get_mode(state.selected_mode_index) {
                        ui.separator();
                        ui.label(RichText::new("Channels:").strong());
                        ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                            for ch in &mode.channels {
                                ui.label(format!("Ch{}: {}", ch.offset + 1, ch.name));
                            }
                        });

                        // Check if fixture has RGB channels
                        let _has_rgb = mode.channels.iter().any(|c| {
                            matches!(
                                c.channel_type,
                                ChannelType::Red | ChannelType::Green | ChannelType::Blue
                            )
                        });
                    }
                }
            }

            if ui.button("Add Fixture").clicked() {
                let start_ch = state
                    .new_fixture_start_channel
                    .parse::<usize>()
                    .unwrap_or(1);

                if let Some(template_id) = state.selected_template_id {
                    let template = state.template_library.get_template(template_id);
                    let num_channels = template
                        .and_then(|t| t.get_mode(state.selected_mode_index))
                        .map(|m| m.channels.len())
                        .unwrap_or(0);

                    let end_ch = start_ch + num_channels - 1;

                    let collision = state.fixtures.iter().any(|f| {
                        let f_template = state.template_library.get_template(f.template_id);
                        let f_num_channels = f_template
                            .and_then(|t| t.get_mode(f.mode_index))
                            .map(|m| m.channels.len())
                            .unwrap_or(0);
                        let f_end_ch = f.start_channel + f_num_channels - 1;

                        start_ch <= f_end_ch && end_ch >= f.start_channel
                    });

                    if collision {
                        state.fixture_error = Some(format!(
                            "Channel collision! Channels {} to {} overlap with existing fixture",
                            start_ch, end_ch
                        ));
                    } else if !state.new_fixture_name.is_empty()
                        && start_ch > 0
                        && start_ch <= DMX_CHANNELS
                        && num_channels > 0
                    {
                        let new_id = state.fixtures.len() + 1;
                        let fixture = Fixture::new(
                            new_id,
                            state.new_fixture_name.clone(),
                            start_ch,
                            template_id,
                            state.selected_mode_index,
                        );

                        state.fixtures.push(fixture);
                        state.new_fixture_name.clear();
                        state.new_fixture_start_channel.clear();
                        state.fixture_error = None;
                    }
                }
            }

            if let Some(error) = &state.fixture_error {
                ui.label(RichText::new(error).color(egui::Color32::RED));
            }

            ui.separator();
            ui.heading("Existing Fixtures");
            ScrollArea::vertical()
                .id_salt("existing_fixtures")
                .max_height(200.0)
                .show(ui, |ui| {
                    let mut to_remove: Option<usize> = None;
                    for fixture in &state.fixtures {
                        let template_name = state
                            .template_library
                            .get_template(fixture.template_id)
                            .map(|t| t.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string());

                        let mode = state
                            .template_library
                            .get_template(fixture.template_id)
                            .and_then(|t| t.get_mode(fixture.mode_index))
                            .map(|m| m.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string());

                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "{} (ID: {}) - {} (Mode: {}) - Ch {}",
                                fixture.name,
                                fixture.id,
                                template_name,
                                mode,
                                fixture.start_channel
                            ));
                            if ui.button("✕").clicked() {
                                to_remove = Some(fixture.id);
                            }
                        });
                    }
                    if let Some(id) = to_remove {
                        state.fixtures.retain(|f| f.id != id);
                    }
                });
        }
        FixturesTab::Grouping => {
            ui.heading("Fixture Groups");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Group Name:");
                ui.add(TextEdit::singleline(&mut state.new_group_name).desired_width(150.0));
                ui.label("Grid #:");
                let mut grid_input = state.new_group_grid_index.unwrap_or(1).to_string();
                ui.add(TextEdit::singleline(&mut grid_input).desired_width(40.0));
                if let Ok(idx) = grid_input.parse::<usize>() {
                    state.new_group_grid_index = Some(idx);
                }
            });

            if ui.button("Create Group").clicked() {
                if !state.new_group_name.is_empty() {
                    let new_id = state.fixture_groups.len() as u32 + 1;
                    let mut group = FixtureGroup::new(new_id, state.new_group_name.clone());
                    group.grid_index = state.new_group_grid_index;
                    state.fixture_groups.push(group);
                    state.new_group_name.clear();
                    state.new_group_grid_index = None;
                }
            }

            ui.separator();

            ui.heading("Add Fixtures to Group");
            ui.label("Select fixtures:");
            ScrollArea::vertical()
                .id_salt("fixtures_select")
                .max_height(100.0)
                .show(ui, |ui| {
                    for fixture in &state.fixtures {
                        let is_selected = state.selected_fixture_ids.contains(&fixture.id);
                        if ui.selectable_label(is_selected, &fixture.name).clicked() {
                            if is_selected {
                                state.selected_fixture_ids.retain(|&id| id != fixture.id);
                            } else {
                                state.selected_fixture_ids.push(fixture.id);
                            }
                        }
                    }
                });

            ui.separator();
            ui.heading("Groups");
            ScrollArea::vertical()
                .id_salt("groups_list")
                .max_height(120.0)
                .show(ui, |ui| {
                    let mut to_remove: Option<u32> = None;
                    for group in &mut state.fixture_groups {
                        ui.horizontal(|ui| {
                            ui.label(&group.name);
                            ui.label(format!("({} fixtures)", group.fixture_ids.len()));
                            if let Some(idx) = group.grid_index {
                                ui.label(format!("[Grid {}]", idx));
                            }

                            if ui.button("Add Selected").clicked() {
                                for &fix_id in &state.selected_fixture_ids {
                                    if !group.fixture_ids.contains(&fix_id) {
                                        group.fixture_ids.push(fix_id);
                                    }
                                }
                            }
                            if ui.button("✕").clicked() {
                                to_remove = Some(group.id);
                            }
                        });
                    }
                    if let Some(id) = to_remove {
                        state.fixture_groups.retain(|g| g.id != id);
                    }
                });

            ui.separator();
            ui.heading("Group Grid");
            ui.label(
                "Click a cell to select that group, then use console to control (e.g., 'at at')",
            );

            let grid_cols = 10;
            let grid_rows = 10;
            let cell_size = 40.0;

            egui::Grid::new("group_grid")
                .num_columns(grid_cols)
                .spacing([5.0, 5.0])
                .show(ui, |ui| {
                    for idx in 1..=grid_cols * grid_rows {
                        let group = state
                            .fixture_groups
                            .iter()
                            .find(|g| g.grid_index == Some(idx));
                        let group_name = group.map(|g| g.name.as_str()).unwrap_or("");
                        let is_selected = group
                            .map(|g| state.selected_group_id == Some(g.id))
                            .unwrap_or(false);

                        let button_text = if group_name.is_empty() {
                            idx.to_string()
                        } else {
                            group_name.chars().take(6).collect()
                        };

                        let button = egui::Button::new(button_text)
                            .min_size(Vec2::new(cell_size, cell_size));

                        if is_selected {
                            ui.scope(|ui| {
                                ui.visuals_mut().widgets.active.bg_fill =
                                    egui::Color32::from_rgb(0, 120, 215);
                                if ui
                                    .add_sized(Vec2::new(cell_size, cell_size), button)
                                    .clicked()
                                {
                                    if let Some(g) = group {
                                        state.selected_group_id = Some(g.id);
                                        state.selected_fixture_ids = g.fixture_ids.clone();
                                    }
                                }
                            });
                        } else if ui
                            .add_sized(Vec2::new(cell_size, cell_size), button)
                            .clicked()
                        {
                            if let Some(g) = group {
                                state.selected_group_id = Some(g.id);
                                state.selected_fixture_ids = g.fixture_ids.clone();
                            } else {
                                state.selected_group_id = None;
                                state.selected_fixture_ids.clear();
                            }
                        }

                        if idx % grid_cols == 0 {
                            ui.end_row();
                        }
                    }
                });
        }
        FixturesTab::Editing => {
            ui.heading("Editing");
            ui.separator();
            ui.label("Editing features coming soon...");
        }
    }
}

pub fn show_midi_osc_tab(ctx: &egui::Context, state: &mut ConsoleState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("OSC Status");
            ui.separator();
            ui.label(
                RichText::new(if state.osc_manager.1.is_some() {
                    "Running"
                } else {
                    "Inactive"
                })
                .color(if state.osc_manager.1.is_some() {
                    Color32::GREEN
                } else {
                    Color32::YELLOW
                }),
            );
        });
        ui.separator();
        ui.heading(RichText::new("OSC Adress").color(Color32::ORANGE));
        ui.horizontal(|ui| {
            ui.add_sized(
                Vec2::new(150.0, 35.0),
                egui::TextEdit::singleline(&mut state.osc_manager.0),
            );
            if ui
                .add_sized(
                    Vec2::new(120.0, 35.0),
                    egui::Button::new(RichText::new("Connect").color(Color32::DARK_GREEN)),
                )
                .clicked()
            {
                use crate::osc::OSCManager;
                state.osc_manager.1 = OSCManager::from(state.osc_manager.0.clone()).ok();
                state.osc_manager.0.clear();
            }
        });

        if state.osc_manager.1.is_some() {
            ui.separator();
            ui.heading("OSC History");
            ui.horizontal(|ui| {
                if let Some(osc_manager) = &mut state.osc_manager.1 {
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                osc_manager.get_osc_history().iter().rev().for_each(|p| {
                                    ui.label(p.to_string());
                                });
                            });
                        });
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Audio OSC Controls");
                    egui::ScrollArea::vertical()
                        .id_salt("Audio")
                        .max_height(600.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Master Volume"));
                                ui.text_edit_singleline(
                                    &mut state.osc_address_manager.master_volume,
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Go"));
                                ui.text_edit_singleline(&mut state.osc_address_manager.audio_go);
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Stop"));
                                ui.text_edit_singleline(&mut state.osc_address_manager.audio_stop);
                            });
                        });
                });
                ui.vertical(|ui| {
                    ui.heading("Light OSC Controls");
                    egui::ScrollArea::vertical()
                        .id_salt("Light")
                        .max_height(600.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Master Dimmer"));
                                ui.text_edit_singleline(&mut state.osc_address_manager.master_dmx);
                            });
                            ui.heading("Executors");
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Executors Identifier"));
                                ui.text_edit_singleline(
                                    &mut state.osc_address_manager.executor_identifier,
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Executors Dimmer"));
                                ui.text_edit_singleline(
                                    &mut state.osc_address_manager.executor_dimmer,
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Executors Go"));
                                ui.text_edit_singleline(&mut state.osc_address_manager.executor_go);
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Executors Go Back"));
                                ui.text_edit_singleline(
                                    &mut state.osc_address_manager.executor_go_back,
                                );
                            });
                        });
                });
            });
        }
    });
}

pub fn show_audio_tab(ctx: &egui::Context, state: &mut ConsoleState) {
    // Update audio engine (for fade handling)
    if let Some(ref mut engine) = state.audio_engine {
        engine.set_master_volume(state.master_volume);
        engine.update();

        // Handle follow/continue for ended tracks
        let ended_tracks = engine.get_ended_tracks();
        for (track_id, action) in ended_tracks {
            if action == AudioAction::Follow {
                if let Some(idx) = state.audio_tracks.iter().position(|t| t.id == track_id) {
                    let next_idx = idx.saturating_add(1) % state.audio_tracks.len();
                    if let Some(next_track) = state.audio_tracks.get(next_idx) {
                        let _ = engine.play(next_track, state.master_volume);
                        state.audio_index = next_idx;
                    }
                }
            }
        }
    }

    egui::SidePanel::left("audio_master_panel")
        .min_width(60.0)
        .max_width(80.0)
        .show(ctx, |ui| {
            ui.heading("Volume");
            ui.separator();
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.label(format!("{}%", (state.master_volume * 100.0) as u32));
                let available_height = ui.available_height();
                ui.add_sized(
                    egui::vec2(30.0, available_height),
                    egui::Slider::new(&mut state.master_volume, 0.0..=1.5).vertical(),
                );
            });
            if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
                state.master_volume = (state.master_volume + 0.01).clamp(0.0, 1.5);
            }
            if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
                state.master_volume = (state.master_volume - 0.01).clamp(0.0, 1.5);
            }
            if ui.input(|i| i.key_pressed(Key::F)) {
                state.master_volume = 1.0;
            }
        });

    egui::SidePanel::right("audio_playback_panel")
        .min_width(120.0)
        .max_width(150.0)
        .show(ctx, |ui| {
            ui.heading("Playback");
            ui.separator();

            let track_count = state.audio_tracks.len();
            let safe_index = state.audio_index.min(track_count.saturating_sub(1));
            state.audio_tracks.iter().for_each(|t| {
                state
                    .audio_engine
                    .as_ref()
                    .map(|e| {
                        if e.is_playing(t.id) {
                            Some((e.get_current_playback(), t.duration))
                        } else {
                            None
                        }
                    })
                    .into_iter()
                    .flatten()
                    .enumerate()
                    .for_each(|(idx, (opt_cur_dur, total_dur))| {
                        let cur_dur = opt_cur_dur[idx];
                        let cur_secs = cur_dur as u64;
                        let cur_hours = cur_secs / 3600;
                        let cur_mins = (cur_secs % 3600) / 60;
                        let cur_secs = cur_secs % 60;

                        let total_secs = total_dur as u64;
                        let total_hours = total_secs / 3600;
                        let total_mins = (total_secs % 3600) / 60;
                        let total_secs = total_secs % 60;

                        ui.label(format!(
                            "{:02}:{:02}:{:02} / {:02}:{:02}:{:02}",
                            cur_hours, cur_mins, cur_secs, total_hours, total_mins, total_secs
                        ));
                    });
            });

            ui.horizontal(|ui| {
                if ui.button("◀").clicked() | ui.input(|i| i.key_pressed(Key::ArrowLeft)) {
                    if track_count > 0 {
                        decrement_audio_index(state, track_count);
                    }
                }
                ui.label(format!("{}/{}", safe_index + 1, track_count.max(1)));

                if ui.button("▶").clicked() | ui.input(|i| i.key_pressed(Key::ArrowRight)) {
                    if track_count > 0 {
                        increment_audio_index(state, track_count);
                    }
                }
            });

            ui.separator();

            if track_count > 0 {
                let current_track = &state.audio_tracks[safe_index];
                ui.label(&current_track.name);
            }

            ui.separator();

            if ui
                .add_sized(
                    Vec2::new(120.0, 50.0),
                    egui::Button::new("[▶] GO").fill(Color32::DARK_GREEN),
                )
                .clicked()
            {
                audio_go(state, track_count);
            }

            if ui.input(|i| i.key_pressed(Key::Space)) {
                audio_go(state, track_count);
            }
            if ui
                .add_sized(
                    Vec2::new(120.0, 50.0),
                    egui::Button::new("[⏹] STOP").fill(Color32::DARK_RED),
                )
                .clicked()
            {
                if let Some(ref engine) = state.audio_engine {
                    engine.stop_all();
                }
            }
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Audio Tracks");
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Add Track").clicked() | ui.input(|i| i.key_pressed(Key::A)) {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Untitled".to_string());
                    let file_path = path.to_string_lossy().to_string();
                    let new_id = state.audio_tracks.len() as u32 + 1;

                    // Get duration using lofty
                    let duration = crate::audio::AudioEngine::get_duration(&file_path);

                    let mut track = AudioTrack::new(new_id, file_name, file_path);
                    track.duration = duration;
                    state.audio_tracks.push(track);
                }
            }

            if ui.button("Stop All").clicked() | ui.input(|i| i.key_pressed(Key::Escape)) {
                if let Some(ref engine) = state.audio_engine {
                    engine.stop_all();
                }
            }
        });

        ui.separator();

        ScrollArea::vertical()
            .id_salt("audio_tracks_list")
            .show(ui, |ui| {
                let mut to_remove: Option<u32> = None;
                let mut move_up: Option<usize> = None;
                let mut move_down: Option<usize> = None;

                for (idx, track) in state.audio_tracks.iter_mut().enumerate() {
                    let is_selected = state.selected_audio_track_id == Some(track.id);
                    let is_playing = state
                        .audio_engine
                        .as_ref()
                        .map(|e| e.is_playing(track.id))
                        .unwrap_or(false);

                    egui::Frame::group(&egui::Style::default()).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Up button
                            if ui.button("⬆️").clicked() {
                                move_up = Some(idx);
                            }
                            // Down button
                            if ui.button("⬇️").clicked() {
                                move_down = Some(idx);
                            }

                            if is_playing {
                                if ui.button("⏹").clicked() {
                                    if let Some(ref engine) = state.audio_engine {
                                        engine.stop(track.id);
                                    }
                                }
                            } else {
                                if ui.button("▶").clicked() {
                                    if let Some(ref engine) = state.audio_engine {
                                        let _ = engine.play(track, state.master_volume);
                                    }
                                }
                            }

                            if ui.selectable_label(is_selected, &track.name).clicked() {
                                state.selected_audio_track_id = Some(track.id);
                            }

                            // Show action flag
                            {
                                match track.action {
                                    AudioAction::None => {}
                                    AudioAction::Follow => {
                                        ui.label("↻ Follow");
                                    }
                                    AudioAction::Continue => {
                                        ui.label("⤵ Continue");
                                    }
                                }
                            }

                            // Show duration
                            let duration_str = format_duration(track.duration);
                            ui.label(duration_str);

                            if ui.button("❌").clicked() {
                                if let Some(ref engine) = state.audio_engine {
                                    engine.stop(track.id);
                                }
                                to_remove = Some(track.id);
                            }
                        });

                        if is_selected {
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label("Action:");
                                let action_text = match track.action {
                                    AudioAction::None => "None",
                                    AudioAction::Follow => "Follow",
                                    AudioAction::Continue => "Continue",
                                };
                                egui::ComboBox::from_id_salt("audio_action")
                                    .selected_text(action_text)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut track.action,
                                            AudioAction::None,
                                            "None",
                                        );
                                        ui.selectable_value(
                                            &mut track.action,
                                            AudioAction::Follow,
                                            "Follow",
                                        );
                                        ui.selectable_value(
                                            &mut track.action,
                                            AudioAction::Continue,
                                            "Continue",
                                        );
                                    });
                            });
                            ui.horizontal(|ui| {
                                ui.label("Fade In:");
                                ui.add(
                                    egui::Slider::new(
                                        &mut track.fade_in,
                                        0.0..=track.duration
                                            - (track.fade_out
                                                + (track.duration
                                                    - track.end_point.unwrap_or(track.duration))
                                                + track.start_point),
                                    )
                                    .text("s"),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("Fade Out:");
                                ui.add(
                                    egui::Slider::new(
                                        &mut track.fade_out,
                                        0.0..=track.duration
                                            - (track.fade_in
                                                + track.start_point
                                                + (track.duration
                                                    - track.end_point.unwrap_or(track.duration))),
                                    )
                                    .text("s"),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("Start:");
                                ui.add(
                                    egui::Slider::new(
                                        &mut track.start_point,
                                        0.0..=track.duration
                                            - track.fade_in
                                            - track.fade_out
                                            - (track.duration
                                                - track.end_point.unwrap_or(track.duration)),
                                    )
                                    .text("s"),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("End:");
                                let mut end_val = track.end_point.unwrap_or(track.duration);
                                ui.add(
                                    egui::Slider::new(
                                        &mut end_val,
                                        track.fade_in + track.fade_out + track.start_point
                                            ..=track.duration,
                                    )
                                    .text("s"),
                                );
                                track.end_point = if end_val > 0.0 && end_val < track.duration {
                                    Some(end_val)
                                } else {
                                    None
                                };
                            });
                            ui.horizontal(|ui| {
                                ui.label("Volume:");
                                ui.add(egui::Slider::new(&mut track.volume, 0.0..=1.0));
                            });
                        }
                    });
                    ui.separator();
                }

                // Handle reordering
                if let Some(up_idx) = move_up {
                    if up_idx > 0 {
                        state.audio_tracks.swap(up_idx, up_idx - 1);
                    }
                }
                if let Some(down_idx) = move_down {
                    if down_idx < state.audio_tracks.len() - 1 {
                        state.audio_tracks.swap(down_idx, down_idx + 1);
                    }
                }

                if let Some(id) = to_remove {
                    state.audio_tracks.retain(|t| t.id != id);
                    if state.selected_audio_track_id == Some(id) {
                        state.selected_audio_track_id = None;
                    }
                }
            });
    });
}

pub fn audio_go(state: &mut ConsoleState, track_count: usize) {
    if track_count > 0 {
        let idx = state.audio_index;
        if let Some(ref engine) = state.audio_engine {
            if let Some(track) = state.audio_tracks.get(idx) {
                let _ = engine.play(track, state.master_volume);

                // Handle continue: play next track at the same time
                if track.action == AudioAction::Continue {
                    if let Some(next_track) = state.audio_tracks.get(idx + 1) {
                        let _ = engine.play(next_track, state.master_volume);
                    }
                }
            }
        }
        // Auto increment index after GO
        increment_audio_index(state, track_count);
    }
}

fn increment_audio_index(state: &mut ConsoleState, track_count: usize) {
    state.audio_index = state.audio_index.saturating_add(1) % track_count;
}
fn decrement_audio_index(state: &mut ConsoleState, track_count: usize) {
    if state.audio_index == 0 {
        state.audio_index = track_count - 1;
    } else {
        state.audio_index = state.audio_index.saturating_sub(1);
    }
}

fn format_duration(seconds: f32) -> String {
    let mins = (seconds as u32) / 60;
    let secs = (seconds as u32) % 60;
    format!("{}:{:02}", mins, secs)
}

pub fn show_liveshow_tab(ctx: &egui::Context, state: &mut ConsoleState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Show");
        if ui.button("Stop").clicked() {
            println!("Adding stop");
        }
        ui.separator();
    });
}
