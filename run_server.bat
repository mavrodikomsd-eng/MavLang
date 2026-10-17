@echo off
setlocal
set PATH=C:\Users\admin\.lmstudio\extensions\backends\llama.cpp-win-x86_64-nvidia-cuda12-avx2-2.58.0;%PATH%
llama-server.exe -m "C:\models\qwen2.5-coder-0.5b-instruct-f16.gguf" -c 2048 --gpu-layers 99 --port 8080
pause
