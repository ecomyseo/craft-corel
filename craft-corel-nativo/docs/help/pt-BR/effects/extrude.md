# Extrusões

Uma extrusão transforma um objeto plano em um corpo de aparência sólida, com
lados que vão do contorno dele até uma cópia, iluminados e sombreados. É um
efeito vivo: mude o objeto e o corpo acompanha.

## Para extrudar um objeto {#create}

Com a ferramenta:

1. Escolha a ferramenta [[tool.extrude]] no submenu Efeitos.
2. Arraste a partir de um objeto. Em uma extrusão paralela o corpo segue o
   arrasto; nas outras, o fim do arrasto é o ponto de fuga.
3. Arraste a alça (um quadrado com uma cruz) para mover o ponto de fuga.

Com o painel ([[menu.effects > menu.effects.extrude]], <kbd>Ctrl+F10</kbd>):
selecione um objeto, defina as opções e clique em [[docker.apply]].

![O painel Extrusão](../../img/effects-extrude.png)

Grupos e bitmaps não podem ser extrudados.

## Configurações da extrusão {#settings}

O painel tem cinco páginas, escolhidas com os botões no topo; a barra de
propriedades tem as configurações principais e um menu 3D com o resto.

- **Câmera**: o tipo de extrusão: pequena atrás (de início), grande atrás,
  pequena na frente, grande na frente, paralela atrás ou paralela na frente.
  Os tipos de trás crescem atrás do objeto, os da frente na frente dele. A
  **profundidade** (1 a 99, 20 de início), o **ponto de fuga** (X e Y,
  contados a partir da origem da página ou do centro do objeto) e se ele fica
  **preso ao objeto** (move com ele) ou à página. Nos tipos paralelos, X e Y
  são o deslocamento.
- **Rotação**: gira o corpo no espaço em torno de X, Y e Z (-360 a 360 graus);
  **Redefinir** o devolve.
- **Iluminação**: o ângulo (135 graus de início), a intensidade (50 de início)
  e a elevação da luz, e mais duas luzes que você pode ligar.
- **Cor**: **Preenchimento do objeto** (a cor do objeto, de início),
  **Sólida** (uma cor) ou **Sombreada** (um degradê de uma cor para outra em
  cada lado).
- **Chanfro**: uma borda chanfrada na face da frente, com a largura.

**Limpar extrusão** remove a extrusão; **Separar** transforma as faces em
objetos.
Os passos saem agrupados ao lado do objeto. [[menu.object > menu.object.break_extrude_apart]] (<kbd>Ctrl+K</kbd>, também no
menu do botão direito) faz o mesmo.
