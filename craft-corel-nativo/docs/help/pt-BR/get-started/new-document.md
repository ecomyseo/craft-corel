# Criar um desenho

Todo desenho começa com um tamanho de página, um número de páginas, um modo de
cor e uma resolução. Você os escolhe na caixa de diálogo
[[dialog.create_new_document]].

## Para criar um desenho {#create}

1. Escolha [[menu.file > menu.file.new]] (<kbd>Ctrl+N</kbd>), clique em Novo
   na barra de ferramentas padrão, clique no **+** depois da última guia de
   documento ou clique em Novo documento na Tela de boas-vindas.
2. Na caixa de diálogo [[dialog.create_new_document]], defina os valores
   descritos abaixo.
3. Clique em [[dialog.ok]] ou pressione <kbd>Enter</kbd>.

O desenho abre em uma guia própria. A caixa de diálogo lembra os valores para o
próximo desenho.

![A caixa de diálogo Criar um novo documento](../../img/get-started-new-document.png)

## Geral {#general}

- [[dialog.name]] dá nome ao desenho. Começa como o próximo `Untitled-N`.
- [[newdoc.preset]] define de uma vez o modo de cor, o tamanho da página, as
  unidades e a resolução. As predefinições são [[newdoc.preset_default]] e
  [[newdoc.preset_cmyk]] (CMYK, A4, milímetros, 300 dpi),
  [[newdoc.preset_rgb]] (RGB, A4, milímetros, 300 dpi) e
  [[newdoc.preset_web]] (RGB, 1920 x 1080 pixels, 72 dpi). Quando você muda um
  desses valores, a predefinição mostra [[newdoc.preset_custom]], a menos que
  os valores voltem a coincidir com uma predefinição.
- O botão ao lado da predefinição contém [[newdoc.save_preset]], que pede um
  [[newdoc.preset_name]] e acrescenta os valores atuais à lista, e
  [[newdoc.delete_preset]], que remove uma predefinição que você salvou. As
  predefinições de fábrica não podem ser excluídas.
- [[newdoc.pages]] define quantas páginas o desenho tem, de 1 a 999. Cada
  página se chama "Page N" e tem uma camada.
- [[newdoc.color_mode]] é CMYK, para impressão, ou RGB, para telas.

## Dimensões {#dimensions}

- [[newdoc.page_size]] lista os tamanhos de papel, ou Personalizado. Escolher
  um tamanho mantém a orientação atual. Uma largura e uma altura digitadas que
  coincidam com um tamanho de papel, em qualquer orientação, selecionam o nome
  dele.
- [[newdoc.width]] e [[newdoc.height]] definem o tamanho da página, nas
  unidades escolhidas na lista ao lado da largura. Essas unidades passam a ser
  as unidades de desenho da barra de propriedades.
- [[newdoc.orientation]]: [[dialog.portrait]] ou [[dialog.landscape]].
- [[newdoc.resolution]] em [[newdoc.dpi]] (300 de início). É usada pelos
  efeitos que viram pixels e para medir tamanhos em pixels: 1920 pixels são
  162,56 mm a 300 dpi e 677,33 mm a 72 dpi. Quando as unidades são pixels,
  mudar a resolução mantém o tamanho da página em pixels.

## Configurações de cor {#color-settings}

Abra [[newdoc.color_settings]] para escolher o [[newdoc.rgb_profile]], o
[[newdoc.cmyk_profile]] e o [[newdoc.gray_profile]] do desenho e a
[[newdoc.intent]]. Começam com sRGB, o perfil CMYK incluído, escala de cinza
com ganho de ponto de 20% e colorimétrico relativo. Veja
[Gerenciamento de cores](../color/color-management.md).

## Pular a caixa de diálogo {#skip}

Marque [[newdoc.dont_show]] para criar novos desenhos de imediato, com os
valores usados por último. Para ver a caixa de diálogo de novo, ative a opção
correspondente na página [[options.general]] de
[[menu.tools > menu.tools.options > menu.tools.options_app]]. Veja
[Opções do TraceDraw](../customize/options-app.md#general).

## Outras formas de começar {#other-ways}

- [[menu.file > menu.file.new_from_template]] começa a partir de um modelo.
  Veja [Modelos](../templates-styles/templates.md).
- [[menu.file > menu.file.open]] abre um desenho existente. Veja
  [Abrir e salvar desenhos](open-and-save.md).
