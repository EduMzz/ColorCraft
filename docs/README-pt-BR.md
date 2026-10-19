# ColorCraft 0.1.14

Editor standalone de paletas paramétricas em Rust, inicialmente para Windows x64. Aplicativo independente de Eduardo; utiliza bibliotecas Craft, sem representar uma admissão oficial na suite ArtCraft. O Colorator web original continua na pasta superior.

**Escopo da primeira proposta de merge: somente ColorCraft.** O fork do VectorCraft é uma prova de conceito da integração entre aplicativos e permanece uma entrega experimental separada. Veja [preparação e limites da proposta](MERGE-PREPARATION.md).

## Executar

Extraia o ZIP inteiro numa pasta com permissão de escrita e abra `ColorCraft.exe`. Não precisa de navegador, Node, Python, servidor ou instalação do Rust. A interface usa OpenGL; é necessário um driver gráfico compatível. O executável inclui o runtime C/C++ estaticamente.

O marcador `portable.txt` mantém preferências e recuperação em `ColorCraftData`, ao lado do executável. Não apague essa pasta para atualizar o programa. Os projetos ficam onde você escolher ao salvar. As paletas publicadas ficam sempre em `%APPDATA%\ArtCraft\Palettes\library.json`, inclusive no modo portátil, para que outros programas ArtCraft as encontrem no mesmo lugar. Na primeira abertura, versões novas mesclam bibliotecas legadas conhecidas sem remover os arquivos antigos.

Na Home, use **Nova paleta**, **Abrir** ou o exemplo. Também é possível arrastar arquivos para a janela. Para migrar uma paleta do Colorator, exporte seu JSON no webapp, abra-o aqui e salve como `.colorcraft`. O JSON importado permanece intacto.

Para mostrar o ícone do ColorCraft no Explorador e abrir esses documentos com duplo clique, execute `Register-ColorCraftFiles.ps1` na pasta extraída. O registro é feito somente no perfil atual do Windows e não exige administrador. Para desfazer, execute `Unregister-ColorCraftFiles.ps1`. Se mover a pasta portátil, registre novamente no novo local.

## Integração local com VectorCraft

Esta versão recebe lotes confirmados pelo protótipo VectorCraft e abre projetos/cores por `colorcraft://open?palette=<id>&color=<id>`. Execute `Register-ColorCraftProtocol.ps1` na pasta do pacote para registrar o protocolo por usuário. A biblioteca pública continua em `%APPDATA%\ArtCraft\Palettes`; a fila local fica em `%LOCALAPPDATA%\ColorCraft\Bridge\v1`. Não há serviço de rede. Um envio é uma operação de desfazer, seguida de salvamento e publicação, sem substituir cores existentes. Veja [contrato e recuperação de falhas](BRIDGE-v1.md).

## Nesta versão

- Home, documentos em abas, menus, temas claro/escuro e painéis Paleta, Cor, Vínculos, Detalhes, Fontes e Papéis semânticos que podem ser reposicionados e agrupados.
- Rolagem junto à borda dos painéis e encaixe por alvos explícitos; painéis podem ser desencaixados por arraste. Indicadores de relações técnicas e bloqueio de edição direta independente do cadeado.
- Cards com cinco níveis fixos no dropdown dentro da navegação do painel Paleta, da amostra de aproximadamente 5 mm ao card completo; árvore com seleção da linha inteira e cadeados clicáveis.
- Workspaces com predefinições Essentials, Essentials Classic, Edição de cor e Relações; crie espaços nomeados a partir da disposição atual, renomeie, exclua e redefina os padrões. O menu Janela mantém os comandos de painéis e docking.
- Modal de composições com duas colunas, cards de receitas, informações offline, pílulas de sugestões, substituição por clique na paleta e resolução explícita de conflitos.
- Lápis para renomear a paleta e a cor no próprio título; barra de claridade V (HSV) abaixo da roda.
- Seleção múltipla, copiar/recortar/colar entre documentos, duplicação, movimentação, bloqueio e histórico de desfazer/refazer.
- Roda cromática ampla, campos HEX/RGB/HSV/HSL/OKLCH; cores nativas sRGB, Display P3, Lab, cinza e CMYK com perfis ICC embutidos/importados.
- Vínculos bidirecionais de canais RGB/HSV entre duas ou mais cores numa operação e ciclos compatíveis; nove receitas HSV, cores intermediárias em sRGB/OKLab/OKLCH e destaque de intermediárias.
- Contraste mínimo WCAG 2.2 entre cores opacas sRGB. Entrada numérica incompatível é rejeitada; arraste para no limite sem compensar a outra cor. Outras relações e bloqueios continuam sendo respeitados.
- Metadados, campos próprios, relações documentais, papéis semânticos e modos com herança do padrão.
- Projeto editável, publicação local automática e exportações `.colorcraft-palette`, JSON, CSS, DTCG, GPL, `.color-palette`, CSV e SVG, com confirmação das perdas pertinentes.
- Arquivos próprios `.colorcraft` e `.colorcraft-palette` usam o ícone de arquivo do ColorCraft no Windows; scripts portáteis permitem registrar e remover associações apenas para o usuário atual.
- A captura de fontes abre em janela nativa independente, com redimensionamento e controles vetoriais; marcadores existentes podem ser selecionados e reposicionados com lente de precisão, e a máscara do pincel exibe o raio e pode ser limpa.
- Gravação atômica, detecção de conflito de arquivo, cópia anterior e recuperação de rascunhos separada dos projetos.
- Salvamento automático periódico dos projetos nomeados e publicação da revisão correspondente na biblioteca compartilhada, com nova tentativa quando a publicação estiver pendente.

## Limites conhecidos

Primeira versão funcional. A integração com VectorCraft existe como PoC separado; a integração de produção nos programas da suíte será proposta posteriormente. Serviços de nuvem, CLI/MCP de automação, mais modelos de harmonia e outros formatos de intercâmbio continuam no plano. Os painéis flutuam dentro da janela principal; a captura de fontes usa uma janela nativa independente.

Na captura, use a barra vertical de ícones: conta-gotas seleciona e move marcadores existentes ou cria uma amostra no espaço livre; pincel cria uma máscara separada de uma captura pontual e adiciona traços à máscara selecionada; mão desloca a imagem; lupa amplia ao clicar e reduz com Alt ou botão direito. A lista e as opções têm painéis e rolagens independentes. Aplicar e Cancelar permanecem no rodapé.

PDFs com indicação de perfil ICC oferecem continuar capturando os pixels da prévia PDFium ou cancelar para converter uma cópia para sRGB em outro aplicativo. A escolha também é exigida ao atualizar ou relincar uma fonte desse tipo. Aceitar a prévia não garante equivalência com outros aplicativos. Raster e SVG com perfis não suportados ainda exigem conversão externa.

A tela usa prévia sRGB; não há detecção/calibração automática do perfil do monitor. Valores nativos e bytes ICC são preservados separadamente da prévia. O perfil CMYK genérico é **sintético**, não uma caracterização de impressora. Para impressão, forneça o perfil real e faça a validação no fluxo de saída. Perfis não suportados produzem erro de conversão. Transparência não faz parte do modelo desta versão; importações que a exigem são recusadas integralmente.

Os formatos opcionais não são todos reimportáveis: o aplicativo abre projetos, snapshots próprios, JSON do Colorator, GPL e Simple Color Palette. CSS, DTCG, CSV e SVG são saídas nesta versão. O JSON Schema descreve a estrutura; o núcleo também valida referências, ciclos, gamut e consistência das relações.

Leia [o manual](MANUAL.md), [o contrato dos formatos e biblioteca](FORMATOS.md), [a validação](VALIDACAO.md) e [os créditos](../NOTICE.md).

## Compilar

Rust estável com alvo `x86_64-pc-windows-msvc` e Visual Studio Build Tools C++/Windows SDK. Na pasta `colorcraft`:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check -p colorcraft-core --target wasm32-unknown-unknown
cargo run -p colorcraft
```

`tools/package.ps1` gera o executável e o ZIP portátil com exemplos e licenças. Dependências compartilhadas estão fixadas no commit `82a24b006eefa036b43c09a67fb9a3fbd17ffe63`; versões resolvidas em `Cargo.lock`. O build usa CRT estático no Windows.

Código original sob MIT OU Apache-2.0. O ícone fornecido por Eduardo tem termos próprios descritos em `assets/ASSETS.md`.

- Interface available in Brazilian Portuguese and English; switch languages in Edit > Preferences > Interface language.


Guia para desenvolvedores: [Adicionar vínculos e harmonias](ADDING-RELATIONS.md). Notas desta versão: [0.1.12](CHANGELOG-0.1.12.md).

O [painel Conflitos](CHANGELOG-0.1.13.md) registra bloqueios e limites durante a sessão, com cadeias navegáveis e remoção contextual de relações. Não é salvo dentro das paletas.
