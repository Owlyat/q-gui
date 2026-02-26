use crate::dmx_types::DMXBufferValue;
use open_dmx::DMX_CHANNELS;
use scan_fmt::scan_fmt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsoleError {
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
    #[error("Invalid channel: {0}. Must be between 1 and {1}")]
    InvalidChannel(String, usize),
    #[error("Invalid fixture id: {0}")]
    InvalidFixtureId(String),
    #[error("Invalid level: {0}. Must be between 0 and 255")]
    InvalidLevel(String),
    #[error("Missing arguments for command: {0}")]
    MissingArgs(String),
}

#[derive(Clone, Serialize, Deserialize, Debug, strum::Display)]
pub enum ConsoleCommand {
    #[strum(serialize = "Chan {ch} at {value}")]
    DimChannel { ch: usize, value: u8 },
    #[strum(serialize = "Fix {fixture_id} at {value}")]
    DimFixture { fixture_id: usize, value: u8 },
    #[strum(serialize = "Blackout")]
    Blackout,
    #[strum(serialize = "Clear")]
    Clear,
    #[strum(serialize = "Move Exec {exec_from} Cue {cue_from} To Exec {exec_to} Cue {cue_to}")]
    MoveExecCue {
        exec_from: usize,
        cue_from: usize,
        exec_to: usize,
        cue_to: usize,
    },
}
impl TryFrom<String> for ConsoleCommand {
    type Error = ConsoleError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let s = value.trim().to_lowercase();
        if s.eq_ignore_ascii_case("b/o")
            | s.eq_ignore_ascii_case("blackout")
            | s.eq_ignore_ascii_case("bo")
        {
            return Ok(ConsoleCommand::Blackout);
        }
        if s.eq_ignore_ascii_case("clear") | s.eq_ignore_ascii_case("clr") {
            return Ok(ConsoleCommand::Clear);
        }
        if let Ok((ch, value)) = scan_fmt!(&s, "chan {} at {}", usize, u8) {
            return Ok(ConsoleCommand::DimChannel { ch, value });
        }
        if let Ok((fixture_id, value)) = scan_fmt!(&s, "fix {} at {}", usize, u8) {
            return Ok(ConsoleCommand::DimFixture { fixture_id, value });
        }
        if let Ok((exec_from, cue_from, exec_to, cue_to)) = scan_fmt!(
            &s,
            "move exec {} cue {} to exec {} cue {}",
            usize,
            usize,
            usize,
            usize
        ) {
            return Ok(ConsoleCommand::MoveExecCue {
                exec_from,
                cue_from,
                exec_to,
                cue_to,
            });
        }
        Err(ConsoleError::UnknownCommand(value))
    }
}
impl ConsoleCommand {
    pub fn parse(input: &str) -> Result<ConsoleCommand, ConsoleError> {
        ConsoleCommand::try_from(input.to_string())
    }
}

pub fn execute_console_command(state: &mut crate::ConsoleState) {
    let command = state.command_input.clone();
    state.command_error = None;

    match ConsoleCommand::parse(&command) {
        Ok(cmd) => match cmd {
            ConsoleCommand::Blackout => {
                state.command_history.push(cmd);
                state.master_dimmer = if state.master_dimmer != 0.0 { 0.0 } else { 1.0 };
            }
            ConsoleCommand::Clear => {
                state.command_history.push(cmd);
                state.buffer.clear();
            }
            ConsoleCommand::DimChannel { ch, value } => {
                if let Some(existing) = state.buffer.iter_mut().find(|v| v.chan == ch) {
                    existing.dmx = value;
                } else {
                    state.buffer.push(DMXBufferValue {
                        chan: ch,
                        dmx: value,
                    });
                }
                state.command_history.push(cmd);
            }
            ConsoleCommand::DimFixture { fixture_id, value } => {
                if let Some(fixture) = state.fixtures.iter().find(|f| f.id == fixture_id) {
                    let channel = fixture.start_channel;
                    if let Some(existing) = state.buffer.iter_mut().find(|v| v.chan == channel) {
                        existing.dmx = value;
                    } else {
                        state.buffer.push(DMXBufferValue {
                            chan: channel,
                            dmx: value,
                        });
                    }
                    state.command_history.push(cmd);
                } else {
                    state.command_error = Some(format!("Fixture {fixture_id} not found"));
                }
            }
            ConsoleCommand::MoveExecCue {
                exec_from,
                cue_from,
                exec_to,
                cue_to,
            } => {
                let exec_from = exec_from.saturating_sub(1);
                let exec_to = exec_to.saturating_sub(1);
                let same_exec = exec_from == exec_to;
                let cue_from = cue_from.saturating_sub(1);
                let cue_to = cue_to.saturating_sub(1);
                if same_exec {
                    let exec = &mut state.executors[exec_from];
                    let current_cue = exec.cue_list[cue_from].clone();
                    if let Some(cue_to_be_swapped) = exec.cue_list.get_mut(cue_to) {
                        let cue_clone = cue_to_be_swapped.clone();
                        *cue_to_be_swapped = current_cue;
                        exec.cue_list[cue_from] = cue_clone;
                    } else {
                        let current_cue = exec.cue_list.remove(cue_from);
                        exec.cue_list.insert(cue_to, current_cue);
                    }
                } else {
                    let cue_a = state.executors[exec_from].cue_list.remove(cue_from);
                    if let Some(cue) = state.executors[exec_to].cue_list.get_mut(cue_to) {
                        let cue_b = cue.clone();
                        *cue = cue_a.clone();
                        state.executors[exec_from].cue_list.insert(cue_from, cue_b);
                    } else {
                        state.executors[exec_to].cue_list.insert(cue_to, cue_a);
                    }
                }

                state.command_history.push(cmd);
            }
        },
        Err(e) => {
            state.command_error = Some(e.to_string());
        }
    }
}
