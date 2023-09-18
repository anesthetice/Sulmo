use serde::{Deserialize, Serialize};
use serde_json::{self};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use sysinfo::{RefreshKind, SystemExt};

use crate::{conversation::ConversationChunk, utils::pathbuf_helper};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    // maximum alloted time for a prompt to finish before it's killed
    pub timeout: f64,

    // tick rate (in ms) that dictates how quickly how certain aspects of the application are refreshed
    pub tick_rate: u64,

    // how much time (in ms) should the TUI startup be delayed by
    pub startup_freeze: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            timeout: 420.0,
            tick_rate: 200,
            startup_freeze: 1000,
        }
    }
}

impl AppConfig {
    const FILEPATH: &'static str = "./configs/sulmo.conf";
    fn to_pretty_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
    pub fn from_file() -> Option<Self> {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .open(Self::FILEPATH)
            .ok()?;
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer).ok()?;
        serde_json::from_slice::<Self>(&buffer).ok()
    }
    pub fn save(&self) -> std::io::Result<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(Self::FILEPATH)?;
        file.write_all(self.to_pretty_json().as_bytes())?;
        Ok(())
    }
    pub fn to_print(&self) -> Vec<String> {
        vec![
            format!("generation timeout           :    '{}'", self.timeout),
            format!("tick rate                    :    '{}'", self.tick_rate),
        ]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelConfig {
    // -n N, --n-predict N
    tokens_to_predict: i32,

    // -t N, --threads N
    threads_used: u8,

    // -ngl N, --n-gpu-layers N
    layers_offloaded_to_gpu: u8,

    // -c N, --ctx-size N
    prompt_context_size: u16,

    // --temp
    randomness: f64,

    // --repeat-penalty N
    repeat_penalty: f64,

    // text that is always added before a prompt, space not included
    prompt_prefix: String,

    // text that is always added after a prompt, space not included
    prompt_suffix: String,

    // should the text that is added before and after a prompt be displayed
    pub ps_displayed: bool,

    // anything extra (i.e. --tfs 0.95)
    other: String,

    past_chunks: Vec<ConversationChunk>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            tokens_to_predict: -1,
            threads_used: {
                let mut info = sysinfo::System::new();
                info.refresh_cpu();
                // uses half the threads by default
                info.physical_core_count().unwrap_or(2_usize) as u8
            },
            layers_offloaded_to_gpu: 32,
            prompt_context_size: 2048,
            randomness: 0.75,
            repeat_penalty: 1.15,
            prompt_prefix: String::from("###Instruction: "),
            prompt_suffix: String::from(" ###Response: "),
            ps_displayed: false,
            other: String::from(""),
            past_chunks: Vec::new(),
        }
    }
}

impl ModelConfig {
    pub const DEFAULT_FILEPATH: &'static str = "./configs/model.conf";

    pub fn to_args(&self) -> Vec<String> {
        let mut args = vec![
            "--n-predict".to_string(),
            self.tokens_to_predict.to_string(),
            "--threads".to_string(),
            self.threads_used.to_string(),
            "--n-gpu-layers".to_string(),
            self.layers_offloaded_to_gpu.to_string(),
            "--ctx-size".to_string(),
            self.prompt_context_size.to_string(),
            "--temp".to_string(),
            self.randomness.to_string(),
            "--repeat-penalty".to_string(),
            self.repeat_penalty.to_string(),
        ];
        if !self.other.is_empty() {
            self.other
                .split(' ')
                .for_each(|slice| args.push(slice.to_string()));
        }
        args
    }
    fn to_pretty_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
    pub fn default_from_file() -> Option<Self> {
        Self::from_file(Self::DEFAULT_FILEPATH)
    }
    pub fn from_file<P: AsRef<Path>>(filepath: P) -> Option<Self> {
        let mut file = fs::OpenOptions::new().read(true).open(filepath).ok()?;
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer).ok()?;
        serde_json::from_slice::<Self>(&buffer).ok()
    }
    pub fn save<P: AsRef<Path>>(&self, filepath: P) -> std::io::Result<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filepath)?;
        file.write_all(self.to_pretty_json().as_bytes())?;
        Ok(())
    }
    pub fn to_prompt(&self, prompt: &str) -> String {
        format!("{}{}{}", self.prompt_prefix, prompt, self.prompt_suffix)
    }
    pub fn to_print(&self) -> Vec<String> {
        vec![
            format!(
                "tokens to predict            :    '{}'",
                self.tokens_to_predict
            ),
            format!("threads used                 :    '{}'", self.threads_used),
            format!(
                "layers offloaded to gpu      :    '{}'",
                self.layers_offloaded_to_gpu
            ),
            format!(
                "prompt context size          :    '{}'",
                self.prompt_context_size
            ),
            format!("randomness                   :    '{}'", self.randomness),
            format!(
                "repeat penalty               :    '{}'",
                self.repeat_penalty
            ),
            format!("prompt prefix                :    '{}'", self.prompt_prefix),
            format!("prompt suffix                :    '{}'", self.prompt_suffix),
            format!("prefix/suffix displayed      :    '{}'", self.ps_displayed),
            format!("other arguments              :    '{}'", self.other),
        ]
    }
    pub fn try_update<P: AsRef<Path>>(&mut self, model_filepath: P, other: &[ConversationChunk]) {
        if other != self.past_chunks {
            self.past_chunks = other.to_vec();
            if let Some(filepath) = pathbuf_helper(
                model_filepath.as_ref(),
                &PathBuf::from("./configs/"),
                "conf",
            ) {
                let _ = self.save(filepath);
            }
        }
    }
    pub fn get_past_chunks(&self) -> Vec<ConversationChunk> {
        self.past_chunks.clone()
    }
}
