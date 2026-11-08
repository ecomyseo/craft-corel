# TraceDraw について、および更新

TraceDraw は無料でオープンソースです。このページでは、使っているバージョンの
確認方法、新しいバージョンの入手方法、助けを求める場所を説明します。

## バージョン情報ボックス {#about}

[[menu.help > menu.help.about]] を選ぶと、実行中のバージョン、作者、
ライセンス、ソースコードへのリンク、プログラムの構成要素が表示されます。
[[dialog.close]]、<kbd>Enter</kbd> または <kbd>Esc</kbd> で閉じます。

![TraceDraw のバージョン情報ボックス](../../img/welcome-about-and-updates-about.png)

## 更新の確認 {#updates}

デスクトップ版の TraceDraw は、新しいバージョンが公開されたときに知らせる
ことができます。

今すぐ確認するには:

1. [[menu.help > menu.help.updates]] を選びます。
2. TraceDraw はプロジェクトのリリースページに最新バージョンを問い合わせ、
   お使いのバージョンより新しいかどうかを知らせます。テスト版とナイトリー
   ビルドが提示されることはありません。
3. 新しいバージョンがある場合は、操作を選びます。
   - **ダウンロード** は、お使いのシステム用のファイルのダウンロードを開き
     ます (Windows ではインストーラーまたはポータブルアーカイブ、Mac では
     プロセッサーに合ったディスクイメージ、Linux では AppImage または
     アーカイブ)。インストールはシステムのインストーラーが行います。
   - **リリースノート** は、そのバージョンの変更点の一覧を開きます。
   - **このバージョンをスキップ** は、そのバージョンを今後提示しません。
   - [[dialog.close]] はボックスを閉じます。

TraceDraw が Linux の AppImage として動いているときは、ボックスに
**インストール** も表示されることがあります。新しい AppImage を実行中のものの
隣にダウンロードし、ダウンロードが完全で正規のものかを確認して、古いファイル
と置き換えます。新しいバージョンは次に TraceDraw を開いたときに起動します。
確認できない点があれば、古いファイルはそのまま残り、ボックスが理由を示します。

確認に失敗した場合 (インターネット接続がない場合など) は、ボックスがそれを
知らせ、リリースページを提示します。

![更新ボックス](../../img/welcome-about-and-updates-updates.png)

### 自動確認 {#automatic-check}

[[menu.tools > menu.tools.options > menu.tools.options_global]] の
[[options.general]] ページには、起動時に更新を確認する設定があります。
最初はオンになっており、TraceDraw は 1 日に 1 回起動時に静かに確認し、
スキップしていない新しいバージョンが出たときだけ知らせます。

TraceDraw は初回起動時に、アップデートを確認するかどうかを尋ねます: [[updates.policy_always]] または [[updates.policy_never]]。回答するまで、TraceDraw が自分から接続することはありません。回答はこの設定に反映され、ここでいつでも変更できます。

> **注:** ブラウザー版は常に最新です。そこでは
> [[menu.help > menu.help.updates]] がリリースページを開きます。

## ウェルカム画面 {#welcome-screen}

ウェルカム画面は TraceDraw の起動時に最初に表示されるタブです。ここから
図面を作成したり、最近使った図面 (それぞれサムネイル付き。
[[menu.file.clear_recent]] で一覧を消去) を開いたり、テンプレートから始めたり、
ニュースを読んだり、学習資料を探したりできます。
[[menu.help > menu.help.welcome_screen]] を選ぶと再び表示されます。
[[menu.help > menu.help.video_tutorials]]、[[menu.help.quick_start_guide]]、
[[menu.help.user_guide]] はその学習ページを、[[menu.help.whats_new]] は
ニュースを開きます。

## コミュニティとサポート {#community}

- [[menu.help > menu.help.community]] はプロジェクトのディスカッション
  フォーラムを開きます。質問したり、アイデアを共有したりできます。
- [[menu.help > menu.help.support]] は問題の追跡システムを開きます。問題を
  報告できます。
- [[menu.help > menu.help.message_settings]] は TraceDraw のメッセージを
  設定するオプションを開きます。
