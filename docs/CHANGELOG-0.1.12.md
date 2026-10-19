# ColorCraft 0.1.12

- Modal Relacionar cores reorganizado: participantes em duas colunas, canais RGB/HSV em linhas próprias, nomes completos sob as letras e ações separadas do conteúdo rolável.
- Aba Vínculo limitante com coluna de seleção de tipos; Contraste é o primeiro tipo disponível, sem alterar a matemática WCAG.
- Catálogo do núcleo reúne metadados, parâmetros e construtores de vínculos paramétricos, limitantes e harmonias. Receitas e controles leem o mesmo registro; guia em ADDING-RELATIONS.md.
- Correção do popup Adicionar modo: clicar e digitar no nome não fecha o menu. Enter cria a associação vazia e a operação pode ser desfeita.
- Painéis ocultos podem ser reabertos em workspaces preenchidos ou vazios, com destino válido de docking.
- Exclusão de captura usa modal próprio; o fundo fica indisponível enquanto a decisão está pendente. Escape fecha a confirmação, preservando o rascunho e as cores da paleta.
- Borracha por traço na barra de ferramentas da captura. Remove traços completos da máscara selecionada; arrastos rápidos também detectam interseções entre eventos. Outras capturas e páginas são preservadas.
- Máscaras são exibidas como cobertura contínua, com preenchimento uniforme e contorno suavizado, sem acúmulo de transparência nas junções. A amostragem continua usando a geometria original na resolução da fonte.
- Criar composições sem substituições não resolve novamente as originais: evita falsos conflitos de precisão e mantém todos os valores existentes. Originais bloqueadas podem criar novas harmonias, gradientes e vínculos que preservem seus valores. Substituições e edições continuam sujeitas ao solver e aos bloqueios.

Os projetos permanecem no schema v4. IDs e fórmulas das harmonias existentes, perfis, metadados e formatos de intercâmbio foram preservados. Distribuição em pasta e ZIP novos; versões e instâncias anteriores não são substituídas.
