use crate::dmx_types::DMX_CHANNELS;

pub fn mix_executor_outputs(state: &mut crate::ui::ConsoleState) {
    let mut dmx_chans = [0u8; DMX_CHANNELS];

    // Calculate the executors values
    state.executors.iter_mut().for_each(|exec| {
        exec.update_fade();
        if exec.fader_level > 0.0 {
            if let Some(current_cue) = &exec.cue_list.get(exec.current_cue_index) {
                current_cue
                    .levels
                    .iter()
                    .enumerate()
                    .for_each(|(idx, cue_dmx_level)| {
                        dmx_chans[idx.saturating_sub(1)] = ((*cue_dmx_level as f32 * exec.current_output_level) // this needs to be interpolated with the value of the last cue so if the last cue was chan 5 at 150 and current is at 20, we interpolate from 150 to 20
                                * state.master_dimmer)
                            as u8;
                    });
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
