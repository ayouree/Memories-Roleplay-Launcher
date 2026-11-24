# Memories Roleplay Launcher (Tauri)
Prasyarat build: Node 18+, Rust (rustup, toolchain MSVC + Visual Studio Build Tools "Desktop development with C++"), WebView2 (sudah ada di Windows 10/11 modern).

    npm install
    npm run dev                  # development
    npm test                     # cargo test: keybind (rbind aman), XML GTA (sampel asli), parser crash
    npm run build                # installer NSIS: src-tauri/target/release/bundle/nsis/
                                 # exe portable : src-tauri/target/release/memories-launcher.exe

Semua UI ada di src/, semua logika sistem di src-tauri/src/ (Rust). Konfigurasi gerbang/tautan: src/shared/app-config.json (dibaca UI dan Rust).

## Keputusan teknis
- Nickname: dibaca/ditulis di LevelDB FiveM (data/nui-storage/Local Storage/leveldb). Key dicari dari data (bukan dikarang), pembacaan dari salinan sementara, penulisan hanya jika FiveM tertutup, dengan backup + verifikasi + pemulihan otomatis.
- Lokasi FiveM: pilihan pemain > default > registry (fivem://). Cache/crash mengikuti lokasi itu.
- Connect: lewat explorer.exe; cadangan metode "Langsung" (+connect) di Pengaturan > FiveM.
- Ping: TCP connect tiap 5 detik (berhenti saat jendela tersembunyi). Port di app-config.json.
- Memori: Tauri/WebView2 (tanpa Chromium bawaan), WebView2 tanpa GPU, heap JS dibatasi.
