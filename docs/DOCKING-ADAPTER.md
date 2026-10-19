# Adaptador de apresentação do docking

O modelo, as ações transacionais e a serialização continuam sendo `craft-ui::docking`, revisão `82a24b006eefa036b43c09a67fb9a3fbd17ffe63`. O cache Cargo e o upstream não são modificados.

`crates/desktop/src/docking/surface.rs` adapta a apresentação de `crates/ui/src/docking/surface.rs` dessa revisão de https://github.com/storytold/craft-libs. Copyright (c) 2026 ArtCraft Team and the craft-libs contributors. MIT OU Apache-2.0; avisos e licenças são preservados em `NOTICE.md` e no pacote `third-party/craft-libs`.

Modificações locais: alvos explícitos de encaixe, prévia correspondente à área atingida, fallback flutuante também dentro do corpo do painel original, desencaixe por arraste e ícones vetoriais. O resolvedor local delega a ordenação na faixa de abas ao resolvedor compartilhado e preserva a oclusão pelo grupo à frente. O corpo de um painel deixa de ser, por inteiro, uma área de encaixe. O desenho dos alvos e o cálculo da proposta usam os mesmos retângulos.

Os painéis flutuantes permanecem dentro da janela do ColorCraft, como antes. Os testes exercitam sequências de eventos de ponteiro no renderer, cancelamento por Escape, agrupamento pela aba, divisão pelo alvo, ausência de botões de desencaixar nos cabeçalhos, preservação do projeto e largura das áreas de rolagem.
