# 新功能

TraceDraw 的每個版本都會加入功能和修正。本頁說明如何得知變更內容，並列出最近
版本的重點。

## 在程式中查看新內容 {#in-the-program}

- 選擇 [[menu.help > menu.help.whats_new]] 開啟歡迎畫面的新功能頁面。頁面由新到舊列出
  最近十個版本，每個版本都有日期和簡短標題。按一下某個版本即可閱讀變更的
  簡短說明；最新版本已經展開。說明下方的 **想了解更多？** 會開啟該版本在
  GitHub 上的頁面，其中有完整的變更清單。
- 開啟 [[menu.help > menu.help.highlight_whats_new]]，在功能表中標出本版本的新
  內容：每個新命令的結尾、通往新命令的子功能表箭頭旁，以及包含新命令的功能表
  標題上方都會出現一個小圓點。關閉該命令後標記消失，下次啟動 TraceDraw 時該
  命令又會關閉。
- 每個版本的完整變更清單在專案的
  [變更記錄](https://github.com/tracedraw/tracedraw/blob/main/CHANGELOG.md) 中。

## 最近版本的重點 {#highlights}

- **觸控和手機。** 標準工具列上的 Shift、Ctrl 和 Alt 按鈕為手指保持按下；在 Android 上介面會避開相機挖孔和圓角；Apple Pencil 可在 iPad 的 Safari 中使用。
- **檔案。** .cdr 檔案保留重疊曲線的填色規則，開啟時保留定位點、多欄文字、所有頁面共用的參考線和主頁面物件。
- **圖片、列印和工作區。** 裁切工具的裁切圖片像素和 [[menu.bitmaps > menu.bitmaps.crop_to_frame]] 可以裁掉圖片的像素；可以灰階列印和反向順序列印；[[menu.view > menu.view.scrollbars]] 可以隱藏捲軸。
- **TraceCut。** [[menu.tools > menu.tools.tracecut]] 開啟適用於切割繪圖機的工具：繪圖機輸出、切割線、排料和測量。請參閱 [TraceCut](../objects/tracecut.md)。
- **鍵盤、手機和觸控筆。** 選單和清單可以用鍵盤操作。在 Android 手機上介面完整顯示在螢幕上，使用觸控筆時放在螢幕上的手不再誤畫；繪圖板的筆可在使用 Wayland 的 Linux 上運作。
- **檔案。** .cdr 檔案開啟時保留定位點、主版頁面、漸層和變形；PDF 中的圖片以完整解析度匯入；JPEG 和 TIFF 可匯出為 CMYK。
- **物件縮圖。** [[docker.objects]] 面板在每個物件的名稱旁顯示其縮小的
  圖形；在清單上按 Ctrl 捲動滾輪或拖曳底部的滑桿可調整大小。請參閱
  [組織物件與圖層](../objects/objects-and-layers.md#objects-docker)。
- **使用你的語言的說明。** 每個 **?** 按鈕和 <kbd>F1</kbd> 都會以 TraceDraw
  設定的語言開啟本說明中關於眼前對話方塊、面板或工具的頁面。
- **文件屬性。** [[menu.file > menu.file.document_properties]] 保存繪圖的
  語言、標題、主旨、作者、版權、標籤、備註和評等，並附有繪圖內容的完整報告。
  請參閱 [文件屬性](../pages-layout/document-properties.md)。
- **色彩管理。** [[menu.tools > menu.tools.color_management]] 設定繪圖的
  描述檔和新繪圖的預設值，並為開啟或匯入的檔案設定原則。請參閱
  [色彩管理](../color/color-management.md)。
- **條碼和 QR 碼。** 三步驟精靈可放置條碼，QR 碼則是可在
  [[docker.properties]] 面板中編輯的動態物件。請參閱
  [條碼和 QR 碼](../objects/barcodes.md)。
- **效果。** [[menu.effects]] 功能表集中了各效果面板；漸變可以按固定間距沿
  路徑排列，並以順時針或逆時針旋轉色彩。請參閱 [漸變](../effects/blend.md)。
- **路徑文字** 有自己的方向、位置、距離和位移設定。請參閱
  [新增和編輯文字](../text/adding-text.md)。
- **字型管理員** 整理已安裝的字型、資料夾、我的最愛和集合。請參閱
  [字型和字符](../text/fonts.md)。
