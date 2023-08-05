use std::{
    process::{Command, Stdio, Child, ChildStdout},
    io::Read,
    path::PathBuf,
};
use unicode_segmentation::UnicodeSegmentation;
use crate::configs::{LlamaConfig, AppConfig};

pub struct Conversation {
    pub model: PathBuf,
    usr_chunk: ConversationChunk,
    pro_chunk: ConversationChunk,
    past_chunks: Vec<ConversationChunk>,
    stripped: bool,
    buffer:  [u8; 2048],
    child: Option<(Child, ChildStdout)>,
}

impl Conversation {
    pub fn new(model: PathBuf) -> Self {
        Self {
            model: model,
            usr_chunk: ConversationChunk::new(),
            pro_chunk: ConversationChunk::new(),
            past_chunks: Vec::new(),
            stripped: false,
            buffer: [0; 2048],
            child: None,
        }
    }
    pub fn run(&mut self, llama_config: &LlamaConfig, app_config: &AppConfig) {
        if self.child.is_none() {
            let mut args: Vec<String> = llama_config.to_args();

            self.usr_chunk.input = llama_config.to_prompt(&self.usr_chunk.raw_input);
            args.push("--model".to_string()); args.push(self.model.to_str().unwrap().to_string());
            args.push("--prompt".to_string()); args.push(self.usr_chunk.input.clone());

            let mut child = Command::new("llama-cpp/main")
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
                .expect("failed to execute llama-cpp/main");

            let child_stdout = child.stdout.take().unwrap();
            self.child = Some((child, child_stdout));
            self.pro_chunk = self.usr_chunk.clone();
            self.usr_chunk.clear();
            self.stripped = false;
        }
    }
    pub fn check(&mut self) {
        if let Some(child) = self.child.as_mut() {
            match child.1.read(&mut self.buffer) {
                Ok(0) => self.child = None,
                Ok(n) => {
                    let text_chunk = String::from_utf8_lossy(&self.buffer[..n]);
                    self.pro_chunk.output.push_str(&text_chunk);
                    // a not very clean way of removing the echoed instruction fed to the LLM
                    if !self.stripped && self.pro_chunk.output.len() >= self.pro_chunk.input.len() {
                        let index = self.pro_chunk.input.len();
                        self.pro_chunk.output = self.pro_chunk.output[index..].to_string();
                        self.stripped=true;
                    }
                },
                Err(error) => panic!("{}", error),
            }
        }
    }
    pub fn get_input(&self) -> &str {
        self.usr_chunk.raw_input.as_str()
    }
    pub fn pop_back_input(&mut self) {
        if !self.usr_chunk.raw_input.is_empty() {
            match self.usr_chunk.raw_input.graphemes(true).last() {
                Some(cluster) => {self.usr_chunk.raw_input = self.usr_chunk.raw_input.strip_suffix(cluster).unwrap().to_string()},
                None => (),
            }
        }
    }
    pub fn get_output(&self) -> &str {
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
}