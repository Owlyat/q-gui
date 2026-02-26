use crate::dmx_types::{AudioAction, AudioTrack};
use lofty::prelude::*;
use parking_lot::Mutex;
use rodio::{Decoder, DeviceSinkBuilder, Source};
use std::fs::File;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tween::Tweener;

pub struct AudioEngine {
    active_players: Arc<Mutex<Vec<ActivePlayback>>>,
    ended_tracks: Arc<Mutex<Vec<(u32, AudioAction)>>>,
}

struct ActivePlayback {
    track_id: u32,
    player: Arc<rodio::Player>,
    _sink: rodio::MixerDeviceSink,
    volume: f32,
    master_volume: f32,
    action: AudioAction,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            active_players: Arc::new(Mutex::new(Vec::new())),
            ended_tracks: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn get_duration(file_path: &str) -> f32 {
        if let Ok(tagged_file) = lofty::read_from_path(file_path) {
            if let Some(dur) = tagged_file.properties().duration().as_secs().checked_add(0) {
                return dur as f32;
            }
        }

        if let Ok(file) = File::open(file_path) {
            if let Ok(source) = Decoder::try_from(file) {
                return source
                    .total_duration()
                    .map(|d| d.as_secs_f32())
                    .unwrap_or(0.0);
            }
        }

        0.0
    }
    pub fn play(&self, track: &AudioTrack, master_volume: f32) -> Result<(), String> {
        self.stop(track.id);

        let file =
            File::open(&track.file_path).map_err(|e| format!("Failed to open file: {}", e))?;

        let source =
            Decoder::try_from(file).map_err(|e| format!("Failed to decode file: {}", e))?;

        let mut sink = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| format!("Failed to open audio device: {}", e))?;

        sink.log_on_drop(false);

        let mixer = sink.mixer();

        let player = rodio::Player::connect_new(mixer);
        let player_arc = Arc::new(player);
        let player_for_fade_in = Arc::clone(&player_arc);

        if track.fade_in > 0.0 {
            player_arc.set_volume(0.0);
        }

        let fade_in = track.fade_in;
        let fade_out = track.fade_out;
        let track_id = track.id;
        let action = track.action.clone();
        let ended_tracks = Arc::clone(&self.ended_tracks);

        player_arc.append(source);
        let _ = player_arc.try_seek(Duration::from_secs(track.start_point));

        if fade_in > 0.0 {
            let vol = track.volume * master_volume;
            player_for_fade_in.set_volume(0.0);
            std::thread::spawn(async move || {
                println!("Started tween");
                let mut tween = Tweener::sine_in_out(0.0, vol, fade_in);
                while !tween.is_finished() {
                    let v = tween.move_by(0.200);
                    player_for_fade_in.set_volume(v);
                    thread::sleep(Duration::from_millis(200));
                }
                println!("Finished tween");
            });
        }

        let playback = ActivePlayback {
            track_id: track.id,
            player: player_arc,
            _sink: sink,
            volume: track.volume,
            master_volume,
            action: track.action.clone(),
        };

        self.active_players.lock().push(playback);

        Ok(())
    }

    pub fn stop(&self, track_id: u32) {
        let mut players = self.active_players.lock();
        players.retain(|p| {
            if p.track_id == track_id {
                p.player.stop();
                false
            } else {
                true
            }
        });
    }

    pub fn stop_all(&self) {
        let mut players = self.active_players.lock();
        for p in players.drain(..) {
            p.player.stop();
        }
    }

    pub fn update(&self) {
        let mut players = self.active_players.lock();
        let mut ended = Vec::new();

        players.retain(|p| {
            if p.player.is_paused() {
                return true;
            }
            // Keep player if it still has audio (not empty), remove if empty (finished)
            if p.player.empty() {
                ended.push((p.track_id, p.action.clone()));
                return false;
            }
            p.player.set_volume(p.volume * p.master_volume);
            true
        });

        drop(players);

        if !ended.is_empty() {
            let mut tracks = self.ended_tracks.lock();
            tracks.extend(ended);
        }
    }

    pub fn set_master_volume(&self, volume: f32) {
        let mut players = self.active_players.lock();
        for p in players.iter_mut() {
            p.master_volume = volume;
            p.player.set_volume(p.volume * p.master_volume);
        }
    }
    pub fn get_current_playback(&self) -> Vec<f32> {
        let mut players = self.active_players.lock();
        let mut res = vec![];
        for p in players.iter_mut() {
            res.push(p.player.get_pos().as_secs_f32());
        }
        res
    }

    pub fn get_ended_tracks(&self) -> Vec<(u32, AudioAction)> {
        let mut tracks = self.ended_tracks.lock();
        let result = tracks.drain(..).collect();
        result
    }

    pub fn is_playing(&self, track_id: u32) -> bool {
        let players = self.active_players.lock();
        players
            .iter()
            .any(|p| p.track_id == track_id && !p.player.empty() && !p.player.is_paused())
    }
}
