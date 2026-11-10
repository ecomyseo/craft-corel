# Novidades

> *Tradução automática: esta página pode estar atrasada em relação à versão em inglês.*

Cada versão do TraceDraw traz recursos e correções. Esta página explica como
saber o que mudou e lista os destaques das versões mais recentes.

## Ver as novidades no programa {#in-the-program}

- Escolha [[menu.help > menu.help.whats_new]] para abrir a página de novidades
  da Tela de boas-vindas. Ela lista as dez últimas versões, da mais nova
  para a mais antiga, cada uma com a data e um título curto. Clique numa
  versão para ler um resumo curto do que mudou; a mais nova já vem aberta.
  **Quer saber mais?** embaixo de um resumo abre a página dessa versão no
  GitHub, com a lista completa de mudanças.
- Ative [[menu.help > menu.help.highlight_whats_new]] para marcar nos menus o
  que é novo nesta versão: um pequeno ponto aparece no fim de cada comando
  novo, ao lado da seta de um submenu que leva a um deles e sobre o título de
  um menu que contém um. As marcas somem quando você desativa o comando, e ele
  volta desativado na próxima vez que o TraceDraw é iniciado.
- A lista completa de mudanças de cada versão está no
  [registro de alterações](https://github.com/tracedraw/tracedraw/blob/main/CHANGELOG.md)
  do projeto.

## Destaques das versões recentes {#highlights}

- **Toque e celulares.** Os botões Shift, Ctrl e Alt da barra padrão ficam
  pressionados para um dedo; no Android a interface fica longe do entalhe da
  câmera e dos cantos arredondados; o Apple Pencil funciona no Safari do iPad.
- **Arquivos.** Arquivos .cdr mantêm a regra de preenchimento das curvas
  sobrepostas e abrem com suas tabulações, texto em várias colunas, guias
  compartilhadas por todas as páginas e os objetos da página mestra.
- **Imagens, impressão e área de trabalho.** Recortar os pixels da imagem na
  ferramenta Recortar e [[menu.bitmaps > menu.bitmaps.crop_to_frame]] recortam
  os pixels de uma imagem; impressão em tons de cinza e em ordem inversa;
  [[menu.view > menu.view.scrollbars]] oculta as barras de rolagem.
- **TraceCut.** [[menu.tools > menu.tools.tracecut]] abre as ferramentas para
  plotters de recorte: saída para o plotter, linhas de corte, encaixe e
  medição. Veja [TraceCut](../objects/tracecut.md).
- **Teclado, celulares e canetas.** Menus e listas funcionam pelo teclado. Em
  celulares Android a interface cabe na tela e uma mão apoiada não desenha
  mais enquanto se usa uma caneta; a caneta de uma mesa digitalizadora
  funciona no Linux com Wayland.
- **Arquivos.** Arquivos .cdr abrem com suas paradas de tabulação, páginas
  mestras, degradês e distorções; imagens de PDF entram em resolução total;
  JPEG e TIFF são exportados em CMYK.
- **Miniaturas dos objetos.** O painel [[docker.objects]] mostra cada objeto
  desenhado em tamanho pequeno ao lado do nome; Ctrl+roda sobre a lista ou o
  controle deslizante na parte de baixo define o tamanho. Veja
  [Organizar objetos e camadas](../objects/objects-and-layers.md#objects-docker).
- **Tablets e Android.** A versão para navegador se instala como app num
  tablet ou computador, e um app para Android roda em tablets, celulares e
  Chromebooks. Veja [Tablets e o app para Android](../get-started/tablets-and-android.md).
- **Pressão da caneta.** A pressão de uma mesa digitalizadora define a
  largura das linhas de Mão livre e das pinceladas. Veja
  [Desenhar com uma mesa digitalizadora](../lines-shapes/pen-pressure.md).
- **Pacotes e fontes ausentes.** Um pacote reúne um desenho, as imagens e as
  fontes em uma pasta, e uma caixa de diálogo escolhe as fontes que
  substituem as que faltam. Veja [Empacotar um desenho](../file-formats/package.md)
  e [Fontes ausentes](../text/missing-fonts.md).
- **Converter cores.** Todas as cores de uma seleção ou de um desenho,
  bitmaps inclusive, podem ir para CMYK ou RGB em um passo. Veja
  [Converter cores](../color/convert-colors.md).
- **Impressão.** Livretos podem ser grampeados ou colados na lombada, e
  arquivos PDF podem levar marcas de impressão. Veja
  [Imprimir livretos](../printing/booklet.md) e
  [Marcas de impressão em arquivos PDF](../file-formats/pdf-prepress-marks.md).
- **Localizar por propriedades e encaixar objetos num caminho.** Localize
  objetos por preenchimento, contorno, largura, fonte ou tipo e substitua-os;
  coloque objetos e cópias ao longo de um caminho. Veja
  [Localizar por propriedades](../objects/find-by-properties.md) e
  [Encaixar objetos num caminho](../objects/fit-objects-to-path.md).
- **Prévia de antes e depois.** Toda caixa de diálogo que muda uma imagem é
  uma janela redimensionável, com uma prévia comum que você aproxima e move.
  Veja [A prévia de antes e depois](../bitmaps/before-after-preview.md).
- **Ajuda no seu idioma.** Todo botão **?** e a tecla <kbd>F1</kbd> abrem a
  página desta Ajuda sobre a caixa de diálogo, o painel ou a ferramenta à sua
  frente, no idioma em que o TraceDraw está configurado.
- **Propriedades do documento.** [[menu.file > menu.file.document_properties]]
  guarda o idioma, título, assunto, autor, direitos autorais, etiquetas, notas
  e classificação de um desenho, sobre um relatório completo do que o desenho
  contém. Veja [Propriedades do documento](../pages-layout/document-properties.md).
- **Gerenciamento de cores.** [[menu.tools > menu.tools.color_management]]
  define os perfis do desenho e os padrões para novos desenhos, com políticas
  para arquivos abertos ou importados. Veja
  [Gerenciamento de cores](../color/color-management.md).
- **Códigos de barras e QR codes.** Um assistente de três etapas insere
  códigos de barras, e os QR codes são objetos vivos que você edita no painel
  [[docker.properties]]. Veja [Códigos de barras e QR codes](../objects/barcodes.md).
- **Efeitos.** O menu [[menu.effects]] agrupa os painéis de efeitos; as
  misturas podem seguir um caminho com espaçamento fixo e girar as cores no
  sentido horário ou anti-horário. Veja [Misturas](../effects/blend.md).
- **Texto em um caminho** tem configurações próprias de orientação, posição,
  distância e deslocamento. Veja [Adicionar e editar texto](../text/adding-text.md).
- **Gerenciador de fontes** organiza fontes instaladas, pastas, favoritas e
  coleções. Veja [Fontes e glifos](../text/fonts.md).
