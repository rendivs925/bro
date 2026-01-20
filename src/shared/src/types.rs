use serde::{Deserialize, Serialize};
use std::fmt;

pub type Result<T> = anyhow::Result<T>;

// Common types
pub type CommandId = String;
pub type SessionId = String;
pub type WorkflowId = String;
pub type PluginId = String;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ScriptType {
    Bash,
    Python,
    JavaScript,
    Rust,
    Ruby,
    PowerShell,
    Custom(String),
}

impl fmt::Display for ScriptType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptType::Bash => write!(f, "bash"),
            ScriptType::Python => write!(f, "python"),
            ScriptType::JavaScript => write!(f, "javascript"),
            ScriptType::Rust => write!(f, "rust"),
            ScriptType::Ruby => write!(f, "ruby"),
            ScriptType::PowerShell => write!(f, "powershell"),
            ScriptType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

// Security levels for script execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    Sandboxed, // Restricted execution
    Trusted,   // Full access to user-approved paths
    Isolated,  // Container/VM execution
}

impl fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityLevel::Sandboxed => write!(f, "sandboxed"),
            SecurityLevel::Trusted => write!(f, "trusted"),
            SecurityLevel::Isolated => write!(f, "isolated"),
        }
    }
}

// Audio types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSample {
    pub data: Vec<i16>,
    pub sample_rate: u32,
    pub channels: u8,
}

impl AudioSample {
    pub fn new(data: Vec<i16>, sample_rate: u32, channels: u8) -> Self {
        Self {
            data,
            sample_rate,
            channels,
        }
    }

    /// Get the duration of the audio sample in seconds
    pub fn duration_seconds(&self) -> f32 {
        self.data.len() as f32 / self.sample_rate as f32 / self.channels as f32
    }

    /// Get the number of samples per channel
    pub fn samples_per_channel(&self) -> usize {
        self.data.len() / self.channels as usize
    }

    /// Convert audio to mono by averaging channels
    pub fn to_mono(&self) -> AudioSample {
        if self.channels <= 1 {
            return self.clone();
        }

        let samples_per_channel = self.samples_per_channel();
        let mut mono_data = Vec::with_capacity(samples_per_channel);

        for frame_idx in 0..samples_per_channel {
            let mut sum: i32 = 0;
            for ch in 0..self.channels as usize {
                let sample_idx = frame_idx * self.channels as usize + ch;
                sum += self.data[sample_idx] as i32;
            }
            mono_data.push((sum / self.channels as i32) as i16);
        }

        AudioSample::new(mono_data, self.sample_rate, 1)
    }

    /// Convert to 16kHz mono (optimal for speech recognition)
    pub fn to_16khz_mono(&self) -> Result<AudioSample> {
        let mono = self.to_mono();
        mono.resample(16000)
    }

    /// Resample audio to target sample rate using simple linear interpolation
    pub fn resample(&self, target_sample_rate: u32) -> Result<AudioSample> {
        if self.sample_rate == target_sample_rate {
            return Ok(self.clone());
        }

        let ratio = target_sample_rate as f32 / self.sample_rate as f32;
        let new_length = (self.data.len() as f32 * ratio) as usize;

        let mut resampled_data = Vec::with_capacity(new_length);

        for i in 0..new_length {
            let src_idx = i as f32 / ratio;
            let idx_floor = src_idx.floor() as usize;
            let idx_ceil = (idx_floor + 1).min(self.data.len() - 1);

            let fraction = src_idx - idx_floor as f32;
            let sample = self.data[idx_floor] as f32 * (1.0 - fraction)
                + self.data[idx_ceil] as f32 * fraction;

            resampled_data.push(sample as i16);
        }

        Ok(AudioSample::new(
            resampled_data,
            target_sample_rate,
            self.channels,
        ))
    }

    /// Get the RMS (Root Mean Square) level of the audio
    pub fn rms_level(&self) -> f32 {
        if self.data.is_empty() {
            return 0.0;
        }

        let sum_squares: f64 = self
            .data
            .iter()
            .map(|&x| (x as f64 / i16::MAX as f64).powi(2))
            .sum();

        (sum_squares / self.data.len() as f64).sqrt() as f32
    }
}
