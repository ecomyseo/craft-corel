# Esboço e reconhecimento de formas

Duas ferramentas transformam traços rápidos à mão em objetos limpos: a
ferramenta [[tool.shape_recognition]] reconhece a forma que você quis desenhar,
e a ferramenta [[tool.sketch]] monta uma linha suave a partir de vários traços.

## Reconhecimento de formas {#shape-recognition}

A ferramenta [[tool.shape_recognition]] (<kbd>Shift+S</kbd>) transforma um
traço à mão livre na forma com que ele se parece, do jeito que você desenhou:
um retângulo inclinado continua inclinado.

1. Escolha a ferramenta [[tool.shape_recognition]].
2. Desenhe a forma em um único traço. Um traço cujas pontas ficam próximas é
   considerado fechado.
3. Solte o botão.

O que você obtém:

- Um traço aberto quase reto vira uma linha reta; senão, uma curva suave.
- Um traço fechado com quatro cantos perto do ângulo reto vira um retângulo.
  Quando está quase nivelado, fica nivelado.
- Um traço fechado e arredondado vira uma elipse, nivelada quando quase
  nivelada e nunca inclinada quando quase redonda.
- Um traço fechado com três cantos vira um triângulo, e um com quatro outros
  cantos, um quadrilátero.
- Qualquer outra coisa vira uma curva fechada suave.

O novo objeto recebe o preenchimento e o contorno padrão e fica selecionado.

Na barra de propriedades, **Reconhecimento de formas** define com que
facilidade os traços viram formas, e **Suavização** quanto as curvas que sobram
são suavizadas. Cada um vai de Nenhum, Mínimo, Baixo, Médio (de início), Alto
até Máximo. Com o reconhecimento em Nenhum, todo traço continua uma curva.

## Esboço {#sketch}

A ferramenta [[tool.sketch]] (<kbd>S</kbd>) permite montar uma linha a partir de
vários traços curtos, como faria com um lápis.

1. Escolha a ferramenta [[tool.sketch]].
2. Desenhe um traço. Ele é suavizado e selecionado.
3. Desenhe o próximo traço começando perto do fim do primeiro. Com **Unir**
   ativado, ele é acrescentado à mesma curva.

Na barra de propriedades, **Unir** acrescenta um traço à curva aberta
selecionada quando ele começa perto o bastante do fim dela; a distância ao lado
define quão perto (5 mm de início). **Suavização** define quanto cada traço é
suavizado, de Nenhum a Máximo.
