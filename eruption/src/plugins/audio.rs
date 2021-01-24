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

use lazy_static::lazy_static;
use log::*;
use mlua::prelude::*;
use parking_lot::{Mutex, RwLock};
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};
use std::sync::Arc;
use std::{
    any::Any,
    time::{Duration, Instant},
};

use crate::events;
use crate::plugins::{self, Plugin};

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum AudioPluginError {
    #[error("Pulse Audio error: {description}")]
    PulseError { description: String },

    #[error("File I/O error: {description}")]
    IoError { description: String },

    #[error("Playback error: {description}")]
    PlaybackError { description: String },

    #[error("Audio grabber error: {description}")]
    GrabberError { description: String },
}

/// How many sound effects may be played back simultaneously
pub const MAX_IN_FLIGHT_SFX: usize = 2;

/// The allocated size of the audio grabber buffer
pub const AUDIO_GRABBER_BUFFER_SIZE: usize = 44100 * 2 / 16;

/// Number of FFT frequency buckets of the spectrum analyzer
pub const FFT_SIZE: usize = 512;

/// Thread termination request flag of the audio grabber thread
pub static AUDIO_GRABBER_THREAD_SHALL_TERMINATE: AtomicBool = AtomicBool::new(false);

// Used as a flag whether the 'audio/grabber' thread is running
static AUDIO_GRABBER_THREAD_RUNNING: AtomicBool = AtomicBool::new(false);

/// Number of currently playing sound effects
pub static ACTIVE_SFX: AtomicUsize = AtomicUsize::new(0);

/// Running average of the loudness of the signal in the audio grabber buffer
static CURRENT_RMS: AtomicIsize = AtomicIsize::new(0);

static ERROR_RATE_LIMIT_MILLIS: u64 = 10000;

lazy_static! {
    /// Pluggable audio backend. Currently supported backends are "Null" and PulseAudio
    pub static ref AUDIO_BACKEND: Arc<Mutex<Option<Box<dyn backends::AudioBackend + 'static + Sync + Send>>>> =
        // Arc::new(Mutex::new(backends::PulseAudioBackend::new().expect("Could not instantiate the audio backend!")));
        Arc::new(Mutex::new(None));

    /// Do not spam the logs on error, limit the amount of error messages per time unit
    static ref RATE_LIMIT_TIME: Arc<RwLock<Instant>> = Arc::new(RwLock::new(Instant::now().checked_sub(Duration::from_millis(ERROR_RATE_LIMIT_MILLIS)).unwrap_or_else(|| Instant::now())));

    /// Holds audio data recorded by the audio grabber
    static ref AUDIO_GRABBER_BUFFER: Arc<RwLock<Vec<i16>>> = Arc::new(RwLock::new(vec![0; AUDIO_GRABBER_BUFFER_SIZE / 2]));

    /// Spectrum analyzer state
    static ref AUDIO_SPECTRUM: Arc<RwLock<Vec<f32>>> = Arc::new(RwLock::new(vec![0.0; FFT_SIZE / 2]));

    /// Global "sound effects enabled" flag
    pub static ref ENABLE_SFX: AtomicBool = AtomicBool::new(false);

    // Sound FX audio buffers
    /// Key down SFX
    pub static ref SFX_KEY_DOWN: Option<Vec<u8>> = util::load_sfx("key-down.wav").ok();
    /// Key up SFX
    pub static ref SFX_KEY_UP: Option<Vec<u8>> = util::load_sfx("key-up.wav").ok();
}

// Enable computation of RMS and Spectrum Analyzer data?
static AUDIO_GRABBER_PERFORM_RMS_COMPUTATION: AtomicBool = AtomicBool::new(false);
static AUDIO_GRABBER_PERFORM_FFT_COMPUTATION: AtomicBool = AtomicBool::new(false);

pub fn reset_audio_backend() {
    AUDIO_GRABBER_THREAD_SHALL_TERMINATE.store(true, Ordering::SeqCst);
    // AUDIO_BACKEND.lock().take();

    AUDIO_GRABBER_PERFORM_RMS_COMPUTATION.store(false, Ordering::SeqCst);
    AUDIO_GRABBER_PERFORM_FFT_COMPUTATION.store(false, Ordering::SeqCst);

    *RATE_LIMIT_TIME.write() = Instant::now()
        .checked_sub(Duration::from_millis(ERROR_RATE_LIMIT_MILLIS))
        .unwrap();
}

fn try_start_audio_backend() -> Result<()> {
    // AUDIO_GRABBER_THREAD_SHALL_TERMINATE.store(false, Ordering::SeqCst);
    // AUDIO_GRABBER_THREAD_RUNNING.store(false, Ordering::SeqCst);

    AUDIO_BACKEND
        .lock()
        .replace(Box::new(backends::PulseAudioBackend::new().map_err(
            |e| {
                *RATE_LIMIT_TIME.write() = Instant::now();

                error!("Could not initialize the audio backend: {}", e);
                e
            },
        )?));

    Ok(())
}

fn try_start_audio_grabber() -> Result<()> {
    let start_backend = AUDIO_BACKEND.lock().is_none();
    if start_backend {
        try_start_audio_backend()?;
    }

    // start the audio grabber thread
    if let Some(backend) = AUDIO_BACKEND.lock().as_ref() {
        backend.start_audio_grabber()?;
        Ok(())
    } else {
        Err(AudioPluginError::GrabberError {
            description: "Audio backend not initialized".into(),
        }
        .into())
    }
}

/// A plugin that performs audio-related tasks like playing or capturing sounds
pub struct AudioPlugin {}

impl AudioPlugin {
    pub fn new() -> Self {
        AudioPlugin {}
    }

    pub fn get_audio_loudness() -> isize {
        AUDIO_GRABBER_PERFORM_RMS_COMPUTATION.store(true, Ordering::Relaxed);

        if !AUDIO_GRABBER_THREAD_RUNNING.load(Ordering::SeqCst) {
            if RATE_LIMIT_TIME.read().elapsed().as_millis() > ERROR_RATE_LIMIT_MILLIS as u128 {
                try_start_audio_grabber()
                    .unwrap_or_else(|e| error!("Could not start the audio grabber: {}", e));
            }
        }

        CURRENT_RMS.load(Ordering::SeqCst)
    }

    pub fn get_audio_spectrum() -> Vec<f32> {
        AUDIO_GRABBER_PERFORM_FFT_COMPUTATION.store(true, Ordering::Relaxed);

        if !AUDIO_GRABBER_THREAD_RUNNING.load(Ordering::SeqCst) {
            if RATE_LIMIT_TIME.read().elapsed().as_millis() > ERROR_RATE_LIMIT_MILLIS as u128 {
                try_start_audio_grabber()
                    .unwrap_or_else(|e| error!("Could not start the audio grabber: {}", e));
            }
        }

        AUDIO_SPECTRUM.read().clone()
    }

    pub fn get_audio_raw_data() -> Vec<i16> {
        if !AUDIO_GRABBER_THREAD_RUNNING.load(Ordering::SeqCst) {
            if RATE_LIMIT_TIME.read().elapsed().as_millis() > ERROR_RATE_LIMIT_MILLIS as u128 {
                try_start_audio_grabber()
                    .unwrap_or_else(|e| error!("Could not start the audio grabber: {}", e));
            }
        }

        AUDIO_GRABBER_BUFFER.read().to_vec()
    }

    pub fn get_audio_volume() -> isize {
        let start_backend = AUDIO_BACKEND.lock().is_none();
        if start_backend {
            try_start_audio_backend().unwrap_or_else(|e| error!("{}", e));
        }

        if let Some(backend) = &*AUDIO_BACKEND.lock() {
            backend.get_master_volume().unwrap_or(0) * 100 / std::u16::MAX as isize
        } else {
            0
        }
    }
}

#[async_trait::async_trait]
impl Plugin for AudioPlugin {
    fn get_name(&self) -> String {
        "Audio".to_string()
    }

    fn get_description(&self) -> String {
        "Audio related functions".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        events::register_observer(|event: &events::Event| {
            match event {
                events::Event::KeyDown(_index) => {
                    if ENABLE_SFX.load(Ordering::SeqCst)
                        && SFX_KEY_DOWN.is_some()
                        && ACTIVE_SFX.load(Ordering::SeqCst) <= MAX_IN_FLIGHT_SFX
                    {
                        let mut start_backend = false;

                        if let Some(backend) = AUDIO_BACKEND.lock().as_ref() {
                            backend
                                .play_sfx(&SFX_KEY_DOWN.as_ref().unwrap())
                                .unwrap_or_else(|e| error!("{}", e));
                        } else {
                            start_backend = true;
                        }

                        if start_backend {
                            try_start_audio_backend()?;
                        }
                    }
                }

                events::Event::KeyUp(_index) => {
                    if ENABLE_SFX.load(Ordering::SeqCst)
                        && SFX_KEY_UP.is_some()
                        && ACTIVE_SFX.load(Ordering::SeqCst) <= MAX_IN_FLIGHT_SFX
                    {
                        let mut start_backend = false;

                        if let Some(backend) = AUDIO_BACKEND.lock().as_ref() {
                            backend
                                .play_sfx(&SFX_KEY_UP.as_ref().unwrap())
                                .unwrap_or_else(|e| error!("{}", e));
                        } else {
                            start_backend = true;
                        }

                        if start_backend {
                            try_start_audio_backend()?;
                        }
                    }
                }

                _ => (),
            };

            Ok(true) // event has been processed
        });

        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()> {
        let globals = lua_ctx.globals();

        let get_audio_loudness =
            lua_ctx.create_function(move |_, ()| Ok(AudioPlugin::get_audio_loudness()))?;
        globals.set("get_audio_loudness", get_audio_loudness)?;

        let get_audio_spectrum =
            lua_ctx.create_function(move |_, ()| Ok(AudioPlugin::get_audio_spectrum()))?;
        globals.set("get_audio_spectrum", get_audio_spectrum)?;

        let get_audio_raw_data =
            lua_ctx.create_function(move |_, ()| Ok(AudioPlugin::get_audio_raw_data()))?;
        globals.set("get_audio_raw_data", get_audio_raw_data)?;

        let get_audio_volume =
            lua_ctx.create_function(move |_, ()| Ok(AudioPlugin::get_audio_volume()))?;
        globals.set("get_audio_volume", get_audio_volume)?;

        Ok(())
    }

    async fn main_loop_hook(&self, _ticks: u64) {}

    fn sync_main_loop_hook(&self, _ticks: u64) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

mod util {
    use super::AudioPluginError;
    use super::Result;
    use byteorder::{LittleEndian, WriteBytesExt};
    use std::path::{Path, PathBuf};

    pub fn load_sfx<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
        #[cfg(debug_assertions)]
        let prefix = PathBuf::from("support/sfx");

        #[cfg(not(debug_assertions))]
        let prefix = PathBuf::from("/usr/share/eruption/sfx");

        let mut reader = hound::WavReader::open(prefix.join(path)).map_err(|e| {
            AudioPluginError::PlaybackError {
                description: format!("Could not load waveform audio file: {}", e),
            }
        })?;

        let buffer: Vec<i16> = reader
            .samples::<i16>()
            .map(|s| s.expect("Could not read sample data!"))
            .collect();

        let mut writer: Vec<u8> = vec![];
        for s in buffer {
            writer
                .write_i16::<LittleEndian>(s)
                .map_err(|e| AudioPluginError::IoError {
                    description: format!("{}", e),
                })?;
        }

        Ok(writer)
    }
}

mod backends {
    use super::AudioPluginError;
    use super::Result;
    use super::ACTIVE_SFX;
    use super::AUDIO_GRABBER_BUFFER;
    use super::AUDIO_GRABBER_BUFFER_SIZE;
    use super::AUDIO_GRABBER_THREAD_RUNNING;
    use super::AUDIO_GRABBER_THREAD_SHALL_TERMINATE;
    use super::AUDIO_SPECTRUM;
    use super::CURRENT_RMS;
    use super::ENABLE_SFX;
    use super::FFT_SIZE;

    use log::*;
    use std::sync::Arc;
    use std::{sync::atomic::Ordering, thread};

    use libpulse_binding as pulse;
    use libpulse_simple_binding as psimple;
    use psimple::Simple;
    use pulse::sample;
    use pulse::stream::Direction;
    use pulsectl::controllers::DeviceControl;
    use pulsectl::controllers::SinkController;

    use rustfft::num_complex::Complex;
    use rustfft::Fft;
    use rustfft::{algorithm::Radix4, FftDirection};
    use std::f32::consts::PI;

    /// Audio backend trait, defines an interface to the player and
    /// grabber functionality
    pub trait AudioBackend {
        fn play_sfx(&self, data: &'static [u8]) -> Result<()>;
        fn start_audio_grabber(&self) -> Result<()>;

        fn get_master_volume(&self) -> Result<isize>;
    }

    /// An audio backend that does nothing
    pub struct NullBackend {}

    impl AudioBackend for NullBackend {
        fn play_sfx(&self, _data: &'static [u8]) -> Result<()> {
            Ok(())
        }

        fn start_audio_grabber(&self) -> Result<()> {
            Ok(())
        }

        fn get_master_volume(&self) -> Result<isize> {
            Ok(0)
        }
    }

    /// PulseAudio backend
    pub struct PulseAudioBackend {
        handle: Arc<psimple::Simple>,
    }

    #[allow(unused)]
    impl PulseAudioBackend {
        pub fn new() -> Result<Self> {
            let handle = Arc::new(Self::init_playback()?);
            let result = PulseAudioBackend { handle };

            Ok(result)
        }

        pub fn init_playback() -> Result<psimple::Simple> {
            let spec = sample::Spec {
                format: sample::Format::S16NE,
                channels: 2,
                rate: 44100,
            };

            assert!(spec.is_valid());

            let result = Simple::new(
                None,
                "eruption",
                Direction::Playback,
                None,
                "Keyboard Effects",
                &spec,
                None,
                None,
            )
            .map_err(|e| AudioPluginError::PulseError {
                description: format!("Could not open Pulse Audio: {}", e),
            })?;

            Ok(result)
        }

        pub fn init_grabber() -> Result<psimple::Simple> {
            let spec = sample::Spec {
                format: sample::Format::S16NE,
                channels: 2,
                rate: 44100,
            };

            assert!(spec.is_valid());

            let result = Simple::new(
                None,
                "eruption",
                Direction::Record,
                None,
                "Audio Grabber",
                &spec,
                None,
                None,
            )
            .map_err(|e| AudioPluginError::PulseError {
                description: format!("Could not open Pulse Audio: {}", e),
            })?;

            Ok(result)
        }
    }

    impl AudioBackend for PulseAudioBackend {
        fn play_sfx(&self, data: &'static [u8]) -> Result<()> {
            if !ENABLE_SFX.load(Ordering::SeqCst) {
                return Ok(());
            }

            let pa = self.handle.clone();

            let builder = thread::Builder::new().name("audio/playback".into());
            builder
                .spawn(move || {
                    ACTIVE_SFX.fetch_add(1, Ordering::SeqCst);

                    pa.write(&data)
                        .map_err(|e| AudioPluginError::PlaybackError {
                            description: format!("Error during writing of playback buffer: {}", e),
                        })
                        .unwrap();
                    pa.drain()
                        .map_err(|e| AudioPluginError::PlaybackError {
                            description: format!("Error during playback: {}", e),
                        })
                        .ok();

                    ACTIVE_SFX.fetch_sub(1, Ordering::SeqCst);
                })
                .unwrap_or_else(|e| {
                    error!("Could not spawn a thread: {}", e);
                    panic!()
                });

            Ok(())
        }

        fn start_audio_grabber(&self) -> Result<()> {
            if AUDIO_GRABBER_THREAD_RUNNING.load(Ordering::SeqCst) {
                return Err(AudioPluginError::GrabberError {
                    description: "Thread already running".into(),
                }
                .into());
            }

            AUDIO_GRABBER_THREAD_RUNNING.store(true, Ordering::SeqCst);

            let builder = thread::Builder::new().name("audio/grabber".into());
            builder
                .spawn(move || -> Result<()> {
                    let grabber = Self::init_grabber().unwrap();

                    'RECORDER_LOOP: loop {
                        let mut tmp: Vec<u8> = vec![0; AUDIO_GRABBER_BUFFER_SIZE];

                        grabber
                            .read(&mut tmp)
                            .map_err(|e| AudioPluginError::GrabberError {
                                description: format!("Error during recording: {}", e),
                            })?;

                        let mut buffer = AUDIO_GRABBER_BUFFER.write();
                        buffer.clear();
                        buffer.reserve(AUDIO_GRABBER_BUFFER_SIZE);
                        buffer.extend(
                            tmp.chunks_exact(2)
                                .map(|c| i16::from_ne_bytes([c[0], c[1]])),
                        );

                        // compute root mean square (RMS) of the recorded samples
                        if super::AUDIO_GRABBER_PERFORM_RMS_COMPUTATION.load(Ordering::Relaxed) {
                            let sqr_sum = buffer
                                .iter()
                                .map(|s| *s as f32)
                                .fold(0.0, |sqr_sum, s| sqr_sum + s * s);

                            let sqr_sum = (sqr_sum / buffer.len() as f32).sqrt();

                            CURRENT_RMS.store(sqr_sum.round() as isize, Ordering::SeqCst);
                        }

                        // compute spectrum analyzer
                        if super::AUDIO_GRABBER_PERFORM_FFT_COMPUTATION.load(Ordering::Relaxed) {
                            let mut data: Vec<Complex<f32>> = buffer
                                .iter()
                                .take(FFT_SIZE)
                                .map(|e| Complex::from(*e as f32))
                                .collect();

                            let fft = Radix4::new(FFT_SIZE, FftDirection::Forward);
                            fft.process(&mut data);

                            // apply post processing steps: normalization, window function and smoothing
                            let one_over_fft_len_sqrt = 1.0 / ((FFT_SIZE / 2) as f32).sqrt();

                            let mut phase = 0.0;
                            const DELTA: f32 = (2.0 * PI) / (FFT_SIZE / 2) as f32;

                            let result: Vec<f32> = data[(FFT_SIZE / 2)..]
                                .iter()
                                // normalize
                                .map(|e| ((e.re as f32) * one_over_fft_len_sqrt).abs())
                                // apply Hamming window
                                .map(|e| {
                                    phase += DELTA;
                                    e * (0.54 - 0.46 * phase.cos())
                                })
                                .collect();

                            for (i, e) in AUDIO_SPECTRUM.write().iter_mut().enumerate() {
                                *e = (*e + result[i]) / 2.0;
                            }
                        }

                        if AUDIO_GRABBER_THREAD_SHALL_TERMINATE.load(Ordering::SeqCst) {
                            AUDIO_GRABBER_THREAD_SHALL_TERMINATE.store(false, Ordering::SeqCst);
                            AUDIO_GRABBER_THREAD_RUNNING.store(false, Ordering::SeqCst);

                            break 'RECORDER_LOOP;
                        }
                    }

                    Ok(())
                })
                .unwrap_or_else(|e| {
                    error!("Could not spawn a thread: {}", e);
                    panic!()
                });

            Ok(())
        }

        fn get_master_volume(&self) -> Result<isize> {
            let mut handler = SinkController::create();
            let result = handler
                .get_default_device()
                .map_err(|_e| AudioPluginError::PulseError {
                    description: "Could not query PulseAudio".to_owned(),
                })?
                .volume
                .avg()
                .0;

            Ok(result as isize)
        }
    }
}
