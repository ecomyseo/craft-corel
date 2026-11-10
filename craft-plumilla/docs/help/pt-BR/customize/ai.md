# Desenhar e gerar com IA

O TraceDraw pode desenhar a partir de uma descrição em palavras e fazer imagens
com um serviço de IA que você escolhe e configura com a sua própria chave. O
TraceDraw não tem IA própria nem conta: nada é enviado até você executar um
comando de IA, e então só o que esse comando precisa vai para o serviço que você
configurou. Cada pedido é cobrado pelo seu serviço; servidores de modelos locais
no seu computador não custam nada.

## Configurar o serviço de IA {#service}

[[menu.tools > menu.tools.options > menu.tools.options_ai]] tem duas páginas.

![A página Serviço das opções de IA](../../img/customize-ai-service.png)

Na página **Serviço**, configure um serviço de IA para todos os recursos de IA:

1. Escolha uma **Predefinição**: Anthropic, OpenAI, Google Gemini, OpenCode
   Zen, OpenCode Go ou um servidor de modelos no seu computador (Ollama, LM
   Studio, servidor llama.cpp, vLLM). Uma predefinição preenche o tipo de serviço e o endereço;
   **Personalizado** aceita qualquer endereço que fale a mesma linguagem das
   conclusões de chat da OpenAI.
2. Digite a **chave de API** da sua conta nesse serviço (servidores locais não
   precisam). Marque **Lembrar a chave neste computador** para guardá-la; ela
   fica então em texto simples nas suas preferências.
3. Escolha o **Modelo**, e o **Modelo de imagem** para serviços que fazem
   imagens (OpenAI, Google Gemini, ou um endereço personalizado com um modelo
   de imagem).
4. Clique em [[aiui.test]]. Ele pede ao serviço os modelos e diz quantos há, ou
   mostra o erro.

**Mostrar a barra de prompt** coloca a barra de prompt de IA acima da paleta de
cores.

## Geral {#general}

![A página Geral das opções de IA](../../img/customize-ai-general.png)

- [[aiui.enable]] (ligado): desligado, todos os comandos de IA ficam
  esmaecidos.
- [[aiui.services]]: se [[aiui.drawings]] e [[aiui.pictures]] estão
  [[aiui.ready]] ou [[aiui.not_ready]].
- [[aiui.features]]: o que cada comando de IA faz.
- [[aiui.privacy_title]]: só o que um comando precisa, para o serviço que você
  configurou, quando você o executa.

## Desenhar com palavras {#draw-with-words}

Digite um pedido como "uma bicicleta vermelha com rodas azuis" ou "aumente o
círculo e ponha o título embaixo dele", na barra de prompt ou no modo
**Desenhar com palavras** do painel [[docker.ai_generate]]
([[menu.object > menu.object.ai_generate > menu.object.ai_draw_words]]), e
pressione **Desenhar** ou <kbd>Enter</kbd>. O serviço escreve um script para o
desenho, que o TraceDraw executa como um passo de desfazer. A conversa fica
guardada, então você pode pedir mudanças; **Nova conversa** recomeça. **Abrir
em Scripts** mostra o script.

## O painel Gerar com IA {#generate}

O painel [[docker.ai_generate]] ([[menu.tools > menu.tools.ai_generate]], ou o
botão Gerar com IA da barra de ferramentas padrão) faz imagens com o modelo de
imagem do seu serviço.

![O painel Gerar com IA](../../img/customize-ai-generate.png)

Escolha um modo com os botões no topo:

- **Criar imagem**: uma imagem a partir de uma descrição. **Inspire-me**
  preenche uma ideia. Escolha a proporção.
- **Remixar imagem**: muda uma imagem como você descrever (o bitmap
  selecionado de início). Os pedidos prontos incluem mockup, restaurar foto,
  colorir esboço, trocar o fundo, mudar a estação e recolorir.
- **Criar vetores**: uma ilustração transformada em formas editáveis, seja uma
  imagem vetorizada em curvas, seja um desenho SVG escrito pelo modelo de
  texto.
- **Desenhar com palavras** (veja acima).

Todo modo tem um **Modelo**, uma imagem de referência opcional (**Usar
seleção** ou **Importar arquivo**), o **Prompt** e ladrilhos de **Estilo** e
**Paleta de cores**. Embaixo, defina o número de **Imagens** (1 a 4) e
**Inserir no desenho**, depois clique em **Gerar**. Os resultados ficam no
painel; clique em um para inseri-lo.

## Outros comandos de IA {#other}

- [[menu.bitmaps > menu.bitmaps.remove_background]] mantém o assunto de uma
  imagem e deixa o resto transparente.
- [[menu.bitmaps.ai_art_styles]] repinta a seleção em um de oito estilos.

Enquanto a IA está desligada ou não configurada, os comandos ficam esmaecidos, e
a dica deles diz por quê; o botão [[aiui.set_up]] abre a página de opções
certa.
