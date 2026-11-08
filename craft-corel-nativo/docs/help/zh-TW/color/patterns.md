# 圖樣填色

圖樣填色會在物件內部重複拼貼。拼貼可以是雙色、一組向量物件（全彩向量圖樣）或點陣圖。拼貼固定在頁面上，因此移動物件時不會滑動。

## 從您的物件建立圖樣 {#create-pattern}

1. 繪製構成一個拼貼的物件並選取它們。
2. 選擇 [[menu.object > menu.object.create > menu.object.create_pattern]]。
3. 在 [[dialog.create_pattern]] 對話方塊中，選擇 [[dialog.create_pattern_type]]：[[dialog.create_pattern_vector]] 會將物件保留為向量，在任何大小下都保持銳利；[[dialog.create_pattern_bitmap]] 則會將其製成圖片。
4. 按一下 [[dialog.ok]]。

![建立圖樣對話方塊](../../img/color-patterns-create-pattern.png)

此圖樣會成為新物件的填色。若要套用到已繪製的物件，請在 [[docker.properties]] 面板的填色區段中選擇 [[docker.fill_pattern]]。

## 圖樣設定 {#settings}

在 [[docker.properties]] 面板的填色區段中，選擇 [[docker.fill_pattern]]，然後選擇圖樣種類及其設定。對於向量圖樣，面板會顯示拼貼包含的物件數量，以及其寬度和高度：變更這些值時，拼貼的內容也會隨之縮放。

## 匯出檔案中的圖樣 {#export}

SVG 檔案會將向量圖樣保留為向量。PDF 和 EPS 檔案則會以匯出解析度將其轉為圖片，並裁剪至物件範圍。
