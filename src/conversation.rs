use crate::configs::{AppConfig, LlamaConfig};
use std::{
    io::Read,
    path::PathBuf,
    process::{Child, ChildStdout, Command, Stdio},
    time::{Instant, Duration},
};
use unicode_segmentation::UnicodeSegmentation;

pub struct Conversation {
    pub model: PathBuf,
    // the conversation chunk that the user can modify
    pub config: LlamaConfig,
    usr_chunk: ConversationChunk,
    // the conversation chunk that may be being processed
    pro_chunk: ConversationChunk,
    past_chunks: Vec<ConversationChunk>,
    stripped: bool,
    buffer: [u8; 2048],
    child: Option<(Child, ChildStdout, Instant)>,
}

impl Conversation {
    pub fn new(model: PathBuf, config: LlamaConfig) -> Self {
        Self {
            model,
            config,
            usr_chunk: ConversationChunk::new(),
            pro_chunk: ConversationChunk::new(),
            past_chunks: Vec::new(),
            stripped: false,
            buffer: [0; 2048],
            child: None,
        }
    }
    pub fn run(&mut self) {
        if self.child.is_none() {
            let mut args: Vec<String> = self.config.to_args();

            self.usr_chunk.input = self.config.to_prompt(&self.usr_chunk.raw_input);
            args.push("--model".to_string());
            args.push(self.model.to_str().unwrap().to_string());
            args.push("--prompt".to_string());
            args.push(self.usr_chunk.input.clone());

            let mut child = Command::new("llama-cpp/main")
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
                .expect("failed to execute llama-cpp/main");

            let child_stdout = child.stdout.take().unwrap();
            self.child = Some((child, child_stdout, Instant::now()));
            if !self.pro_chunk.raw_input.is_empty() {
                self.past_chunks.push(self.pro_chunk.clone())
            };
            self.pro_chunk = self.usr_chunk.clone();
            self.usr_chunk.clear();
            self.stripped = false;
        }
    }
    pub fn check(&mut self, app_config: &AppConfig) -> () {
        if let Some(child) = self.child.as_mut() {
            if child.2.elapsed() > Duration::from_secs_f64(app_config.timeout) {
                self.child = None;
                return;
            }
            match child.1.read(&mut self.buffer) {
                Ok(0) => self.child = None,
                Ok(n) => {
                    let text_chunk = String::from_utf8_lossy(&self.buffer[..n]);
                    self.pro_chunk.output.push_str(&text_chunk);
                    // a not very clean way of removing the echoed instruction fed to the LLM
                    if !self.stripped && self.pro_chunk.output.len() >= self.pro_chunk.input.len() {
                        let index = self.pro_chunk.input.len();
                        self.pro_chunk.output = self.pro_chunk.output[index..].to_string();
                        self.stripped = true;
                    }
                }
                Err(error) => panic!("{}", error),
            }
        }
    }
    pub fn get_usr_input(&self) -> &str {
        self.usr_chunk.raw_input.as_str()
    }
    pub fn get_pro_input(&self) -> &str {
        self.pro_chunk.raw_input.as_str()
    }
    pub fn pop_back_input(&mut self) {
        if !self.usr_chunk.raw_input.is_empty() {
            if let Some(cluster) = self.usr_chunk.raw_input.graphemes(true).last() {
                self.usr_chunk.raw_input = self
                    .usr_chunk
                    .raw_input
                    .strip_suffix(cluster)
                    .unwrap()
                    .to_string()
            }
        }
    }
    pub fn get_pro_output(&self) -> &str {
        self.pro_chunk.output.as_str()
    }
    pub fn push_char(&mut self, chr: char) {
        self.usr_chunk.raw_input.push(chr);
    }
    pub fn clear_output(&mut self) {
        self.pro_chunk.output.clear();
    }
    pub fn reset_child(&mut self) {
        self.child = None;
    }
    pub fn get_past_conversations_str(&self) -> Vec<(&str, &str)> {
        let mut vector: Vec<(&str, &str)> = Vec::new();
        self.past_chunks.iter().for_each(|chunk| {
            vector.push((chunk.raw_input.as_str(), chunk.output.as_str()));
        });
        vector
    }
    pub fn pop_front(&mut self) {
        self.child = None;
        if self.pro_chunk.is_empty() {
            self.past_chunks.pop();
        } else {
            self.pro_chunk.clear();
        }
    }
}

#[derive(Clone)]
struct ConversationChunk {
    // the input after processing
    input: String,
    // the input without any processing, what the user typed
    raw_input: String,
    // the output given by the LLM
    output: String,
}

impl ConversationChunk {
    fn new() -> Self {
        Self {
            input: String::new(),
            raw_input: String::new(),
            output: String::new(),
        }
    }
    fn clear(&mut self) {
        self.input.clear();
        self.raw_input.clear();
        self.output.clear();
    }
    fn is_empty(&self) -> bool {
        self.raw_input.is_empty() || self.output.is_empty()
    }
}
