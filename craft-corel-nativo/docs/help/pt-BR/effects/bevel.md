# Chanfros

Um chanfro faz as bordas de um objeto parecerem em relevo ou entalhadas,
iluminando um lado da borda e sombreando o outro. É um efeito vivo, definido no
painel [[docker.bevel]]; abra-o com [[menu.effects > menu.effects.bevel]].

![O painel Chanfro](../../img/effects-bevel.png)

## Para chanfrar um objeto {#create}

1. Selecione um objeto preenchido.
2. No painel [[docker.bevel]], escolha o estilo:
   - [[docker.soft_edge]]: a borda desce para dentro do objeto, iluminada de um
     lado e sombreada do outro;
   - [[docker.emboss]]: o objeto parece saltar da página.
3. Defina o visual do chanfro:
   - [[docker.depth]]: até onde o chanfro chega a partir da borda, para
     dentro do objeto na borda suave e para fora no relevo;
   - [[docker.level]]: em quantas etapas a luz e a sombra são divididas ao
     longo do chanfro. 1 dá dois tons chapados; mais etapas os degradam
     suavemente da borda para dentro;
   - [[docker.light_angle]]: a direção de onde vem a luz;
   - [[docker.intensity]]: a força da luz e da sombra;
   - [[docker.light_color]] e [[docker.shadow_color]]: as cores do lado
     iluminado e do lado sombreado;
   - [[docker.emboss_color]]: a cor a partir da qual o chanfro é
     tonalizado. Marque a caixa e escolha uma cor própria; desligada, usa o
     preenchimento do objeto.
4. Clique em [[docker.apply]].

Para copiar o chanfro de outro objeto, use
[[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]].
[[menu.object > menu.object.clear_effect]] o remove. O painel de extrusão
também tem um chanfro para a face da frente; veja
[Extrusões](extrude.md#settings).
