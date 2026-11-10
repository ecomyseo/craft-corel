# Paletas de cores

> *Tradução automática: esta página pode estar atrasada em relação à versão em inglês.*

A paleta de cores embaixo da janela é o jeito mais rápido de colorir objetos.
Abaixo dela, a **paleta do documento** reúne as cores que o desenho usa. Você
pode abrir mais paletas, criar as suas e editá-las.

## Usar a paleta {#using}

| Ação | Resultado |
|---|---|
| Clicar em uma cor | preenche a seleção (sem nada selecionado, define o preenchimento dos novos objetos) |
| Clicar com o botão direito em uma cor | colore o contorno da seleção |
| <kbd>Ctrl</kbd>+clicar em uma cor | mistura um décimo dela no preenchimento da seleção |
| Clicar e segurar em uma cor | mostra tons mais claros, mais escuros, mais apagados e mais vivos para escolher |
| Arrastar uma cor até um objeto | preenche o objeto, ou colore o contorno quando solta na borda |
| Clicar na amostra riscada | remove o preenchimento (botão direito: o contorno) |

As setas nas pontas rolam a paleta, e o botão na ponta direita mostra todas as
cores de uma vez. Passe o ponteiro sobre uma cor para ver o nome e os valores. O
botão de menu da paleta, na ponta esquerda, abre o menu da paleta. A roda do
mouse sobre a paleta a rola; com <kbd>Ctrl</kbd> pressionado, a roda aumenta
(roda para cima) ou diminui (roda para baixo) as amostras, de 8 a 48 px, e o
tamanho fica guardado. O menu da paleta tem **Tamanho das amostras**, com cinco
tamanhos, de **Extrapequeno** a **Extragrande**, e **Linhas**, de 1 a 8 linhas
de amostras, ou **Ajustar** para mostrar todas as cores.

A paleta pode ir para outra borda da janela, ou flutuar: arraste-a pela alça
pontilhada. Com duas ou três linhas, a janela que mostra todas as cores abre
sem cobrir nenhuma delas.

### Várias paletas

Você pode ter várias paletas abertas ao mesmo tempo, cada uma numa barra
própria: marque-a em [[menu.window > menu.window.color_palettes]], clique no
olho dela no Gerenciador de paletas, ou escolha-a em **Abrir como barra de
paleta** no menu de uma paleta. A paleta do documento também pode ter uma
barra própria.

- Cada barra pode ir para qualquer borda da janela (nas bordas esquerda e
  direita as amostras ficam em colunas) ou flutuar numa janela pequena:
  arraste-a pela alça pontilhada ou pelo título, ou escolha um lugar em
  **Posição** no menu dela.
- Arraste a borda direita ou de baixo de uma paleta flutuante para mudar o
  tamanho: as amostras se arrumam em linhas numa janela larga e em colunas
  numa janela alta.
- Cada barra tem o próprio **Tamanho das amostras** e as próprias **Linhas**.
- **Fechar paleta** no menu, ou o botão de fechar da janela, a fecha.
- As paletas abertas, onde estão, seus tamanhos e linhas ficam guardados, e
  vão junto nos arquivos de espaço de trabalho.

### Dar nome às paletas

Qualquer paleta, integrada ou a do documento, pode ter um nome seu: escolha
**Renomear...** no menu dela ou no Gerenciador de paletas, ou clique duas vezes
no nome dela ali. O nome aparece em todo lugar onde a paleta é listada;
**Restaurar nome** devolve o nome original. Os arquivos das paletas integradas
nunca mudam; uma paleta sua salva como `.gpl` ou `.tdpal` guarda o nome no
arquivo.

Em cada lado, ela pode ficar na borda da janela ou ao lado do desenho, entre o desenho e os painéis ou a caixa de ferramentas daquele lado. Enquanto você a arrasta, uma faixa azul mostra onde ela vai ficar; clique com o botão direito na alça para escolher um lugar em uma lista.

## A paleta do documento {#document-palette}

Toda cor aplicada a um objeto entra na paleta do documento, que é salva com o
desenho. O conta-gotas dela acrescenta a cor sob o ponteiro, e o menu tem
**Atualizar automaticamente** (para parar de acrescentar cores sozinha),
**Acrescentar da seleção**, **Acrescentar do documento**, **Excluir cor** e
**Redefinir paleta** (mantém só as cores que o desenho ainda usa). Você também
pode arrastar cores ou objetos até ela. A cor usada por último tem uma
moldura branca: uma cor pega com o conta-gotas que a paleta já tem recebe a
moldura, para você ver onde está, e aplicar uma cor da paleta move a moldura
para ela. Para mostrá-la ou ocultá-la, veja
[Opções de personalização](../customize/customization.md#color-palette).

## O painel Paletas {#palettes-docker}

O painel [[docker.palettes]] lista as paletas que você tem em três seções,
**Integradas**, **Minhas paletas** e **Paleta do documento**. Abra-o com
[[menu.window > menu.window.color_palettes > menu.window.palette_manager]].

![O painel Paletas](../../img/color-palettes.png)

- Cada paleta mostra as primeiras cores, quantas cores tem, o modelo de cor e
  onde está sendo exibida. Clique no olho para abri-la numa barra própria, ou
  para fechar a barra dela.
- Clique numa paleta para selecioná-la: o controle **Tamanho das amostras**
  passa a ajustar as amostras da barra dela enquanto você arrasta (as marcas
  são os cinco tamanhos nomeados), e **Linhas** define as linhas dela.
- O botão de três pontos, ou o botão direito, abre o menu da paleta: abrir ou
  fechar a barra, [[docker.edit]], **Renomear...**, **Restaurar nome**,
  **Duplicar**, [[docker.palette_lock]] (paletas que você abriu ou criou,
  para que o editor de paletas não as altere; [[docker.palette_unlock]]
  permite editá-las de novo), **Excluir paleta** (paletas que
  você abriu ou criou; o arquivo fica), **Mover para cima** e **Mover para
  baixo**. Você também pode arrastar uma paleta para outro lugar da seção.
- Digite na caixa de filtro para listar só as paletas cujo nome contém o
  texto.
- Os botões ao lado da caixa de filtro: [[menu.window.palette_open]] abre um
  arquivo de paleta (`.tdpal`, `.gpl`, `.ase` ou `.aco`; abrir o mesmo arquivo
  de novo não acrescenta uma segunda cópia); [[menu.window.palette_from_document]]
  e [[menu.window.palette_from_selection]] preenchem a paleta do documento com
  as cores do desenho ou da seleção.
- As cores da paleta do documento aparecem na seção dela; clique em uma para
  preencher a seleção, com o botão direito para colorir o contorno.
  **Adicionar cores da seleção**, **Adicionar do documento** e **Limpar** a
  alteram.

## Editar uma paleta {#palette-editor}

[[menu.window > menu.window.color_palettes > menu.window.palette_editor]] abre o
editor de paletas.

![O Editor de paletas](../../img/color-palettes-palette-editor.png)

1. Escolha a [[dialog.palette]] a editar, ou clique em [[dialog.new_palette]]
   para começar uma.
2. Digite o [[dialog.name]].
3. Clique em uma cor para selecioná-la; [[dialog.add_color]] acrescenta a cor
   atual e [[dialog.remove_color]] remove a selecionada.
4. Clique em [[dialog.save_palette]] para salvá-la como arquivo.
5. Clique em [[dialog.close]].

As paletas de fábrica não podem ser editadas; crie uma paleta nova ou edite uma
cópia.
