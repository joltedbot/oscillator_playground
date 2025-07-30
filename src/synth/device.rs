use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, StreamConfig, default_host};
use std::error::Error;
pub struct AudioDevice {
    sample_rate: f32,
    number_of_channels: usize,
    stream_config: StreamConfig,
    output_device: Device,
}

impl AudioDevice {
    pub fn new() -> Self {
        let host = default_host();
        let output_device = host.output_devices().unwrap().next().unwrap();
        let stream_config: StreamConfig = output_device.default_output_config().unwrap().into();
        let sample_rate = stream_config.sample_rate.0 as f32;
        let number_of_channels = stream_config.channels as usize;

        Self {
            sample_rate,
            number_of_channels,
            stream_config,
            output_device,
        }
    }

    pub fn update_audio_device(&mut self, audio_device_name: &str) -> Result<(), Box<dyn Error>> {
        let host = default_host();
        let mut device_list = host.output_devices()?;
        self.output_device = device_list
            .find(|device| device.name().unwrap() == audio_device_name)
            .ok_or("Audio device not found")?;
        self.stream_config = self.output_device.default_output_config()?.into();
        self.sample_rate = self.stream_config.sample_rate.0 as f32;
        self.number_of_channels = self.stream_config.channels as usize;
        Ok(())
    }

    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn get_number_of_channels(&self) -> usize {
        self.number_of_channels
    }

    pub fn get_stream_config(&self) -> &StreamConfig {
        &self.stream_config
    }

    pub fn get_output_device(&self) -> &Device {
        &self.output_device
    }
}
