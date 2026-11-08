# O painel Propriedades

O painel [[docker.properties]] (<kbd>Alt+Enter</kbd>) mostra e altera as
propriedades dos objetos selecionados: contorno, preenchimento, transparência e
mais, conforme o que estiver selecionado. Ele vem aberto, ao lado de
[[docker.hints]] e [[docker.objects]]. Abra-o de novo com
[[menu.object > menu.object.properties]].

![O painel Propriedades](../../img/objects-properties-docker.png)

## Com objetos selecionados {#selection}

O alto do painel dá o nome do objeto selecionado (e quantos mais estão
selecionados) e a posição e o tamanho. Abaixo, uma fileira de botões abre e
fecha cada seção que o objeto tem:

- **QR code**, para um QR code: o conteúdo e a aparência; veja
  [Códigos de barras e QR codes](barcodes.md#qr-codes).
- [[docker.outline]]: cor, largura, pintura, pontas, cantos, tracejados, setas
  e mais; veja [Definir contornos](../lines-shapes/outlines.md).
- [[docker.fill]]: o tipo de preenchimento e as configurações; veja
  [Preenchimentos](../color/fills.md). <kbd>F11</kbd> abre esta seção.
- [[docker.transparency]]: quanto do que está embaixo aparece através; veja
  [Transparência](../color/transparency.md).
- [[kind.rectangle]], para retângulos: os cantos; veja
  [Cantos](../lines-shapes/rectangles-and-ellipses.md#rectangle-corners).
- [[docker.fx]]: os efeitos de bitmap aplicados ao objeto; veja
  [Efeitos de bitmap](../effects/bitmap-effects.md).
- [[docker.character]], para texto: a fonte, o tamanho, [[docker.bold]] e
  [[docker.italic]], e um botão para [[docker.edit_with_text_tool]].
- [[docker.summary]]: um resumo do objeto.

As seções que você fechou continuam fechadas na próxima vez. Uma alteração vale
para todos os objetos selecionados e é um passo de desfazer.

## Sem nada selecionado {#defaults}

Sem nada selecionado, o painel diz [[docker.no_objects_selected]] e mostra as
[[docker.default_properties]]: o preenchimento e o contorno que novas formas e
linhas recebem. Mude-os aqui para desenhar com outras cores desde o início.
Veja [Estilos de objeto e padrões](../templates-styles/object-styles.md#document-defaults).
