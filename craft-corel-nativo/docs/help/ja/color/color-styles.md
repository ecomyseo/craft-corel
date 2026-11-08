# カラースタイル

**カラースタイル** は、図面のオブジェクトが使う名前付きの色です。スタイルを変えると、それで塗りつぶしたすべてのオブジェクトも変わります。デザインの色を一貫させ、別の配色をすばやく試せます。スタイルは [[docker.color_styles]] パネルに一覧されます。[[menu.window > menu.window.dockers]] から開きます。

![カラースタイルパネル](../../img/color-color-styles.png)

## カラースタイルを作る {#create}

- [[docker.new_from_selection]] は、選択したオブジェクトの塗りつぶしの色のうち、まだスタイルになっていないものそれぞれにスタイルを追加します。
- [[docker.new_color_style]] は、[[docker.color]] パネルの現在の色のスタイルを追加します。

## スタイルを使う、編集する {#edit}

各スタイルには、色、名前、三つのボタンが表示されます:

- 色をクリックすると変更できます。その色で塗りつぶしたオブジェクトも追従します。
- 名前をクリックすると名前を変更できます。
- [[docker.fill]] は選択範囲をそのスタイルで塗りつぶし、[[docker.outline]] はそのアウトラインに色を付けます。
- **x** はスタイルを削除します (オブジェクトの色はそのまま残ります)。

## ハーモニー {#harmonies}

[[docker.harmony_editor]] は、[[docker.base_color]] から、調和するスタイルのセットを作ります:

1. [[docker.base_color]] を選びます。
2. ハーモニーをクリックします: [[harmony.analogous]]、[[harmony.complementary]]、[[harmony.split_complementary]]、[[harmony.triadic]]、[[harmony.tetradic]]、[[harmony.monochromatic]]。

ハーモニーの色が、その名前を付けたスタイルとして追加されます。
