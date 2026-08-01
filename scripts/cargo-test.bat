@echo off
rem scripts/cargo-test.bat
rem
rem Runs `cargo test` for the LiteMark Rust core.
rem
rem History: the GNU toolchain (ADR 0001) needed rustup's self-contained
rem bin directory on PATH for dlltool/ld, so this wrapper prepended it. After
rem switching to the MSVC target (ADR 0003), rustc discovers the linker and SDK
rem via the standard vcvars/SDK path, so no PATH manipulation is needed and a
rem plain `cargo test` works. This wrapper is kept for convenience so existing
rem workflows (`scripts/cargo-test.bat`) keep working.

cargo test --manifest-path src-tauri/Cargo.toml %*
