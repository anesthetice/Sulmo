### Sulmo - llama.cpp tui written in rust

A pretty whacky barebones terminal user interface for prompting GGUF models using llama.cpp

built in rust using ratatui

### installation

I recommend reading the "it doesn't work" section as well

#### for linux (from source)
1. clone this repository, and open a terminal in the directory
2. execute the "llama-install.sh" script

#### for windows (from source)
1. clone this repository, and open a terminal in the directory
2. build the application by running "cargo build --release" in your terminal
3. download and build llama.cpp inside the directory (https://github.com/ggerganov/llama.cpp)

#### for windows (from precompiled binary)
1. donwload the latest precompiled .exe version of Sulmo
2. download and build llama.cpp inside the directory (https://github.com/ggerganov/llama.cpp)

### it doesn't work

If you execute Sulmo in a terminal, it will most likely tell you exactly why it's not working, but to get you up to speed, Sulmo needs just a couple of things to work.

1. llama.cpp to be in the same directory and compiled (i.e. ./llama.cpp/main exist)
2. you have at least one model in the ./models directory (GGUF model if you are using the latest llama.cpp release, .bin files only)

And that's it, anything else will be automatically generated when launching Sulmo.

### finding models

You can assemble GGUF models yourself, or you can simply find them online, a great place to start would be Hugging Face : https://huggingface.co

A special thanks to Hugging Face user TheBloke https://huggingface.co/TheBloke for his amazing work.

Lastly I recommend downloading Q5_K_M models as they seem to have the best size to quality ratio.
