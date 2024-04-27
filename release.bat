wasm-pack.exe build --release src-wasm/ --target web
DEL src-wasm\pkg\README.md
DEL src-wasm\pkg\package.json
DEL src-wasm\pkg\.gitignore

DEL /Q src-front\wasm
MKDIR src-front\wasm\

COPY src-wasm\pkg\* src-front\wasm\

cargo.exe tauri build
