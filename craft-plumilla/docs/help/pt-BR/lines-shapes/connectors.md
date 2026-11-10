# Linhas de conexão e âncoras

As linhas de conexão unem dois objetos e continuam presas a eles: quando você
move ou redimensiona os objetos, o conector acompanha. São úteis em diagramas,
fluxogramas e organogramas.

## Âncoras {#anchor-points}

Um conector se prende a uma **âncora** de cada objeto. Todo objeto tem quatro
âncoras, no meio de cada lado da caixa delimitadora. Você pode acrescentar suas
próprias âncoras com a ferramenta [[tool.anchor_editing]]; elas se movem e
escalonam com o objeto.

## Desenhar um conector {#connector-tools}

Três ferramentas do submenu Conector desenham conectores:

- [[tool.connector]]: uma linha reta;
- [[tool.right_angle_connector]]: segmentos horizontais e verticais com cantos
  retos;
- [[tool.rounded_connector]]: o mesmo com cantos arredondados.

Para unir dois objetos:

1. Escolha uma ferramenta de conector.
2. Arraste de um objeto ao outro. Enquanto você arrasta, as âncoras dos dois
   objetos aparecem, e a linha começa na âncora que será usada.
3. Solte sobre o outro objeto.

Quando você começa ou termina o arraste em uma âncora, essa âncora é usada;
senão, é usado o par de âncoras mais próximo dos dois objetos. O conector é uma
linha aberta sem preenchimento e com o contorno padrão.

### Configurações do conector {#connector-settings}

A barra de propriedades, mostrada com as ferramentas de conector e quando a
seleção é toda de conectores, tem os três tipos de conector (com conectores
selecionados, eles mudam como esses correm), o **raio do canto** dos conectores
arredondados (3 mm de início) e a largura, o estilo e as setas do contorno dos
conectores selecionados.

### Continuar preso {#attached}

- Quando você move, escalona ou remodela os objetos unidos, o conector corre de
  novo entre as âncoras deles.
- Quando você move o conector junto com os objetos, ele continua preso.
- Quando você move só o conector, ele se solta e vira uma linha simples.
  Desfazer o prende de novo.
- Quando um dos objetos é excluído, o conector fica onde está.

## Editar âncoras {#anchors}

A ferramenta [[tool.anchor_editing]] acrescenta, move e exclui âncoras:

1. Escolha a ferramenta [[tool.anchor_editing]] e clique em um objeto. As
   âncoras aparecem como losangos.
2. Clique dentro do objeto para acrescentar uma âncora ali.
3. Arraste uma âncora que você acrescentou para movê-la.
4. Selecione uma âncora e pressione <kbd>Delete</kbd>, clique duas vezes nela
   ou clique em **Excluir âncora** na barra de propriedades para removê-la.
   **Remover todas as âncoras personalizadas** remove todas as que você
   acrescentou.

As quatro âncoras no meio dos lados não podem ser removidas.
