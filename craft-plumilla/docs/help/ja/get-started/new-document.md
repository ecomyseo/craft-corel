# 図面を作成する

どの図面も、ページサイズ、ページ数、カラーモード、解像度を持って始まります。これらは [[dialog.create_new_document]] ダイアログボックスで選びます。

## 図面を作成するには {#create}

1. [[menu.file > menu.file.new]] (<kbd>Ctrl+N</kbd>) を選ぶか、標準ツールバーの「新規」をクリックするか、最後のドキュメントタブの後ろの **+** をクリックするか、ようこそ画面の「新規ドキュメント」をクリックします。
2. [[dialog.create_new_document]] ダイアログボックスで、以下に説明する値を設定します。
3. [[dialog.ok]] をクリックするか <kbd>Enter</kbd> を押します。

図面は専用のタブで開きます。ダイアログボックスは、次の図面のために設定した値を記憶します。

![新しいドキュメントの作成ダイアログボックス](../../img/get-started-new-document.png)

## 一般 {#general}

- [[dialog.name]] は図面の名前です。最初は次の `Untitled-N` になっています。
- [[newdoc.preset]] は、カラーモード、ページサイズ、単位、解像度を一度に設定します。プリセットは [[newdoc.preset_default]] と [[newdoc.preset_cmyk]] (CMYK、A4、ミリメートル、300 dpi)、[[newdoc.preset_rgb]] (RGB、A4、ミリメートル、300 dpi)、[[newdoc.preset_web]] (RGB、1920 x 1080 ピクセル、72 dpi) です。これらの値のどれかを変えると、値が再びプリセットと一致しない限り、プリセットは [[newdoc.preset_custom]] と表示されます。
- プリセットの横のボタンには [[newdoc.save_preset]] と [[newdoc.delete_preset]] があります。前者は [[newdoc.preset_name]] を尋ねて現在の値を一覧に追加し、後者は保存したプリセットを削除します。組み込みのプリセットは削除できません。
- [[newdoc.pages]] は図面のページ数を 1 から 999 の範囲で設定します。各ページは「ページ N」という名前で、レイヤーを一つ持ちます。
- [[newdoc.color_mode]] は、印刷用の CMYK か、画面用の RGB です。

## サイズ {#dimensions}

- [[newdoc.page_size]] は用紙サイズ、またはカスタムを一覧します。サイズを選ぶと、現在の向きはそのまま保たれます。入力した幅と高さがどちらかの向きで用紙サイズと一致すると、その名前が選ばれます。
- [[newdoc.width]] と [[newdoc.height]] はページサイズを、幅の横の一覧で選んだ単位で設定します。その単位がプロパティバーの図面単位になります。
- [[newdoc.orientation]]: [[dialog.portrait]] または [[dialog.landscape]]。
- [[newdoc.resolution]] は [[newdoc.dpi]] 単位です (最初は 300)。ピクセルに変換される効果と、ピクセル単位でのサイズ計測に使われます。1920 ピクセルは 300 dpi では 162.56 mm、72 dpi では 677.33 mm です。単位がピクセルのときに解像度を変えると、ページのピクセル単位のサイズが保たれます。

## カラー設定 {#color-settings}

[[newdoc.color_settings]] を開いて、図面の [[newdoc.rgb_profile]]、[[newdoc.cmyk_profile]]、[[newdoc.gray_profile]] と [[newdoc.intent]] を選びます。最初は sRGB、組み込みの CMYK プロファイル、ドットゲイン 20% のグレースケール、相対的な色域を維持、になっています。[カラーマネジメント](../color/color-management.md) を参照してください。

## ダイアログボックスを省略する {#skip}

[[newdoc.dont_show]] にチェックを付けると、前回使った値で新しい図面がすぐに作成されます。ダイアログボックスを再び表示するには、[[menu.tools > menu.tools.options > menu.tools.options_app]] の [[options.general]] ページで対応する設定をオンにします。[TraceDraw のオプション](../customize/options-app.md#general) を参照してください。

## そのほかの始め方 {#other-ways}

- [[menu.file > menu.file.new_from_template]] はテンプレートから始めます。[テンプレート](../templates-styles/templates.md) を参照してください。
- [[menu.file > menu.file.open]] は既存の図面を開きます。[図面を開く、保存する](open-and-save.md) を参照してください。
