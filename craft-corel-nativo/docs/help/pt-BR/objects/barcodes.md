# Códigos de barras e QR codes

O TraceDraw faz códigos de barras para produtos, etiquetas e estoque, e QR codes
que os celulares leem. Os dois são feitos de formas vetoriais comuns, então
imprimem nítidos em qualquer tamanho.

## Fazer um código de barras {#barcode}

Escolha [[menu.object > menu.object.insert > menu.object.insert_barcode]] para
abrir o [[barcode.title]]. Ele tem três etapas; uma [[barcode.sample_preview]]
em cada uma mostra o código como ele será feito. Use [[barcode.next]] e
[[barcode.back]] para andar entre as etapas, e [[barcode.finish]] na última.

![O Assistente de código de barras](../../img/objects-barcodes.png)

### Etapa 1: o padrão e os dados {#barcode-data}

Escolha um dos padrões e digite os dados. Escolher um padrão coloca um exemplo
no campo, e o assistente diz o que o padrão aceita:

- **Codabar**: até 30 dígitos e os sinais `- $ : / . +`, entre caracteres de
  início e fim de A a D.
- **Code 39**: letras maiúsculas, dígitos e `- . espaço $ / + %`.
- **Code 128**: qualquer caractere imprimível.
- **EAN-13**, **EAN-8** e **UPC-A**: 12, 7 ou 11 dígitos, ou um a mais com o
  dígito verificador, seguidos opcionalmente de um espaço e um complemento de 2
  ou 5 dígitos.
- **UPC-E**: 6 dígitos, ou o sistema numérico (0 ou 1) e 6 dígitos.
- **Interleaved 2 of 5**: até 40 dígitos.

Dados que o padrão não aceita mantêm você nesta etapa, com uma mensagem.

### Etapa 2: impressora e tamanhos {#barcode-sizes}

- [[barcode.printer]]: a resolução da impressora, de 150 a 2400
  [[barcode.dpi]] (300 de início). As barras têm um número inteiro de pontos da
  impressora, para imprimirem por igual.
- [[barcode.units]] para os tamanhos abaixo.
- [[barcode.reduction]]: pontos tirados de cada barra para compensar o
  espalhamento da tinta (1 de início). O código mantém a largura.
- [[barcode.magnification]]: 100% é o tamanho padrão.
- [[barcode.bar]] (1 polegada de início), com a [[barcode.symbol_width]]
  resultante.
- [[barcode.ratio]]: 2,0 a 3,0, para Codabar, Code 39 e Interleaved 2 of 5.
- [[barcode.advanced]] acrescenta [[barcode.check_digit]] (Code 39 e
  Interleaved 2 of 5) e a [[barcode.quiet_zone]], uma margem branca atrás das
  barras em [[barcode.modules]].

### Etapa 3: o texto {#barcode-text}

- A [[barcode.font]], o [[barcode.size]] (12 pontos de início), o
  [[barcode.weight]] e o [[barcode.alignment]] do texto legível.
- Opções, disponíveis quando o padrão as usa: [[barcode.center_sys_check]],
  [[barcode.text_above]], [[barcode.addon_text]], [[barcode.addon_bottom]],
  [[barcode.fact_data]], [[barcode.asterisks]], [[barcode.quiet_marks]] e
  [[barcode.start_stop]].
- [[barcode.human_readable]]: desative para barras sem texto.

[[barcode.finish]] coloca o código de barras no meio da página como um grupo e
o seleciona.

## QR codes {#qr-codes}

Escolha [[menu.object > menu.object.insert > menu.object.insert_qr]]. Um QR code
é colocado no meio da página na hora, e o painel [[docker.properties]] abre na
seção [[qr.section]]:

1. Escolha o tipo de conteúdo: [[qr.kind_url]], [[qr.kind_email]],
   [[qr.kind_phone]], [[qr.kind_sms]], [[qr.kind_contact]], [[qr.kind_text]] ou
   [[qr.kind_location]].
2. Preencha os campos, como [[qr.field_url]], ou [[qr.field_first_name]],
   [[qr.field_last_name]], [[qr.field_phone]] e [[qr.field_email]] para um
   contato. Um campo vale quando você sai dele. Campos que os tipos têm em comum
   são mantidos quando você muda o tipo.
3. Defina a aparência: [[qr.dark_fill]] e [[qr.light_fill]] (qualquer
   preenchimento), [[qr.outline]] e [[qr.pixel_size]] (2,5 mm de início).
4. Clique em [[qr.validate]] para conferir se o código pode ser lido. Uma
   mensagem verde confirma; uma vermelha diz por que não, por exemplo quando os
   pixels não são escuros o bastante em relação ao fundo.

O QR code continua editável: mova, escalone ou gire como qualquer objeto e mude
o conteúdo depois. Para transformá-lo em formas simples, desagrupe-o. Conteúdo
que não pode ser codificado, ou em excesso, não muda nada e aparece em
vermelho.
