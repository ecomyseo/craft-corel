# 效果面板

一个对象可以同时带有多个实时效果，依次应用。[[docker.effects]] 面板按应用顺序从上到下列出所选对象的效果。使用 [[menu.window > menu.window.dockers > menu.window.dockers_effects]] 打开它。

![效果面板](../../img/effects-effects-docker.png)

## 管理效果 {#stack}

列表的每一行都有：

- 一个打开或关闭效果的复选框。关闭的效果保留其位置和设置，但不会被绘制、导出或打印；
- 效果的名称，关闭时显示为暗淡；
- 上下箭头，将效果移到更早或更晚应用，这会改变结果（例如封套的轮廓图，或轮廓图的封套）；
- 一个删除效果的叉号。

每项更改为一个撤消步骤。每种效果仍使用其自身的工具或面板进行编辑。

## 其他效果命令 {#commands}

- [[menu.effects > menu.effects.flatten]] 将所选对象的效果变为普通对象。
- [[menu.effects.picker]] 显示可应用到所选内容的效果列表。
- [[menu.object > menu.object.clear_effect]] 删除所选内容的效果。
- [[menu.object > menu.object.copy_effect]] 从另一个对象复制效果，[[menu.object.clone_effect]] 创建跟随它的克隆；请参阅 [复制和克隆效果](copy-and-clone.md)。
