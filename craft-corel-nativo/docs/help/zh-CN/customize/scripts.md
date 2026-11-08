# 脚本和自动化

脚本可以自动完成重复性工作：根据数据绘制形状、一次更改许多对象，或添加一种文件格式。脚本在 [[docker.scripts]] 面板（[[menu.tools > menu.tools.scripts > menu.tools.scripts_docker]]）中用 JavaScript 编写。

![脚本面板](../../img/customize-scripts.png)

## 编写和运行脚本 {#run}

脚本使用 TraceDraw 的对象：`Application`、`ActiveDocument`、`ActivePage`、`ActiveLayer`，以及形状、填充、轮廓、颜色、曲线和效果。例如：

```js
for (let i = 0; i < 5; i++) {
  ActiveLayer.CreateRectangle(10 + i * 25, 10, 20, 20);
}
```

- [[docker.run]] 运行脚本。其更改为一个撤消步骤。
- [[docker.record]] 将你的操作录制为脚本（也可用 [[menu.tools > menu.tools.scripts > menu.tools.scripts_record]]）。
- [[docker.load]] 和 [[docker.save]] 打开和保存脚本；[[menu.tools.scripts_run]] 运行脚本文件。
- [[docker.clear]] 清空编辑器；[[docker.output]] 显示脚本打印的内容及其错误。

## 调试 {#debug}

在桌面版中，单击编辑器的边距可设置断点，然后单击 [[docker.debug]]。脚本在断点处停止：可以 **继续**、**单步** 或 **停止** 它，也可以输入表达式，在脚本暂停处对其求值。

## 脚本过滤器 {#filters}

定义了 `TraceDrawFilter`（带有名称、扩展名以及 `Import` 或 `Export` 函数）的脚本会向 [[menu.file.import]] 和 [[menu.file.export]] 添加一种文件格式。单击 [[docker.install_filter]] 保留它；[[docker.script_filters]] 列出已安装的过滤器，并带有 [[docker.edit]] 和 [[docker.remove]]。

## AI 助手（MCP） {#mcp}

TraceDraw 支持模型上下文协议（Model Context Protocol），因此你计算机上的 AI 助手可以打开、编辑、检查、导出和保存绘图：

- `tracedraw --mcp` 在没有窗口的情况下为助手运行 TraceDraw。
- 要让助手编辑你窗口中打开的绘图，请在 [[menu.tools > menu.tools.options > menu.tools.options_global]] 的 [[options.general]] 页面上打开 [[options.mcp_window]]，并将 `tracedraw --mcp-window` 作为命令提供给助手。每项更改都是一个可以撤回的撤消步骤。只有你计算机上持有 TraceDraw 所写秘密令牌的程序才能连接。

## 命令行工具 {#cli}

每个下载包中还有 `tracedraw-cli`，它无需打开程序即可在格式之间转换绘图，适合批量处理。

## 宏 {#macros}

TraceDraw 可以运行为绘图程序编写的 VBA 宏项目（桌面版）。[[menu.tools > menu.tools.macros > menu.tools.macros_load]] 添加一个项目（`.gms`）或模块文件（`.bas`、`.cls`、`.frm`）；项目会被保留并在启动时加载。[[docker.macros]] 列出其中的宏：双击某个宏或使用 [[menu.tools > menu.tools.macros > menu.tools.macros_run]]（<kbd>Alt+Shift+F8</kbd>）运行它。宏的窗体、消息框和输入框会显示在屏幕上，<kbd>Esc</kbd> 或 [[menu.tools > menu.tools.macros > menu.tools.macros_stop]] 可停止它，一次运行所做的全部更改为一个撤销步骤。[[menu.tools > menu.tools.macros > menu.tools.macro_editor]]（<kbd>Alt+F11</kbd>）显示并编辑代码。调用 Windows 函数或编辑器自身对象模型的宏无法运行。
