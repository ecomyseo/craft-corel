# 颜色管理

屏幕用红、绿、蓝三色光显示颜色；打印机则混合青、品红、黄和黑色油墨。颜色管理使用 **颜色配置文件** 描述每台设备，使颜色从屏幕到印刷页面都保持相同的外观。设置位于 [[menu.tools > menu.tools.color_management]]，即 [[color_mgmt.title]] 对话框。在你单击 [[dialog.ok]] 之前，什么都不会改变。

![颜色管理设置对话框](../../img/color-color-management.png)

## 绘图的配置文件 {#document}

[[color_mgmt.tab_document]] 选项卡设置活动绘图的配置文件。

- [[color_mgmt.current_profiles]] 显示绘图当前使用的 RGB、CMYK 和灰度配置文件。
- 选择新配置文件的应用方式：
  - [[color_mgmt.assign]]：颜色值保持不变，其外观按新配置文件改变；
  - [[color_mgmt.convert]]：转换 CMYK 颜色，使其在新配置文件中看起来相同。
- 选择 [[color_mgmt.rgb_profile]]、[[color_mgmt.cmyk_profile]] 和 [[color_mgmt.grayscale]] 配置文件。每个列表先列出默认配置文件，然后是在你的计算机上找到的所有配置文件。[[color_mgmt.load_profiles]] 添加配置文件（`.icc`、`.icm`）。
- [[color_mgmt.primary_mode]]：CMYK 或 RGB。
- [[color_mgmt.intent]]：如何处理无法显示的颜色：[[color_mgmt.intent_relative]]（起初选用）、[[color_mgmt.intent_perceptual]]、[[color_mgmt.intent_saturation]] 或 [[color_mgmt.intent_absolute]]。

CMYK 配置文件随绘图一起保存，从版本 16 起也保存到 `.cdr` 文件中。

## 新绘图的默认值 {#default}

[[color_mgmt.tab_default]] 选项卡设置新绘图使用的配置以及颜色的转换方式。

- [[color_mgmt.preset]]：适用于北美、欧洲、日本或网页和屏幕工作的现成设置。[[color_mgmt.save_preset]] 以 [[color_mgmt.preset_name]] 保存你自己的设置；[[color_mgmt.delete_preset]] 删除一个预设。
- 默认的 [[color_mgmt.rgb_profile]]、[[color_mgmt.cmyk_profile]] 和 [[color_mgmt.gray_profile]]、[[color_mgmt.primary_mode]] 和 [[color_mgmt.intent]]。
- [[color_mgmt.conversion]]：
  - [[color_mgmt.engine]]：使用配置文件的 [[color_mgmt.engine_icc]]，或 [[color_mgmt.engine_builtin]]；
  - [[color_mgmt.preserve_black]]：只用黑色油墨的颜色在转换后仍只用黑色油墨；
  - [[color_mgmt.map_gray]]（起初打开）：转换为 CMYK 的灰色只使用黑色油墨；
  - [[dialog.black_point]]（起初打开）：使最暗的色调保持区分。
- [[color_mgmt.open_policy]]：打开文件时其配置文件如何处理，分别针对 RGB、CMYK 和灰度：[[color_mgmt.open_use_embedded]]、[[color_mgmt.open_assign_default]] 或 [[color_mgmt.open_convert_default]]。当文件使用其他配置文件或没有配置文件时，[[color_mgmt.warn_mismatch_box]] 和 [[color_mgmt.warn_missing_box]] 会在状态栏中显示消息。
- [[color_mgmt.import_policy]]：对导入或粘贴的内容使用 [[color_mgmt.import_convert]] 或 [[color_mgmt.import_assign]]。
- [[color_mgmt.spot_definition]]：专色由 [[color_mgmt.spot_lab]]、[[color_mgmt.spot_cmyk]] 还是 [[color_mgmt.spot_rgb]] 定义。

## 预览印刷颜色 {#proof}

[[menu.view > menu.view.proof_colors]]，或状态栏右端的校样按钮，按印刷效果显示绘图：CMYK 油墨无法印刷的颜色看起来更暗淡。[[docker.color]] 面板也会对这类颜色发出警告。

未加载配置文件时，TraceDraw 使用其自身的铜版纸印刷油墨模型转换颜色。
