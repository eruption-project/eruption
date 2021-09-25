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
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::Arc;
use std::{
    any::Any,
    time::{Duration, Instant},
};

use crate::events;
use crate::plugins::{self, Plugin};

pub mod protocol {
    include!(concat!(env!("OUT_DIR"), "/audio_proxy.rs"));
}

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum AudioPluginError {
    #[error("Audio grabber error: {description}")]
    GrabberError { description: String },
}

/// The allocated size of the audio grabber buffer
pub const AUDIO_GRABBER_BUFFER_SIZE: usize = 512;

/// Number of FFT frequency buckets of the spectrum analyzer
pub const FFT_SIZE: usize = 512;

/// Running average of the loudness of the signal in the audio grabber buffer
static CURRENT_RMS: AtomicIsize = AtomicIsize::new(0);

static ERROR_RATE_LIMIT_MILLIS: u64 = 10000;

lazy_static! {
    /// Pluggable audio backends. Currently supported backends are "Null" and "ProxyBackend"
    pub static ref AUDIO_BACKEND: Arc<Mutex<Option<Box<dyn backends::AudioBackend + 'static + Sync + Send>>>> =
        Arc::new(Mutex::new(None));

    /// Do not spam the logs on error, limit the amount of error messages per time unit
    static ref RATE_LIMIT_TIME: Arc<RwLock<Instant>> = Arc::new(RwLock::new(Instant::now().checked_sub(Duration::from_millis(ERROR_RATE_LIMIT_MILLIS)).unwrap_or_else(Instant::now)));

    /// Holds audio data recorded by the audio grabber
    static ref AUDIO_GRABBER_BUFFER: Arc<RwLock<Vec<i16>>> = Arc::new(RwLock::new(vec![0; AUDIO_GRABBER_BUFFER_SIZE]));

    /// Spectrum analyzer state
    static ref AUDIO_SPECTRUM: Arc<RwLock<Vec<f32>>> = Arc::new(RwLock::new(vec![0.0; FFT_SIZE / 2]));

    /// Global "sound effects enabled" flag
    pub static ref ENABLE_SFX: AtomicBool = AtomicBool::new(false);
}

// Enable computation of RMS and Spectrum Analyzer data?
static AUDIO_GRABBER_PERFORM_RMS_COMPUTATION: AtomicBool = AtomicBool::new(false);
static AUDIO_GRABBER_PERFORM_FFT_COMPUTATION: AtomicBool = AtomicBool::new(false);

pub fn reset_audio_backend() {
    AUDIO_GRABBER_PERFORM_RMS_COMPUTATION.store(false, Ordering::SeqCst);
    AUDIO_GRABBER_PERFORM_FFT_COMPUTATION.store(false, Ordering::SeqCst);

    *RATE_LIMIT_TIME.write() = Instant::now()
        .checked_sub(Duration::from_millis(ERROR_RATE_LIMIT_MILLIS))
        .unwrap();
}

fn try_start_audio_backend() -> Result<()> {
    AUDIO_BACKEND
        .lock()
        .replace(Box::new(backends::ProxyBackend::new().map_err(|e| {
            *RATE_LIMIT_TIME.write() = Instant::now();

            error!("Could not initialize the audio backend: {}", e);
            e
        })?));

    Ok(())
}

fn start_audio_proxy_thread() -> Result<()> {
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

        CURRENT_RMS.load(Ordering::SeqCst)
    }

    pub fn get_audio_spectrum() -> Vec<f32> {
        AUDIO_GRABBER_PERFORM_FFT_COMPUTATION.store(true, Ordering::Relaxed);

        AUDIO_SPECTRUM.read().clone()
    }

    pub fn get_audio_raw_data() -> Vec<i16> {
        AUDIO_GRABBER_BUFFER.read().to_vec()
    }

    pub fn get_audio_volume() -> isize {
        if let Some(backend) = &*AUDIO_BACKEND.lock() {
            backend.get_master_volume().unwrap_or(0) * 100 / std::u16::MAX as isize
        } else {
            0
        }
    }

    pub fn is_audio_muted() -> bool {
        if let Some(backend) = &*AUDIO_BACKEND.lock() {
            backend.is_audio_muted().unwrap_or(true)
        } else {
            false
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

    async fn initialize(&mut self) -> plugins::Result<()> {
        start_audio_proxy_thread()?;

        events::register_observer(|event: &events::Event| {
            match event {
                events::Event::KeyDown(_index) => {
                    if ENABLE_SFX.load(Ordering::SeqCst) {
                        if let Some(_backend) = AUDIO_BACKEND.lock().as_ref() {
                            // backend
                            //     .play_sfx(SFX_KEY_DOWN.as_ref().unwrap())
                            //     .unwrap_or_else(|e| error!("{}", e));
                        }
                    }
                }

                events::Event::KeyUp(_index) => {
                    if ENABLE_SFX.load(Ordering::SeqCst) {
                        if let Some(_backend) = AUDIO_BACKEND.lock().as_ref() {
                            // backend
                            //     .play_sfx(SFX_KEY_UP.as_ref().unwrap())
                            //     .unwrap_or_else(|e| error!("{}", e));
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

        let is_audio_muted =
            lua_ctx.create_function(move |_, ()| Ok(AudioPlugin::is_audio_muted()))?;
        globals.set("is_audio_muted", is_audio_muted)?;

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

mod backends {
    use crate::constants;
    use crate::plugins::audio::protocol;

    use super::AudioPluginError;
    use super::Result;
    use super::AUDIO_GRABBER_BUFFER;
    use super::AUDIO_GRABBER_BUFFER_SIZE;
    use super::AUDIO_SPECTRUM;
    use super::CURRENT_RMS;
    use super::FFT_SIZE;

    use lazy_static::lazy_static;
    use log::*;
    use nix::unistd::unlink;
    use parking_lot::Mutex;
    use prost::Message;
    use std::fs;
    use std::io::Read;
    use std::io::Write;
    use std::os::unix::net::UnixListener;
    use std::os::unix::prelude::PermissionsExt;
    use std::path::Path;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::AtomicI32;
    use std::sync::Arc;
    use std::{sync::atomic::Ordering, thread};

    use rustfft::num_complex::Complex;
    use rustfft::Fft;
    use rustfft::{algorithm::Radix4, FftDirection};
    use std::f32::consts::PI;

    use protocol::response::Payload;

    lazy_static! {
        pub static ref LISTENER: Arc<Mutex<Option<UnixListener>>> = Arc::new(Mutex::new(None));

        /// Audio device master volume
        static ref MASTER_VOLUME: AtomicI32 = AtomicI32::new(0);

        /// Audio device master volume
        static ref AUDIO_MUTED: AtomicBool = AtomicBool::new(false);
    }

    /// Audio backend trait, defines an interface to the player and
    /// grabber functionality
    pub trait AudioBackend {
        fn play_sfx(&self, data: &'static [u8]) -> Result<()>;
        fn start_audio_grabber(&self) -> Result<()>;

        fn get_master_volume(&self) -> Result<isize>;

        fn is_audio_muted(&self) -> Result<bool>;
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

        fn is_audio_muted(&self) -> Result<bool> {
            Ok(false)
        }
    }

    /// An audio backend that uses the eruption-audio-proxy
    pub struct ProxyBackend {}

    impl ProxyBackend {
        pub fn new() -> Result<Self> {
            // unlink any leftover audio socket
            let _result = unlink(constants::AUDIO_SOCKET_NAME)
                .map_err(|e| debug!("Unlink of audio socket failed: {}", e));

            // bind and store audio socket
            let listener = UnixListener::bind(Path::new(constants::AUDIO_SOCKET_NAME))?;

            listener.set_nonblocking(false)?;

            // widen permissions of audio socket, so that all users may connect to it
            let mut perms = fs::metadata(constants::AUDIO_SOCKET_NAME)?.permissions();
            perms.set_mode(0o666);
            fs::set_permissions(constants::AUDIO_SOCKET_NAME, perms)?;

            LISTENER.lock().replace(listener);

            Ok(Self {})
        }

        fn run_io_loop() -> Result<()> {
            let mut cntr = 0;

            'IO_LOOP: loop {
                if crate::QUIT.load(Ordering::SeqCst) {
                    break 'IO_LOOP;
                }

                if let Some(listener) = LISTENER.lock().as_ref() {
                    listener.set_nonblocking(false)?;

                    match listener.accept() {
                        Ok((mut stream, _addr)) => {
                            info!("Audio proxy connected");

                            stream.set_nonblocking(false)?;

                            'RECEIVE_LOOP: loop {
                                let mut tmp = bytes::BytesMut::new();
                                tmp.resize(constants::BUFFER_CAPACITY + 64, 0x00);

                                match stream.read(&mut tmp) {
                                    Ok(0) => {
                                        info!("Audio proxy disconnected");

                                        break 'RECEIVE_LOOP;
                                    }

                                    Ok(n) => {
                                        debug!("Read {} bytes from audio socket", n);

                                        let result =
                                            protocol::Response::decode_length_delimited(&mut tmp);
                                        match result {
                                            Ok(response) => match response.response_type() {
                                                protocol::response::ResponseType::AudioData => {
                                                    trace!("Received audio data");

                                                    if let Some(Payload::Data(tmp)) = response.payload {
                                                        let mut buffer =
                                                            AUDIO_GRABBER_BUFFER.write();
                                                        buffer.clear();

                                                        buffer
                                                            .reserve(AUDIO_GRABBER_BUFFER_SIZE / 2);
                                                        buffer.extend(tmp.chunks_exact(2).map(
                                                            |c| i16::from_ne_bytes([c[0], c[1]]),
                                                        ));

                                                        if buffer.len() < FFT_SIZE {
                                                            buffer.resize(FFT_SIZE, 0x0000);
                                                        }

                                                        // compute root mean square (RMS) of the recorded samples
                                                        if super::AUDIO_GRABBER_PERFORM_RMS_COMPUTATION
                                                            .load(Ordering::Relaxed)
                                                        {
                                                            let sqr_sum = buffer
                                                                .iter()
                                                                .map(|s| *s as f32)
                                                                .fold(0.0, |sqr_sum, s| sqr_sum + s * s);

                                                            let sqr_sum =
                                                                (sqr_sum / buffer.len() as f32).sqrt();

                                                            CURRENT_RMS.store(
                                                                sqr_sum.round() as isize,
                                                                Ordering::SeqCst,
                                                            );
                                                        }

                                                        // compute spectrum analyzer
                                                        if super::AUDIO_GRABBER_PERFORM_FFT_COMPUTATION
                                                            .load(Ordering::Relaxed)
                                                        {
                                                            let mut data: Vec<Complex<f32>> = buffer
                                                                .iter()
                                                                .take(FFT_SIZE)
                                                                .map(|e| Complex::from(*e as f32))
                                                                .collect();

                                                            let fft = Radix4::new(
                                                                FFT_SIZE,
                                                                FftDirection::Forward,
                                                            );
                                                            fft.process(&mut data);

                                                            // apply post processing steps: normalization, window function and smoothing
                                                            let one_over_fft_len_sqrt =
                                                                1.0 / ((FFT_SIZE / 2) as f32).sqrt();

                                                            let mut phase = 0.0;
                                                            const DELTA: f32 =
                                                                (2.0 * PI) / (FFT_SIZE / 2) as f32;

                                                            let result: Vec<f32> = data[(FFT_SIZE / 2)..]
                                                                .iter()
                                                                // normalize
                                                                .map(|e| {
                                                                    ((e.re as f32) * one_over_fft_len_sqrt)
                                                                        .abs()
                                                                })
                                                                // apply Hamming window
                                                                .map(|e| {
                                                                    phase += DELTA;
                                                                    e * (0.54 - 0.46 * phase.cos())
                                                                })
                                                                .collect();

                                                            for (i, e) in AUDIO_SPECTRUM
                                                                .write()
                                                                .iter_mut()
                                                                .enumerate()
                                                            {
                                                                *e = (*e + result[i]) / 2.0;
                                                            }
                                                        }
                                                    } else {
                                                        error!("Invalid payload received");
                                                    };
                                                }

                                                protocol::response::ResponseType::AudioVolume => {
                                                    if let Some(Payload::Volume(val)) = response.payload {
                                                        MASTER_VOLUME.store(val, Ordering::SeqCst);
                                                    } else {
                                                        error!("Invalid payload received");
                                                    };
                                                }

                                                protocol::response::ResponseType::AudioMutedState => {
                                                    if let Some(Payload::Muted(val)) = response.payload {
                                                        AUDIO_MUTED.store(val, Ordering::SeqCst);
                                                    } else {
                                                        error!("Invalid payload received");
                                                    };
                                                }

                                                // _ => {
                                                //     error!("Protocol error");
                                                // }
                                            },

                                            Err(e) => {
                                                error!("Protocol error: {}", e);

                                                // break 'RECEIVE_LOOP;
                                            }
                                        }
                                    }

                                    Err(_e) => {
                                        return Err(AudioPluginError::GrabberError {
                                            description: "Lost connection to proxy".to_owned(),
                                        }
                                        .into());
                                    }
                                }

                                if crate::QUIT.load(Ordering::SeqCst) {
                                    break 'RECEIVE_LOOP;
                                }

                                // TODO: implement some kind of message queuing mechanism for
                                // ProxyCommand::GetMasterVolume and ProxyCommand::IsAudioMuted
                                if cntr % 150 == 0 {
                                    let mut command = protocol::Command::default();
                                    command.set_command_type(
                                        protocol::command::CommandType::AudioVolume,
                                    );

                                    let mut buf = command.encode_length_delimited_to_vec();
                                    buf.resize(constants::BUFFER_CAPACITY + 64, 0x00);

                                    match stream.write_all(&buf) {
                                        Ok(_n) => {}

                                        Err(_e) => {
                                            return Err(AudioPluginError::GrabberError {
                                                description: "Lost connection to proxy".to_owned(),
                                            }
                                            .into());
                                        }
                                    }

                                    stream.flush()?;
                                }

                                if cntr % 200 == 0 {
                                    let mut command = protocol::Command::default();
                                    command.set_command_type(
                                        protocol::command::CommandType::AudioMutedState,
                                    );

                                    let mut buf = command.encode_length_delimited_to_vec();
                                    buf.resize(constants::BUFFER_CAPACITY + 64, 0x00);

                                    match stream.write_all(&buf) {
                                        Ok(_n) => {}

                                        Err(_e) => {
                                            return Err(AudioPluginError::GrabberError {
                                                description: "Lost connection to proxy".to_owned(),
                                            }
                                            .into());
                                        }
                                    }

                                    stream.flush()?;
                                }

                                cntr += 1;
                            }
                        }

                        Err(_e) => {
                            return Err(AudioPluginError::GrabberError {
                                description: "Lost connection to proxy".to_owned(),
                            }
                            .into());
                        }
                    }
                }
            }

            Ok(())
        }
    }

    impl AudioBackend for ProxyBackend {
        fn play_sfx(&self, _data: &'static [u8]) -> Result<()> {
            Ok(())
        }

        fn start_audio_grabber(&self) -> Result<()> {
            let builder = thread::Builder::new().name("audio/proxy".into());
            builder
                .spawn(move || {
                    Self::run_io_loop().unwrap_or_else(|e| {
                        error!("Error occurred in audio proxy thread: {}", e);
                    });
                })
                .unwrap_or_else(|e| {
                    error!("Could not spawn a thread: {}", e);
                    panic!()
                });

            Ok(())
        }

        fn get_master_volume(&self) -> Result<isize> {
            Ok(MASTER_VOLUME.load(Ordering::SeqCst) as isize)
        }

        fn is_audio_muted(&self) -> Result<bool> {
            Ok(AUDIO_MUTED.load(Ordering::SeqCst))
        }
    }
}
