# Android 可下載預覽版

版本：0.1.0-alpha.1。目標為 Android 8.0 / API 26 以上的 ARM64 手機／平板。
通用 APK 同時包含 ARM64 與 x86_64 原生函式庫；需要 OpenGL ES 3.0。

## 可操作範圍

- 原生 Rust / egui 觸控介面，無網路權限、無帳號。
- 三段程式繪製的動態示範素材與 30 fps 播放頭。
- 手機單欄；較寬的橫向／平板視窗使用預覽與工具雙欄，內容可捲動。
- 選取片段、裁切來源起迄、分割、左右移動、復原／重做。
- 每次剪輯後儲存在 App 私有目錄，重新開啟時載入。
- 驗證存檔版本、大小、片段上限、時間邊界，錯誤顯示於介面。

這個版本**尚不能匯入、播放或匯出裝置裡的真實影片**，不包含音訊。
示範專案格式是這個原型自己的格式，尚非完整桌面 FilmCraft 專案格式。
卸載 App 會移除私有目錄裡的示範專案。背景與程序中止測試不代表斷電儲存保證。

## 取得與安裝

發佈位置：[GitHub Releases](https://github.com/aa0968111723-prog/filmcraft-mobile-rust/releases)。
資產檔名為 `filmcraft-mobile-0.1.0-alpha.1-universal.apk`，附 `SHA256SUMS.txt`。

1. 在 Android 裝置下載 APK。
2. 開啟下載的 APK，依 Android 畫面允許該下載來源安裝應用程式。
3. 安裝後開啟 **FilmCraft Mobile**。

這是直接安裝的早期預覽版，使用專案本機保存的預覽簽署身分；並非 Google Play 發佈版。
iPhone / iPad 不能安裝 APK；本次沒有 IPA / TestFlight 發佈。

## 從原始碼打包（Windows / PowerShell 7）

前置：Rust 1.95 以上、JDK 21、Android SDK platform 36 / build-tools 36.0.0、
NDK 27.2.12479018、`cargo-ndk 4.1.2`。

```powershell
cargo install cargo-ndk --version 4.1.2 --locked
rustup target add aarch64-linux-android x86_64-linux-android
./scripts/build-android.ps1 -SdkRoot '<Android SDK>' -JavaHome '<JDK 21>'
```

腳本以 cargo-ndk 編譯 Rust，再直接用 SDK 的 aapt2 / zipalign / apksigner 打包與簽署。
NativeActivity 不含 Java 程式，因此這版不需要 Gradle；未來接入相簿等功能再加入平台宿主。
此為有意識縮小首版依賴，沒有使用 xbuild 或 cargo-apk 的修補 fork。

輸出在 `target/android/`。APK 內嵌第三方授權文字，另輸出可單獨下載的 notices。
NDK 連結設定使用 16 KiB 最大頁面大小；仍需個別記錄 16 KiB 裝置執行結果。
建置時重新映射原始碼路徑，並在打包前檢查原生函式庫沒有本機使用者目錄路徑。
本機簽署金鑰與密碼在 `.local/signing/`，不進 Git，也不跟 APK 一起發佈；
需要保留這個目錄才能使用相同身分發佈更新。

## 驗證層級

編譯、APK 結構、簽章、模擬器執行與實體裝置分開記錄。
具體發布前結果記於 Release 說明與 [android-validation.md](android-validation.md)。
