# ColorCraft 0.1.13

- Novo painel Conflitos com cadeias visuais de cores e relações até o elemento limitante: proteção manual/direta/de fonte, dependência de gradiente, fronteiras de canais, contraste e incompatibilidades/ciclos.
- Histórico individual durante a sessão, limitado a 500 entradas. Repetições consecutivas são agrupadas com contagem por gesto; sondagens do solver e redesenhos não são eventos. Tentativas repetidas de criar uma relação recusada também são agrupadas.
- Registro de edições recusadas, limites em arrastes e conflitos após alterações explícitas de parâmetros ou substituições de uma composição. Abrir o modal e consultar prévias não cria entradas.
- Indicador no rodapé mostra novidades sem abrir ou trocar o painel automaticamente. Limpar o histórico não modifica o projeto nem seu desfazer/refazer.
- Clicar em uma cor a revela na árvore/grade; clicar em uma relação a destaca no painel Vínculos, mesmo fora do filtro da seleção atual.
- Menu de contexto nas relações das cadeias oferece Desfazer vínculo. A operação remove somente a relação e preserva todas as cores e valores. Desfazer/restaurar mantém a entrada histórica e atualiza a disponibilidade dos alvos. Harmonias e gradientes indicam que a relação inteira será removida.
- Diagnósticos tipados no núcleo preservam procedência, snapshots e identificação de múltiplos bloqueadores. O guia ADDING-RELATIONS.md foi ampliado para desenvolvedores.
- Interface em português/inglês, ícones vetoriais, nomes acessíveis, foco visível e rolagem vertical. Snapshots históricos identificam elementos removidos e restrições de candidatos que não foram aplicados.

Sem alteração do schema v4, das fórmulas ou das regras de aplicação. O histórico de conflitos não é serializado e não torna o projeto modificado. Workspaces salvos são mantidos; os novos layouts incluem Conflitos como aba inicialmente inativa. Distribuição nova em pasta e ZIP, preservando versões e instâncias anteriores.
