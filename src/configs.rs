use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    // maximum alloted time for a prompt to finish before it's killed
    timeout: f64
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            timeout: 120.0,
        }
    }
}



#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LlamaConfig {
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
}

impl Default for LlamaConfig {
    fn default() -> Self {
        return Self {
            tokens_to_predict: -1,
            threads_used: 12,
            layers_offloaded_to_gpu: 32,
            prompt_context_size: 2048,
            randomness: 0.75,
            repeat_penalty: 1.15,
        }
    }
}

impl LlamaConfig {
    pub fn to_args(&self) -> Vec<String> {
        let mut arguments: Vec<String> = Vec::new();
        arguments.push("--n-predict".to_string());
        arguments.push(self.tokens_to_predict.to_string());
        arguments.push("--threads".to_string());
        arguments.push(self.threads_used.to_string());
        arguments.push("--n-gpu-layers".to_string());
        arguments.push(self.layers_offloaded_to_gpu.to_string());
        arguments.push("--ctx-size".to_string());
        arguments.push(self.prompt_context_size.to_string());
        arguments.push("--temp".to_string());
        arguments.push(self.randomness.to_string());
        arguments.push("--repeat-penalty".to_string());
        arguments.push(self.repeat_penalty.to_string());
        arguments
    }
}