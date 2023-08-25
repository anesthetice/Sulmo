### Sulmo - llama.cpp tui written in rust

A pretty whacky barebones terminal user interface for prompting GGUF models using llama.cpp

built in rust using ratatui

### installation

#### for linux (from source)
1. clone this repository, and open a terminal in the directory
2. execute the "llama-install.sh" script

#### for windows (from source)
1. clone this repository, and open a terminal in the directory
2. build the application by running "cargo build --release" in your terminal
3. download and build llama.cpp inside the directory (https://github.com/ggerganov/llama.cpp)

### finding models

You can assemble GGUF models yourself, or you can simply find them online, a great place to start would be Hugging Face : https://huggingface.co

A special thanks to Hugging Face user TheBloke https://huggingface.co/TheBloke for his amazing work.

Lastly I recommend downloading Q5_K_M models as they seem to have the best size to quality ratio.