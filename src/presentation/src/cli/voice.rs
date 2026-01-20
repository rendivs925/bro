//! Voice input handler for CLI voice mode
//!
//! Provides voice-activated command execution using:
//! - Vosk for speech recognition
//! - Piper for text-to-speech feedback
//! - CPAL for microphone input

use infrastructure::adapters::{
    microphone::{MicrophoneCapture, MicrophoneConfig},
    tts_adapter::TtsAdapter,
    vosk_adapter::VoskAdapter,
};
use infrastructure::ollama_client::OllamaClient;
use shared::types::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Voice input handler for CLI voice mode
pub struct VoiceHandler {
    microphone: MicrophoneCapture,
    speech_recognizer: Arc<VoskAdapter>,
    tts_engine: Option<TtsAdapter>,
    ollama_client: OllamaClient,
    wake_words: Vec<String>,
    is_listening: bool,
}

impl VoiceHandler {
    /// Create a new voice handler
    pub async fn new() -> Result<Self> {
        println!("🎤 Initializing voice recognition system...");

        // Initialize microphone with voice command config
        let microphone = MicrophoneCapture::with_config(MicrophoneConfig::for_voice_commands())
            .map_err(|e| anyhow::anyhow!("Failed to initialize microphone: {}", e))?;

        println!("  ✓ Microphone initialized");

        // Try to find Vosk model
        let home_model_path = format!(
            "{}/.local/share/vosk/model",
            std::env::var("HOME").unwrap_or_default()
        );
        let model_paths = vec![
            "model/vosk-model-en-us-0.22",
            "model/vosk-model-small-en-us-0.15",
            "models/vosk-model-en-us-0.22",
            "models/vosk-model-small-en-us-0.15",
            "models/vosk-model-en-us-0.22-lgraph",
            "/usr/share/vosk/model",
            &home_model_path,
        ];

        let mut speech_recognizer = None;
        for path in &model_paths {
            if std::path::Path::new(path).exists() {
                match VoskAdapter::new(path, 16000.0) {
                    Ok(adapter) => {
                        println!("  ✓ Speech recognition loaded from {}", path);
                        speech_recognizer = Some(Arc::new(adapter));
                        break;
                    }
                    Err(e) => {
                        tracing::debug!("Failed to load Vosk model from {}: {}", path, e);
                    }
                }
            }
        }

        let speech_recognizer = speech_recognizer.ok_or_else(|| {
            anyhow::anyhow!(
                "Vosk model not found. Please download a model from https://alphacephei.com/vosk/models\n\
                 and place it in one of: {:?}",
                model_paths
            )
        })?;

        // Try to initialize TTS (optional - will work without it)
        let tts_engine = match TtsAdapter::new() {
            Ok(tts) => {
                println!("  ✓ Text-to-speech initialized");
                Some(tts)
            }
            Err(e) => {
                println!("  ⚠ Text-to-speech unavailable: {}", e);
                None
            }
        };

        // Initialize Ollama client for command interpretation
        let ollama_client = OllamaClient::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize Ollama client: {}", e))?;

        println!("  ✓ AI command interpreter ready");

        Ok(Self {
            microphone,
            speech_recognizer,
            tts_engine,
            ollama_client,
            wake_words: vec!["bro".to_string(), "hey bro".to_string()],
            is_listening: false,
        })
    }

    /// Start voice input mode
    pub async fn start_voice_mode(&mut self) -> Result<()> {
        println!();
        println!("🎤 Voice Mode Active");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Say 'bro' followed by your command");
        println!("Say 'stop', 'exit', or 'quit' to end voice mode");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();

        // Speak welcome message if TTS available
        if let Some(ref tts) = self.tts_engine {
            let _ = self
                .speak(tts, "Voice mode active. Say bro followed by your command.")
                .await;
        }

        self.is_listening = true;

        // Create channel for audio samples
        let (tx, mut rx) = mpsc::channel::<Vec<i16>>(32);

        // Start continuous microphone capture
        let _recording = self.microphone.start_continuous(move |samples| {
            let _ = tx.blocking_send(samples.to_vec());
        })?;

        println!("🎧 Listening...");

        // Main voice processing loop
        while self.is_listening {
            // Wait for audio with timeout
            match tokio::time::timeout(tokio::time::Duration::from_secs(30), rx.recv()).await {
                Ok(Some(audio_chunk)) => {
                    if let Err(e) = self.process_audio_chunk(audio_chunk).await {
                        eprintln!("Voice processing error: {}", e);
                    }
                }
                Ok(None) => {
                    // Channel closed
                    break;
                }
                Err(_) => {
                    // Timeout - just continue listening
                    println!("  (still listening...)");
                }
            }
        }

        println!();
        println!("🎤 Voice mode stopped");

        if let Some(ref tts) = self.tts_engine {
            let _ = self.speak(tts, "Voice mode ended.").await;
        }

        Ok(())
    }

    /// Process an audio chunk for voice commands
    async fn process_audio_chunk(&mut self, audio_chunk: Vec<i16>) -> Result<bool> {
        use domain::services::SpeechRecognitionService;
        use shared::types::AudioSample;

        // Skip very short audio chunks
        if audio_chunk.len() < 1600 {
            return Ok(true);
        }

        // Create audio sample for recognition
        let audio_sample = AudioSample {
            data: audio_chunk,
            sample_rate: 16000,
            channels: 1,
        };

        // Recognize speech
        let result = self.speech_recognizer.recognize(audio_sample).await?;
        let text = result.text.trim().to_lowercase();

        if text.is_empty() {
            return Ok(true); // Continue listening
        }

        println!("  Heard: \"{}\"", text);

        // Check for stop commands
        if text == "stop" || text == "exit" || text == "quit" {
            if let Some(ref tts) = self.tts_engine {
                let _ = self.speak(tts, "Stopping voice mode").await;
            }
            self.is_listening = false;
            return Ok(false);
        }

        // Check for wake word and extract command
        if let Some(command) = self.extract_command(&text) {
            println!("  Command: \"{}\"", command);

            // Process the command with AI
            let response = self.process_voice_command(&command).await?;

            println!();
            println!("{}", response);
            println!();

            // Speak the response if TTS available
            if let Some(ref tts) = self.tts_engine {
                // Truncate long responses for speech
                let speech_text = if response.len() > 500 {
                    format!("{}... Response truncated for speech.", &response[..500])
                } else {
                    response
                };
                let _ = self.speak(tts, &speech_text).await;
            }
        }

        Ok(true) // Continue listening
    }

    /// Extract command from recognized text (looking for wake word)
    fn extract_command(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();

        for wake_word in &self.wake_words {
            if text_lower.contains(wake_word) {
                if let Some(pos) = text_lower.find(wake_word) {
                    let after_wake = text[pos + wake_word.len()..].trim();
                    if !after_wake.is_empty() {
                        return Some(after_wake.to_string());
                    }
                }
            }
        }

        None
    }

    /// Process a voice command using AI
    async fn process_voice_command(&self, command: &str) -> Result<String> {
        let prompt = format!(
            r#"You are a voice assistant for a CLI tool. The user said: "{}"

Interpret this as a command and respond appropriately. If it's a question, answer it concisely.
If it's a request to run a command, explain what would happen.
Keep your response brief and suitable for text-to-speech (under 200 words).

Response:"#,
            command
        );

        let response = self.ollama_client.generate_response(&prompt).await?;
        Ok(response.trim().to_string())
    }

    /// Speak text using TTS
    async fn speak(&self, tts: &TtsAdapter, text: &str) -> Result<()> {
        use domain::services::TextToSpeechService;
        use infrastructure::adapters::audio_player::AudioPlayer;

        let samples = tts.synthesize(text, None).await?;

        // Play the audio
        let player = AudioPlayer::new()?;
        player.play_pcm_data(&samples, 22050).await?;

        Ok(())
    }

    /// Configure wake word sensitivity
    pub fn set_wake_word_sensitivity(&mut self, _sensitivity: f32) {
        // Vosk doesn't have direct sensitivity control
        // This could be implemented by adjusting VAD threshold
        tracing::info!("Wake word sensitivity adjustment not yet supported by Vosk");
    }

    /// Add custom wake words
    pub fn add_wake_words(&mut self, words: Vec<String>) {
        self.wake_words.extend(words);
        tracing::info!("Added {} custom wake words", self.wake_words.len());
    }
}

/// Voice command processing result
#[derive(Debug)]
pub struct VoiceCommandResult {
    pub text_response: Option<String>,
    pub audio_feedback: Option<String>,
    pub action_taken: bool,
}
