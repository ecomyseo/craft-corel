# 將繪圖保存在 GitHub 上

TraceDraw 可以從 GitHub 儲存庫開啟繪圖，並以提交 (commit) 的方式存回，桌面版和瀏覽器版皆可。每次儲存都會成為儲存庫歷程中的一個版本，之後可以再次開啟。這些指令位於 [[menu.file > menu.file.github]]。

## 開始之前：權杖 {#token}

公開儲存庫不需要權杖即可開啟。若要開啟私人儲存庫以及進行儲存，請在 GitHub 上建立一個細粒度個人存取權杖 (fine-grained personal access token)，並授予該儲存庫內容的讀取和寫入權限，然後將它貼到 [[github.token]] 欄位中。

TraceDraw 執行期間會保留權杖。勾選 [[github.remember_token]] 可在下次繼續使用；此時權杖會以純文字儲存在您的偏好設定中（在瀏覽器中則儲存在該瀏覽器的儲存空間）。對話方塊也會記住上次使用的儲存庫、分支和路徑。

## 從 GitHub 開啟繪圖 {#open}

1. 選擇 [[menu.file > menu.file.github > menu.file.github_open]]。
2. 以 `owner/name` 的形式或其 github.com 位址輸入 [[github.repository]]。
3. 輸入 [[github.branch]]，或保留空白以使用 `main`。
4. 輸入 [[github.path]] 的路徑，例如 `art/logo.tdraw`。
5. 按一下 [[dialog.ok]]。

繪圖會在新的分頁中開啟，並以檔案名稱命名。它會記住來源位置以及是哪一個版本。

![從 GitHub 開啟對話方塊](../../img/get-started-github-open.png)

## 將繪圖儲存到 GitHub {#save}

對於從 GitHub 開啟的繪圖，[[menu.file.save]]（<kbd>Ctrl+S</kbd>）會將它存回。對於任何繪圖，請選擇 [[menu.file > menu.file.github > menu.file.github_save]]。

1. 檢查儲存庫、分支和路徑。
2. 撰寫一則 [[github.message]]，說明變更了什麼（預設為 "Update" 加上檔案名稱）。
3. 按一下 [[dialog.ok]]。

繪圖會連同其字型儲存為 `.tdraw` 檔案，並成為該分支上的一個新提交。[[menu.file.save_as]] 仍然會將檔案儲存在您的電腦上。

![儲存到 GitHub 對話方塊](../../img/get-started-github-save.png)

### GitHub 拒絕儲存時 {#conflict}

如果檔案在您開啟後已在 GitHub 上變更，或者該路徑已經有檔案而您的繪圖並非來自該檔案，GitHub 就會拒絕儲存，對話方塊會告知此情況。請儲存到其他路徑或分支，或先開啟較新的版本。對話方塊也會說明其他問題：權杖遭拒或沒有存取權限、要求過多、儲存庫、分支或檔案不存在、路徑是資料夾、檔案不是繪圖，或繪圖大於 100 MB。

## 開啟較早的版本 {#history}

對於從 GitHub 開啟的繪圖，請選擇 [[menu.file > menu.file.github > menu.file.github_history]]。方塊中會列出變更過該檔案的提交，最新的在最前面，並顯示其簡短 ID、日期、作者和訊息。選取一個並按一下「開啟」：該版本會以新繪圖的形式開啟，以檔案名稱和提交命名，且不與 GitHub 連結。

![GitHub 上的歷程記錄對話方塊](../../img/get-started-github-history.png)
