use crate::dmx_types::{FadeDirection, DMX_CHANNELS};

pub fn mix_executor_outputs(state: &mut crate::ui::ConsoleState) {
    let mut dmx_chans = [0u8; DMX_CHANNELS];

    // Calculate the executors values
    state.executors.iter_mut().for_each(|exec| {
        exec.update_fade();
        if exec.fader_level > 0.0 {
            if let Some(current_cue) = &exec.cue_list.get(exec.current_cue_index) {
                // Check if we should interpolate (fading and direction is set)
                if exec.is_fading {
                    if let Some(direction) = exec.last_direction {
                        // Calculate previous cue index based on direction
                        let prev_cue_idx = match direction {
                            FadeDirection::Positive => {
                                (exec.current_cue_index + exec.cue_list.len() - 1)
                                    % exec.cue_list.len()
                            }
                            FadeDirection::Negative => {
                                (exec.current_cue_index + 1) % exec.cue_list.len()
                            }
                        };

                        if let Some(prev_cue) = exec.cue_list.get(prev_cue_idx) {
                            let progress = exec.current_output_level;

                            for (idx, cue_dmx_level) in current_cue.levels.iter().enumerate() {
                                let prev_level = prev_cue.levels[idx] as f32;
                                let curr_level = *cue_dmx_level as f32;
                                let interpolated =
                                    prev_level + (curr_level - prev_level) * progress;
                                dmx_chans[idx] = (interpolated * state.master_dimmer) as u8;
                            }
                        }
                    } else {
                        // No direction set - use current cue directly (no interpolation)
                        current_cue
                            .levels
                            .iter()
                            .enumerate()
                            .for_each(|(idx, cue_dmx_level)| {
                                dmx_chans[idx] = ((*cue_dmx_level as f32
                                    * exec.current_output_level)
                                    * state.master_dimmer)
                                    as u8;
                            });
                    }
                } else {
                    // Not fading - use current cue directly
                    current_cue
                        .levels
                        .iter()
                        .enumerate()
                        .for_each(|(idx, cue_dmx_level)| {
                            dmx_chans[idx] = ((*cue_dmx_level as f32 * exec.current_output_level)
                                * state.master_dimmer)
                                as u8;
                        });
                }
            }
        }
    });

    // Buffer is sent above every dmx values
    state.buffer.iter().for_each(|v| {
        if let Some(chan) = dmx_chans.get_mut(v.chan.saturating_sub(1)) {
            *chan = v.dmx;
        }
    });

    if dmx_chans.to_vec() != state.channels {
        state.channels = dmx_chans.to_vec().clone();
        println!("Channels updated");
        if let Some(dmx) = &mut state.dmx_serial {
            dmx.set_channels(dmx_chans);
        }
    }
    if let Some(dmx) = &mut state.dmx_serial {
        // Set the serial state
        match dmx.check_agent() {
            Ok(()) => state.dmx_connected = true,
            Err(e) => {
                state.dmx_connected = false;
                state.dmx_serial_error = e.to_string();
            }
        }
    }
}
