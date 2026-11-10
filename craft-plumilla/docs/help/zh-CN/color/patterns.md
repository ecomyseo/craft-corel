# 图样填充

图样填充在对象内部重复一个图块。图块可以是双色、一组矢量对象（全色矢量图样）或位图。图块固定在页面上，因此移动对象时图样不会滑动。

## 用你的对象创建图样 {#create-pattern}

1. 绘制构成一个图块的对象并选择它们。
2. 选择 [[menu.object > menu.object.create > menu.object.create_pattern]]。
3. 在 [[dialog.create_pattern]] 对话框中选择 [[dialog.create_pattern_type]]：[[dialog.create_pattern_vector]] 将对象保留为矢量，任何大小都清晰；[[dialog.create_pattern_bitmap]] 将它们制作成图片。
4. 单击 [[dialog.ok]]。

![创建图样对话框](../../img/color-patterns-create-pattern.png)

该图样成为新对象的填充。要将其赋予已绘制的对象，请在 [[docker.properties]] 面板的填充部分选择 [[docker.fill_pattern]]。

## 图样设置 {#settings}

在 [[docker.properties]] 面板的填充部分选择 [[docker.fill_pattern]]，然后选择图样类型及其设置。对于矢量图样，面板显示图块包含多少个对象及其宽度和高度：更改它们会同时缩放图块的内容。

## 导出文件中的图样 {#export}

SVG 文件将矢量图样保留为矢量。PDF 和 EPS 文件将其作为导出分辨率下的图片，并剪裁到对象。
