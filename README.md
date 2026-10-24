# FilmCraft Mobile Rust

以手機與平板為優先的開源影片剪輯專案，採用 **Rust 為主，平台整合為輔**。
這是獨立的行動版實驗專案，基於 FilmCraft 的開源核心方向，非上游官方行動版。

**目前狀態：原生 Android 示範剪輯預覽版。可操作生成素材的時間軸，尚不能匯入／匯出真實影片。**

取得 APK：[Android 預覽版 Release](https://github.com/aa0968111723-prog/filmcraft-mobile-rust/releases/tag/v0.1.0-alpha.1)。
安裝與建置說明見 [docs/android-preview.md](docs/android-preview.md)，驗證範圍見 [docs/android-validation.md](docs/android-validation.md)。
iPhone / iPad 尚無安裝檔。

## 平台與語言

| 項目 | 開發方向 |
| --- | --- |
| 優先平台 | Android 手機、Android 平板 |
| 後續平台 | iPhone、iPad；需要 macOS、Xcode 與裝置驗證 |
| 剪輯邏輯、時間、專案、命令 | Rust，逐步接入 FilmCraft 核心 |
| 觸控介面 | Rust / eframe 0.36.2 / egui；Android NativeActivity + OpenGL ES 預覽宿主 |
| 硬體編解碼 | Rust 安全介面包裝 Android MediaCodec / Apple VideoToolbox，待實作 |
| 相簿、檔案選擇、分享、生命週期 | 系統 SDK；需要時使用少量 Kotlin / Swift 橋接 |
| 輔助檔案 | Gradle、Android Manifest、Xcode 專案、CI YAML 等打包設定 |

「純 Rust」是應用邏輯與可共享核心的目標，不代表沒有系統 SDK、平台函式庫或建置工具。
行動裝置的長時間剪輯應優先驗證硬體編解碼、記憶體、耗電與散熱，不能只憑 Rust 語言保證效能。

## 已完成

- Cargo workspace 與 Rust 安全規則。
- 可打包的 Rust Android App：示範素材、播放頭、裁切、分割、排序、復原／重做與本機存檔。
- 以固定 Git commit **實際依賴**原有 `filmcraft-time`，保留整數 Tick 的精確時間。
- 驗證剪裁範圍、拒絕時間溢位、計算有限的 CPU 預覽快取容量。
- 手機／平板寬度版面策略，供之後介面使用；目前沒有畫面。
- 主機端 smoke example、核心測試與 GitHub Actions 設定。
- [開源補齊方案](docs/open-source-reuse.md)：已核對候選元件的來源、授權與版本差異。
- 獨立 Android ARM64 平台依賴編譯實驗，驗證 android-activity / ndk / CPAL / JNI 可共同編譯；尚非 App。

## 開發

需 Rust 1.95 以上、Cargo、Git。首次建置需要網路取得固定版本的 FilmCraft 及其依賴。

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
cargo run -p filmcraft-mobile-smoke --locked
```

上述指令執行主機端品質檢查，不會產生行動 App。
APK 打包與簽署指令見 [Android 預覽版說明](docs/android-preview.md)。

## 下一步

1. 在實體 Android 手機／平板驗證觸控宿主、畫面與生命週期。
2. 匯入一支本機影片、硬體預覽與拖曳播放頭。
3. 串接既有 FilmCraft 專案／剪輯／命令模組，完成裁切、分割與復原。
4. 實作自動儲存、程序被系統終止後復原、匯出與分享。
5. 平板雙欄與橫向編輯，再於 macOS 驗證 iPhone／iPad。

完整範圍見 [ROADMAP.md](ROADMAP.md)，技術決策見 [docs/architecture.md](docs/architecture.md)。

## 來源與授權

- 本機來源：原有 FilmCraft checkout，未修改它。
- 核心依賴：[aa0968111723-prog/filmcraft](https://github.com/aa0968111723-prog/filmcraft)，固定於
  `8309f21a6c3f34ddf190af7fd9bfd913053a1fd5`。
- 上游：[storytold/filmcraft](https://github.com/storytold/filmcraft)。
- 授權：MIT OR Apache-2.0；見 [NOTICE](NOTICE)、[LICENSE-MIT](LICENSE-MIT)、[LICENSE-APACHE](LICENSE-APACHE)。
- 不包含上游 ArtCraft 商標圖稿、影片素材、字型或桌面 App 的整套資產。
