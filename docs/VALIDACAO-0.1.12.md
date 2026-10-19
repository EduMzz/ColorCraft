# Validação do ColorCraft 0.1.12

79 testes aprovados: 34 desktop, 5 unidades do núcleo, 32 fluxos do núcleo, 1 unidade de armazenamento e 7 fluxos de armazenamento. Formatação, Clippy com `-D warnings` e build release verificados.

- Eventos de ponteiro e teclado do egui cobrem clicar no campo do popup de modo, digitar, criar com Enter, desfazer e preservar os valores das cores.
- Todos os painéis são ocultados e reabertos em layouts preenchidos e vazios sem erro de destino. Projetos não são modificados pela navegação.
- Canais têm letra e nome em botões com largura uniforme. Rodapé permanece acessível em janela lógica de 600 × 453 pontos, equivalente à área reduzida a 150%.
- Exclusão de captura usa modal, bloqueia o canvas, permite cancelar/confirmar e trata Escape. O rascunho só é aplicado pelo botão Aplicar capturas.
- Borracha apaga um traço completo por interseção, incluindo deslocamentos rápidos entre eventos, preservando outras capturas/páginas. Máscara vazia não pode ser aplicada.
- Cobertura de pintura não acumula transparência em juntas/cruzamentos. O cache visual independe do zoom; a amostragem mantém a geometria normalizada e resolução de origem.
- Regressão reproduz resíduo numérico aceito em vínculo existente que gerava falso conflito ao fixar as duas originais. Prévia e aplicação de gradiente sem substituições preservam cada item existente, as relações e desfazer/refazer.
- Cores com bloqueio manual, bloqueio direto ou vínculo de fonte podem criar vínculos, gradientes e harmonias sem alterar seus valores. Substituições e edições continuam passando pelo solver; testes existentes de conflitos, ciclos e bloqueios permanecem aprovados.
- Catálogo verifica IDs únicos, parâmetros válidos e construtores; as fórmulas e IDs existentes foram preservados.

Conferência visual em português/inglês, temas claro/escuro, 1440 × 920 a 100% e 900 × 680 a 150%, incluindo modal de relações, limite, papéis e captura. Screenshots automáticos do workbench usam o mesmo componente dentro de uma janela nativa de QA, pois o backend Glow não entrega screenshots de viewports imediatos secundários. A janela independente foi conferida separadamente por captura do Windows: modal de exclusão claro a 100%, borracha escura a 150%, clique e arraste real de marcador mantendo as duas capturas existentes.

QA usa perfis isolados e arquivos de demonstração. Nenhum projeto do usuário foi aberto ou modificado para testar. Processos de QA são encerrados; instâncias do usuário e distribuições anteriores são preservadas. A distribuição inclui o runtime PDFium e os avisos de licença.
