# Scripts e automação

Os scripts automatizam trabalho repetitivo: desenhar formas a partir de dados,
mudar muitos objetos de uma vez ou acrescentar um formato de arquivo. Eles são
escritos em JavaScript no painel [[docker.scripts]] ([[menu.tools > menu.tools.scripts >
menu.tools.scripts_docker]]).

![O painel Scripts](../../img/customize-scripts.png)

## Escrever e executar scripts {#run}

Um script trabalha com os objetos do TraceDraw: `Application`,
`ActiveDocument`, `ActivePage`, `ActiveLayer` e as formas, preenchimentos,
contornos, cores, curvas e efeitos. Por exemplo:

```js
for (let i = 0; i < 5; i++) {
  ActiveLayer.CreateRectangle(10 + i * 25, 10, 20, 20);
}
```

- [[docker.run]] executa o script. As alterações dele são um passo de desfazer.
- [[docker.record]] grava o que você faz como script (também
  [[menu.tools > menu.tools.scripts > menu.tools.scripts_record]]).
- [[docker.load]] e [[docker.save]] abrem e salvam scripts;
  [[menu.tools.scripts_run]] executa um arquivo de script.
- [[docker.clear]] esvazia o editor; a [[docker.output]] mostra o que o script
  imprime e os erros.

## Depuração {#debug}

No desktop, clique na margem do editor para definir um ponto de parada, depois
clique em [[docker.debug]]. O script para no ponto de parada: **Continuar**,
**Passo** ou **Parar**, e digite uma expressão para avaliá-la onde o script
está pausado.

## Filtros de script {#filters}

Um script que define `TraceDrawFilter`, com um nome, extensões e uma função
`Import` ou `Export`, acrescenta um formato de arquivo a
[[menu.file.import]] e [[menu.file.export]]. Clique em
[[docker.install_filter]] para guardá-lo; [[docker.script_filters]] lista os
filtros instalados com [[docker.edit]] e [[docker.remove]].

## Assistentes de IA (MCP) {#mcp}

O TraceDraw fala o Model Context Protocol, então assistentes de IA no seu
computador podem abrir, editar, inspecionar, exportar e salvar desenhos:

- `tracedraw --mcp` executa o TraceDraw sem janela para um assistente.
- Para deixar um assistente editar o desenho aberto na sua janela, ligue
  [[options.mcp_window]] na página [[options.general]] de
  [[menu.tools > menu.tools.options > menu.tools.options_global]], e dê ao
  assistente `tracedraw --mcp-window` como comando. Cada alteração é um passo
  de desfazer que você pode voltar. Só programas no seu computador com o token
  secreto que o TraceDraw grava podem se conectar.

## A ferramenta de linha de comando {#cli}

Todo download também traz `tracedraw-cli`, que converte desenhos entre formatos
sem abrir o programa, para trabalho em lote.

## Macros {#macros}

O TraceDraw executa projetos de macros VBA escritos para programas de desenho (versão para desktop). [[menu.tools > menu.tools.macros > menu.tools.macros_load]] adiciona um projeto (`.gms`) ou arquivos de módulo (`.bas`, `.cls`, `.frm`); os projetos ficam guardados e carregam ao iniciar. O [[docker.macros]] lista as macros: clique duas vezes numa delas ou use [[menu.tools > menu.tools.macros > menu.tools.macros_run]] (<kbd>Alt+Shift+F8</kbd>) para executá-la. Formulários, caixas de mensagem e de entrada da macro aparecem na tela, <kbd>Esc</kbd> ou [[menu.tools > menu.tools.macros > menu.tools.macros_stop]] interrompe, e tudo o que uma execução muda é um único passo de desfazer. [[menu.tools > menu.tools.macros > menu.tools.macro_editor]] (<kbd>Alt+F11</kbd>) mostra e edita o código. Macros que chamam funções do Windows ou o modelo de objetos do próprio editor não funcionam.
