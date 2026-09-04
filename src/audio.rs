use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample};

type SharedAudioBuffer = Arc<Mutex<VecDeque<f32>>>;

pub struct AudioOutput {
    pub stream: cpal::Stream,
    pub sample_rate: u32,
}

pub fn build_audio_stream(buffer: SharedAudioBuffer) -> AudioOutput {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");
    let config = device.default_output_config().expect("no default config");
    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;
    // Wait for about two video frames of audio before beginning. If the
    // emulator falls behind, rebuffer instead of alternating sound and zeros.
    let prebuffer_samples = (sample_rate as usize / 30).max(1);
    let stream_config = config.config();

    let stream = match config.sample_format() {
        SampleFormat::F32 => build_stream::<f32>(&device, &stream_config, buffer, channels, prebuffer_samples),
        SampleFormat::I16 => build_stream::<i16>(&device, &stream_config, buffer, channels, prebuffer_samples),
        SampleFormat::U16 => build_stream::<u16>(&device, &stream_config, buffer, channels, prebuffer_samples),
        format => panic!("unsupported output sample format: {format:?}"),
    };

    stream.play().expect("failed to start stream");
    AudioOutput { stream, sample_rate }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    buffer: SharedAudioBuffer,
    channels: usize,
    prebuffer_samples: usize,
) -> cpal::Stream
where
    T: Sample + SizedSample + FromSample<f32>,
{
    let mut playing = false;
    device.build_output_stream(
        config,
        move |data: &mut [T], _| {
            let mut buf = buffer.lock().unwrap();
            if !playing && buf.len() >= prebuffer_samples {
                playing = true;
            }

            for frame in data.chunks_mut(channels) {
                let sample = if playing {
                    match buf.pop_front() {
                        Some(sample) => sample,
                        None => {
                            playing = false;
                            0.0
                        }
                    }
                } else {
                    0.0
                };
                frame.fill(T::from_sample(sample));
            }
        },
        |err| eprintln!("audio stream error: {err}"),
        None,
    ).expect("failed to build output stream")
}
