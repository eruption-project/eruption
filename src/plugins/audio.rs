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

use failure::Fail;
use lazy_static::lazy_static;
use log::*;
use rlua;
use rlua::Context;
use std::any::Any;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::FFTplanner;

use crate::events;
use crate::plugins::{self, Plugin};

pub type Result<T> = std::result::Result<T, AudioPluginError>;

#[derive(Debug, Fail)]
pub enum AudioPluginError {
    //#[fail(display = "Unknown error: {}", description)]
    //UnknownError { description: String },
    #[fail(display = "Pulse Audio error: {}", description)]
    PulseError { description: String },

    #[fail(display = "File I/O error: {}", description)]
    IoError { description: String },

    #[fail(display = "Playback error: {}", description)]
    PlaybackError { description: String },

    #[fail(display = "Audio grabber error: {}", description)]
    GrabberError { description: String },
}

/// How many sound effects may be played back simultaneously
pub const MAX_IN_FLIGHT_SFX: usize = 2;

/// The allocated size of the audio grabber buffer
pub const AUDIO_GRABBER_BUFFER_SIZE: usize = 44100 / 16 / 2;

/// Thread termination request flag of the audio player thread
pub static AUDIO_PLAYBACK_THREAD_TERMINATED: AtomicBool = AtomicBool::new(false);

/// Thread termination request flag of the audio grabber thread
pub static AUDIO_GRABBER_THREAD_TERMINATED: AtomicBool = AtomicBool::new(false);

/// Number of currently playing sound effects
pub static ACTIVE_SFX: AtomicUsize = AtomicUsize::new(0);

/// Running average of the loudness of the signal in the audio grabber buffer
static CURRENT_RMS: AtomicIsize = AtomicIsize::new(0);

lazy_static! {
    /// Pluggable audio backend. Currently supported backends are "Null", ALSA and PulseAudio
    pub static ref AUDIO_BACKEND: Arc<Mutex<Option<Box<dyn backends::AudioBackend + 'static + Sync + Send>>>> =
        //Arc::new(Mutex::new(backends::AlsaBackend::new().expect("Could not instantiate the audio backend!")));
        //Arc::new(Mutex::new(backends::PulseAudioBackend::new().expect("Could not instantiate the audio backend!")));
        Arc::new(Mutex::new(None));

    /// Holds audio data recorded by the audio grabber
    static ref AUDIO_GRABBER_BUFFER: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(vec![0; AUDIO_GRABBER_BUFFER_SIZE / 2]));

    /// Global "sound effects enabled" flag
    pub static ref ENABLE_SFX: AtomicBool = AtomicBool::new(false);

    // Sound FX audio buffers
    /// Key down SFX
    pub static ref SFX_KEY_DOWN: Option<Vec<u8>> = util::load_sfx("typewriter1.wav").ok();
    /// Key up SFX
    pub static ref SFX_KEY_UP: Option<Vec<u8>> = util::load_sfx("typewriter1.wav").ok();
}

#[inline]
fn try_start_audio_backend() -> Result<()> {
    AUDIO_BACKEND
        .lock()
        .unwrap()
        .replace(Box::new(backends::PulseAudioBackend::new().map_err(|e| {
            error!("Could not initialize the audio backend: {}", e);
            e
        })?));

    Ok(())
}

#[inline]
fn try_start_audio_grabber() -> Result<()> {
    let start_backend = AUDIO_BACKEND.lock().unwrap().is_none();
    if start_backend {
        try_start_audio_backend()?;
    }

    // start the audio grabber thread
    if let Some(backend) = AUDIO_BACKEND.lock().unwrap().as_ref() {
        backend.start_audio_grabber()?;
        Ok(())
    } else {
        Err(AudioPluginError::GrabberError {
            description: "Audio backend not initialized".into(),
        })
    }
}

/// A plugin that performs audio-related tasks like playing or capturing sounds
pub struct AudioPlugin {}

impl AudioPlugin {
    pub fn new() -> Self {
        AudioPlugin {}
    }

    pub fn get_audio_loudness() -> isize {
        try_start_audio_grabber()
            .unwrap_or_else(|e| error!("Could not start the audio grabber: {}", e));

        CURRENT_RMS.load(Ordering::SeqCst)
    }

    pub fn get_audio_spectrum() -> Vec<f32> {
        try_start_audio_grabber()
            .unwrap_or_else(|e| error!("Could not start the audio grabber: {}", e));

        const FFT_STEP: usize = 1;
        const FFT_SIZE: usize = AUDIO_GRABBER_BUFFER_SIZE / 2 / FFT_STEP;

        let raw_data = Self::get_audio_raw_data();
        let mut data: Vec<Complex<f32>> = raw_data
            .iter()
            .step_by(FFT_STEP)
            .map(|e| Complex::from(*e as f32))
            .collect();
        let mut output = vec![Complex::zero(); FFT_SIZE];

        let inverse = false;
        let mut planner = FFTplanner::new(inverse);
        let fft = planner.plan_fft(FFT_SIZE);
        fft.process(&mut data, &mut output);

        let one_over_fft_len_sqrt = 1.0 / (FFT_SIZE as f32).sqrt();
        let result = output
            .iter()
            .map(|e| ((e.re as f32) * one_over_fft_len_sqrt).abs())
            .collect();

        //debug!("{:?}", result);

        result
    }

    pub fn get_audio_raw_data() -> Vec<i16> {
        try_start_audio_grabber()
            .unwrap_or_else(|e| error!("Could not start the audio grabber: {}", e));

        AUDIO_GRABBER_BUFFER.lock().unwrap().to_vec()
    }
}

impl Plugin for AudioPlugin {
    fn get_name(&self) -> String {
        "Audio".to_string()
    }

    fn get_description(&self) -> String {
        "Audio related functions".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        // NOTE: Due to limitations of the plugin system, we can not
        //       capture `self` in the event handler closure below.
        events::register_observer(events::EventClass::Keyboard, |event| {
            match *event {
                events::Event::KeyDown(_index) => {
                    if ENABLE_SFX.load(Ordering::SeqCst)
                        && SFX_KEY_DOWN.is_some()
                        && ACTIVE_SFX.load(Ordering::SeqCst) <= MAX_IN_FLIGHT_SFX
                    {
                        let mut start_backend = false;

                        if let Some(backend) = AUDIO_BACKEND.lock().unwrap().as_ref() {
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

                        if let Some(backend) = AUDIO_BACKEND.lock().unwrap().as_ref() {
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
            };

            Ok(true) // event has been processed
        });

        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: Context) -> rlua::Result<()> {
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

        Ok(())
    }

    fn main_loop_hook(&self, _ticks: u64) {}

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
    use hound;
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
    use super::AUDIO_GRABBER_THREAD_TERMINATED;
    use super::AUDIO_PLAYBACK_THREAD_TERMINATED;
    use super::CURRENT_RMS;
    use super::ENABLE_SFX;

    use lazy_static::lazy_static;
    #[allow(unused)]
    use log::{debug, error};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;

    use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
    use cpal::{StreamData, UnknownTypeInputBuffer, UnknownTypeOutputBuffer};

    use libpulse_binding as pulse;
    use libpulse_simple_binding as psimple;
    use psimple::Simple;
    use pulse::sample;
    use pulse::stream::Direction;

    /// Audio backend trait, defines an interface to the player and
    /// grabber functionality
    pub trait AudioBackend {
        fn play_sfx(&self, data: &'static [u8]) -> Result<()>;
        fn start_audio_grabber(&self) -> Result<()>;
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
    }

    /// Advanced Linux Sound Architecture backend
    pub struct AlsaBackend {
        host: Arc<cpal::Host>,
        event_loop_playback: Arc<cpal::EventLoop>,
        event_loop_grabber: Arc<cpal::EventLoop>,
    }

    #[allow(unused)]
    impl AlsaBackend {
        pub fn new() -> Result<Self> {
            let host = cpal::default_host();

            for (index, device) in host.devices().unwrap().enumerate() {
                debug!("Device: {}: {}", index, device.name().unwrap());
            }

            let event_loop_playback = host.event_loop();
            let event_loop_grabber = host.event_loop();

            let result = AlsaBackend {
                host: Arc::new(host),
                event_loop_playback: Arc::new(event_loop_playback),
                event_loop_grabber: Arc::new(event_loop_grabber),
            };

            Ok(result)
        }
    }

    #[allow(unused)]
    impl AudioBackend for AlsaBackend {
        fn play_sfx(&self, data: &'static [u8]) -> Result<()> {
            if ENABLE_SFX.load(Ordering::SeqCst) == false {
                return Ok(());
            }

            AUDIO_PLAYBACK_THREAD_TERMINATED.store(false, Ordering::SeqCst);

            let samples = data.len();

            let host = self.host.clone();
            let event_loop = self.event_loop_playback.clone();

            let builder = thread::Builder::new().name("audio/handler".into());
            builder
                .spawn(move || -> Result<()> {
                    let device = host
                        .default_output_device()
                        //.map_err(|e| AudioPluginError::PlaybackError { description: format!("{}", e) });
                        .expect("no output device available");

                    debug!("Default output device: {}", device.name().unwrap());

                    let mut supported_formats_range = device
                        .supported_output_formats()
                        .expect("error while querying formats");

                    //for f in supported_formats_range {
                    //debug!("{:?}", f);
                    //}

                    //let format = supported_formats_range.next().expect("No supported formats!").with_max_sample_rate();

                    let format = cpal::Format {
                        channels: 2,
                        sample_rate: cpal::SampleRate(44100),
                        data_type: cpal::SampleFormat::I16,
                    };

                    debug!("Default output format: {:?}", format);

                    let stream_id =
                        event_loop
                            .build_output_stream(&device, &format)
                            .map_err(|e| AudioPluginError::PlaybackError {
                                description: format!("{}", e),
                            })?;

                    let event_loop_c = event_loop.clone();
                    let stream_id_c = stream_id.clone();

                    let builder = thread::Builder::new().name("audio/playback".into());
                    builder
                        .spawn(move || {
                            ACTIVE_SFX.fetch_add(1, Ordering::SeqCst);

                            event_loop
                                .play_stream(stream_id)
                                .expect("failed to play_stream");

                            event_loop.run(move |stream_id, stream_result| {
                                let stream_data = match stream_result {
                                    Ok(data) => data,
                                    Err(err) => {
                                        eprintln!(
                                            "an error occurred on stream {:?}: {}",
                                            stream_id, err
                                        );
                                        return;
                                    }
                                };

                                if AUDIO_PLAYBACK_THREAD_TERMINATED.load(Ordering::SeqCst) == true {
                                    return;
                                }

                                match stream_data {
                                    StreamData::Output {
                                        buffer: UnknownTypeOutputBuffer::U16(mut buffer),
                                    } => {
                                        let mut di = data
                                            .chunks_exact(2)
                                            .map(|c| u16::from_ne_bytes([c[0], c[1]]));

                                        for elem in buffer.iter_mut() {
                                            *elem = di.next().unwrap_or(0);
                                        }
                                    }

                                    StreamData::Output {
                                        buffer: UnknownTypeOutputBuffer::I16(mut buffer),
                                    } => {
                                        let mut di = data
                                            .chunks_exact(2)
                                            .map(|c| u16::from_ne_bytes([c[0], c[1]]))
                                            .map(|c| (c as f32) as i16);

                                        for elem in buffer.iter_mut() {
                                            *elem = di.next().unwrap_or(0);
                                        }
                                    }

                                    StreamData::Output {
                                        buffer: UnknownTypeOutputBuffer::F32(mut buffer),
                                    } => {
                                        let mut di = data
                                            .chunks_exact(2)
                                            .map(|c| u16::from_ne_bytes([c[0], c[1]]))
                                            .map(|c| f32::from(c));

                                        for elem in buffer.iter_mut() {
                                            *elem = di.next().unwrap_or(0.0);
                                        }
                                    }

                                    _ => error!("Unsupported sample format!"),
                                };
                            });
                        })
                        .unwrap_or_else(|e| {
                            error!("Could not spawn a thread: {}", e);
                            panic!()
                        });

                    thread::sleep(std::time::Duration::from_millis(
                        ((44100 * 2) / samples * 100) as u64,
                    ));

                    //event_loop_c.pause_stream(stream_id_c);
                    event_loop_c.destroy_stream(stream_id_c);

                    AUDIO_PLAYBACK_THREAD_TERMINATED.store(true, Ordering::SeqCst);
                    ACTIVE_SFX.fetch_sub(1, Ordering::SeqCst);

                    Ok(())
                })
                .unwrap_or_else(|e| {
                    error!("Could not spawn a thread: {}", e);
                    panic!()
                });

            Ok(())
        }

        fn start_audio_grabber(&self) -> Result<()> {
            lazy_static! {
                static ref AUDIO_GRABBER_THREAD_RUNNING: AtomicBool = AtomicBool::new(false);
            }

            if AUDIO_GRABBER_THREAD_RUNNING.load(Ordering::SeqCst) {
                return Ok(());
                //return Err(AudioPluginError::GrabberError{ description: "Thread already running".into() });
            }

            let host = self.host.clone();
            let event_loop = self.event_loop_grabber.clone();

            let builder = thread::Builder::new().name("audio/grabber".into());
            builder
                .spawn(move || -> Result<()> {
                    AUDIO_GRABBER_THREAD_RUNNING.store(true, Ordering::SeqCst);

                    let device = host
                        .default_input_device()
                        .expect("Failed to get default input device");

                    debug!("Default input device: {}", device.name().unwrap());

                    let format = device
                        .default_input_format()
                        .expect("Failed to get default input format");

                    debug!("Default input format: {:?}", format);

                    let stream_id =
                        event_loop
                            .build_input_stream(&device, &format)
                            .map_err(|e| AudioPluginError::GrabberError {
                                description: format!(
                                    "Could not create audio grabber stream: {}",
                                    e
                                ),
                            })?;
                    event_loop.play_stream(stream_id).map_err(|e| {
                        AudioPluginError::GrabberError {
                            description: format!("Could not start audio grabber stream: {}", e),
                        }
                    })?;

                    event_loop.run(move |stream_id, stream_result| {
                        let stream_data = match stream_result {
                            Ok(data) => data,
                            Err(err) => {
                                eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                                return;
                            }
                        };

                        if AUDIO_GRABBER_THREAD_TERMINATED.load(Ordering::SeqCst) == true {
                            return;
                        }

                        let mut buffer = AUDIO_GRABBER_BUFFER.lock().unwrap();

                        match stream_data {
                            StreamData::Input {
                                buffer: UnknownTypeInputBuffer::U16(input_buffer),
                            } => {
                                // copy recorded samples to the global buffer
                                buffer.clear();
                                buffer.extend(input_buffer.iter().map(|s| *s as i16));
                            }

                            StreamData::Input {
                                buffer: UnknownTypeInputBuffer::I16(input_buffer),
                            } => {
                                // copy recorded samples to the global buffer
                                buffer.clear();
                                buffer.extend(input_buffer.iter());
                            }

                            StreamData::Input {
                                buffer: UnknownTypeInputBuffer::F32(input_buffer),
                            } => {
                                // copy recorded samples to the global buffer
                                buffer.clear();
                                buffer.extend(input_buffer.iter().map(|s| s.round() as i16));
                            }

                            _ => error!("Grabber: Unsupported sample format!"),
                        }

                        // compute root mean square (RMS) of the recorded samples
                        let sqr_sum = buffer
                            .iter()
                            .map(|s| *s as f32)
                            .fold(0.0, |sqr_sum, s| sqr_sum + s * s);

                        let sqr_sum = (sqr_sum / buffer.len() as f32).sqrt();

                        CURRENT_RMS.store(sqr_sum.round() as isize, Ordering::SeqCst);
                    });
                })
                .unwrap_or_else(|e| {
                    error!("Could not spawn a thread: {}", e);
                    panic!()
                });

            Ok(())
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
                format: sample::SAMPLE_S16NE,
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
                format: sample::SAMPLE_S16NE,
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
            if ENABLE_SFX.load(Ordering::SeqCst) == false {
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
                        .unwrap();

                    ACTIVE_SFX.fetch_sub(1, Ordering::SeqCst);
                })
                .unwrap_or_else(|e| {
                    error!("Could not spawn a thread: {}", e);
                    panic!()
                });

            Ok(())
        }

        fn start_audio_grabber(&self) -> Result<()> {
            lazy_static! {
                static ref AUDIO_GRABBER_THREAD_RUNNING: AtomicBool = AtomicBool::new(false);
            }

            if AUDIO_GRABBER_THREAD_RUNNING.load(Ordering::SeqCst) {
                return Ok(());
                //return Err(AudioPluginError::GrabberError{ description: "Thread already running".into() });
            }

            let builder = thread::Builder::new().name("audio/grabber".into());
            builder
                .spawn(move || -> Result<()> {
                    AUDIO_GRABBER_THREAD_RUNNING.store(true, Ordering::SeqCst);

                    let grabber = Self::init_grabber().unwrap();

                    'RECORDER_LOOP: loop {
                        let mut tmp: Vec<u8> = Vec::with_capacity(AUDIO_GRABBER_BUFFER_SIZE);
                        tmp.resize(AUDIO_GRABBER_BUFFER_SIZE, 0);

                        grabber
                            .read(&mut tmp)
                            .map_err(|e| AudioPluginError::GrabberError {
                                description: format!("Error during recording: {}", e),
                            })?;

                        let mut buffer = AUDIO_GRABBER_BUFFER.lock().unwrap();
                        buffer.clear();
                        buffer.extend(
                            tmp.chunks_exact(2)
                                .map(|c| i16::from_ne_bytes([c[0], c[1]])),
                        );

                        // compute root mean square (RMS) of the recorded samples
                        let sqr_sum = buffer
                            .iter()
                            .map(|s| *s as f32)
                            .fold(0.0, |sqr_sum, s| sqr_sum + s * s);

                        let sqr_sum = (sqr_sum / buffer.len() as f32).sqrt();

                        CURRENT_RMS.store(sqr_sum.round() as isize, Ordering::SeqCst);

                        if AUDIO_GRABBER_THREAD_TERMINATED.load(Ordering::SeqCst) {
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
    }
}
