# 颜色样式

**颜色样式** 是绘图中对象使用的命名颜色。更改样式后，用它填充的每个对象也随之改变，这使设计的颜色保持一致，并能快速尝试其他配色方案。样式列在 [[docker.color_styles]] 面板中；可从 [[menu.window > menu.window.dockers]] 打开它。

![颜色样式面板](../../img/color-color-styles.png)

## 创建颜色样式 {#create}

- [[docker.new_from_selection]] 为所选对象中尚未成为样式的每种填充颜色添加一个样式。
- [[docker.new_color_style]] 用 [[docker.color]] 面板的当前颜色添加一个样式。

## 使用和编辑样式 {#edit}

每个样式显示其颜色、名称和三个按钮：

- 单击颜色可更改它：用该颜色填充的对象随之改变；
- 单击名称可重命名；
- [[docker.fill]] 用样式填充所选对象，[[docker.outline]] 为其轮廓着色；
- **x** 删除样式（对象保留其颜色）。

## 和谐 {#harmonies}

[[docker.harmony_editor]] 从一个 [[docker.base_color]] 生成一组相互协调的样式：

1. 选择 [[docker.base_color]]。
2. 单击一种和谐方式：[[harmony.analogous]]、[[harmony.complementary]]、[[harmony.split_complementary]]、[[harmony.triadic]]、[[harmony.tetradic]] 或 [[harmony.monochromatic]]。

和谐的颜色会作为样式添加，并以该和谐方式命名。
