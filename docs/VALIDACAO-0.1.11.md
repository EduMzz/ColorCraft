# Validação do ColorCraft 0.1.11

Verificação final: 69 testes aprovados (27 desktop, 4 amostragem, 30 fluxos do núcleo, 1 armazenamento e 7 fluxos de armazenamento); formatação e Clippy com -D warnings aprovados.

- Regressões de inserção na árvore: pasta, cor, seleção múltipla com níveis diferentes, colagem de múltiplas cores e clipboard interno; o destino em cards continua sendo a pasta aberta.
- Eventos de ponteiro do egui verificam arrastar cores da grade e da árvore para as associações padrão e por modo. Desfazer restaura a associação vazia sem modificar cores.
- Eventos de teclado verificam criação/reutilização de tags com Enter, preservando a grafia existente e evitando duplicatas.
- Associações vazias validadas em gravação/reabertura, snapshot público, importação e CSS. Exclusão de cor preserva o papel; desfazer recupera tudo.
- Cabeçalhos verificados sem botão de desencaixar. Seletor de workspace centralizado verticalmente; controles de janela ficam nos limites em larguras lógicas de 1440, 900, 600 e 480 pontos.
- Capturas nativas isoladas em 1440 × 920 a 100% e 900 × 680 a 150% cobrem diálogos novos, modos, papéis, tags e cartões de vínculos nos temas claro e escuro. Conferidos rodapés e rolagem em janela reduzida. As capturas de QA não usam a biblioteca ou as preferências do usuário.
- Distribuição gerada em nova pasta e ZIP, preservando versões anteriores e processos já abertos.

Os testes de gesto são eventos reais enviados ao egui, não automação de mouse do Windows. As capturas nativas verificam o resultado visual; não substituem uma matriz exaustiva de dispositivos e escalas de monitor.
