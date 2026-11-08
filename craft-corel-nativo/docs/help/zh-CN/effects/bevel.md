# 斜角

斜角通过照亮边缘的一侧并为另一侧加上阴影，使对象的边缘看起来凸起或雕刻。它是一种实时效果，在 [[docker.bevel]] 面板中设置；使用 [[menu.effects > menu.effects.bevel]] 打开该面板。

![斜角面板](../../img/effects-bevel.png)

## 为对象添加斜角 {#create}

1. 选择一个已填充的对象。
2. 在 [[docker.bevel]] 面板中选择样式：
   - [[docker.soft_edge]]：边缘向对象内部倾斜，一侧受光，另一侧有阴影；
   - [[docker.emboss]]：对象看起来像是从页面中压出来的。
3. 设置斜角的外观：
   - [[docker.depth]]：斜角从边缘延伸的距离，柔和边缘伸入对象内部，浮雕则伸向对象外部；
   - [[docker.level]]：沿斜角将光与影分成的层数。1 为两个平整色调，层数越多，从边缘向内的过渡越平滑；
   - [[docker.light_angle]]：光线射来的方向；
   - [[docker.intensity]]：光与影的强度；
   - [[docker.light_color]] 和 [[docker.shadow_color]]：受光一侧和阴影一侧的颜色；
   - [[docker.emboss_color]]：斜角着色所依据的颜色。勾选复选框并选择自己的颜色；关闭时使用对象自身的填充。
4. 单击 [[docker.apply]]。

要从另一个对象复制斜角，请使用 [[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]]。[[menu.object > menu.object.clear_effect]] 删除斜角。立体化面板也为正面提供斜角；请参阅 [立体化](extrude.md#settings)。
