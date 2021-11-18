/*
    This file is part of Eruption.

    Eruption is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Eruption is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Eruption.  If not, see <http://www.gnu.org/licenses/>.
*/

use std::sync::Arc;

pub use backends::{AudioBackend, PulseAudioBackend};
use lazy_static::lazy_static;
use parking_lot::RwLock;

use crate::constants;

pub type Result<T> = std::result::Result<T, eyre::Error>;

lazy_static! {
    pub static ref AUDIO_BUFFER: Arc<RwLock<Vec<u8>>> =
        Arc::new(RwLock::new(vec![0x00; constants::AUDIO_BUFFER_SIZE]));
}

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("Connection error: {description}")]
    ConnectionError { description: String },

    // #[error("File I/O error: {description}")]
    // IoError { description: String },

    // #[error("Playback error: {description}")]
    // PlaybackError { description: String },
    #[error("Audio grabber error: {description}")]
    GrabberError { description: String },

    #[error("Audio player error: {description}")]
    PlayerError { description: String },
}

mod backends {
    use std::sync::Arc;

    use libpulse_binding::{sample, stream::Direction};
    use libpulse_simple_binding::Simple;
    use parking_lot::Mutex;
    use pulsectl::controllers::{DeviceControl, SinkController};
    use std::cell::RefCell;

    use crate::audio::AudioError;

    use super::Result;

    thread_local! {
        pub static SINK_CONTROLLER: RefCell<SinkController> = RefCell::new(SinkController::create());
    }

    pub trait AudioBackend {
        fn device_name(&self) -> Result<String>;

        fn open(&mut self) -> Result<()>;
        fn close(&mut self) -> Result<()>;

        fn get_audio_volume(&self) -> Result<i32>;
        fn set_audio_volume(&mut self, vol: i32) -> Result<()>;
        fn is_audio_muted(&self) -> Result<bool>;

        fn play_samples(&self, data: &'static [u8]) -> Result<()>;
        fn record_samples(&self) -> Result<()>;
    }

    pub struct PulseAudioBackend {
        pub recorder_handle: Arc<Mutex<Option<Simple>>>,
        pub player_handle: Arc<Mutex<Option<Simple>>>,
        pub is_open: bool,
    }

    impl PulseAudioBackend {
        pub fn new() -> Self {
            Self {
                recorder_handle: Arc::new(Mutex::new(None)),
                player_handle: Arc::new(Mutex::new(None)),
                is_open: false,
            }
        }
    }

    impl AudioBackend for PulseAudioBackend {
        fn device_name(&self) -> Result<String> {
            Ok("PulseAudio/PipeWire Device".to_string())
        }

        fn open(&mut self) -> Result<()> {
            let spec = sample::Spec {
                format: sample::Format::S16NE,
                channels: 2,
                rate: 44100,
            };

            assert!(spec.is_valid());

            let result = Simple::new(
                None,
                "Eruption",
                Direction::Record,
                Some("analog-stereo.monitor"),
                "Audio Grabber",
                &spec,
                None,
                None,
            )
            .map_err(|e| AudioError::ConnectionError {
                description: format!("Could not open PulseAudio/PipeWire recording device: {}", e),
            })?;

            *self.recorder_handle.lock() = Some(result);

            let result = Simple::new(
                None,
                "Eruption",
                Direction::Playback,
                None,
                "Audio Playback",
                &spec,
                None,
                None,
            )
            .map_err(|e| AudioError::ConnectionError {
                description: format!("Could not open PulseAudio/PipeWire playback device: {}", e),
            })?;

            *self.player_handle.lock() = Some(result);

            self.is_open = true;

            Ok(())
        }

        fn close(&mut self) -> Result<()> {
            if self.is_open {
                *self.recorder_handle.lock() = None;
                *self.player_handle.lock() = None;
                self.is_open = false;
            }

            Ok(())
        }

        fn get_audio_volume(&self) -> Result<i32> {
            SINK_CONTROLLER.with(|handler| {
                let mut handler = handler.borrow_mut();

                let result = handler
                    .get_default_device()
                    .map_err(|_e| AudioError::ConnectionError {
                        description: "Could not query PulseAudio/PipeWire".to_owned(),
                    })?
                    .volume
                    .avg()
                    .0;

                Ok(result as i32)
            })
        }

        fn set_audio_volume(&mut self, _vol: i32) -> Result<()> {
            todo!()
        }

        fn is_audio_muted(&self) -> Result<bool> {
            SINK_CONTROLLER.with(|handler| {
                let mut handler = handler.borrow_mut();

                let result = handler
                    .get_default_device()
                    .map_err(|_e| AudioError::ConnectionError {
                        description: "Could not query PulseAudio/PipeWire".to_owned(),
                    })?
                    .mute;

                Ok(result)
            })
        }

        fn play_samples(&self, data: &'static [u8]) -> Result<()> {
            let player = self.player_handle.lock();
            if let Some(player) = player.as_ref() {
                player.write(&data).map_err(|e| AudioError::PlayerError {
                    description: format!("Error during playback: {}", e),
                })?;

                Ok(())
            } else {
                Err(AudioError::PlayerError {
                    description: "Audio subsystem is not available".to_string(),
                }
                .into())
            }
        }

        fn record_samples(&self) -> Result<()> {
            let mut buf = super::AUDIO_BUFFER.write();

            let grabber = self.recorder_handle.lock();
            if let Some(grabber) = grabber.as_ref() {
                grabber
                    .read(&mut buf)
                    .map_err(|e| AudioError::GrabberError {
                        description: format!("Error during recording: {}", e),
                    })?;

                Ok(())
            } else {
                Err(AudioError::GrabberError {
                    description: "Audio subsystem is not available".to_string(),
                }
                .into())
            }
        }
    }
}
