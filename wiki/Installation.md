# Installation & Setup

ContextFlow requires a few prerequisites to build from source, as it relies on Rust, Node.js, and C++ build tools.

## Prerequisites
| Tool | Version | Notes |
|------|---------|-------|
| Windows 10/11 | 22H2+ | Required for WinRT speech APIs |
| Rust | stable | `rustup toolchain install stable` |
| Node.js | 20+ | LTS recommended |
| pnpm | 9+ | `npm install -g pnpm` |
| VS Build Tools | 2022+ | C++ workload + Windows 11 SDK |
| CMake | 3.x | For `whisper.cpp` |

## Setup

1. **Clone the repository:**
   ```powershell
   git clone https://github.com/your-org/contextflow.git
   cd contextflow
   ```

2. **Install Node.js dependencies:**
   ```powershell
   pnpm install
   ```

3. **Verify the Rust workspace compiles:**
   ```powershell
   cargo check --workspace
   ```

4. **Run in Development Mode:**
   ```powershell
   pnpm tauri dev
   ```
   *Note: The first launch will automatically download the whisper model (~142 MB) from HuggingFace.*

## Building for Release
Use the included build script to compile the release version of the application:
```powershell
.\build.ps1
```
