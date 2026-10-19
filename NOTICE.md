# Créditos e licenças

ColorCraft: concepção, direção e ícone de Eduardo. Implementação original em Rust derivada funcionalmente do seu Colorator. Código original disponível sob MIT OU Apache-2.0, à escolha do destinatário. Veja `LICENSE-MIT` e `LICENSE-APACHE`. Imagens têm termos separados em `assets/ASSETS.md`.

Bibliotecas `craft-ui`, `craft-cms`, `craft-settings`, `craft-atomic`: Copyright (c) 2026 ArtCraft Team and the craft-libs contributors, MIT OU Apache-2.0, revisão `82a24b006eefa036b43c09a67fb9a3fbd17ffe63` de https://github.com/storytold/craft-libs. Esta distribuição usa as bibliotecas sem representar uma versão oficial da suite. Avisos upstream mantidos em `third-party/craft-libs/`.

Interface e execução: egui/eframe/epaint e contribuidores; fontes padrão e respectivas licenças; rfd; image; serde/serde_json; uuid; toml; blake3; fs2; time; winresource e demais dependências transitivas. O pacote contém `third-party/dependencies.csv` e os arquivos de licença das dependências resolvidas para o alvo Windows. `Cargo.lock` registra as versões exatas. Quando o pacote Cargo omite os avisos, eles são obtidos no commit upstream indicado em seus metadados; o arquivo `SOURCES.txt` registra a origem. O código-fonte inalterado da dependência `option-ext` acompanha sua pasta sob MPL-2.0, juntamente com a licença. Para dependências com alternativas de licença, a distribuição utiliza a alternativa permissiva aplicável, preservando os avisos originais.

O renderer local de docking adapta `crates/ui/src/docking/surface.rs` de craft-libs na revisão acima (mesmo copyright e licenças), com alvos de encaixe explícitos. O modelo compartilhado permanece inalterado. Veja `docs/DOCKING-ADAPTER.md`.

Matemática implementada usando conceitos públicos e referências W3C CSS Color, WCAG 2.2, ICC e as matrizes de OKLab de Björn Ottosson. Nenhum código de Leonardo, Atmos, Adobe ou outro concorrente foi copiado. `.color-palette` e Design Tokens são saídas de interoperabilidade; suas marcas e especificações continuam pertencendo aos respectivos responsáveis.

Não há updater, telemetria, servidor, conta ou envio à nuvem. Links de ajuda abrem o navegador apenas quando acionados pelo usuário.

Ícone Home: Lucide house (lucide-static), conforme usado no VectorCraft, sob ISC. Aviso completo em docs/LUCIDE-ICONS-LICENSE.txt.

Integração local v1: crate colorcraft-bridge, código original MIT OU Apache-2.0, compartilhado com o fork independente EduMzz/vectorcraft-colorcraft. O receptor não é um servidor de rede.
