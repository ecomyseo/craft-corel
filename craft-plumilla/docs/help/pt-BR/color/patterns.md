# Preenchimentos de padrão

Um preenchimento de padrão repete um ladrilho pelo interior de um objeto. O
ladrilho pode ser de duas cores, um conjunto de objetos vetoriais (um padrão
vetorial colorido) ou um bitmap. Os ladrilhos ficam ancorados na página, então
não deslizam quando você move o objeto.

## Criar um padrão com seus objetos {#create-pattern}

1. Desenhe os objetos que formam um ladrilho e selecione-os.
2. Escolha [[menu.object > menu.object.create > menu.object.create_pattern]].
3. Na caixa de diálogo [[dialog.create_pattern]], escolha o
   [[dialog.create_pattern_type]]: [[dialog.create_pattern_vector]] mantém os
   objetos como vetores, nítidos em qualquer tamanho;
   [[dialog.create_pattern_bitmap]] faz uma imagem deles.
4. Clique em [[dialog.ok]].

![A caixa de diálogo Criar padrão](../../img/color-patterns-create-pattern.png)

O padrão passa a ser o preenchimento dos novos objetos. Para dá-lo a objetos já
desenhados, escolha [[docker.fill_pattern]] na seção de preenchimento do painel
[[docker.properties]].

## Configurações do padrão {#settings}

Na seção de preenchimento do painel [[docker.properties]], escolha
[[docker.fill_pattern]], depois o tipo de padrão e as configurações. Para um
padrão vetorial, o painel mostra quantos objetos o ladrilho contém e a largura e
a altura dele: mudá-las escalona o conteúdo do ladrilho junto.

## Padrões em arquivos exportados {#export}

Arquivos SVG mantêm os padrões vetoriais como vetores. Arquivos PDF e EPS os
recebem como imagens na resolução de exportação, recortadas no objeto.
