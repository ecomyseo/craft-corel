# Desenhar com simetria

A simetria desenha cópias espelhadas vivas de um objeto em relação a uma ou
mais linhas de espelho. As cópias acompanham toda alteração feita no original,
então você desenha uma metade (ou uma fatia) e vê o todo. Os comandos ficam em
[[menu.object > menu.object.symmetry]].

## Para criar uma simetria {#create}

1. Selecione um objeto.
2. Escolha [[menu.object > menu.object.symmetry > menu.object.symmetry_create]]
   (<kbd>Alt+S</kbd>).

Uma linha de espelho vertical é colocada na borda direita do objeto, para que a
cópia apareça ao lado dele, e o TraceDraw entra direto na edição da simetria.

## Editar uma simetria {#edit}

[[menu.object.symmetry_edit]] (<kbd>Alt+E</kbd>) edita a simetria do objeto
selecionado. Uma barra no canto superior esquerdo da janela de desenho define:

- o número de linhas de espelho, de 1 a 12, distribuídas por igual em volta do
  centro. Com mais linhas você obtém um caleidoscópio: com *n* linhas há
  2*n* - 1 cópias;
- o centro (X e Y);
- o ângulo da primeira linha (0 a 360 graus);
- Mostrar visualização completa (cópias desenhadas por inteiro, ou como
  contornos vermelhos quando desligado) e Mostrar linhas de simetria.

Edite os originais normalmente, com qualquer ferramenta; as cópias acompanham.
Um clique ou um marquee seleciona um dos originais dentro da simetria, e os
objetos que você desenha, cola ou duplica entram nela. Arraste uma linha de
espelho com a ferramenta Seleção para mover as linhas; clique em uma linha para
mostrar alças redondas nas pontas e arraste uma para girar as linhas. Enquanto
um objeto com simetria está selecionado ou em edição, as linhas de espelho
aparecem tracejadas.

Clique em Concluir na barra, ou escolha [[menu.object.symmetry_finish]]
(<kbd>Alt+Shift+E</kbd>), para parar de editar.

## Remover ou manter as cópias {#remove}

- [[menu.object.symmetry_remove]] (<kbd>Alt+Shift+S</kbd>) remove a simetria;
  as cópias somem.
- [[menu.object.symmetry_break]] (<kbd>Alt+X</kbd>) transforma as cópias em
  objetos próprios, que não acompanham mais o original.
