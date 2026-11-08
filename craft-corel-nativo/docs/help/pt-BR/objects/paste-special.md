# Colar especial

[[menu.edit > menu.edit.paste_special]] escolhe como o conteúdo da área de
transferência entra no desenho, quando ele pode entrar de mais de uma forma:
por exemplo, texto copiado de outro programa pode virar um objeto de texto, e
um SVG copiado de uma página web pode virar formas editáveis ou uma imagem.

![A caixa de diálogo Colar especial](../../img/objects-paste-special.png)

## Para colar em outra forma {#paste-as}

1. Copie algo, no TraceDraw ou em outro programa.
2. Escolha [[menu.edit > menu.edit.paste_special]].
3. Em [[dialog.paste_as]], escolha uma das formas que a área de transferência
   oferece:
   - [[dialog.paste_objects]]: objetos copiados no TraceDraw, como eram;
   - [[dialog.paste_svg]]: texto SVG, transformado em formas editáveis;
   - [[dialog.paste_bitmap]]: uma imagem, como bitmap;
   - [[dialog.paste_text]]: texto simples, como uma linha de texto.
4. Clique em [[dialog.ok]].

Quando a área de transferência não tem nada que possa ser colado, a caixa de
diálogo diz [[dialog.clipboard_empty]].

O [[menu.edit.paste]] comum (<kbd>Ctrl+V</kbd>) cola na primeira forma
disponível, e [[menu.edit.paste_in_view]] (<kbd>Ctrl+Shift+V</kbd>) cola no
meio da janela.
