# Validação do ColorCraft 0.1.13

94 testes aprovados: 41 desktop, 13 unidades do núcleo, 32 fluxos do núcleo, 1 unidade de armazenamento e 7 fluxos de armazenamento. `cargo fmt --all -- --check`, Clippy do workspace/todos os alvos com `-D warnings` e build release com lockfile aprovados.

- Procedência da cadeia de saturação seguida de harmonia termina na cor que impõe o mínimo ou o bloqueio, excluindo cores apenas conectadas sem responsabilidade pelo bloqueio.
- Cobertura de bloqueios manual, direto e de fonte, dependência de gradiente, máximos de canais, contraste, incompatibilidades, ciclos, múltiplos bloqueadores e substituições de composição. Ciclos identificam suas relações; fronteiras empatadas geram cadeias separadas.
- O arraste limitado mantém um resultado válido e registra a fronteira encontrada. A matemática e os testes anteriores do solver permanecem preservados; produzir um diagnóstico não modifica o projeto nem seu histórico.
- Contagem por gesto, agrupamento consecutivo, atualização de snapshots, limite de 500 entradas, limpeza, separação entre documentos e descarte com o documento cobertos. O histórico não é serializado e não torna o projeto modificado.
- Abrir ou redesenhar a composição, inclusive reparar slots desatualizados passivamente, não gera eventos. Uma substituição explícita conflitante gera uma ocorrência, sem repetir as sondagens internas.
- Eventos reais de entrada do egui cobrem o menu contextual de uma relação, remoção pelo comando existente, preservação dos valores, desfazer e refazer. O registro histórico permanece e a disponibilidade acompanha o projeto atual.
- Navegação revela cores ocultas pela busca e expande ancestrais na árvore. A relação focada aparece em Vínculos mesmo quando a seleção atual normalmente a ocultaria. Navegação e registro não abrem o painel automaticamente nem alteram cores.
- Os snapshots preservam nomes e amostras após exclusões e distinguem itens removidos de candidatos que nunca foram aplicados. Tentativas equivalentes de criar relações recusadas são agrupadas mesmo quando recebem novos IDs.

16 screenshots nativos de QA cobrem as cadeias de mínimo e bloqueio, português/inglês, claro/escuro e 1440 × 920 a 100% / 900 × 680 a 150%. Inspeção visual confirma sequência conectada, textos traduzidos, ícones vetoriais, rodapé e comando de limpeza acessíveis, com rolagem vertical na janela reduzida. Arquivos em `qa/0.1.13`. Essas capturas validam a renderização; a interação é exercitada pelos testes de eventos do egui, sem afirmar automação física do mouse nesta entrega.

QA usa perfis isolados e paletas de demonstração. Nenhum projeto ou preferência do usuário foi usado para testar. A entrega é gerada em pasta e ZIP novos, com runtime PDFium, avisos de licença e guia de extensão de relações atualizado, preservando distribuições anteriores e sem encerrar a instância do usuário.
