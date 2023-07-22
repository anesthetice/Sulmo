use crossterm::style::Stylize;
use std::{
    path::PathBuf,
    fs::{read_dir, create_dir},
};
use crate::configs::{AppConfig,LlamaConfig};

/// returns a vector containing the relative paths of the models found in ./models
pub fn load_ggml_models() -> Vec<PathBuf> {
    let mut ggml_models: Vec<PathBuf> = Vec::new();

    let models_path: PathBuf = PathBuf::from("./models");
    if models_path.is_dir() {
        match read_dir(models_path) {
            Ok(element) => {
                for entry in element {
                    if entry.is_ok() {
                        let valid_entry = entry.unwrap().path();
                        if valid_entry.to_str().is_some_and(|string| {string.ends_with(".bin")}) {
                            println!("         Found \"{}\".", valid_entry.file_name().unwrap_or("?".as_ref()).to_str().unwrap_or("?"));
                            ggml_models.push(valid_entry);
                        }
                    }
                }
            },
            Err(error) => {
                println!("[{}] Failed to read the /models directory: {}.", "FAILED".red(), error);
                panic!("");
            },
        }
    } else {
        match create_dir("./models") {
            Ok(()) => (),
            Err(error) => {
                println!("[{}] Failed to find and create the /models directory: {}.", "FAILED".red(), error);
                panic!("");
            }
        }
    }
    if ggml_models.len() == 0 {
        println!("[ {} ] Failed to find a single ggml model.", "!!!!".yellow())
    } else {
        println!("[  {}  ] Loaded ggml models.", "OK".green());
    }
    ggml_models
}

pub fn load_llama_configuration() -> LlamaConfig {
    let configuration = match LlamaConfig::from_file() {
        Some(config) => config,
        None => {
            println!("         Failed to load llama configuration. Generating new configuration...");
            let config = LlamaConfig::default();
            match config.save() {
                Ok(()) => println!("[  {}  ] Created and saved llama configuration.", "OK".green()),
                Err(error) => println!("[ {} ] Created but failed to save llama configuration\n=> {}", "!!!!".yellow(), error),
            }
            config
        }
    };
    println!("[  {}  ] Loaded llama configuration.", "OK".green());
    configuration
}

pub fn load_app_configuration() -> AppConfig {
    let configuration = match AppConfig::from_file() {
        Some(config) => config,
        None => {
            println!("         Failed to load sulmo configuration. Generating new configuration...");
            let config = AppConfig::default();
            match config.save() {
                Ok(()) => println!("[  {}  ] Created and saved sulmo configuration.", "OK".green()),
                Err(error) => println!("[ {} ] Created but failed to save sulmo configuration\n=> {}", "!!!!".yellow(), error),
            }
            config
        }
    };
    println!("[  {}  ] Loaded sulmo configuration.", "OK".green());
    configuration
}