# 色彩管理

螢幕以紅、綠、藍光顯示色彩；印表機則混合青色、洋紅色、黃色和黑色油墨。色彩管理使用**色彩描述檔**來描述每個裝置，讓色彩從螢幕到印刷頁面都保持相同的外觀。這些設定位於 [[menu.tools > menu.tools.color_management]]，也就是 [[color_mgmt.title]] 對話方塊。在您按一下 [[dialog.ok]] 之前不會有任何變更。

![色彩管理設定對話方塊](../../img/color-color-management.png)

## 繪圖的描述檔 {#document}

[[color_mgmt.tab_document]] 分頁設定作用中繪圖的描述檔。

- [[color_mgmt.current_profiles]] 顯示繪圖目前使用的 RGB、CMYK 和灰階描述檔。
- 選擇套用新描述檔的方式：
  - [[color_mgmt.assign]]：色彩值保持不變，其外觀則依新描述檔改變；
  - [[color_mgmt.convert]]：轉換 CMYK 色彩，使其在新描述檔中看起來相同。
- 選擇 [[color_mgmt.rgb_profile]]、[[color_mgmt.cmyk_profile]] 和 [[color_mgmt.grayscale]] 描述檔。每個清單都先列出預設描述檔，然後是您電腦上找到的所有描述檔。[[color_mgmt.load_profiles]] 會加入描述檔檔案（`.icc`、`.icm`）。
- [[color_mgmt.primary_mode]]：CMYK 或 RGB。
- [[color_mgmt.intent]]：無法顯示的色彩如何納入可顯示範圍：[[color_mgmt.intent_relative]]（預設）、[[color_mgmt.intent_perceptual]]、[[color_mgmt.intent_saturation]] 或 [[color_mgmt.intent_absolute]]。

CMYK 描述檔會隨繪圖保存，也會寫入版本 16 以上的 `.cdr` 檔案。

## 新繪圖的預設值 {#default}

[[color_mgmt.tab_default]] 分頁設定新繪圖所使用的設定，以及色彩的轉換方式。

- [[color_mgmt.preset]]：適用於北美、歐洲、日本或網頁與螢幕工作的現成設定。[[color_mgmt.save_preset]] 會以 [[color_mgmt.preset_name]] 儲存您自己的設定；[[color_mgmt.delete_preset]] 會移除一個預設集。
- 預設的 [[color_mgmt.rgb_profile]]、[[color_mgmt.cmyk_profile]] 和 [[color_mgmt.gray_profile]]、[[color_mgmt.primary_mode]] 以及 [[color_mgmt.intent]]。
- [[color_mgmt.conversion]]：
  - [[color_mgmt.engine]]：[[color_mgmt.engine_icc]]（使用描述檔）或 [[color_mgmt.engine_builtin]]；
  - [[color_mgmt.preserve_black]]：只使用黑色油墨的色彩在轉換後仍只使用黑色油墨；
  - [[color_mgmt.map_gray]]（預設開啟）：轉換為 CMYK 的灰色只使用黑色油墨；
  - [[dialog.black_point]]（預設開啟）：讓最暗的色調保持區別。
- [[color_mgmt.open_policy]]：您開啟的檔案其描述檔會如何處理，分別針對 RGB、CMYK 和灰階：[[color_mgmt.open_use_embedded]]、[[color_mgmt.open_assign_default]] 或 [[color_mgmt.open_convert_default]]。當檔案使用其他描述檔或沒有描述檔時，[[color_mgmt.warn_mismatch_box]] 和 [[color_mgmt.warn_missing_box]] 會在狀態列中顯示訊息。
- [[color_mgmt.import_policy]]：對於您匯入或貼上的內容，選擇 [[color_mgmt.import_convert]] 或 [[color_mgmt.import_assign]]。
- [[color_mgmt.spot_definition]]：特別色是以 [[color_mgmt.spot_lab]]、[[color_mgmt.spot_cmyk]] 還是 [[color_mgmt.spot_rgb]] 定義。

## 預覽印刷色彩 {#proof}

[[menu.view > menu.view.proof_colors]]，或狀態列右端的校樣按鈕，會以印刷時的樣子顯示繪圖：CMYK 油墨無法印出的色彩看起來會比較暗淡。[[docker.color]] 面板也會針對這類色彩發出警告。

沒有載入描述檔時，TraceDraw 會以其本身對塗佈紙上印刷油墨的模型來轉換色彩。
