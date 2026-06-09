$env:BINDGEN_EXTRA_CLANG_ARGS = "--target=x86_64-pc-windows-msvc"
$env:LIBCLANG_PATH = "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\Llvm\x64\bin"
$env:CUDACXX = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.0\bin\nvcc.exe"
$env:CUDAFLAGS = "--allow-unsupported-compiler"
$env:GGML_CUDA_ARCHITECTURES = "75;80;86;87;88;89;90"
cmd.exe /c '"C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat" && pnpm tauri build'
