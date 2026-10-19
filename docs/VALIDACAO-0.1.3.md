# Validação da versão 0.1.3

- `cargo fmt --all`, `cargo check --locked -p colorcraft` e `cargo clippy --workspace --all-targets -- -D warnings` concluídos no Windows.
- A validação não executou a suíte de testes automatizados.
- Capturas nativas do editor em tema escuro (1440×900 e 1060×720) e do gerenciamento de workspaces nos temas escuro e claro (1440×900) revisadas.
- A composição dos workspaces é salva nas preferências locais, separada dos dados dos projetos.
- A distribuição final deve ser aberta para inspeção interativa de criação, alternância, renomeação e exclusão de workspaces.
