# Dados do objeto

O painel [[docker.object_data]] associa suas próprias informações a um objeto,
em campos com nome: um código de peça, um preço, uma observação. Os dados são
salvos com o desenho e acompanham o objeto quando você o copia.

![O painel Dados do objeto](../../img/objects-object-data.png)

## Acrescentar e editar dados {#fields}

1. Selecione um objeto. Sem nada selecionado, o painel diz
   [[docker.select_one_object]].
2. O painel lista os campos do objeto em duas colunas, [[docker.field]] e
   [[docker.value]]. Clique em um nome ou valor para editá-lo.
3. Clique em [[docker.add_field]] para acrescentar um campo, ou em um dos
   botões de campos comuns: Name, Cost, Comments e CDRStaticID.
4. Clique no **x** no fim de uma linha para remover o campo.

Quando os objetos selecionados têm um campo Cost com um número, o painel mostra
o custo total deles.

Alguns recursos guardam dados próprios nos objetos, como linhas de cota,
balões, conectores e âncoras; raramente é preciso mudar esses campos à mão.
