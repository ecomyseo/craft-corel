# Reamostrar, endireitar e expandir bitmaps

## Reamostrar {#resample}

[[menu.bitmaps > menu.bitmaps.resample]] dá aos bitmaps selecionados um novo
número de pixels; o tamanho deles na página continua o mesmo.

![A caixa de diálogo Reamostrar](../../img/bitmaps-resample-and-straighten-resample.png)

1. Digite o novo tamanho em **pixels**, em **porcentagem** ou como a
   **resolução** que ele dá no tamanho da página. Com **Manter proporção**
   ligado, a altura acompanha a largura.
2. Escolha o método:
   - **Suave** (fotos), de início;
   - **Nítido**, um pouco mais definido;
   - **Pixels duros** (pixel art): cada pixel continua um quadrado de uma cor;
   - **Ampliar com detalhes**: suave, depois com nitidez quando a imagem
     cresce.
3. Clique em [[dialog.ok]].

A caixa de diálogo mostra os tamanhos original e novo e a memória que a nova
imagem ocupa. Com vários bitmaps selecionados, cada um é escalado pelo mesmo
fator.

## Endireitar imagem {#straighten}

[[menu.bitmaps > menu.bitmaps.straighten_image]] corrige fotos inclinadas,
vistas de lado ou entortadas pela lente.

![A caixa de diálogo Endireitar imagem](../../img/bitmaps-resample-and-straighten-straighten.png)

1. Use a **Grade** (tamanho e cor ajustáveis) para alinhar a imagem.
2. Ajuste os controles: **Corrigir distorção da lente** (-100 a 100),
   **Girar imagem** (-15 a 15 graus), **Perspectiva vertical** e
   **Perspectiva horizontal** (-100 a 100). Os botões de girar viram a imagem
   um quarto de volta antes.
3. Com **Recortar imagem** ligado, o resultado é recortado no maior retângulo
   com as proporções originais; **Recortar e reamostrar para o tamanho
   original** o devolve ao número original de pixels. Sem recorte, os cantos
   ficam transparentes.
4. Clique em [[dialog.ok]]. **Redefinir** devolve todas as configurações.

Amplie e desloque a prévia com as ferramentas acima dela. O resultado
substitui os pixels, então os efeitos do bitmap se tornam permanentes.

## Expandir bitmaps {#inflate}

Alguns efeitos, como desfoques, passam das bordas de um bitmap.

- [[menu.bitmaps > menu.bitmaps.inflate > menu.bitmaps.inflate_auto]] (ligado
  de início) aumenta os bitmaps no quanto os efeitos precisam, com uma margem
  transparente.
- [[menu.bitmaps.inflate_manual]] aumenta à mão os bitmaps selecionados: para
  a largura e a altura, digite o novo tamanho em pixels ou uma porcentagem
  (100 a 1000%), com **Manter proporção**. A imagem continua centrada.

![A caixa de diálogo Expandir bitmap manualmente](../../img/bitmaps-resample-and-straighten-inflate.png)

## Outros comandos de bitmap {#other}

- [[menu.bitmaps.crop_bitmap]] recorta um bitmap com a ferramenta
  [[tool.crop]]; arrastar um canto de um bitmap com a ferramenta
  [[tool.shape]] também o recorta.
- [[menu.bitmaps.edit_bitmap]] abre o bitmap em outro programa.
- [[menu.bitmaps.remove_background_local]] deixa transparente o fundo de uma
  foto neste computador (veja [Remover o fundo](remove-background.md));
  [[menu.bitmaps.remove_background]] faz isso com o serviço de IA.
- [[menu.bitmaps.break_link]] e [[menu.bitmaps.update_from_link]] trabalham com
  bitmaps vinculados a um arquivo.
- [[menu.bitmaps.save_bitmap_as]] salva um bitmap como arquivo PNG.
