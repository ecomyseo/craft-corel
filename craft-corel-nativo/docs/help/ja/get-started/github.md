# 図面を GitHub に保管する

TraceDraw は、デスクトップでもブラウザーでも、GitHub のリポジトリから図面を開き、コミットとして保存し直せます。保存するたびにリポジトリの履歴に一つのバージョンができ、後でそれを開き直せます。コマンドは [[menu.file > menu.file.github]] にあります。

## 始める前に: トークン {#token}

公開リポジトリはトークンなしで開けます。非公開のリポジトリを開いたり保存したりするには、GitHub でリポジトリのコンテンツに読み書きのアクセス権を持つ fine-grained personal access token を作成し、[[github.token]] フィールドに貼り付けます。

トークンは TraceDraw の実行中は保持されます。次回のために残すには [[github.remember_token]] にチェックを付けます。この場合、トークンは環境設定に平文で保存されます (ブラウザーでは、そのブラウザーのストレージに保存されます)。ダイアログボックスは、最後に使ったリポジトリ、ブランチ、パスも記憶します。

## GitHub から図面を開く {#open}

1. [[menu.file > menu.file.github > menu.file.github_open]] を選びます。
2. [[github.repository]] を `owner/name` の形式か、その github.com のアドレスで入力します。
3. [[github.branch]] を入力します。空のままにすると `main` になります。
4. [[github.path]] のパスを入力します。例: `art/logo.tdraw`。
5. [[dialog.ok]] をクリックします。

図面はファイル名の付いた新しいタブで開きます。図面は、どこから来たか、どのバージョンだったかを記憶しています。

![GitHub から開くダイアログボックス](../../img/get-started-github-open.png)

## GitHub に図面を保存する {#save}

GitHub から開いた図面は、[[menu.file.save]] (<kbd>Ctrl+S</kbd>) でそのまま保存し直せます。どの図面でも、[[menu.file > menu.file.github > menu.file.github_save]] を選べます。

1. リポジトリ、ブランチ、パスを確認します。
2. 何を変更したかを書いた [[github.message]] を入力します (最初は "Update" とファイル名が入っています)。
3. [[dialog.ok]] をクリックします。

図面はフォントを含む `.tdraw` ファイルとして、そのブランチの新しいコミットとして保存されます。[[menu.file.save_as]] は、これまでどおりコンピューター上にファイルを保存します。

![GitHub に保存するダイアログボックス](../../img/get-started-github-save.png)

### GitHub が保存を拒否したとき {#conflict}

開いた後に GitHub 上でファイルが変更された場合や、そのパスに既にファイルがあり、図面がそこから開いたものでない場合、GitHub は保存を拒否し、ダイアログボックスにその旨が表示されます。別のパスかブランチに保存するか、先に新しいバージョンを開いてください。ダイアログボックスは、ほかの問題も説明します: トークンが拒否された、またはアクセス権がない、リクエストが多すぎる、リポジトリ、ブランチ、ファイルが存在しない、パスがフォルダーである、ファイルが図面ではない、図面が 100 MB より大きい、といった問題です。

## 以前のバージョンを開く {#history}

GitHub から開いた図面で、[[menu.file > menu.file.github > menu.file.github_history]] を選びます。ボックスには、そのファイルを変更したコミットが新しい順に、短い ID、日付、作成者、メッセージとともに一覧されます。一つを選んで「開く」をクリックすると、そのバージョンがファイル名とコミット名の付いた新しい図面として開きます。この図面は GitHub とは結び付いていません。

![GitHub の履歴ダイアログボックス](../../img/get-started-github-history.png)
