# 新增功能

TraceDraw 的每个版本都会增加功能和修复。本页介绍如何了解更改内容，并列出最近
版本的亮点。

## 在程序中查看新内容 {#in-the-program}

- 选择 [[menu.help > menu.help.whats_new]] 打开欢迎屏幕的新增功能页面。页面按从新到旧
  列出最近十个版本，每个版本都有日期和简短标题。单击某个版本即可阅读变化的
  简短介绍；最新版本已经展开。介绍下方的 **想了解更多？** 会打开该版本在
  GitHub 上的页面，其中有完整的更改列表。
- 打开 [[menu.help > menu.help.highlight_whats_new]]，在菜单中标出本版本的新
  内容：每个新命令的末尾、通向新命令的子菜单箭头旁，以及包含新命令的菜单标题
  上方都会出现一个小圆点。关闭该命令后标记消失，下次启动 TraceDraw 时该命令
  又会关闭。
- 每个版本的完整更改列表在项目的
  [更改日志](https://github.com/tracedraw/tracedraw/blob/main/CHANGELOG.md) 中。

## 最近版本的亮点 {#highlights}

- **触控和手机。** 标准工具栏上的 Shift、Ctrl 和 Alt 按钮为手指保持按下；在 Android 上界面会避开摄像头挖孔和圆角；Apple Pencil 可在 iPad 的 Safari 中使用。
- **文件。** .cdr 文件保留重叠曲线的填充规则，打开时保留制表位、多栏文本、所有页面共用的辅助线和主页面对象。
- **图片、打印和工作区。** 裁剪工具的裁剪图片像素和 [[menu.bitmaps > menu.bitmaps.crop_to_frame]] 可以裁掉图片的像素；可以灰度打印和倒序打印；[[menu.view > menu.view.scrollbars]] 可以隐藏滚动条。
- **TraceCut。** [[menu.tools > menu.tools.tracecut]] 打开用于切割绘图仪的工具：绘图仪输出、切割线、排料和测量。参见 [TraceCut](../objects/tracecut.md)。
- **键盘、手机和触控笔。** 菜单和列表可以用键盘操作。在 Android 手机上界面完整显示在屏幕上，使用触控笔时放在屏幕上的手不再误画；数位板的笔可在使用 Wayland 的 Linux 上工作。
- **文件。** .cdr 文件打开时保留制表位、主页面、渐变和变形；PDF 中的图片以完整分辨率导入；JPEG 和 TIFF 可导出为 CMYK。
- **对象缩略图。** [[docker.objects]] 面板在每个对象的名称旁显示其缩小的
  图形；在列表上按 Ctrl 滚动滚轮或拖动底部的滑块可调整大小。参见
  [组织对象和图层](../objects/objects-and-layers.md#objects-docker)。
- **使用你的语言的帮助。** 每个 **?** 按钮和 <kbd>F1</kbd> 都会以 TraceDraw
  设置的语言打开本帮助中关于眼前对话框、面板或工具的页面。
- **文档属性。** [[menu.file > menu.file.document_properties]] 保存绘图的
  语言、标题、主题、作者、版权、标签、备注和评分，并附有绘图内容的完整报告。
  请参阅 [文档属性](../pages-layout/document-properties.md)。
- **颜色管理。** [[menu.tools > menu.tools.color_management]] 设置绘图的
  配置文件和新绘图的默认值，并为打开或导入的文件设定策略。请参阅
  [颜色管理](../color/color-management.md)。
- **条形码和二维码。** 三步向导可放置条形码，二维码则是可在
  [[docker.properties]] 面板中编辑的动态对象。请参阅
  [条形码和二维码](../objects/barcodes.md)。
- **效果。** [[menu.effects]] 菜单集中了各效果面板；调和可以按固定间距沿路径
  排列，并按顺时针或逆时针旋转颜色。请参阅 [调和](../effects/blend.md)。
- **路径文本** 有自己的方向、位置、距离和偏移设置。请参阅
  [添加和编辑文本](../text/adding-text.md)。
- **字体管理器** 整理已安装的字体、文件夹、收藏和集合。请参阅
  [字体和字形](../text/fonts.md)。
