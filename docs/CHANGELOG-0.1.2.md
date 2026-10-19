# ColorCraft 0.1.2

- A área de rolagem ocupa a largura disponível do painel, independentemente da quantidade de colunas de cards. A margem fica no conteúdo e as barras junto à borda direita.
- Arrastar uma aba sobre a área livre de um painel o deixa flutuante. Uma cruz de alvos pequenos indica onde dividir ou agrupar; soltar diretamente na faixa de abas continua agrupando. Escape cancela. Há também um botão de desencaixar no cabeçalho e a ação no menu de contexto da aba.
- Cards e linhas da árvore mostram um ícone vetorial de corrente quando a cor participa de vínculos de canais, harmonias, gradientes ativos ou limites de contraste. A dica descreve os tipos. Intermediárias dependentes têm uma segunda indicação de gradiente. Relações documentais não acionam esse indicador.
- Novo botão de mão bloqueia apenas a edição direta de uma cor. Ela continua acompanhando as outras participantes das relações. O cadeado mantém o bloqueio total. Os estados são independentes, têm desfazer/refazer e são preservados ao salvar, copiar e duplicar.
- A mão aparece nos cards com controles, na árvore, no painel Cor e no menu de contexto. Os níveis de amostra continuam mostrando somente a cor; use o menu de contexto para os bloqueios e para consultar as relações.
- Roda, claridade, HEX, canais e conversão respeitam o bloqueio direto. Substituição numa composição exige liberação explícita; liberação e aplicação formam uma transação única.

Projetos continuam na versão 3, com o campo opcional `directLocked` por cor (ausente significa falso). O snapshot para consumo continua na versão 1 e não exporta bloqueios de edição. As versões anteriores não implementam o bloqueio direto; use 0.1.2 para respeitá-lo. O modelo e a serialização de docking são mantidos, incluindo posições salvas. Nenhuma matemática de cor foi alterada.

A distribuição tem pasta e ZIP próprios. Não substitui nem encerra as versões já abertas.
