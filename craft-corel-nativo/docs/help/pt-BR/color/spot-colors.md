# Cores especiais

Uma **cor especial** é uma tinta com nome impressa em uma chapa própria, como
um metálico ou uma cor de marca, em vez de ser misturada com ciano, magenta,
amarelo e preto. Cada desenho guarda a lista das suas tintas especiais. Cada
tinta tem um substituto feito de cores de processo, usado para mostrá-la na
tela e onde a tinta não pode ser usada.

## Acrescentar uma tinta especial {#add}

1. Abra o painel [[docker.color]] e misture a cor com a aparência da tinta.
2. Em [[docker.spot_inks]], digite um [[docker.ink_name]].
3. Clique em [[docker.new_ink]].

Os nomes são únicos dentro de um desenho.

## Usar tintas especiais {#use}

A lista [[docker.spot_inks]] mostra cada tinta com uma amostra, o nome e os
botões [[docker.fill]] e [[docker.outline]], que a aplicam à seleção na
[[docker.tint]] definida (0 a 100%, 100% de início). O X no fim de uma linha
remove a tinta da lista; os objetos que a usam mantêm a cor substituta.

## Catálogos de cores {#color-books}

Um arquivo de paleta pode ser um catálogo de tintas especiais (por exemplo, uma
paleta `.ase` com entradas especiais). As entradas especiais abrem como cores
especiais, com os nomes do catálogo; aplicar uma acrescenta a tinta ao desenho.
O TraceDraw não traz catálogos licenciados, mas um que você tenha abre como
qualquer paleta.

## Cores especiais na impressão e nas exportações {#output}

- Em arquivos PDF, cada tinta vira uma cor de separação com o nome dela, então a
  gráfica recebe a tinta.
- Ao imprimir separações, cada tinta ganha uma chapa própria depois das chapas
  de ciano, magenta, amarelo e preto. Veja [Impressão](../printing/print.md).
- Na tela e em arquivos de bitmap, SVG e `.cdr`, uma cor especial aparece como o
  substituto na sua tinta.
