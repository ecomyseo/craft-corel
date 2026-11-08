# Guardar desenhos no GitHub

O TraceDraw pode abrir desenhos de um repositório do GitHub e salvá-los de
volta como commits, no computador e no navegador. Cada salvamento vira uma
versão no histórico do repositório, que você pode abrir de novo depois. Os
comandos ficam em [[menu.file > menu.file.github]].

## Antes de começar: o token {#token}

Repositórios públicos abrem sem token. Para abrir os privados e para salvar,
crie no GitHub um token de acesso pessoal refinado com acesso de leitura e
escrita ao conteúdo do repositório, e cole-o no campo [[github.token]].

O token fica guardado enquanto o TraceDraw está aberto. Marque
[[github.remember_token]] para mantê-lo para a próxima vez; ele é então
guardado em texto simples nas suas preferências (no navegador, no
armazenamento daquele navegador). As caixas de diálogo também lembram o último
repositório, branch e caminho.

## Abrir um desenho do GitHub {#open}

1. Escolha [[menu.file > menu.file.github > menu.file.github_open]].
2. Digite o [[github.repository]] como `dono/nome` ou o endereço dele no
   github.com.
3. Digite a [[github.branch]], ou deixe em branco para `main`.
4. Digite o caminho do [[github.path]], por exemplo `art/logo.tdraw`.
5. Clique em [[dialog.ok]].

O desenho abre em uma nova guia, com o nome do arquivo. Ele lembra de onde veio
e qual versão era.

![A caixa de diálogo Abrir do GitHub](../../img/get-started-github-open.png)

## Salvar um desenho no GitHub {#save}

Para um desenho aberto do GitHub, [[menu.file.save]] (<kbd>Ctrl+S</kbd>) o salva
de volta. Para qualquer desenho, escolha
[[menu.file > menu.file.github > menu.file.github_save]].

1. Confira o repositório, a branch e o caminho.
2. Escreva uma [[github.message]] que diga o que mudou (começa com "Update" e o
   nome do arquivo).
3. Clique em [[dialog.ok]].

O desenho é salvo como arquivo `.tdraw`, com as fontes, como um novo commit
nessa branch. [[menu.file.save_as]] continua salvando um arquivo no seu
computador.

![A caixa de diálogo Salvar no GitHub](../../img/get-started-github-save.png)

### Quando o GitHub recusa um salvamento {#conflict}

Se o arquivo mudou no GitHub depois que você o abriu, ou já existe um arquivo
naquele caminho e o seu desenho não veio dele, o GitHub recusa o salvamento e a
caixa de diálogo avisa. Salve em outro caminho ou branch, ou abra a versão mais
nova primeiro. A caixa de diálogo também explica outros problemas: um token
recusado ou sem acesso, pedidos demais, um repositório, branch ou arquivo que
não existe, um caminho que é uma pasta, um arquivo que não é um desenho, ou um
desenho com mais de 100 MB.

## Abrir uma versão anterior {#history}

Para um desenho aberto do GitHub, escolha
[[menu.file > menu.file.github > menu.file.github_history]]. A caixa lista os
commits que mudaram o arquivo, do mais novo ao mais antigo, com id curto, data,
autor e mensagem. Selecione um e clique em Abrir: essa versão abre como um novo
desenho, com o nome do arquivo e do commit, que não fica ligado ao GitHub.

![A caixa de diálogo Histórico no GitHub](../../img/get-started-github-history.png)
