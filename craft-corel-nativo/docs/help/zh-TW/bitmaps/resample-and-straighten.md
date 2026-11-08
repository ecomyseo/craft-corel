# 重新取樣、拉直和擴大點陣圖

## 重新取樣 {#resample}

[[menu.bitmaps > menu.bitmaps.resample]] 會為選取的點陣圖設定新的像素數量；它們在頁面上的大小保持不變。

![重新取樣對話方塊](../../img/bitmaps-resample-and-straighten-resample.png)

1. 以**像素**、**百分比**，或以其在頁面上的大小所產生的**解析度**輸入新的大小。開啟**維持長寬比**時，高度會隨寬度變更。
2. 選擇方法：
   - **平滑**（相片），此為預設；
   - **銳利**，稍微清晰一些；
   - **硬像素**（像素藝術）：每個像素都維持為單一色彩的方塊；
   - **放大並保留細節**：平滑處理，並在圖片放大時加以銳利化。
3. 按一下 [[dialog.ok]]。

對話方塊會顯示原始大小和新大小，以及新圖片佔用的記憶體。選取多個點陣圖時，每個點陣圖都會以相同的倍數縮放。

## 拉直影像 {#straighten}

[[menu.bitmaps > menu.bitmaps.straighten_image]] 可修正傾斜、以斜角拍攝或因鏡頭而彎曲的相片。

![拉直影像對話方塊](../../img/bitmaps-resample-and-straighten-straighten.png)

1. 使用**格線**（可調整其大小和色彩）來對齊圖片。
2. 設定各滑桿：**修正鏡頭扭曲**（-100 到 100）、**旋轉影像**（-15 到 15 度）、**垂直透視**和**水平透視**（-100 到 100）。旋轉按鈕會先將圖片轉動四分之一圈。
3. 開啟**裁切影像**時，結果會裁切為保有原始比例的最大矩形；**裁切並重新取樣為原始大小**會將其恢復為原始的像素數量。不裁切時，角落會是透明的。
4. 按一下 [[dialog.ok]]。**重設**會將所有設定恢復原狀。

使用預覽上方的工具縮放和平移預覽。結果會取代像素，因此點陣圖上的效果會變成永久性的。

## 擴大點陣圖 {#inflate}

某些效果（例如模糊）會擴散到點陣圖的邊緣之外。

- [[menu.bitmaps > menu.bitmaps.inflate > menu.bitmaps.inflate_auto]]（預設開啟）會依效果所需擴大點陣圖，並加上透明邊界。
- [[menu.bitmaps.inflate_manual]] 可手動擴大選取的點陣圖：對於寬度和高度，以像素或百分比（100 到 1000%）輸入新的大小，並可使用**維持長寬比**。圖片會保持置中。

![手動擴大點陣圖對話方塊](../../img/bitmaps-resample-and-straighten-inflate.png)

## 其他點陣圖指令 {#other}

- [[menu.bitmaps.crop_bitmap]] 會使用 [[tool.crop]] 工具裁切點陣圖；使用 [[tool.shape]] 工具拖曳點陣圖的角落也可以裁切它。
- [[menu.bitmaps.edit_bitmap]] 會在另一個程式中開啟點陣圖。
- [[menu.bitmaps.remove_background]] 會讓相片的背景變成透明。
- [[menu.bitmaps.remove_background_local]]: [移除背景](remove-background.md).
- [[menu.bitmaps.break_link]] 和 [[menu.bitmaps.update_from_link]] 適用於連結至檔案的點陣圖。
- [[menu.bitmaps.save_bitmap_as]] 會將點陣圖儲存為 PNG 檔案。
