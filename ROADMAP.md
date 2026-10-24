# Roadmap

## M0 — 獨立開源儲存庫與可攜核心

- [x] 建立 Rust workspace、授權與來源紀錄。
- [x] 固定 commit 引用 FilmCraft 精確時間模組。
- [x] 加入剪裁驗證、版面策略、預覽快取計算與測試。
- [x] 提供 host smoke example 與 CI 設定。
- [ ] GitHub CI 實際執行並通過（發布後確認）。

## M1 — Android 宿主與實機預覽

- [x] 查核開源補齊方案、授權證據、維護狀態與版本差異（docs/open-source-reuse.md）。
- [x] 獨立 Android ARM64 probe：android-activity / ndk / CPAL / JNI 共同編譯通過。
- [ ] 驗證 Rust UI 宿主，建立 Gradle / Manifest 與 APK。
- [ ] 手機直向、橫向、平板分割視窗、安全區與觸控。
- [ ] 系統選擇一支影片，處理 content URI。
- [ ] MediaCodec 硬體解碼與 GPU 預覽，記錄失敗時的錯誤行為。
- [ ] 音訊與影片同步、seek、暫停與返回前景。

## M2 — 最小可用剪輯

- [ ] 接入 FilmCraft project / edit / engine 命令。
- [ ] 分割、裁切、排序、復原與重做。
- [ ] 儲存、重新開啟、程序遭終止後復原。
- [ ] 一段影片連同音訊匯出 MP4；驗證格式支援與分享。
- [ ] 測量 1080p 工作階段的記憶體、耗電、散熱與匯出時間。

## M3 — 平板完整工作流程

- [ ] 雙欄預覽／屬性、多軌時間軸、雙指縮放、筆與鍵盤。
- [ ] 字幕、文字、基本色彩與音量。
- [ ] 與桌面專案格式的相容性測試。

## M4 — iPhone / iPad

- [ ] macOS / Xcode 宿主、模擬器與真機簽署。
- [ ] 相簿、檔案、VideoToolbox、音訊工作階段與生命週期。
- [ ] iPad 旋轉／視窗縮放與觸控驗證。
- [ ] TestFlight 與商店發佈準備。

初期不承諾完整桌面功能、4K 多軌、HDR、雲端同步或商店上架日期。
