//! Audio playback module

pub mod ffmpeg;
pub mod pipewire;
pub mod sinks;

pub use sinks::{AudioSink, list_sinks, move_sink_input, set_default_sink, get_own_sink_input_index};
