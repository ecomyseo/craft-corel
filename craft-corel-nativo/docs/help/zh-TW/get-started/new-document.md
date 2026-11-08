# 建立繪圖

每個繪圖都從頁面大小、頁數、色彩模式和解析度開始。您可以在 [[dialog.create_new_document]] 對話方塊中選擇這些設定。

## 建立繪圖 {#create}

1. 選擇 [[menu.file > menu.file.new]]（<kbd>Ctrl+N</kbd>）、按一下標準工具列上的「新增」、按一下最後一個文件分頁後面的 **+**，或按一下歡迎畫面上的「新增文件」。
2. 在 [[dialog.create_new_document]] 對話方塊中，設定下列各項值。
3. 按一下 [[dialog.ok]] 或按 <kbd>Enter</kbd>。

繪圖會在自己的分頁中開啟。對話方塊會記住您的設定值，供下一個繪圖使用。

![建立新文件對話方塊](../../img/get-started-new-document.png)

## 一般 {#general}

- [[dialog.name]] 為繪圖命名。預設為下一個 `Untitled-N`。
- [[newdoc.preset]] 會一次設定色彩模式、頁面大小、單位和解析度。預設集包括 [[newdoc.preset_default]] 和 [[newdoc.preset_cmyk]]（CMYK、A4、公釐、300 dpi）、[[newdoc.preset_rgb]]（RGB、A4、公釐、300 dpi）以及 [[newdoc.preset_web]]（RGB、1920 x 1080 像素、72 dpi）。當您變更其中某個值時，預設集會顯示 [[newdoc.preset_custom]]，除非這些值再次符合某個預設集。
- 預設集旁邊的按鈕包含 [[newdoc.save_preset]]（會要求輸入 [[newdoc.preset_name]]，並將目前的值加入清單）和 [[newdoc.delete_preset]]（會移除您儲存的預設集）。內建的預設集無法刪除。
- [[newdoc.pages]] 設定繪圖的頁數，從 1 到 999。每個頁面都命名為「頁面 N」，並且有一個圖層。
- [[newdoc.color_mode]] 可選 CMYK（用於印刷）或 RGB（用於螢幕）。

## 尺寸 {#dimensions}

- [[newdoc.page_size]] 列出各種紙張大小，或「自訂」。選擇大小時會保留目前的方向。若您輸入的寬度和高度符合某個紙張大小（無論哪個方向），就會選取該大小的名稱。
- [[newdoc.width]] 和 [[newdoc.height]] 設定頁面大小，單位為您在寬度旁邊清單中選擇的單位。這些單位會成為屬性列的繪圖單位。
- [[newdoc.orientation]]：[[dialog.portrait]] 或 [[dialog.landscape]]。
- [[newdoc.resolution]]，以 [[newdoc.dpi]] 為單位（預設為 300）。它用於會轉換為像素的效果，以及以像素測量大小：1920 像素在 300 dpi 時為 162.56 mm，在 72 dpi 時為 677.33 mm。當單位為像素時，變更解析度會保留頁面的像素大小。

## 色彩設定 {#color-settings}

開啟 [[newdoc.color_settings]] 以選擇繪圖的 [[newdoc.rgb_profile]]、[[newdoc.cmyk_profile]] 和 [[newdoc.gray_profile]]，以及 [[newdoc.intent]]。預設值為 sRGB、內建的 CMYK 描述檔、20% 網點擴大的灰階，以及相對色度。請參閱[色彩管理](../color/color-management.md)。

## 略過對話方塊 {#skip}

勾選 [[newdoc.dont_show]] 即可直接以上次使用的值建立新繪圖。若要再次顯示對話方塊，請在 [[menu.tools > menu.tools.options > menu.tools.options_app]] 的 [[options.general]] 頁面上開啟對應的設定。請參閱 [TraceDraw 選項](../customize/options-app.md#general)。

## 其他開始方式 {#other-ways}

- [[menu.file > menu.file.new_from_template]] 從範本開始。請參閱[範本](../templates-styles/templates.md)。
- [[menu.file > menu.file.open]] 開啟現有的繪圖。請參閱[開啟和儲存繪圖](open-and-save.md)。
