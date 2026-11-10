# Tabulações, colunas, marcadores e capitulares

O menu [[menu.text]] tem quatro caixas de diálogo para compor texto de
parágrafo. Cada uma age sobre o texto de parágrafo selecionado. As
configurações também estão na seção **Moldura** do painel [[docker.text]].

## Tabulações {#tabs}

[[menu.text > menu.text.tabs]] define para onde salta o caractere
<kbd>Tab</kbd>. De início, há uma parada de tabulação a cada 12,7 mm (meia
polegada); [[options.default_tab_spacing]] nas Opções muda isso.

1. Selecione o texto de parágrafo.
2. Escolha [[menu.text > menu.text.tabs]].
3. Em [[dialog.tab_stops]], mude as posições, ou clique em [[dialog.add_tab]]
   para acrescentar uma.
4. Clique em [[dialog.close]].

![A caixa de diálogo Tabulações](../../img/text-paragraphs-tabs.png)

## Colunas {#columns}

[[menu.text > menu.text.columns]] divide uma moldura de texto em colunas; o
texto passa de uma coluna para a seguinte.

1. Selecione o texto de parágrafo.
2. Escolha [[menu.text > menu.text.columns]].
3. Defina o número de [[dialog.columns]] e o [[dialog.gutter]], o espaço entre
   as colunas (5 mm de início).
4. Clique em [[dialog.close]].

![A caixa de diálogo Colunas](../../img/text-paragraphs-columns.png)

Para pular para a próxima coluna em um ponto, insira
[[menu.text.code_column_break]] em [[menu.text.insert_formatting_code]].

## Marcadores {#bullets}

[[menu.text > menu.text.bullets]] coloca um marcador antes de cada parágrafo.

1. Selecione o texto de parágrafo.
2. Escolha [[menu.text > menu.text.bullets]].
3. Marque [[dialog.use_bullets]] e defina o [[dialog.bullet_symbol]] do
   marcador e o [[dialog.bullet_indent]] (5 mm de início).
4. Clique em [[dialog.close]].

![A caixa de diálogo Marcadores](../../img/text-paragraphs-bullets.png)

## Capitulares {#drop-cap}

Uma capitular é uma primeira letra grande que ocupa várias linhas.

1. Selecione o texto de parágrafo.
2. Escolha [[menu.text > menu.text.drop_cap]].
3. Marque [[dialog.use_drop_cap]] e defina as [[dialog.lines_dropped]] (3 de
   início).
4. Clique em [[dialog.close]].

A letra é escalonada para que o topo encontre o topo da primeira linha e ela
assente na linha de base da última linha que ocupa; as linhas ao lado são
encurtadas para dar espaço.

![A caixa de diálogo Capitular](../../img/text-paragraphs-drop-cap.png)

## Hifenização {#hyphenation}

[[menu.text > menu.text.use_hyphenation]] deixa as palavras do texto
selecionado quebrarem no fim das linhas (durante a digitação, os parágrafos
no cursor; sem texto selecionado, vale para o texto novo), com pelo menos duas letras antes e três depois da quebra. Uma
vírgula ou outra pontuação colada à palavra não impede a quebra, e palavras
com apóstrofo (como o italiano *dell'anno*) também quebram. Palavras com letra
maiúscula nunca são quebradas. Para decidir você mesmo onde uma
palavra pode quebrar, veja [Hífens personalizados](writing-tools.md#custom-hyphens).
