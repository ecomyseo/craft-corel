# Desenhar polígonos, estrelas, espirais e outras formas

Estas ferramentas desenham formas prontas arrastando uma caixa. Como nos
retângulos, mantenha <kbd>Ctrl</kbd> pressionado para manter a forma regular e
<kbd>Shift</kbd> para desenhar a partir do centro; veja
[Desenhar com uma caixa](rectangles-and-ellipses.md#drawing).

## Polígono {#polygon}

A ferramenta [[tool.polygon]] (<kbd>Y</kbd>) desenha um polígono regular.
Defina o número de **pontas** (3 a 500, 5 de início) na barra de propriedades
antes ou depois de desenhar. O primeiro canto fica no centro superior.

Os polígonos são simétricos: quando você move um nó com a ferramenta
[[tool.shape]], os nós correspondentes de todos os lados se movem junto, então
você transforma um pentágono em flor com um único arraste.

## Estrela {#star}

A ferramenta [[tool.star]] desenha uma estrela. A barra de propriedades começa
com dois botões que escolhem o que ela desenha:

- **Estrela**: uma estrela com o número de pontas (5 de início) e a
  **nitidez** dados (0,5 de início; 0 faz um polígono, 1 as pontas mais finas).
- **Estrela complexa**: uma estrela cujos lados se cruzam, feita ligando cada
  ponta a uma mais adiante (9 pontas e nitidez 2 de início). O meio fica vazio.
  Com alguns números de pontas, ela é feita de várias formas fechadas, como
  seis pontas dão dois triângulos.

## Espiral {#spiral}

A ferramenta [[tool.spiral]] (<kbd>A</kbd>) desenha uma espiral, uma curva
aberta. Na barra de propriedades, defina as **voltas** (1 a 100, 4 de início) e
escolha uma espiral **simétrica**, com voltas igualmente espaçadas, ou
**logarítmica**, com voltas cada vez mais afastadas. Mantenha <kbd>Ctrl</kbd>
pressionado para uma espiral redonda.

## Formas comuns {#common-shapes}

A ferramenta [[tool.common_shapes]] desenha formas prontas. Clique no botão de
forma na barra de propriedades para escolher uma de cinco coleções:

- **Formas básicas**: paralelogramo, trapézio, losango, triângulos, hexágono,
  octógono, sinal de mais, coração, raio, lua crescente, anel, cilindro, cubo,
  moldura, gota, rosto sorridente e símbolo de proibido.
- **Setas**: em todas as direções, divisa, pentágono, entalhada, listrada e
  dobrada.
- **Fluxograma**: processo, decisão, dados, documento, terminal, conector,
  armazenamento e os outros símbolos usuais.
- **Faixas**: fitas, ondas, pergaminho, explosão, estouro e flâmula.
- **Balões**: retangular, arredondado, oval, nuvem e balões de linha.

Depois, arraste uma caixa. A forma vira uma única curva com o preenchimento e o
contorno padrão. Partes que devem parecer separadas, como as faces de um cubo,
são partes separadas da curva, e furos como o meio de um anel continuam furos
seja qual for o preenchimento.

## Linhas de ação {#action-lines}

A ferramenta [[tool.action_lines]] desenha linhas de velocidade no estilo de
história em quadrinhos: um grupo de **linhas** (2 a 500, 12 de início)
**paralelas** (de início) ou **radiais**. Os comprimentos variam, mas a mesma
caixa sempre dá as mesmas linhas.

## Papel milimetrado {#graph-paper}

A ferramenta [[tool.graph_paper]] (<kbd>D</kbd>) desenha uma grade de células:
um grupo de retângulos que preenchem exatamente a caixa. Defina as **linhas** e
**colunas** (4 por 3 de início) na barra de propriedades. Mantenha
<kbd>Ctrl</kbd> pressionado para células quadradas.

Os valores padrão de pontas, voltas, linhas e colunas também estão nas páginas
de ferramentas das opções; veja [Opções das ferramentas](../customize/tool-options.md).
