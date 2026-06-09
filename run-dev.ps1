$env:BINDGEN_EXTRA_CLANG_ARGS = "--target=x86_64-pc-windows-msvc"
$env:LIBCLANG_PATH = "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\Llvm\x64\bin"
$env:CUDACXX = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.0\bin\nvcc.exe"
$env:CUDAFLAGS = "--allow-unsupported-compiler"
$env:GGML_CUDA_ARCHITECTURES = "75;80;86;87;88;89;90"

# Copy bundled model to app data dir for dev mode (Tauri doesn't extract resources in `tauri dev`)
$modelSrc = Join-Path $PSScriptRoot "apps\desktop\src-tauri\models\ggml-large-v3-turbo.bin"
$modelDst = Join-Path $env:APPDATA "contextflow\ggml-large-v3-turbo.bin"
if ((Test-Path -LiteralPath $modelSrc) -and -not (Test-Path -LiteralPath $modelDst)) {
    $null = New-Item -ItemType Directory -Force -Path (Split-Path -Parent $modelDst)
    Copy-Item -LiteralPath $modelSrc -Destination $modelDst
    Write-Output "Copied model to $modelDst"
}

cmd.exe /c '"C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat" && cargo clean -p whisper-rs-sys && pnpm tauri dev'
