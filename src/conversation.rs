use std::{
    process::{Command, Stdio, Child, ChildStdout},
    io::Read,
    path::PathBuf,
};
use unicode_segmentation::UnicodeSegmentation;
use crate::configs::{LlamaConfig, AppConfig};

pub struct Conversation {
    pub model: PathBuf,
    pub input: String,
    pub output: String,
    pub buffer:  [u8; 2048],
    pub child: Option<(Child, ChildStdout)>
}

impl Conversation {
    pub fn new(model: PathBuf) -> Self {
        Self {
            model: model,
            input: String::new(),
            output: String::new(),
            buffer: [0; 2048],
            child: None,
        }
    }
    pub fn run(&mut self, llama_config: &LlamaConfig, app_config: &AppConfig) {
        if self.child.is_none() {
            let mut args: Vec<String> = llama_config.to_args();
            args.push("--model".to_string()); args.push(self.model.to_str().unwrap().to_string());
            args.push("--prompt".to_string()); args.push(self.input.clone());
            let mut child = Command::new("llama-cpp/main")
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
                .expect("failed to execute llama-cpp/main");
            let child_stdout = child.stdout.take().unwrap();
            self.child = Some((child, child_stdout)); 
            self.input.clear();
        }
    }
    pub fn check(&mut self) {
        if let Some(child) = self.child.as_mut() {
            match child.1.read(&mut self.buffer) {
                Ok(0) => self.child = None,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&self.buffer[..n]);
                    self.output.push_str(&chunk);
                },
                Err(error) => panic!("{}", error),
            }
        }
    }
    pub fn get_input(&self) -> &str {
        self.input.as_str()
    }
    pub fn pop_back_input(&mut self) {
        if !self.input.is_empty() {
            match self.input.graphemes(true).last() {
                Some(cluster) => {self.input = self.input.strip_suffix(cluster).unwrap().to_string()},
                None => (),
            }
        }
    }
    pub fn get_output(&self) -> &str {
        self.output.as_str()
    }
}
