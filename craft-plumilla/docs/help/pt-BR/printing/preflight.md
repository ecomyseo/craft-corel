# Verificação prévia

A verificação prévia procura em um desenho o que costuma dar errado na
impressão, antes de você imprimir ou mandar para uma gráfica. Escolha
[[menu.file > menu.file.preflight]]. A caixa de diálogo Imprimir também faz as
verificações e mostra quantos problemas encontrou.

![A caixa de diálogo Verificação prévia](../../img/printing-preflight.png)

## As verificações {#checks}

| Verificação | Encontra |
|---|---|
| Objeto de texto vazio | texto sem nada dentro |
| O texto não cabe no quadro | texto de parágrafo com linhas ocultas |
| Fonte não instalada | texto em uma fonte que não está instalada |
| Texto muito pequeno | texto menor que 5 pontos impresso |
| Texto pequeno em preto composto | texto preto pequeno feito de várias tintas |
| Cor RGB em um desenho CMYK | cores de tela em um desenho feito para impressão |
| Objeto fora da página | objetos fora da página e da sangria |
| Bitmap de baixa resolução | bitmaps abaixo de 200 dpi como colocados |
| Bitmap de resolução muito alta | bitmaps acima de 1200 dpi |
| Contorno muito fino | contornos mais finos que 0,1 mm |
| Branco definido para sobreimprimir | branco que sumiria na impressão |
| Página vazia | páginas sem objetos |
| Objetos que não serão impressos | objetos em camadas ocultas ou que não imprimem |

Avisos são problemas prováveis; notas merecem uma olhada.

## Usar a lista {#list}

O topo da caixa mostra o resultado. Cada tipo de problema tem um título com a
contagem; clique nele para abrir ou fechar. Clique em um problema para ir à
página dele e selecionar o objeto. Abaixo da lista, a caixa diz o que fazer com
o tipo escolhido.

- **Verificar de novo** refaz as verificações depois que você corrige as
  coisas.
- **Escolher verificações** liga e desliga verificações e define os limites:
  as resoluções de bitmap, os menores tamanhos de texto e o contorno mais
  fino. **Padrões** os devolve.
