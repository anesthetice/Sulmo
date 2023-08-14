#!/bin/bash

# a very simple script to install and rename llama.cpp on linux
# please follow the instructions at https://github.com/ggerganov/llama.cpp if you wish to install this on windows or macOS

git clone https://github.com/ggerganov/llama.cpp.git
cd llama.cpp
make
cd ..
mv llama.cpp llama-cpp