# パターン塗りつぶし

パターン塗りつぶしは、オブジェクトの内側全体にタイルを繰り返します。タイルは 2 色、ベクターオブジェクトのセット (フルカラーのベクターパターン)、またはビットマップです。タイルはページに固定されているので、オブジェクトを移動してもずれません。

## 自分のオブジェクトからパターンを作る {#create-pattern}

1. 1 枚のタイルを構成するオブジェクトを描き、それらを選択します。
2. [[menu.object > menu.object.create > menu.object.create_pattern]] を選びます。
3. [[dialog.create_pattern]] ダイアログボックスで [[dialog.create_pattern_type]] を選びます: [[dialog.create_pattern_vector]] はオブジェクトをベクターのまま保ち、どのサイズでもくっきりします。[[dialog.create_pattern_bitmap]] はそれらの画像を作ります。
4. [[dialog.ok]] をクリックします。

![パターンの作成ダイアログボックス](../../img/color-patterns-create-pattern.png)

パターンは新しいオブジェクトの塗りつぶしになります。既に描いたオブジェクトに適用するには、[[docker.properties]] パネルの塗りつぶしセクションで [[docker.fill_pattern]] を選びます。

## パターンの設定 {#settings}

[[docker.properties]] パネルの塗りつぶしセクションで [[docker.fill_pattern]] を選び、続いてパターンの種類とその設定を選びます。ベクターパターンでは、タイルに含まれるオブジェクトの数と、その幅と高さが表示されます。幅と高さを変えると、タイルの内容も一緒に拡大縮小されます。

## エクスポートしたファイルでのパターン {#export}

SVG ファイルでは、ベクターパターンはベクターのまま保たれます。PDF と EPS ファイルでは、エクスポートの解像度の画像になり、オブジェクトの形で切り抜かれます。
