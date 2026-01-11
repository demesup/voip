use std::sync::atomic::{AtomicU16, Ordering::Relaxed};
use std::sync::{Arc, Mutex};

use crate::jitter::JitterBuffer;
use crate::packet::AudioPacket;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamError};
use tokio::sync::broadcast::Sender;

pub struct AudioState {
    host: Host,
    input: Option<Stream>,
    output: Option<Stream>,
}

impl AudioState {
    pub fn new(host: Host) -> Self {
        Self {
            host,
            input: None,
            output: None,
        }
    }

    pub fn start(
        &mut self,
        input_channel: Sender<Vec<u8>>,
        output_jitter: Arc<Mutex<JitterBuffer>>,
    ) {
        let output_device = match self.host.default_output_device() {
            Some(dev) => dev,
            None => {
                eprintln!("No output device available");
                return;
            }
        };

        let input_device = self.host.default_input_device();

       
        if let Some(ref input_dev) = input_device {
            if let (Ok(input_name), Ok(output_name)) =
                (input_dev.name(), output_device.name()) {
                if input_name == output_name {
                    log::warn!("Input and output devices are the same. This may cause audio feedback.");
                }
            }
        }

        self.output = output_stream_fn(output_device, output_jitter.clone()).unwrap_or(None);
        if let Some(input_device) = input_device {
            self.input = input_stream_fn(input_device, input_channel).unwrap_or(None);
        } else {
            self.input = None;
        }

        if self.output.is_none() {
            self.clear();
            eprintln!("Failed to create output stream");
        }
    }

    pub fn clear(&mut self) {
        self.input = None;
        self.output = None;
    }
}

pub fn input_stream_fn(
    input_device: Device,
    channel: Sender<Vec<u8>>,
) -> Result<Option<Stream>, ()> {
    let config = match input_device.default_input_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Input config error: {}", e);
            return Err(());
        }
    };

    log::info!("Using input config: {:?}", config);

    let seq = AtomicU16::new(0);
    let sample_buffer = Arc::new(Mutex::new(Vec::new()));
    let last_log = Arc::new(Mutex::new(std::time::Instant::now()));

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            let last_log_clone = last_log.clone();
            input_device.build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    let mut buffer = sample_buffer.lock().unwrap();
                    for &s in data {
                        buffer.push((s * i16::MAX as f32) as i16);
                    }
                    while buffer.len() >= 960 {
                        let chunk: Vec<i16> = buffer.drain(0..960).collect();
                        let max_sample = chunk.iter().map(|s| s.abs()).max().unwrap_or(0);
                        let is_silence = max_sample < 100;

                        let packet = AudioPacket {
                            seq: seq.fetch_add(1, Relaxed),
                            samples: chunk,
                        };

                       
                        let mut last = last_log_clone.lock().unwrap();
                        if last.elapsed().as_secs() >= 2 {
                            log::info!("🎤 Input: Seq {} | Max: {} | {}",
                                packet.seq,
                                max_sample,
                                if is_silence { "🔇 SILENCE" } else { "🔊 AUDIO" }
                            );
                            *last = std::time::Instant::now();
                        }

                        let _ = channel.send(packet.serialize());
                    }
                },
                err_fn,
                None,
            )
        },
        cpal::SampleFormat::I16 => input_device.build_input_stream(
            &config.into(),
            move |data: &[i16], _| {
                let mut buffer = sample_buffer.lock().unwrap();
                for &s in data {
                    buffer.push(s);
                }
                while buffer.len() >= 960 {
                    let chunk: Vec<i16> = buffer.drain(0..960).collect();
                    let packet = AudioPacket {
                        seq: seq.fetch_add(1, Relaxed),
                        samples: chunk,
                    };
                    let _ = channel.send(packet.serialize());
                }
            },
            err_fn,
            None,
        ),
        cpal::SampleFormat::U16 => input_device.build_input_stream(
            &config.into(),
            move |data: &[u16], _| {
                let mut buffer = sample_buffer.lock().unwrap();
                for &s in data {
                    buffer.push((s as i32 - 32768) as i16);
                }
                while buffer.len() >= 960 {
                    let chunk: Vec<i16> = buffer.drain(0..960).collect();
                    let packet = AudioPacket {
                        seq: seq.fetch_add(1, Relaxed),
                        samples: chunk,
                    };
                    let _ = channel.send(packet.serialize());
                }
            },
            err_fn,
            None,
        ),
        _ => {
            eprintln!("Unsupported sample format: {:?}", config.sample_format());
            return Err(());
        }
    };

    match stream {
        Ok(s) => {
            if let Err(e) = s.play() {
                eprintln!("Failed to play input stream: {}", e);
                return Err(());
            }
            log::info!("Sending audio...");
            Ok(Some(s))
        }
        Err(e) => {
            eprintln!("Input stream error: {}", e);
            Err(())
        }
    }
}

pub fn output_stream_fn(
    output_device: Device,
    buffer: Arc<Mutex<JitterBuffer>>,
) -> Result<Option<Stream>, ()> {
    let config = match output_device.default_output_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Output config error: {}", e);
            return Err(());
        }
    };

    log::info!("Using output config: {:?}", config);

    let last_log = Arc::new(Mutex::new(std::time::Instant::now()));
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            let last_log_clone = last_log.clone();
            output_device.build_output_stream(
                &config.into(),
                move |data: &mut [f32], _| {
                    let mut jb = buffer.lock().unwrap();
                    let buffer_size = jb.buffer.len();
                    let mut max_sample = 0i16;

                    for s in data {
                        let sample = jb.pop_sample();
                        max_sample = max_sample.max(sample.abs());
                        *s = sample as f32 / i16::MAX as f32;
                    }

                    let is_silence = max_sample < 100;

                   
                    let mut last = last_log_clone.lock().unwrap();
                    if last.elapsed().as_secs() >= 2 {
                        log::info!("🔊 Output: Buffer {} samples | Max: {} | {}",
                            buffer_size,
                            max_sample,
                            if is_silence { "🔇 SILENCE" } else { "🎵 PLAYING" }
                        );
                        *last = std::time::Instant::now();
                    }
                },
                err_fn,
                None,
            )
        },
        cpal::SampleFormat::I16 => output_device.build_output_stream(
            &config.into(),
            move |data: &mut [i16], _| {
                let mut jb = buffer.lock().unwrap();
                for s in data {
                    *s = jb.pop_sample();
                }
            },
            err_fn,
            None,
        ),
        cpal::SampleFormat::U16 => output_device.build_output_stream(
            &config.into(),
            move |data: &mut [u16], _| {
                let mut jb = buffer.lock().unwrap();
                for s in data {
                    let sample = jb.pop_sample();
                    *s = (sample as i32 + 32768) as u16;
                }
            },
            err_fn,
            None,
        ),
        _ => {
            eprintln!("Unsupported sample format: {:?}", config.sample_format());
            return Err(());
        }
    };

    match stream {
        Ok(s) => {
            if let Err(e) = s.play() {
                eprintln!("Failed to play output stream: {}", e);
                return Err(());
            }
            log::info!("Receiving audio...");
            Ok(Some(s))
        }
        Err(e) => {
            eprintln!("Output stream error: {}", e);
            Err(())
        }
    }
}

fn err_fn(err: StreamError) {
    eprintln!("Stream error: {}", err);
}