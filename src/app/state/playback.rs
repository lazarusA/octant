//! Animation playback and timestep navigation state.

#[derive(Debug)]
pub struct PlaybackState {
    pub is_playing: bool,
    pub playback_fps: f32,
    pub loop_playback: bool,
    pub enable_prefetch: bool,
    pub last_step_time: web_time::Instant,
    pub current_timestep: usize,
    /// Seconds of steps loaded ahead, in every drawn animated layer, before
    /// playback starts or resumes after reaching a step not yet loaded.
    pub buffer_secs: f32,
    /// Whether playback waits for its buffer to fill (`may_show`).
    pub buffering: bool,
    /// Steps ready ahead and steps wanted while buffering (the indicator).
    pub buffer_progress: (usize, usize),
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
            buffer_secs: 3.0,
            buffering: false,
            buffer_progress: (0, 0),
        }
    }
}
