# Unir curvas

O painel [[docker.join_curves]] fecha os espaços entre as pontas de curvas, por
exemplo linhas que deveriam se encontrar em um canto mas param antes. Abra-o com
[[menu.object > menu.object.join_curves]].

![O painel Unir curvas](../../img/lines-shapes-join-curves.png)

## Para unir curvas {#join}

1. Selecione as curvas. Partes abertas dentro de uma mesma curva também são
   unidas.
2. Escolha como as pontas são unidas:
   - **Estender**: as duas pontas continuam na sua direção até se encontrarem,
     formando um canto. Quando as direções nunca se encontram à frente das
     pontas, uma linha reta as une.
   - **Chanfrar**: uma linha reta entre as pontas.
   - **Filete**: como Estender, depois o canto é arredondado com o **raio** (2
     mm de início).
   - **Curva de Bézier**: uma curva suave que sai e chega nas direções das
     pontas.
3. Defina a **tolerância de vão** (5 mm de início). Pontas mais distantes não
   são unidas.
4. Clique em [[docker.apply]].

O par de pontas mais próximo é unido primeiro, depois o seguinte, até não
sobrar nenhum par dentro da tolerância; as duas pontas de uma mesma parte a
fecham. A curva unida fica no lugar, com o preenchimento e o contorno da curva
selecionada por último, e as outras curvas são removidas. Quando nenhuma ponta
está próxima o bastante, nada muda e a barra de status avisa.

Você também pode unir dois nós de ponta de uma curva com a ferramenta
[[tool.shape]]; veja
[Moldar curvas com a ferramenta Forma](shape-tool.md#property-bar). Desenhar a
partir da ponta de uma curva selecionada também a continua; veja
[Continuar uma curva](lines.md#continue).
