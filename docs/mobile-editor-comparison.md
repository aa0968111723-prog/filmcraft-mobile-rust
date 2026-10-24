# 手機／平板開源剪輯專案對照

查核日期：2026-10-25。此頁是選型與補齊計畫；以下新候選尚未編譯、安裝或接入本產品。
上游展示、官方文件、產品自身驗證分別記錄，沒有把上游宣稱視為本機通過。

## 專案比較

| 專案 | 類型／平台 | 授權查核 | 可對應的缺口 | 採用判斷 |
| --- | --- | --- | --- | --- |
| [Open Video Editor](https://github.com/devhyper/open-video-editor/tree/2f3b1534cdf4ea8d96fca027916857500e80a423) | Android 完整 App | GPL-3.0；依檔案聲明確認版本選項；[授權原文](https://github.com/devhyper/open-video-editor/blob/2f3b1534cdf4ea8d96fca027916857500e80a423/LICENSE) | 裁切、旋轉、縮放、灰階；有官方安裝入口與手機畫面；[原始說明](https://github.com/devhyper/open-video-editor/blob/2f3b1534cdf4ea8d96fca027916857500e80a423/README.md) | 用於功能與操作流程對照。現階段不複製程式碼或資產進 MIT／Apache 核心。 |
| [Miniter](https://github.com/mlm-games/Miniter/tree/3c446fa751690cf9e145b93f02ea3bbb66853c94) | Android／Linux 剪輯 App | GPL-3.0；須逐檔確認；[授權原文](https://github.com/mlm-games/Miniter/blob/3c446fa751690cf9e145b93f02ea3bbb66853c94/LICENSE) | 時間軸、片段組合；說明提到自 0.7 起使用 Rust 後端；[原始說明](https://github.com/mlm-games/Miniter/blob/3c446fa751690cf9e145b93f02ea3bbb66853c94/README.md) | 研究 Rust 與行動宿主的分工。上游自述維護模式；存檔重載與 Android 特效仍有限制，不直接搬用。 |
| [Android Video Trimmer](https://github.com/a914-gowtham/android-video-trimmer/tree/9a241bba91272ef43a141e901ee33e34c2ca14b1) | Android 裁切元件與示範 App | MIT 主文，另留 FFmpeg／LGPL 聲明；[授權原文](https://github.com/a914-gowtham/android-video-trimmer/blob/9a241bba91272ef43a141e901ee33e34c2ca14b1/LICENSE) | 裁切把手、縮圖、輸出大小選項；說明表示自 1.8.0 起改用 Media3；[原始說明](https://github.com/a914-gowtham/android-video-trimmer/blob/9a241bba91272ef43a141e901ee33e34c2ca14b1/README.md) | 先隔離研究。固定快照的依賴同時有舊 ExoPlayer 2.17.1 與 Media3 Transformer 1.8.0；需釐清授權與相依圖後才採用。 |
| [DeepMedia Transcoder](https://github.com/deepmedia/Transcoder/tree/a28e1eaf575181f2911f857d978e539b41625de0) | Android 媒體函式庫與示範 App | Apache-2.0，含 NOTICE；[授權原文](https://github.com/deepmedia/Transcoder/blob/a28e1eaf575181f2911f857d978e539b41625de0/LICENSE) | MP4／音訊、裁切、串接、旋轉、縮放、速度、進度與取消回呼；[原始說明](https://github.com/deepmedia/Transcoder/blob/a28e1eaf575181f2911f857d978e539b41625de0/README.md) | 作為 Media3 的備選後端。透過窄平台介面接 Rust 匯出計畫；不在同一條管線同時新增兩套轉碼器。 |
| [Cabbage](https://github.com/VideoFlint/Cabbage/tree/41c78700c32b3814eaa242a3b4fe9323e14d63fc) | Apple 合成框架與 VideoCat 示範 | MIT；[授權原文](https://github.com/VideoFlint/Cabbage/blob/41c78700c32b3814eaa242a3b4fe9323e14d63fc/LICENSE) | 由時間軸產生預覽、縮圖與匯出，支援圖片／影片／聲音與轉場；[原始說明](https://github.com/VideoFlint/Cabbage/blob/41c78700c32b3814eaa242a3b4fe9323e14d63fc/README.md) | Apple 平台合成設計參考；不要另立一份權威時間軸。需要 macOS／Xcode 實際驗證，舊說明的 iOS 9／Swift 4 不等於現代工具鏈支援證明。 |
| [PryntTrimmerView](https://github.com/HHK1/PryntTrimmerView/tree/6abd91b68b69ede226093ebb7f847045d7f4c34b) | Apple 裁切介面與示範 | MIT；[授權原文](https://github.com/HHK1/PryntTrimmerView/blob/6abd91b68b69ede226093ebb7f847045d7f4c34b/LICENSE) | 裁切範圍、縮圖、畫面裁切與播放頭連動；[原始說明](https://github.com/HHK1/PryntTrimmerView/blob/6abd91b68b69ede226093ebb7f847045d7f4c34b/README.md) | 只借鑑觸控互動，或日後作 Apple 原生外殼候選。上游明確說元件本身只提供介面，不能當成影片編輯／匯出引擎。 |
| [MetalPetal](https://github.com/MetalPetal/MetalPetal/tree/f9b78897bd4214bb097f352a1bde0a4f4a1e2ddb) | Apple GPU 影像／影片處理框架 | 核心 MIT；範例另有授權；文件 CC-BY-4.0；[授權原文](https://github.com/MetalPetal/MetalPetal/blob/f9b78897bd4214bb097f352a1bde0a4f4a1e2ddb/LICENSE) | 調色、濾鏡、混合、裁切，提供影片處理方式；[原始說明](https://github.com/MetalPetal/MetalPetal/blob/f9b78897bd4214bb097f352a1bde0a4f4a1e2ddb/README.md) | Apple 特效後端候選；匯出範例另依賴 VideoIO。需分別核對範例／素材／附加套件授權，不能把整個儲存庫一律當成 MIT。 |

完整快照版本、分支、是否封存與最近推送時間見 [固定查核紀錄](research/mobile-editor-snapshots.json)。
GitHub 對 Android Video Trimmer 的自動授權偵測為 `NOASSERTION`；不能以 README 的 MIT 標籤取代原始檔與二進位依賴查核。

## 優先補齊順序

1. **真實影片的匯入、預覽與匯出**：優先 [AndroidX Media3 Transformer](https://developer.android.com/media/media3/transformer) 與官方示範。Rust 持有專案、整數時間、剪輯命令與匯出計畫；平台只處理 URI、播放器與編解碼。Media3 為既有候選，本機開發中的 Android 宿主已有固定 `1.5.1` 設定，但不屬於這次 `0.1.0-alpha.2` 示範預覽版，不將其未驗證功能列入該版。
2. **手機時間軸互動**：對照 Open Video Editor、Miniter、Android Video Trimmer 的操作能力，先補縮圖、裁切把手、拖曳與縮放。用原創 Rust 介面實作；需要重用程式碼時，另確認該檔案的授權、固定版本與聲明。
3. **背景匯出、進度、取消與錯誤復原**：先使用現有 Media3 能力；若出現可重現限制，再隔離比較 Transcoder。驗收需包含取消後清除暫存輸出，以及重開後來源 URI 仍能讀取。
4. **字幕、濾鏡、旋轉與音量**：以同一份 Rust 編輯參數同時驅動預覽與匯出。先驗證一項效果的輸出，再擴充功能，避免只改預覽畫面。
5. **Apple 手機與平板**：Cabbage／PryntTrimmerView／MetalPetal 作平台接縫與互動參考。等取得 macOS 工具鏈後，再實際驗證預覽、音畫同步與匯出；目前沒有 iPhone／iPad 安裝檔。

## 接縫與驗收

- 匯入：可持續存取的來源識別、經驗證的長度／尺寸／旋轉、使用者取消與讀取失敗。
- 時間：Rust 的整數 Tick 是唯一權威；只在平台邊界轉成微秒或平台時間，明確拒絕溢位。
- 預覽：播放、暫停、跳轉、片段切換、聲音、背景返回；記錄實際解碼器與來源格式。
- 匯出：相同裁切範圍與片段順序、聲音、時長、旋轉／比例、進度、取消、失敗時不發布半成品。
- 發布：固定依賴與雜湊、保留授權與來源；分開記錄主機檢查、模擬器、真機與實際素材驗證。

本次沒有複製第三方 App 的商標、截圖、影片、範例素材或程式碼，也沒有變更現有專案的授權。
