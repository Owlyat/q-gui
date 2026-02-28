use std::str::FromStr;

use crate::dmx_types::{ChannelType, DMXBufferValue, Fixture};
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
#[derive(strum::Display, Clone, Serialize, Deserialize, Debug, strum::EnumString)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Clone, Serialize, Deserialize, Debug, strum::Display)]
pub enum ConsoleCommand {
    #[strum(serialize = "Chan {ch} at {value}")]
    DimChannel { ch: usize, value: u8 },
    #[strum(serialize = "Fix {fixture_id} at {value}")]
    DimFixture { fixture_id: u32, value: u8 },
    #[strum(serialize = "Fix {fixture_id} Color R{r} G{g} B{b} W{w}")]
    SetFixtureColor {
        fixture_id: u32,
        r: u8,
        g: u8,
        b: u8,
        w: u8,
    },
    #[strum(serialize = "Blackout")]
    Blackout,
    #[strum(serialize = "Clear")]
    Clear,
    #[strum(serialize = "Move Exec {exec_from} Cue {cue_from} To Exec {exec_to} Cue {cue_to}")]
    MoveExecCueToExecCue {
        exec_from: u32,
        cue_from: u32,
        exec_to: u32,
        cue_to: u32,
    },
    #[strum(serialize = "Move Exec {exec_from} Cue {cue_from} {direction}")]
    MoveExecCueDirection {
        exec_from: u32,
        cue_from: u32,
        direction: Direction,
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
        if let Ok((fixture_id, value)) = scan_fmt!(&s, "fix {} at {}", u32, u8) {
            return Ok(ConsoleCommand::DimFixture { fixture_id, value });
        }
        if let Ok((fixture_id, r, g, b, w)) =
            scan_fmt!(&s, "fix {} color r{} g{} b{} w{}", u32, u8, u8, u8, u8)
        {
            return Ok(ConsoleCommand::SetFixtureColor {
                fixture_id,
                r,
                g,
                b,
                w,
            });
        }
        if let Ok((exec_from, cue_from, exec_to, cue_to)) = scan_fmt!(
            &s,
            "move exec {} cue {} to exec {} cue {}",
            u32,
            u32,
            u32,
            u32
        ) {
            return Ok(ConsoleCommand::MoveExecCueToExecCue {
                exec_from,
                cue_from,
                exec_to,
                cue_to,
            });
        }
        if let Ok((exec_from, cue_from)) = scan_fmt!(&s, "move exec {} cue {} up", u32, u32) {
            return Ok(ConsoleCommand::MoveExecCueDirection {
                exec_from,
                cue_from,
                direction: Direction::Up,
            });
        }
        if let Ok((exec_from, cue_from)) = scan_fmt!(&s, "move exec {} cue {} down", u32, u32) {
            return Ok(ConsoleCommand::MoveExecCueDirection {
                exec_from,
                cue_from,
                direction: Direction::Down,
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
                    state.buffer.push(DMXBufferValue::new(ch, value));
                }
                state.command_history.push(cmd);
            }
            ConsoleCommand::DimFixture { fixture_id, value } => {
                if let Some(fixture) = state.fixtures.iter_mut().find(|f| f.id == fixture_id) {
                    fixture.intensity = value;
                    if let Some(fixture_template) =
                        state.template_library.get_template(fixture.template_id)
                    {
                        let values = fixture.get_fixture_as_buffer(fixture_template);

                        let has_color = fixture.color.has_color();

                        let channels_to_dim: Vec<DMXBufferValue> = values
                            .iter()
                            .filter_map(|(chan_type, buf)| {
                                if has_color {
                                    if chan_type.is(ChannelType::Intensity) {
                                        Some(buf.clone())
                                    } else {
                                        None
                                    }
                                } else {
                                    if chan_type.is(ChannelType::Intensity) {
                                        Some(buf.clone())
                                    } else if chan_type.is(ChannelType::White) {
                                        fixture.color.w = value;
                                        Some(buf.clone())
                                    } else {
                                        None
                                    }
                                }
                            })
                            .collect();

                        channels_to_dim.iter().for_each(|buf| {
                            if let Some(existing) =
                                state.buffer.iter_mut().find(|v| v.chan == buf.chan)
                            {
                                existing.dmx = value;
                            } else {
                                state.buffer.push(DMXBufferValue::new(buf.chan, value));
                            }
                        });
                    } else {
                        let channel = fixture.start_channel;
                        if let Some(existing) = state.buffer.iter_mut().find(|v| v.chan == channel)
                        {
                            existing.dmx = value;
                        } else {
                            state.buffer.push(DMXBufferValue::new(channel, value));
                        }
                    }

                    state.command_history.push(cmd);
                } else {
                    state.command_error = Some(format!("Fixture {fixture_id} not found"));
                }
            }
            ConsoleCommand::SetFixtureColor {
                fixture_id,
                r,
                g,
                b,
                w,
            } => {
                if let Some(fixture) = state.fixtures.iter_mut().find(|f| f.id == fixture_id) {
                    if let Some(fixture_template) =
                        state.template_library.get_template(fixture.template_id)
                    {
                        let values = fixture.get_fixture_as_buffer(fixture_template);

                        for (chan_type, buf) in &values {
                            if matches!(
                                chan_type,
                                ChannelType::Red
                                    | ChannelType::Green
                                    | ChannelType::Blue
                                    | ChannelType::White
                            ) {
                                let new_value = match chan_type {
                                    ChannelType::Red => r,
                                    ChannelType::Green => g,
                                    ChannelType::Blue => b,
                                    ChannelType::White => w,
                                    _ => continue,
                                };
                                if let Some(existing) =
                                    state.buffer.iter_mut().find(|v| v.chan == buf.chan)
                                {
                                    existing.dmx = new_value;
                                } else {
                                    state.buffer.push(DMXBufferValue::new(buf.chan, new_value));
                                }
                            }
                        }

                        fixture.color.r = r;
                        fixture.color.g = g;
                        fixture.color.b = b;
                        fixture.color.w = w;

                        state.command_history.push(cmd);
                    }
                } else {
                    state.command_error = Some(format!("Fixture {fixture_id} not found"));
                }
            }
            ConsoleCommand::MoveExecCueToExecCue {
                exec_from,
                cue_from,
                exec_to,
                cue_to,
            } => {
                let exec_idx_from = (exec_from.saturating_sub(1)) as usize;
                let exec_idx_to = (exec_to.saturating_sub(1)) as usize;
                let exec = &mut state.executors[exec_idx_from];
                let cue_from_idx = exec
                    .cue_list
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, c)| if c.id == cue_from { Some(idx) } else { None })
                    .next();
                let exec = &mut state.executors[exec_idx_to];
                let cue_to_idx = exec
                    .cue_list
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, c)| if c.id == cue_to { Some(idx) } else { None })
                    .next();

                if let Some(idx_from) = cue_from_idx {
                    if let Some(idx_to) = cue_to_idx {
                        if exec_from == exec_to {
                            state.executors[exec_idx_from]
                                .cue_list
                                .swap(idx_from, idx_to);
                        } else {
                            let cue = state.executors[exec_idx_from].cue_list.remove(idx_from);
                            let to_move = state.executors[exec_idx_to].cue_list.remove(idx_to);
                            let mut found_same = false;
                            // Prevent having same Cue ids from two different executors
                            state.executors[exec_idx_to]
                                .cue_list
                                .iter_mut()
                                .for_each(|c| {
                                    if c.id == cue.id {
                                        found_same = true
                                    }
                                    if found_same {
                                        c.id += 1;
                                    }
                                });
                            let mut found_same = false;
                            // Prevent having same Cue ids from two different executors
                            state.executors[exec_idx_from]
                                .cue_list
                                .iter_mut()
                                .for_each(|c| {
                                    if c.id == to_move.id {
                                        found_same = true
                                    }
                                    if found_same {
                                        c.id += 1;
                                    }
                                });
                            state.executors[exec_idx_from]
                                .cue_list
                                .insert(idx_from, to_move);
                            state.executors[exec_idx_to].cue_list.insert(idx_to, cue);
                        }
                    } else {
                        let cue = state.executors[exec_idx_from].cue_list.remove(idx_from);
                        let cue_size = state.executors[exec_idx_to].cue_list.len();
                        if cue_size > 0 {
                            let mut found_same = false;
                            // Prevent having same Cue ids from two different executors
                            state.executors[exec_idx_to]
                                .cue_list
                                .iter_mut()
                                .for_each(|c| {
                                    if c.id == cue.id {
                                        found_same = true
                                    }
                                    if found_same {
                                        c.id += 1;
                                    }
                                });
                            state.executors[exec_idx_to]
                                .cue_list
                                .insert((cue_to.saturating_sub(1)) as usize, cue);
                        } else {
                            state.executors[exec_idx_to].cue_list.push(cue);
                        }
                    }
                }
                /* let exec_from = exec_from.saturating_sub(1);
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

                state.command_history.push(cmd); */
            }
            ConsoleCommand::MoveExecCueDirection {
                exec_from,
                cue_from,
                direction,
            } => {
                let exec_idx = (exec_from.saturating_sub(1)) as usize;
                let cue_size = state.executors[exec_idx].cue_list.len();
                if let Some(exec) = state.executors.get_mut(exec_idx) {
                    let idx = exec
                        .cue_list
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, c)| if c.id == cue_from { Some(idx) } else { None })
                        .next();
                    if let Some(idx) = idx {
                        exec.cue_list.swap(
                            idx,
                            match direction {
                                Direction::Up => idx.saturating_add(1) % cue_size,
                                Direction::Down => {
                                    (cue_size as i16 + idx as i16 - 1) as usize % cue_size
                                }
                            },
                        );
                    }
                }
            }
        },
        Err(e) => {
            state.command_error = Some(e.to_string());
        }
    }
}
