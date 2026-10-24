# 行動版開源補齊方案

查核日期：2026-10-24。優先順序為「沿用既有核心 → 接成熟平台元件 → 自行補接縫」。
以下是本專案的選型判斷，不是各上游對 FilmCraft 的支援承諾。

## 先採用的組合

| 缺口 | 優先專案與授權證據 | 本專案的接入方式 | 尚待驗證 |
| --- | --- | --- | --- |
| 剪輯／專案／匯出核心 | [FilmCraft](https://github.com/aa0968111723-prog/filmcraft/tree/8309f21a6c3f34ddf190af7fd9bfd913053a1fd5)，MIT OR Apache-2.0 | 已引用 time；下一步逐一檢查 project / edit / engine 的行動 target 與桌面耦合 | 專案格式往返、實際匯出 |
| Rust 觸控 UI 的 Android 起點 | [egui hello_android](https://github.com/emilk/egui/tree/bf5ac0dac137af9a4861611f3d2a77ad531f50e3/examples/hello_android)，範例 manifest 為 MIT OR Apache-2.0 | 第一條原型路線；重用技術與宿主結構，重做手機時間軸 | 輸入法、安全區、無障礙、旋轉；不推定 iOS 已完成 |
| Android 生命週期與輸入 | [android-activity](https://github.com/rust-mobile/android-activity/blob/b4ddf059b77be12cdb955d394e65a92e7568d936/android-activity/Cargo.toml)，MIT OR Apache-2.0 | 初期 native-activity；宿主統一管理 Activity，不建立兩套事件迴圈 | 背景／返回、Activity 重建、系統中止後復原 |
| Android 媒體 API | [ndk](https://github.com/rust-mobile/ndk/blob/5ae5c56f68165ce902845b646b63a7cb15a92ead/ndk/Cargo.toml)，MIT OR Apache-2.0 | 啟用 media / nativewindow；將 MediaCodec、MediaFormat 包在平台層 | 解碼器協商、Surface 與 GPU 共享、seek、編碼與 mux |
| 音訊輸出 | [CPAL](https://github.com/RustAudio/cpal)，Apache-2.0 | 優先已發行 0.18.2；接現有 FilmCraft PCM 混音資料，不另做完整音訊引擎 | 音訊焦點、耳機切換、中斷、時鐘同步；呼叫前必須建立 Android context |
| Android 系統橋接 | [jni-rs](https://github.com/jni-rs/jni-rs/blob/dbfb17647988b81ff37b897ce93cc85ee4c84863/Cargo.toml)，MIT OR Apache-2.0 | 相簿／Storage Access Framework／分享的窄介面；必要時搭配小型 Kotlin Activity | content URI 持續授權、取消、非 seekable 來源、執行緒與 JVM 生命週期 |
| Android Rust 函式庫建置 | [cargo-ndk](https://github.com/bbqsrc/cargo-ndk/blob/8cffdefbc9fc29b15ef50390b73258032b26734b/Cargo.toml)，Apache-2.0 OR MIT | 4.1.2 作為建置工具候選；輸出 .so，交給 Gradle 打包 | NDK / Gradle / SDK 版本固定、簽署與 APK 安裝；它本身不是完整 APK 打包器 |
| Apple 媒體與相簿橋接 | [objc2 framework crates](https://github.com/madsmtm/objc2/tree/8a5abacd90d5403fc0463459a35826f892b9142e/framework-crates)，框架預設 Zlib OR Apache-2.0 OR MIT | objc2-video-toolbox / core-media / core-video；檔案與相簿另外評估相關框架 | macOS 上編譯與 iPhone / iPad 真機；bindings 不等於已做好的播放器 |

這些授權是直接元件的 manifest／LICENSE 證據。GitHub 自動偵測常只顯示一種，
甚至顯示 null / NOASSERTION；不能拿它取代各 crate 的授權。
發佈前仍要按實際 Cargo.lock 與啟用功能產生第三方授權清單。

## 已做的驗證：Android 平台依賴可一起編譯

`tools/android-dependency-probe` 是獨立、排除於產品 workspace 外的編譯實驗。
它固定 `android-activity = 0.6.1`、`ndk = 0.9.0`、`cpal = 0.18.2`、`jni = 0.22.4`，
附獨立 Cargo.lock，並引用 AndroidApp、MediaCodec、MediaFormat、HostTrait 與 JavaVM 型別。

```sh
rustup target add aarch64-linux-android
cargo check --manifest-path tools/android-dependency-probe/Cargo.toml --target aarch64-linux-android --locked
cargo clippy --manifest-path tools/android-dependency-probe/Cargo.toml --target aarch64-linux-android --locked -- -D warnings
```

本機已通過上述 Android ARM64 check。這是 Rust 型別／target 編譯證據：
**沒有啟動 Activity、沒有連結成 App、沒有解碼影片、沒有開啟音訊，也沒有生成 APK。**
egui / wgpu / cargo-ndk 與 Apple crates 尚未納入這個 probe。

## 介面備選與平台輔助

| 方案 | 原始證據 | 適用方式與決策 |
| --- | --- | --- |
| Makepad + Robrix | [Makepad MIT 與行動工具鏈](https://github.com/makepad/makepad/tree/44ecb822b48106fa76bde4dfd309e5a8a7ff1932)；[Robrix releases](https://github.com/project-robius/robrix/releases) 提供 Android APK 與 iPhone / iPad TestFlight 路徑 | Rust UI 的強備選。若 egui 宿主卡在輸入法／行動整合，做小型對照原型；不直接搬整套聊天 App。換框架會增加 FilmCraft UI 重用成本，影片貼圖管線需另驗證 |
| AndroidX Media3 | [Apache-2.0 LICENSE](https://github.com/androidx/media/blob/8c6678b657ede1e7883fc164ef73ed483c7796c3/LICENSE)；[Transformer 文件](https://developer.android.com/media/media3/transformer) | 若 Rust 平台管線進度受阻，可用小型 Kotlin 橋接補預覽／匯出。它提供剪裁與轉碼能力，但不應取代 Rust 的專案／剪輯資料模型；尚未接入 |
| Tauri dialog plugin | [Android / iOS 支援與 URI 語意](https://v2.tauri.app/plugin/dialog/)；[原始專案](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/dialog) | 適合 Tauri 宿主。現有 egui 路線不直接引入整套 Tauri，只借鑑其公開 API 分工；若日後切換 Tauri 再驗證直接使用 |

## 已發現的陷阱與排除

- **xbuild 暫不採用**：[專案描述](https://github.com/rust-mobile/xbuild)明確標示 unmaintained。
  舊搜尋摘要仍可能推薦它，不以這些舊摘要決定新專案工具鏈。
- **cargo-apk 僅作範例對照**：egui hello_android README 目前指定一個修補 fork。
  它不是完全無法使用，但第一條長期打包路線選 cargo-ndk + Gradle，減少對該修補的依賴。
- **不沿用桌面 rfd 作手機檔案選擇器**：[rfd README](https://github.com/PolyMeilex/rfd)
  列出的支援平台沒有 Android / iOS。Android content URI 不能直接轉成本機路徑假裝可讀。
- **版本不能全部追最新**：現有 FilmCraft 使用 egui 0.36；查核時 crates.io 的 egui / eframe
  穩定版已是 0.37.0。行動原型先核對 0.36 的 API 與相依圖，不先把上游整套升級。
- CPAL 的 Git master manifest 是 0.19.0，但查核時 crates.io 最新穩定版為 0.18.2。
  本次 probe 用實際已發行的 0.18.2；不可拿 master 文件當成該版完整行為保證。
- 不直接引入以 FFmpeg / GPL / LGPL 元件為核心的整套編輯器；需符合現有 FilmCraft 的
  Rust 核心與授權方向。硬體加速、零拷貝、低耗電都必須在裝置上量測。

## 接入順序與驗收

1. **Android 宿主**：egui 0.36 相容原型 + android-activity；通過輸入法、旋轉、返回前景。
2. **一支影片的預覽**：系統選檔 → URI / 檔案描述元 → FilmCraft demux → MediaCodec → 預覽。
   確認來源可 seek、正確處理 PTS / DTS、B-frame、旋轉與顏色；記錄實際 decoder，不能只看 API 名稱。
3. **聲音與定位**：CPAL 輸出與時鐘對齊；測 seek、耳機切換與中斷後恢復。
4. **最小剪輯閉環**：接 project / edit / engine，分割、裁切、復原、儲存、重開、匯出與分享。
5. **平板與 Apple**：再驗證平板分割視窗、Apple 宿主、VideoToolbox 與相簿。

以上每步都要有「可編譯／可打包／模擬器／真機／實際素材」各別紀錄。
目前完成的是來源選型與 Android 平台依賴編譯實驗，沒有把待驗收項目當作已完成。

完整查核時的 repository head、分支與維護狀態存於
[source-snapshots.json](research/source-snapshots.json)。這些 SHA 是研究快照，
不是產品已採用的 Git 依賴；實際 probe 使用 crates.io 版本及其 Cargo.lock。


## 繁體中文字型（第 2 次預覽版）

採用官方 [Noto Sans CJK TC](https://github.com/notofonts/noto-cjk/tree/f8d157532fbfaeda587e826d4cd5b21a49186f7c/Sans/OTF/TraditionalChinese) 的 Regular OTF，
固定上游版本 `f8d157532fbfaeda587e826d4cd5b21a49186f7c`，未修改或裁切字型。
[官方下載說明](https://github.com/notofonts/noto-cjk/blob/f8d157532fbfaeda587e826d4cd5b21a49186f7c/Sans/README.md)標示 TC 為臺灣繁體中文字形；
[原始授權](https://github.com/notofonts/noto-cjk/blob/f8d157532fbfaeda587e826d4cd5b21a49186f7c/Sans/LICENSE)為 SIL Open Font License 1.1，允許嵌入與隨軟體散布，保留授權文字及字型內的作者資訊。

完整字型內嵌於原生程式，比例字型與等寬字型都使用它補齊中文字元；離線啟動不需要下載或依賴裝置的字型路徑。
字型檔案 SHA-256 與來源保留在 `apps/mobile/assets/fonts/ATTRIBUTION.txt`，授權原文亦納入安裝檔的第三方授權文件。
實際 Android 畫面驗證結果見 [驗證紀錄](android-validation.md)。


## 追加手機剪輯專案對照（2026-10-25）

已查核 7 個手機剪輯 App／媒體元件的固定來源、授權與功能邊界，見 [手機／平板專案對照](mobile-editor-comparison.md)。
優先沿用 Media3 作 Android 媒體接縫，補時間軸互動，再比較備選轉碼與 Apple 效果後端。新候選均屬研究，尚未接入產品。
