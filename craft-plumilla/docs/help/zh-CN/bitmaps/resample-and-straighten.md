# 重新取样、矫正和扩充位图

## 重新取样 {#resample}

[[menu.bitmaps > menu.bitmaps.resample]] 为所选位图设置新的像素数；它们在页面上的大小保持不变。

![重新取样对话框](../../img/bitmaps-resample-and-straighten-resample.png)

1. 以 **像素**、**百分比** 或其在页面上的大小所对应的 **分辨率** 输入新尺寸。打开 **保持纵横比** 时，高度随宽度变化。
2. 选择方法：
   - **平滑**（照片），起初选用；
   - **锐利**，稍微清晰一些；
   - **硬像素**（像素画）：每个像素都保持为一个单色方块；
   - **放大并保留细节**：先平滑，图片放大时再锐化。
3. 单击 [[dialog.ok]]。

对话框显示原始尺寸、新尺寸以及新图片占用的内存。选择多个位图时，每个位图按相同比例缩放。

## 矫正图像 {#straighten}

[[menu.bitmaps > menu.bitmaps.straighten_image]] 校正倾斜、斜视角度拍摄或被镜头弯曲的照片。

![矫正图像对话框](../../img/bitmaps-resample-and-straighten-straighten.png)

1. 用 **网格**（其大小和颜色可调）对齐图片。
2. 设置滑块：**校正镜头畸变**（-100 到 100）、**旋转图像**（-15 到 15 度）、**垂直透视** 和 **水平透视**（-100 到 100）。旋转按钮先将图片转动四分之一圈。
3. 打开 **裁剪图像** 时，结果被裁剪为保持原始比例的最大矩形；**裁剪并重新取样为原始大小** 将其恢复为原始像素数。不裁剪时，四角为透明。
4. 单击 [[dialog.ok]]。**重置** 恢复所有设置。

用预览上方的工具缩放和平移预览。结果会替换像素，因此位图上的效果会变成永久性的。

## 扩充位图 {#inflate}

某些效果（例如模糊）会扩展到位图边缘以外。

- [[menu.bitmaps > menu.bitmaps.inflate > menu.bitmaps.inflate_auto]]（起初打开）按效果所需扩充位图，并加上透明边距。
- [[menu.bitmaps.inflate_manual]] 手动扩充所选位图：为宽度和高度输入以像素或百分比（100 到 1000%）表示的新尺寸，可使用 **保持纵横比**。图片保持居中。

![手动扩充位图对话框](../../img/bitmaps-resample-and-straighten-inflate.png)

## 其他位图命令 {#other}

- [[menu.bitmaps.crop_bitmap]] 用 [[tool.crop]] 工具裁剪位图；用 [[tool.shape]] 工具拖动位图的角也可以裁剪它。
- [[menu.bitmaps.edit_bitmap]] 在另一个程序中打开位图。
- [[menu.bitmaps.remove_background]] 使照片的背景变为透明。
- [[menu.bitmaps.remove_background_local]]: [删除背景](remove-background.md).
- [[menu.bitmaps.break_link]] 和 [[menu.bitmaps.update_from_link]] 用于链接到文件的位图。
- [[menu.bitmaps.save_bitmap_as]] 将位图保存为 PNG 文件。
