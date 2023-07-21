use crossterm::style::Stylize;
use std::{
    path::PathBuf,
    fs,
};

/// returns a vector containing the relative paths of the models found in ./models
pub fn load_ggml_models() -> Vec<PathBuf> {
    let mut ggml_models: Vec<PathBuf> = Vec::new();

    let models_path: PathBuf = PathBuf::from("./models");
    if models_path.is_dir() {
        match fs::read_dir(models_path) {
            Ok(element) => {
                for entry in element {
                    if entry.is_ok() {
                        let valid_entry = entry.unwrap().path();
                        if valid_entry.to_str().is_some_and(|string| {string.ends_with(".bin")}) {
                            println!("         Found {}.", valid_entry.file_name().unwrap_or("?".as_ref()).to_str().unwrap_or("?"));
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
        match fs::create_dir("./models") {
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

pub fn load_llama_configuration() -> crate::configs::LlamaConfig {
    println!("[  {}  ] Loaded llama configuration.", "OK".green());
    crate::configs::LlamaConfig::default()
}

pub fn load_app_configuration() -> crate::configs::AppConfig {
    println!("[  {}  ] Loaded app configuration.", "OK".green());
    crate::configs::AppConfig::default()
}