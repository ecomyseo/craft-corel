# Perspectiva

A perspectiva faz um objeto parecer deitado em um plano que se afasta ao longe.
Você pode acrescentar perspectiva a um objeto já desenhado, ou desenhar direto
em uma grade de perspectiva.

## Acrescentar perspectiva a um objeto {#add}

1. Selecione um objeto.
2. Escolha [[menu.object > menu.object.add_perspective]].
3. Arraste os quatro nós dos cantos com a ferramenta [[tool.shape]] para
   incliná-lo.

Para copiar a perspectiva de outro objeto, use
[[menu.object > menu.object.copy_effect > menu.object.copy_perspective_from]].

## Desenhar em perspectiva {#draw}

[[menu.view > menu.view.draw_in_perspective]] mostra uma grade de perspectiva
sobre a página. Enquanto está ligada, todo objeto feito por uma ferramenta de
desenho (retângulos, elipses, polígonos, curvas, pinceladas, quadros de texto)
é deitado no plano ativo da grade. Objetos colados, duplicados ou movidos com a
ferramenta [[tool.pick]] ficam como estão.

A barra no canto superior esquerdo da janela de desenho tem:

- **Concluir**: sai do modo; os objetos mantêm a perspectiva;
- **1 ponto**, **2 pontos** e **3 pontos**: o tipo de grade;
- os planos: **Chão**, **Esquerdo**, **Direito** (e **Topo** em uma grade de 1
  ponto). O plano ativo é desenhado mais forte;
- **Grade**: mostra as linhas da grade;
- **Profundidade** (5 a 98%): até onde a grade vai em direção aos pontos de
  fuga;
- **Colocar no plano**: deita os objetos selecionados no plano ativo.

Arraste os pontos de fuga (círculos) e os cantos próximos (quadrados) para
mudar a grade. Um objeto deitado em um plano recebe um efeito de perspectiva
comum, cujos quatro cantos você ainda edita com a ferramenta [[tool.shape]].
