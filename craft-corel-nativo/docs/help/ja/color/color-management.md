# カラーマネジメント

画面は赤、緑、青の光で色を表示し、プリンターはシアン、マゼンタ、イエロー、ブラックのインキを混ぜます。カラーマネジメントは **カラープロファイル** で各デバイスを記述し、画面から印刷されたページまで色の見え方を保ちます。設定は [[menu.tools > menu.tools.color_management]]、つまり [[color_mgmt.title]] ダイアログボックスにあります。[[dialog.ok]] をクリックするまでは何も変わりません。

![カラーマネジメント設定ダイアログボックス](../../img/color-color-management.png)

## 図面のプロファイル {#document}

[[color_mgmt.tab_document]] タブは、作業中の図面のプロファイルを設定します。

- [[color_mgmt.current_profiles]] は、図面が現在使っている RGB、CMYK、グレースケールのプロファイルを表示します。
- 新しいプロファイルの適用方法を選びます:
  - [[color_mgmt.assign]]: 色の値はそのままで、見え方が新しいプロファイルに合わせて変わります。
  - [[color_mgmt.convert]]: CMYK の色を変換し、新しいプロファイルでも同じに見えるようにします。
- [[color_mgmt.rgb_profile]]、[[color_mgmt.cmyk_profile]]、[[color_mgmt.grayscale]] のプロファイルを選びます。各一覧には、既定のプロファイルに続いて、コンピューター上で見つかったすべてのプロファイルが並びます。[[color_mgmt.load_profiles]] は、プロファイルのファイル (`.icc`、`.icm`) を追加します。
- [[color_mgmt.primary_mode]]: CMYK または RGB。
- [[color_mgmt.intent]]: 表示できない色をどのように範囲内に収めるか: [[color_mgmt.intent_relative]] (最初の設定)、[[color_mgmt.intent_perceptual]]、[[color_mgmt.intent_saturation]]、[[color_mgmt.intent_absolute]]。

CMYK プロファイルは図面と一緒に保存され、バージョン 16 以降の `.cdr` ファイルにも入ります。

## 新しい図面の既定値 {#default}

[[color_mgmt.tab_default]] タブは、新しい図面が使う設定と、色の変換方法を設定します。

- [[color_mgmt.preset]]: 北米、ヨーロッパ、日本、または Web と画面の作業向けの既成の設定です。[[color_mgmt.save_preset]] は自分の設定を [[color_mgmt.preset_name]] を付けて保存し、[[color_mgmt.delete_preset]] はそれを削除します。
- 既定の [[color_mgmt.rgb_profile]]、[[color_mgmt.cmyk_profile]]、[[color_mgmt.gray_profile]]、[[color_mgmt.primary_mode]]、[[color_mgmt.intent]]。
- [[color_mgmt.conversion]]:
  - [[color_mgmt.engine]]: プロファイルを使う [[color_mgmt.engine_icc]]、または [[color_mgmt.engine_builtin]]。
  - [[color_mgmt.preserve_black]]: 黒インキだけの色は、変換しても黒インキだけのままにします。
  - [[color_mgmt.map_gray]] (最初はオン): CMYK に変換したグレーは黒インキだけを使います。
  - [[dialog.black_point]] (最初はオン): 最も暗い濃淡どうしを区別できるように保ちます。
- [[color_mgmt.open_policy]]: 開いたファイルのプロファイルを RGB、CMYK、グレースケールのそれぞれでどう扱うか: [[color_mgmt.open_use_embedded]]、[[color_mgmt.open_assign_default]]、[[color_mgmt.open_convert_default]]。[[color_mgmt.warn_mismatch_box]] と [[color_mgmt.warn_missing_box]] は、ファイルのプロファイルが違うときやプロファイルがないときに、ステータスバーにメッセージを表示します。
- [[color_mgmt.import_policy]]: インポートまたは貼り付けたものについて、[[color_mgmt.import_convert]] または [[color_mgmt.import_assign]]。
- [[color_mgmt.spot_definition]]: 特色を [[color_mgmt.spot_lab]]、[[color_mgmt.spot_cmyk]]、[[color_mgmt.spot_rgb]] のどれで定義するか。

## 印刷の色をプレビューする {#proof}

[[menu.view > menu.view.proof_colors]]、またはステータスバーの右端の校正ボタンは、図面を印刷したときの見え方で表示します。CMYK インキで印刷できない色はくすんで見えます。[[docker.color]] パネルも、そのような色について警告します。

プロファイルを読み込んでいない場合、TraceDraw はコート紙上のプロセスインキの独自のモデルで色を変換します。
