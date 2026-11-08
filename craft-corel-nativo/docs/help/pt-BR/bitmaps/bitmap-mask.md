# Mascarar cores de bitmaps

O painel [[docker.bitmap_mask]] oculta ou mantém os pixels das cores escolhidas
em um bitmap, por exemplo para deixar transparente um fundo branco. Abra-o com
[[menu.bitmaps > menu.bitmaps.bitmap_mask]].

![O painel Máscara de bitmap](../../img/bitmaps-bitmap-mask.png)

## Para mascarar cores {#mask}

1. Selecione um bitmap.
2. Coloque cores nos dez espaços de cor:
   - clique no conta-gotas e clique em cores do bitmap; cada uma preenche o
     espaço selecionado e o marca;
   - ou clique em **Selecionar cores do bitmap** para preencher os espaços com
     as dez cores mais usadas no bitmap.
3. Selecione um espaço e defina a **tolerância** (0 a 100%): quanto a cor de um
   pixel pode se afastar da cor do espaço e ainda combinar.
4. Escolha **Ocultar selecionadas** (os pixels que combinam ficam
   transparentes) ou **Mostrar selecionadas** (só os pixels que combinam
   ficam, de início).
5. Clique em [[docker.apply]].

**Salvar máscara** e **Abrir máscara** guardam os espaços em um arquivo
`.msk`; **Excluir** esvazia o espaço selecionado. Cada bitmap muda em um passo
de desfazer.
