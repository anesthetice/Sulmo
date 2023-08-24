use crate::configs::{AppConfig, LlamaConfig};
use crossterm::style::Stylize;
use std::{
    fs::{create_dir, read_dir},
    path::PathBuf,
};

/// returns a vector containing the relative paths of the models found in ./models
pub fn load_gguf_models_with_config(default_config: &LlamaConfig) -> Vec<(PathBuf, LlamaConfig)> {
    let mut gguf_models_with_config: Vec<(PathBuf, LlamaConfig)> = Vec::new();

    let models_path: PathBuf = PathBuf::from("./models");
    if models_path.is_dir() {
        match read_dir(models_path) {
            Ok(element) => {
                for entry in element.flatten() {
                    let entry = entry.path();
                    if entry
                        .to_str()
                        .is_some_and(|string| string.ends_with(".bin"))
                    {
                        println!(
                            "         Found \"{}\".",
                            entry
                                .file_name()
                                .unwrap_or("?".as_ref())
                                .to_str()
                                .unwrap_or("?")
                                .cyan()
                        );
                        let mut entry_config = PathBuf::from("./configs");
                        let mut entry_clone = entry.clone();
                        entry_clone.set_extension("conf");
                        entry_config.extend([entry_clone.file_name().unwrap_or("?".as_ref())]);
                        drop(entry_clone);
                        match LlamaConfig::from_file(&entry_config) {
                            Some(config) => {
                                println!("         -> linked with the associated config file");
                                gguf_models_with_config.push((entry, config));
                            }
                            None => match default_config.save(&entry_config) {
                                Ok(()) => {
                                    println!("         -> created and saved a new associated default config file");
                                    gguf_models_with_config.push((entry, default_config.clone()));
                                }
                                Err(error) => {
                                    println!("         -> created but did not save a new associated default config file, {}", error);
                                    gguf_models_with_config.push((entry, default_config.clone()));
                                }
                            },
                        }
                    }
                }
            }
            Err(error) => {
                println!(
                    "[{}] Failed to read the /models directory. => {}",
                    "FAILED".red(),
                    error
                );
                panic!("");
            }
        }
    } else {
        match create_dir("./models") {
            Ok(()) => (),
            Err(error) => {
                println!(
                    "[{}] Failed to find and create the /models directory. => {}",
                    "FAILED".red(),
                    error
                );
                panic!("");
            }
        }
    }
    if gguf_models_with_config.is_empty() {
        println!("[{}] Failed to find a single gguf model.", "FAILED".red())
    } else {
        println!(
            "[  {}  ] Loaded gguf models with their configurations.",
            "OK".green()
        );
    }
    gguf_models_with_config
}

pub fn load_default_llama_configuration() -> LlamaConfig {
    let configuration = match LlamaConfig::default_from_file() {
        Some(config) => config,
        None => {
            println!("         Failed to load the default llama configuration. Generating new configuration...");
            let configuration = LlamaConfig::default();
            match configuration.save(LlamaConfig::DEFAULT_FILEPATH) {
                Ok(()) => println!(
                    "[  {}  ] Created and saved a default llama configuration.",
                    "OK".green()
                ),
                Err(error) => println!(
                    "[ {} ] Created but failed to save a default llama configuration. => {}",
                    "!!!!".yellow(),
                    error
                ),
            }
            configuration
        }
    };
    println!("[  {}  ] Loaded default llama configuration.", "OK".green());
    configuration
}

pub fn load_app_configuration() -> AppConfig {
    let configs_path: PathBuf = PathBuf::from("./configs");
    if !configs_path.is_dir() {
        println!("         Failed to find the ./configs directory. Attempting to create it...");
        match std::fs::create_dir(configs_path) {
            Ok(()) => (),
            Err(error) => {
                println!(
                    "         Failed to create the ./config directory. => {}",
                    error
                );
            }
        }
    }

    let configuration = match AppConfig::from_file() {
        Some(config) => config,
        None => {
            println!(
                "         Failed to load sulmo configuration. Generating a new configuration..."
            );
            let configuration = AppConfig::default();
            match configuration.save() {
                Ok(()) => println!(
                    "[  {}  ] Created and saved sulmo configuration.",
                    "OK".green()
                ),
                Err(error) => println!(
                    "[ {} ] Created but failed to save sulmo configuration. => {}",
                    "!!!!".yellow(),
                    error
                ),
            }
            configuration
        }
    };
    println!("[  {}  ] Loaded sulmo configuration.", "OK".green());
    configuration
}

pub fn check_llama_cpp() {
    let normal_path: PathBuf = PathBuf::from("./llama-cpp/main");
    let alt_path: PathBuf = PathBuf::from("./llama.cpp/main");

    if normal_path.is_file() {
        println!("[  {}  ] Found ./llama-cpp/main", "OK".green());
    } else if alt_path.is_file() {
        match std::fs::rename("./llama.cpp", "./llama-cpp") {
            Ok(()) => println!(
                "[  {}  ] Found ./llama.cpp/main and renamed ./llama.cpp to ./llama-cpp",
                "OK".green()
            ),
            Err(error) => {
                println!("[{}] Found ./llama.cpp/main but failed to rename the folder to ./llama-cpp. => {}", "FAILED".red(), error);
                panic!("");
            }
        }
    } else {
        println!("[{}] Failed to find ./llama-cpp/main or ./llama.cpp/main, please carefully read the README", "FAILED".red());
    }
}
