# 行動版架構決策（2026-10-24）

## Rust 優先與現有核心沿用

原 FilmCraft 已是 Rust workspace，桌面介面使用 egui / eframe，並以獨立 crates
組織 time、project、edit、engine、render、export 與平台媒體 API。
現有桌面 App 同時使用桌面檔案選擇、剪貼簿、音訊及平台設定，不能只改 target 就當成行動版。

本專案先實際接入 `filmcraft-time`，固定來源 commit，避免追蹤 main 造成不受控變動。
後續按模組驗證 project / edit / engine；本階段尚未接入它們，不宣稱既有專案可直接相容。
這是獨立 repository，不是完整複製上游，也不改動原本 checkout。

## 分層

1. Rust 核心：精確時間、剪輯、命令、專案序列化與復原。
2. Rust 行動應用層：觸控狀態、可見時間軸範圍、快取與資源預算。
3. Rust UI 與 GPU：先驗證 egui / wgpu；畫面需重設為手機單欄與平板雙欄。
4. 平台媒體／系統橋接：相簿 URI、沙盒檔案、硬體編解碼、音訊焦點、分享及背景生命週期。

核心不可依賴 UI 或 Android / Apple SDK。FFI 必須集中於未來獨立的平台 crate，
以安全的 Result API 暴露能力，對不支援的格式回報明確錯誤。
不得為追求「純 Rust」而在手機上強制全部使用 CPU 軟體編解碼。

## UI 選型仍待真機驗證

優先評估現有 egui / wgpu，是為了沿用 Rust 技術與部分互動模型；
這不是已完成的 Android / iOS 支援承諾。
需先驗證觸控手勢、輸入法、無障礙、安全區、旋轉、平板視窗縮放及 GPU 影片貼圖。

Dioxus 可以讓應用 UI 使用 Rust，但目前官方行動路徑使用系統 WebView，
不等於整個執行堆疊皆為 Rust。Slint 也可評估，但需另外確認授權與影片表面整合。
初期不引入多套 UI 框架。

## 裝置限制與資料安全

- 用可用邏輯寬度決定版面，840 起使用 expanded，旋轉／分割視窗時重算。
- CPU RGBA 預覽快取獨立限額；不把它誤認為整個程序的記憶體限制。
- 代理影片、低解析度預覽、可取消工作與節流依真機量測再實作。
- 編輯後的專案需以原子替換寫入；程序中止復原尚未實作。
- Android 需處理 content URI 與持續存取權；Apple 需處理相簿與沙盒授權。
- 裝置媒體與憑證不進 Git；簽署憑證使用 CI secrets，不能放入 Cargo 或原始碼。

## 驗證界線

核心通過主機測試或 target check，僅代表相應 Rust 核心可編譯，
不代表可安裝、播放器可用、硬體加速可用或通過商店審核。
里程碑須分別記錄編譯、打包、模擬器、真機與實際剪輯／匯出結果。

## 官方參考

- FilmCraft 架構：https://github.com/storytold/filmcraft/blob/main/docs/architecture.md
- Dioxus mobile：https://dioxuslabs.com/learn/0.7/guides/platforms/mobile/
- Slint mobile：https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/general/
- Rust Android targets：https://doc.rust-lang.org/rustc/platform-support/android.html
- Rust Apple iOS targets：https://doc.rust-lang.org/rustc/platform-support/apple-ios.html
