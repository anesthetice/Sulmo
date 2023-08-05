use std::{
    process::{Command, Stdio, Child, ChildStdout},
    io::Read,
    path::PathBuf,
};
use unicode_segmentation::UnicodeSegmentation;
use crate::configs::{LlamaConfig, AppConfig};

pub struct Conversation {
    pub model: PathBuf,
    pub chunks: Vec<ConversationChunk>,
    pub stripped: bool,
    pub buffer:  [u8; 2048],
    pub child: Option<(Child, ChildStdout)>,
}

impl Conversation {
    pub fn new(model: PathBuf) -> Self {
        Self {
            model: model,
            chunks: vec![ConversationChunk::new()],
            stripped: false,
            buffer: [0; 2048],
            child: None,
        }
    }
    // returns the chunk the user can modifiy
    fn usr_chunk(&mut self) -> &mut ConversationChunk {
        &mut self.chunks[self.chunks.len()-1]
    }
    // returns the chunk that could be being processed if it exists
    fn pro_chunk(&mut self) -> Option<&mut ConversationChunk> {
        let length: usize = self.chunks.len();
        if length > 1 {
            Some(&mut self.chunks[length-2])
        } else {
            None
        }
    }
    pub fn run(&mut self, llama_config: &LlamaConfig, app_config: &AppConfig) {
        if self.child.is_none() {
            let mut args: Vec<String> = llama_config.to_args();
            let mut chunk = self.usr_chunk();

            chunk.input = llama_config.to_prompt(&chunk.raw_input);

            args.push("--model".to_string()); args.push(self.model.to_str().unwrap().to_string());
            args.push("--prompt".to_string()); args.push(chunk.input.clone());
            let mut child = Command::new("llama-cpp/main")
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
                .expect("failed to execute llama-cpp/main");
            let child_stdout = child.stdout.take().unwrap();
            self.child = Some((child, child_stdout));
            self.chunks.push(ConversationChunk::new());
        }
    }
    pub fn check(&mut self) {
        if let Some(child) = self.child.as_mut() {
            match child.1.read(&mut self.buffer) {
                Ok(0) => self.child = None,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&self.buffer[..n]);
                    self.chunk.output.push_str(&chunk);
                    // a not very clean way of removing the echoed instruction fed to the LLM
                    
                    if !self.stripped && self.chunk.output.len() >= self.chunk.input.len() {
                        self.chunk.output = self.chunk.output[self.chunk.raw_input.len()..].to_string();
                        self.stripped=true;
                    }
                },
                Err(error) => panic!("{}", error),
            }
        }
    }
    pub fn get_input(&self) -> &str {
        self.chunk.raw_input.as_str()
    }
    pub fn pop_back_input(&mut self) {
        if !self.chunk.raw_input.is_empty() {
            match self.chunk.raw_input.graphemes(true).last() {
                Some(cluster) => {self.chunk.raw_input = self.chunk.raw_input.strip_suffix(cluster).unwrap().to_string()},
                None => (),
            }
        }
    }
    pub fn get_output(&self) -> &str {
        self.chunk.output.as_str()
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