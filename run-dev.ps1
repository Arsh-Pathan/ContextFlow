$env:BINDGEN_EXTRA_CLANG_ARGS = "--target=x86_64-pc-windows-msvc"
$env:LIBCLANG_PATH = "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\Llvm\x64\bin"
cmd.exe /c '"C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat" && cargo clean -p whisper-rs-sys && pnpm tauri dev'
