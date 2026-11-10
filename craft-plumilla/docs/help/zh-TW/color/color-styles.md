# 色彩樣式

**色彩樣式**是繪圖中的物件所使用的具名色彩。變更樣式時，所有以該樣式填色的物件也會隨之變更，這能讓設計的色彩保持一致，也能快速嘗試其他配色方案。樣式會列在 [[docker.color_styles]] 面板中；從 [[menu.window > menu.window.dockers]] 開啟它。

![色彩樣式面板](../../img/color-color-styles.png)

## 建立色彩樣式 {#create}

- [[docker.new_from_selection]] 會為選取物件中每一個尚未成為樣式的填滿色彩新增一個樣式。
- [[docker.new_color_style]] 會以 [[docker.color]] 面板的目前色彩新增一個樣式。

## 使用和編輯樣式 {#edit}

每個樣式都會顯示其色彩、名稱和三個按鈕：

- 按一下色彩可變更它：以該色彩填色的物件會隨之變更；
- 按一下名稱可重新命名；
- [[docker.fill]] 會以該樣式填滿選取範圍，[[docker.outline]] 會為其外框上色；
- **x** 會移除樣式（物件保留其色彩）。

## 色彩和諧 {#harmonies}

[[docker.harmony_editor]] 會從 [[docker.base_color]] 建立一組搭配協調的樣式：

1. 選擇 [[docker.base_color]]。
2. 按一下一種和諧配色：[[harmony.analogous]]、[[harmony.complementary]]、[[harmony.split_complementary]]、[[harmony.triadic]]、[[harmony.tetradic]] 或 [[harmony.monochromatic]]。

和諧配色的色彩會以樣式的形式加入，並以該和諧配色命名。
