# Validação do ColorCraft

## Regressões da 0.1.9

`cargo test --workspace`: 61 testes aprovados (21 desktop, 4 amostragem, 29 fluxos do núcleo, 7 armazenamento). Formatação e Clippy com `-D warnings` aprovados.

Eventos reais de ponteiro no egui verificam cliques repetidos em marcadores sem criar duplicatas, arraste mantendo IDs, preservação da captura pontual ao pintar, botão Nova máscara criando outra máscara independente, navegação por mão e lupa sem modificar as capturas, e bloqueio de captura/aplicação até aceitar o aviso de cor. Os botões Aplicar e Cancelar são conferidos dentro dos limites em janela ampla, reduzida e espaço lógico equivalente a escala 150%, nos dois temas.

Um PDF de teste contém um perfil ICC sRGB real em um espaço ICCBased; o renderer produz a prévia e um aviso pendente de escolha, em lugar de rejeitar o documento pela presença do marcador. Esse teste não comprova equivalência colorimétrica com leitores externos.

A janela nativa independente foi exibida e inspecionada a aproximadamente 760 × 550, confirmando a barra vertical, os três painéis e o rodapé visível. O usuário interrompeu a automação com Escape antes do teste interativo nativo de clique/arraste; esses gestos foram verificados pelos testes de eventos do egui. Não foi concluída uma matriz nativa de todos os tamanhos, temas e escalas.

As distribuições anteriores são preservadas; nenhum processo anterior do usuário foi encerrado. A instância isolada de QA usa dados temporários.

## Validação da versão 0.1.2

Ambiente: Windows x64, Rust 1.99.1 estável MSVC, build com CRT estático, egui/eframe 0.36.2. `craft-libs` fixada em `82a24b006eefa036b43c09a67fb9a3fbd17ffe63`.

## Verificações automatizadas

- `cargo fmt --all --check`.
- `cargo clippy --workspace --all-targets -- -D warnings`.
- `cargo test --workspace`: **47 testes**, sendo 27 do núcleo, 7 do armazenamento e 13 da aplicação.
- `cargo check -p colorcraft-core --target wasm32-unknown-unknown`: núcleo compila sem dependência de UI ou sistema de arquivos.
- `cargo build --locked --release -p colorcraft`: executável Windows x64 com ícone e informações de versão.

Os testes cobrem razões RGB bidirecionais, diferenças quando há zero, wrap de matiz, ciclos compatíveis, rollback de conflitos, bloqueios, limite de contraste com regiões desconectadas, harmonias, gradientes, destaque de intermediárias, exclusão parcial de composições, substituição mantendo IDs/metadados, um gesto por etapa de desfazer e reconstrução de referências ao duplicar.

Também verificam preservação de cores nativas/bytes ICC, conversão Lab/CMYK, DTCG com perfis padrão, avisos de conversão, precisão linear do Simple Color Palette, escape de CSS, metadados sem efeitos numéricos, herança de modos, migração de arquivos antigos, recusa de versões futuras e referências inválidas.

O armazenamento é exercitado com gravação atômica, revisão, snapshots imutáveis, conflito de arquivo externo, cópia anterior, índice corrompido, biblioteca indisponível sem impedir salvar em outro destino, distinção projeto/snapshot, recuperação separada, importação GPL/Simple e rejeição de transparência ou JSON desconhecido. Os testes usam diretórios temporários isolados.

Os testes da aplicação renderizam Home, os quatro painéis e diálogos nos temas claro/escuro a 1440×920 e 1060×720; exercitam comandos, seleção, clipboard e serialização/transações de docking. Uma regressão encontrada durante a implementação foi corrigida: exibir os sliders podia arredondar o valor existente. O teste agora garante que exibir RGB/HSV/HSL/OKLCH e reorganizar painéis não altera o projeto.

## Interface nativa e distribuição

O executável é aberto no Windows com renderização OpenGL real para capturar Home, editor, composição e conversão. As imagens de revisão são geradas em `qa/` com `--screenshot`; essa opção usa configuração isolada e diretório de dados de teste, sem alterar a biblioteca real. As capturas são verificadas visualmente para identificar cortes, glifos ausentes, proporções e estados de diálogo.

O pacote portátil contém executável, marcador portátil, exemplos, manual, contratos, schemas, licenças e inventário de dependências. `SHA256SUMS.txt` verifica o executável; o `.zip.sha256` ao lado do ZIP verifica o pacote. O exemplo também é aberto pelo executável empacotado.

## Alcance da validação

Os testes de UI não equivalem a uma sessão humana completa com todos os gestos, escalas de DPI, drivers e monitores possíveis. Não foi validada correspondência física de cor em monitor calibrado, caracterização de impressora, interoperabilidade com um consumidor externo ou integração com VectorCraft. O núcleo compilar em WASM não constitui uma versão web entregue. O aplicativo está funcional como primeira versão standalone; esses limites permanecem explícitos.

## Regressões específicas da 0.1.1

Testes com eventos de ponteiro verificam seleção pela amostra e pela área vazia da árvore, Ctrl, cadeados de linhas e cards sem selecionar/arrastar, e barra de claridade preservando H/S e respeitando bloqueio. São verificados os níveis fixos, os cards mínimos sem controles, a ocultação do dropdown na árvore e a restauração do nível anterior; renomeação com Enter/Escape e desfazer; rascunho da composição ao escolher substituições, Escape e proteção contra mutações; preservação da seleção múltipla ao abrir vínculos e seleção por intervalo em pastas expandidas.

O núcleo verifica diagnóstico com IDs de relações, aplicação/remoção como uma transação com rollback e desfazer único, além de vínculos com múltiplas cores, propagação a partir de uma participante secundária e reutilização dos canais existentes. O teste de renderização confirma também que o modal de composição cabe na janela reduzida.

## Regressões específicas da 0.1.2

Eventos de ponteiro completos (pressionar, ultrapassar limiar de arraste, mover, soltar) verificam desencaixe ao soltar sobre o corpo de outro painel, reagrupamento pela faixa de abas, divisão pelo alvo explícito e cancelamento por Escape. Um teste adicional clica no botão de desencaixar e verifica que os quatro viewports de rolagem preenchem a largura de seus painéis em duas dimensões de janela. Essas ações preservam integralmente o projeto.

Os testes clicam no indicador de relações técnicas e no botão de mão nos cards e na árvore, verificando seleção e ausência de mudanças nos valores. O núcleo verifica recusa de edição direta contínua, numérica e nativa; propagação por vínculos, harmonias e gradientes; precedência do cadeado total; independência dos estados; serialização opcional de `directLocked`; cópia/duplicação e metadados; omissão no snapshot; desfazer/refazer e desbloqueio explícito numa substituição com rollback.

Capturas nativas em `qa/v012` verificam os quatro painéis com rolagem e metadados extensos, cards e árvore com indicadores, bloqueio direto e painel flutuante; temas claro/escuro; janela ampla 1440×920 a 100% e reduzida 1060×720 a 150%. Os painéis se ajustam ao espaço: conteúdo longo permanece acessível por rolagem, e a busca muda de linha quando a paleta fica estreita. As capturas usam dados de demonstração isolados. Os executáveis 0.1.0 e 0.1.1 são conferidos por SHA-256 e permanecem intactos; a instância 0.1.1 do usuário não é encerrada.

## Regressões da 0.1.9

Preferência de idioma português (Brasil)/inglês persistida em Preferências, com catálogo compartilhado para menus, painéis, diálogos, captura, composição, tooltips, Home e mensagens do programa. Projetos de preferências anteriores sem o campo `language` continuam usando português. Os nomes de paletas, cores, papéis e workspaces são conteúdo do usuário e permanecem intactos.
