use crate::dmx_types::DMX_CHANNELS;

pub fn mix_executor_outputs(state: &mut crate::ui::ConsoleState) {
    for ch in &mut state.channels {
        *ch = 0;
    }

    for exec in &mut state.executors {
        exec.update_fade();

        if exec.current_output_level > 0.0 && !exec.cue_list.is_empty() {
            for (i, &level) in exec.stored_channels.iter().enumerate() {
                if i < state.channels.len() {
                    let mixed =
                        (state.channels[i] as f32) + (level as f32 * exec.current_output_level);
                    state.channels[i] = mixed.min(255.0) as u8;
                }
            }
        }
    }

    let master = state.master_dimmer;
    if master < 1.0 {
        for ch in &mut state.channels {
            *ch = (*ch as f32 * master) as u8;
        }
    }

    if let Some(ref mut dmx) = state.dmx_serial {
        let channels_array: [u8; DMX_CHANNELS] = state
            .channels
            .clone()
            .try_into()
            .unwrap_or([0; DMX_CHANNELS]);
        let _ = dmx.set_channels(channels_array);
        match dmx.check_agent() {
            Ok(()) => state.dmx_connected = true,
            Err(_) => state.dmx_connected = false,
        }
    }
}
