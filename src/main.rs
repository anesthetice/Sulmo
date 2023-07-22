use crossterm::style::Stylize;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    io::{self, stdout, Read, Write},
    thread,
};

mod setup;
use setup::{
    load_ggml_models,
    load_llama_configuration, load_app_configuration,
};

mod configs;
use configs::{
    LlamaConfig,
    AppConfig,
};


fn main() {
    // setup
    println!("         Loading configurations...");
    let app_config: AppConfig = load_app_configuration();
    let llama_config: LlamaConfig = load_llama_configuration();
    println!("         Loading ggml models...");
    let ggml_models: Vec<PathBuf> =  load_ggml_models();
    println!("         Setup complete.\n\n\n");

    // choosing a model
    for (index, filepath) in ggml_models.iter().enumerate() {
        println!("{}. {}", index, filepath.file_name().unwrap_or("?".as_ref()).to_str().unwrap_or("?"))
    }
    let mut chosen_model: PathBuf = PathBuf::new();
    let mut user_input: String = String::new();
    while !chosen_model.exists() {
        stdout().write_all("Please select a model: ".as_bytes()); stdout().flush(); io::stdin().read_line(&mut user_input).expect("failed to read user input"); print!("\n");
        match user_input.trim().parse::<usize>() {
            Ok(num) => {
                if num < ggml_models.len() {
                    chosen_model = ggml_models[num].clone();
                }
            },
            Err(error) => (),
        }
        user_input.clear()
    }

    // choosing a prompt
    user_input.clear();
    stdout().write_all("Prompt:".as_bytes()); stdout().flush(); io::stdin().read_line(&mut user_input).expect("failed to read user input"); print!("\n");
    let prompt: String = format!("###Instruction: {} \\n###Response: ", user_input.trim());
    drop(user_input);

    let mut args: Vec<String> = llama_config.to_args();
    args.push("--model".to_string()); args.push(chosen_model.to_str().unwrap().to_string());
    args.push("--prompt".to_string()); args.push(prompt.to_string());

    let mut child = Command::new("llama-cpp/main")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
        .expect("failed to execute llama-cpp/main");

    let mut child_stdout = child.stdout.take().unwrap();

    let mut output_string: String = String::new();
    let mut buffer: [u8; 1024] = [0; 1024];
    while child.try_wait().unwrap().is_none() {
        match child_stdout.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buffer[..n]);
                output_string.push_str(&chunk);
            }
            Err(error) => panic!("{}", error),
        };
        stdout().write_all(output_string.as_bytes()); stdout().flush(); output_string.clear();
        thread::sleep(std::time::Duration::from_secs(1));
    }

}