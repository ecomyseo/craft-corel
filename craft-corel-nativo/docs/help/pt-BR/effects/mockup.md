# Fazer um mockup de camiseta

Um mockup mostra um desenho sobre a foto do objeto real, aqui uma camiseta,
para você avaliar antes de imprimir. Quatro recursos desta Ajuda fazem o
trabalho: um envelope curva o desenho sobre o peito, um modo de mesclagem da
transparência deixa as dobras e sombras do tecido aparecerem através da
estampa, e uma moldura de recorte mantém a estampa na camiseta.

![O desenho posto sobre a foto](../../img/effects-mockup-1-design.png)

## Colocar a foto e o desenho {#place}

1. Escolha [[menu.file > menu.file.import]] e importe a foto de uma camiseta
   lisa, de frente, com boa luz.
2. Desenhe ou importe o seu desenho, ponha sobre o peito e ajuste o tamanho.
3. Se o desenho tem vários objetos, selecione todos e escolha
   [[menu.object > menu.object.group]], para os próximos passos o tratarem
   como um só.

## Seguir o peito com um envelope {#envelope}

Tecido não é plano. Uma curva leve faz a estampa parecer deitada na camiseta.

1. Selecione o desenho e escolha a ferramenta [[tool.envelope]] (ou abra
   [[menu.effects > menu.effects.envelope]]).
2. Use **[[envelope.single_arc]]** e arraste um pouco os nós do meio: o de baixo para
   baixo, o de cima para cima e os dos lados para fora. Com moderação.

Cada objeto de um grupo é curvado a partir da caixa do grupo inteiro e mantém
as próprias cores. Veja [Envelopes](envelope.md).

![O desenho curvado com um envelope](../../img/effects-mockup-2-envelope.png)

## Deixar o tecido aparecer: Multiply {#multiply}

Uma estampa pega o sombreado do tecido por baixo dela. O modo de mesclagem
**Multiply** (multiplicar) faz o mesmo: cada cor é multiplicada pelo que está embaixo,
então as dobras e sombras da foto escurecem o desenho onde escurecem a
camiseta.

1. Selecione o desenho e escolha a ferramenta [[tool.transparency]], ou abra
   as configurações de transparência no painel [[docker.properties]].
2. Escolha [[transparency.uniform]] e deixe a quantidade baixa (0 a 10 %).
3. Ponha [[transparency.merge_mode]] em **Multiply**.

Veja [Transparência](../color/transparency.md).

![Transparência uniforme com o modo Multiply](../../img/effects-mockup-3-multiply.png)

## Manter a estampa na camiseta: uma moldura de recorte {#clip}

Uma estampa grande, ou perto de uma manga, pode passar das bordas da camiseta.

![Uma estampa maior que a camiseta](../../img/effects-mockup-4-large.png)

1. Desenhe o contorno da camiseta sobre a foto com a ferramenta
   [[tool.bezier]] ou [[tool.pen]], seguindo as bordas (não precisa ser exato
   onde a estampa não chega). Deixe sem preenchimento e sem contorno.
2. Selecione o desenho, escolha
   [[menu.object > menu.object.clip_frame > menu.object.clip_frame_place_inside]]
   e clique no contorno.

A estampa é cortada na forma da camiseta e mantém o modo Multiply: ela
continua se misturando com a foto embaixo da moldura. Veja
[Colocar objetos dentro de uma moldura](../objects/clip-frames.md).

![O mockup pronto](../../img/effects-mockup-5-final.png)

## Dicas {#tips}

- Uma camiseta branca ou clara mostra melhor o Multiply. Numa camiseta
  escura, experimente **If Lighter** para uma estampa clara, ou mantenha
  **Normal** com um pouco de transparência.
- Para testar outro desenho, use [[menu.object.clip_frame_edit]], ou
  [[menu.object.clip_frame_extract]], troque o desenho e ponha o novo dentro
  de novo.
- Agrupe a foto, a moldura e o desenho para mover o mockup inteiro.
