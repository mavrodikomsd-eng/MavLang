@echo off
cd /d "C:\1\аап"
start "llama-server" "C:\Users\admin\.lmstudio\extensions\backends\llama.cpp-win-x86_64-nvidia-cuda12-avx2-2.58.0\llama-server.exe" -m "C:\models\qwen2.5-coder-0.5b-instruct-f16.gguf" -c 2048 --gpu-layers 99 --port 8080
