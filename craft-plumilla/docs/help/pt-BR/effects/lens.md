# Lentes

Uma lente muda como aparecem os objetos embaixo dela, sem mudá-los: pode
clarear, tingir, ampliar, inverter ou aplicar efeitos de bitmap a tudo o que
cobre. Mova a lente e a vista através dela muda. As lentes são definidas no
painel [[docker.lens]] ([[menu.effects > menu.effects.lens]], <kbd>Alt+F3</kbd>).

![O painel Lente](../../img/effects-lens.png)

## Para fazer uma lente {#create}

1. Desenhe um objeto fechado sobre os objetos a mudar e selecione-o.
2. No painel [[docker.lens]], escolha o tipo de lente e as configurações.
3. Clique em [[docker.apply]].

## Tipos de lente {#kinds}

| Lente | O que faz | Configuração |
|---|---|---|
| Clarear | clareia (ou escurece) o que está embaixo | taxa, -100 a 100 (50) |
| Adicionar cor | acrescenta uma cor, como luz | cor e taxa |
| Limitar cor | deixa passar só uma cor | cor e taxa |
| Mapa de cores personalizado | leva do escuro ao claro em uma faixa de cores | cores inicial e final, direção |
| Olho de peixe | infla o que está embaixo | taxa, -1000 a 1000 (100) |
| Mapa de calor | mostra o brilho como cores quentes e frias | rotação da paleta |
| Inverter | inverte as cores | |
| Ampliar | aumenta o que está embaixo | quantidade, 1 a 100 (2) |
| Tons de cinza tingidos | mostra em tons de uma cor | cor |
| Transparência | deixa a lente translúcida, tingida | taxa e cor |
| Aramado | mostra os objetos como contornos | cores de contorno e preenchimento |
| Efeitos de bitmap | aplica efeitos de bitmap ao que está embaixo | **Adicionar efeito**, em ordem |

## Opções {#options}

- **Congelada**: mantém o que a lente mostra agora, mesmo quando os objetos
  embaixo mudam.
- **Ponto de vista**: mostra outro lugar através da lente.
- **Remover face**: mostra a lente só sobre outros objetos, não sobre a página
  vazia.

Para acrescentar um efeito de bitmap como lente pelo seletor de efeitos,
escolha [[menu.effects > menu.effects.picker]] e aplique-o como lente.
