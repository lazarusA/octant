//! Animation playback and timestep navigation state.

#[derive(Debug)]
pub struct PlaybackState {
    pub is_playing: bool,
    pub playback_fps: f32,
    pub loop_playback: bool,
    pub enable_prefetch: bool,
    pub last_step_time: web_time::Instant,
    pub current_timestep: usize,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            is_playing: false,
            playback_fps: 15.0,
            loop_playback: true,
            enable_prefetch: true,
            last_step_time: web_time::Instant::now(),
            current_timestep: 0,
        }
    }
}
