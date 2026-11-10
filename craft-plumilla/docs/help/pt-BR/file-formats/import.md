# Importar arquivos

[[menu.file > menu.file.import]] (<kbd>Ctrl+I</kbd>) acrescenta o conteúdo de
um arquivo ao desenho aberto, na camada ativa, como um passo de desfazer.
[[menu.file.open]] abre o arquivo como um desenho próprio. Os dois leem os
formatos listados em [Formatos de arquivo suportados](supported-formats.md#open).

## Onde ficam os objetos importados {#placement}

De início, os objetos importados vão para onde o arquivo os tem. Para centrá-los
na vista, mude a configuração na página [[options.file_formats]] das opções
globais; veja [Opções globais](../customize/global-options.md#file-formats).

## Observações sobre alguns formatos {#formats}

- **PDF e AI**: a caixa de diálogo [[pdf_import.title]] pergunta como ler o
  texto: como texto editável, que mantém as fontes quando elas estão
  instaladas, ou como curvas, desenhadas a partir das fontes que o arquivo
  embute, para o texto ficar exatamente como no arquivo. Opções > Formatos de
  arquivo guarda a escolha. Um arquivo protegido pede a senha na caixa de
  diálogo [[pdf_password.title]].
- **EPS e PostScript**: cada página impressa vira uma página.
- **PSD**: cada camada vira um bitmap com opacidade, visibilidade e máscara.
- **DXF**: as unidades de desenho vêm do arquivo.
- **Arquivos de texto** (TXT, RTF, DOCX): viram texto de parágrafo, com
  negrito, itálico, tamanhos e fontes onde o arquivo os tem.
- **Bitmaps**: colocados na resolução deles; veja [Bitmaps](../bitmaps/index.md).

Quando alguns objetos de um arquivo não podem ser lidos, ou faltam fontes, uma
mensagem avisa; veja
[Observações após abrir](../get-started/open-and-save.md#open-notes). Você
também pode arrastar arquivos para a janela, ou colar de outros programas; veja
[Colar especial](../objects/paste-special.md).

## Arquivos PDF de várias páginas {#pdf-pages}

Importar um arquivo PDF ou AI de mais de uma página abre a caixa de diálogo
[[pdf_pages.title]], depois das perguntas sobre o texto e a senha. Escolha
[[pdf_pages.all]] ou digite as páginas, como `1-3, 5`; a lista mostra cada
página com o tamanho e a orientação, e marcar uma página a acrescenta. Depois
escolha onde elas ficam:

- [[pdf_pages.new_pages]]: cada página vira uma página do desenho, com o
  próprio tamanho e orientação, na ordem do arquivo. Assim você junta arquivos
  PDF num só desenho.
- [[pdf_pages.current_page]]: os objetos vão para a camada ativa; com várias
  páginas, os objetos de cada página formam um grupo.

A importação inteira é um passo de desfazer. Um arquivo de uma página entra
sem a caixa de diálogo.
