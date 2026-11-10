# Links e marcadores

O painel [[docker.links]] dá a objetos um hiperlink, para que um clique neles
em um PDF, SVG ou página web exportados abra um endereço, escreva um e-mail ou
vá para outra página. Ele também lista como marcadores os objetos com nome do
desenho. Abra-o em [[menu.window > menu.window.dockers]].

![O painel Links e marcadores](../../img/objects-links.png)

## Para acrescentar um link {#add}

1. Selecione um ou mais objetos.
2. Digite o endereço no campo [[docker.hyperlink]] e saia do campo. Os botões
   [[docker.link_web]], [[docker.link_mail]] e [[docker.link_page]] colocam
   `https://`, `mailto:` ou `#page=` antes do texto, então para um link de
   página você digita só o número dela.
3. Clique em [[docker.remove]] para tirar um link, ou esvazie o campo.

## Marcadores {#bookmarks}

Em [[docker.bookmarks]], o painel lista um botão por objeto que tem nome.
Clique em um para selecionar o objeto e dar zoom nele. Para dar nome a objetos,
veja [Renomear objetos e camadas](objects-and-layers.md#rename).

## Links em arquivos exportados {#exports}

- Em arquivos PDF, cada objeto com link vira uma área clicável sobre a caixa
  dele. Um link de página vai para o alto dessa página do arquivo. Arquivos
  PDF/X não recebem links, porque esses padrões não os permitem.
- Em SVG e páginas web, o próprio objeto é o link.
- Um link em um grupo cobre todos os objetos dele; os links dos objetos de
  dentro são deixados de fora.
