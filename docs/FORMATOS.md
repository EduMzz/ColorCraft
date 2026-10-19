# Contrato de dados 0.1

UTF-8, JSON, números finitos, cores opacas. Todos os caminhos abaixo são locais. Este contrato é uma proposta independente, não um padrão oficial ArtCraft. Leitores devem rejeitar versões principais não suportadas e preservar arquivos de origem. Os schemas acompanham o pacote em `schemas/`; verificações semânticas adicionais pertencem ao núcleo.

## Projeto `.colorcraft`

`schemaVersion: 4`, UUID `id`, `name`, `description`, `revision`, `createdAt`, `updatedAt`, `items`, `profiles`, `links`, `harmonies`, `gradients`, `limits`, `documentaryRelations`, `roles`, `modes` e `sources`. Projetos v3 são migrados ao abrir, preservando cores, metadados e histórico; `schemas/project-v3.schema.json` documenta a estrutura anterior. `items` usa `type: color|folder` e `parentId`, com IDs estáveis. O vetor mantém a ordenação dos irmãos. A pasta não é um agrupamento semântico.

Uma cor antiga pode ter apenas `rgb: [r,g,b]` em sRGB 0–255. Havendo `color: {space, components, profileId}`, esse valor é autoritativo e `rgb` é cache de prévia sRGB, refeito ao abrir. `srgb`, `display-p3`, `cmyk` e `gray` usam componentes 0–1; `lab` usa L*0–100, a*/b*−128–127 e D50. RGB com um ICC personalizado mantém o modelo `srgb` e o perfil explícito: o perfil é quem define a colorimetria, não o nome do modelo. `profiles` contém os bytes ICC originais como vetores de bytes, com `id` e `name`. Perfis internos não sRGB são embutidos ao aplicar uma edição nativa.

`locked` fixa o valor no solver. Desde 0.1.2, o campo opcional `directLocked` impede edição direta de componentes/perfil, mas permite propagação pelas relações; ausente significa `false`. `sourceLocked` indica valor controlado por uma captura. Os estados são independentes; o desbloqueio da fonte exige desvinculação explícita. Ambos são omitidos dos snapshots de exportação para consumo.

Cada `sources[]` guarda caminho vinculado ou bytes base64 incorporados, fingerprint SHA-256, miniatura e capturas. Uma captura armazena página, coordenadas normalizadas, método, parâmetros, máscara e valor em sRGB, com `colorId` opcional para a cor controlada. Limites atuais: 64 MiB por fonte incorporada, 180 MiB totais de bytes originais incorporados e 256 MiB por arquivo de projeto. Alterações da fonte não recalculam valores até atualização explícita; remoção/desvinculação preserva a cor.

Relações v1 da matemática: `links.channels` 0/1/2=R/G/B, 3/4/5=H/S/V. `relations` tem chaves desses índices e `{mode: ratio|offset, value}`. Harmonias guardam offsets HSV congelados em `constraints`, com âncora opcional. Gradientes têm origens `a`/`b`, IDs intermediários, espaço e `detached`. Limites têm `metric: wcag22-srgb` e `minimum` de 1 a 21. Mudança futura de significado exige nova versão/migração explícita; não reinterpretar versões antigas silenciosamente.

O importador aceita JSON Colorator v1/v2 sem sobrescrever a origem. Relações legadas sem âncora continuam sem âncora inventada. Campos desconhecidos na raiz e em itens são preservados; os demais objetos ainda não oferecem garantia geral de preservação de extensões. Nunca salve um arquivo futuro por meio desta versão.

## Paleta `.colorcraft-palette` ou JSON resolvido

`format: colorcraft-palette`, `schemaVersion: 1`, `paletteId`, `revision`, informações do projeto e `previewSpace: srgb`. Mantém itens, ordenação, pastas, IDs, metadados, perfis, papéis, modos e relações documentais. Cada cor tem `color` nativa e `previewSrgb` 0–255. A prévia pode ter sido limitada para a tela; nunca substitui a cor nativa. Não contém bloqueios nem regras de edição obrigatórias. `extensions` preserva extensões da raiz do projeto.

`technicalAppendix`, quando solicitado, declara `informationalOnly: true`, `mathVersion: 1` e as regras para documentação. Consumidores podem ignorá-lo. O importador do ColorCraft também o ignora e cria um projeto novo com os valores resolvidos; para editar regras use o projeto original. `paletteId`/IDs dos itens identificam a fonte; importação registra `sourcePaletteId` numa nova identidade de projeto.

## Biblioteca local

No Windows, o caminho canônico compartilhado entre aplicativos é `%APPDATA%/ArtCraft/Palettes/library.json` (em geral `C:\\Users\\<usuário>\\AppData\\Roaming\\ArtCraft\\Palettes\\library.json`). O índice `library.json` é criado mesmo quando ainda não há paletas publicadas. Snapshots imutáveis ficam sob a mesma raiz. Todos os programas ArtCraft que consumirem paletas ColorCraft devem procurar primeiro esse caminho diretamente; não precisam iniciar o ColorCraft nem ler preferências dele. A biblioteca é por usuário do Windows e não é sincronizada.

Ao iniciar, versões novas mesclam bibliotecas conhecidas de versões anteriores, sem apagar as origens: `%LOCALAPPDATA%/colorcraft/library`, o caminho indicado por `%LOCALAPPDATA%/colorcraft/library-location.json`, o diretório de dados da instalação atual e qualquer diretório de biblioteca anteriormente configurado. O índice e os snapshots legados são validados antes da cópia. Conflitos ou dados inválidos são reportados e preservados para resolução, sem sobrescrita silenciosa. Em instalações portáteis, preferências e recuperação continuam em `ColorCraftData`; a biblioteca publicada continua no caminho canônico compartilhado. Configuração instalada padrão do Windows em `%APPDATA%/colorcraft/config.toml`; portátil em `ColorCraftData/config/config.toml`. `workspace.json` preserva compatibilidade com a disposição de UI anterior. `workspaces.json` armazena os layouts nomeados de painéis e o workspace ativo; esses arquivos não contêm paletas, valores ou metadados do projeto.

`library.json` tem `schemaVersion: 1`, `palettes: [...]`. Cada entrada contém `paletteId`, `revision`, `name`, `colors`, `updatedAt`, `snapshot`, `sourcePath`, `preview`. `snapshot` é um caminho relativo à raiz da biblioteca, com `/`, para um arquivo imutável em `snapshots/<hash-do-id>/<revisão>-<hash-do-conteúdo>.colorcraft-palette`.

Leitores devem ler o índice atomicamente, validar caminhos dentro da raiz, ler o snapshot apontado e guardar os valores usados em seu próprio documento. Não acompanhar alterações de cor silenciosamente: informar uma nova revisão e deixar o aplicativo consumidor decidir quando aceitá-la. Observar o índice ou consultá-lo periodicamente; não depender de um serviço residente. Não há integração de escrita de volta ao projeto ou protocolo de rede nesta entrega. Snapshots antigos não são removidos automaticamente.

Gravação do projeto precede publicação. Falha no índice não desfaz o projeto salvo; a publicação é comunicada como pendente. Índices de versão futura/corrompidos não são recriados por cima. Arquivos de lock são marcadores de exclusão entre processos; a exclusão efetiva é do sistema operacional e é liberada ao sair.

## Exportadores opcionais

| Formato | Conteúdo e limites |
|---|---|
| CSS | Variáveis sRGB fracionárias, aliases semânticos e seletores de modos. Perfis e regras não acompanham. |
| DTCG 2025.10 | Tokens de cor sRGB, Display P3 ou Lab quando o perfil corresponde ao padrão. Demais perfis/modelos convertem para sRGB com aviso; origem e perfis acompanham extensão `colorcraft`. Papéis são aliases; modos ficam na extensão. |
| GPL | Cabeçalho GIMP Palette, sRGB de 8 bits e nomes, com arredondamento. |
| Simple `.color-palette` | JSON `{name, colors:[{name, components:[r,g,b]}]}`, componentes sRGB **lineares**, precisão 0,00001. Pode representar sRGB estendido na saída; importação estendida/transparente é recusada nesta versão. Exige ao menos uma cor. |
| CSV | Separador `;`, aspas duplicadas, componentes/perfil nativos e coluna HEX explicitamente de prévia sRGB. Bytes ICC não acompanham. |
| SVG | Folha de amostras com nome e HEX de prévia sRGB. Sem estrutura editável. |

CSS/CSV/SVG/GPL podem limitar cores ao gamut sRGB; o aviso exige confirmação. JSON próprio e projeto mantêm o nativo. Arquivos opcionais são criados por implementação própria a partir dos contratos públicos, sem copiar código concorrente.

## Referências

- [Design Tokens Color](https://www.designtokens.org/tr/2025.10/format/#color)
- [Simple Color Palette](https://github.com/simple-color-palette/spec)
- [CSS Color 4](https://www.w3.org/TR/css-color-4/)
- [WCAG 2.2: contraste](https://www.w3.org/TR/WCAG22/#contrast-minimum)
- [OKLab, publicação de Björn Ottosson](https://bottosson.github.io/posts/oklab/)
- [craft-libs, revisão utilizada](https://github.com/storytold/craft-libs/tree/82a24b006eefa036b43c09a67fb9a3fbd17ffe63)

## Associações semânticas vazias

`roles[].colorId` pode ser uma string vazia quando o papel não tem cor padrão. Em `modes[].overrides`, chave ausente herda o padrão; chave presente com string vazia significa associação explicitamente vazia. IDs não vazios precisam referenciar cores válidas. Excluir uma cor limpa as associações atingidas sem excluir os papéis. Projetos e snapshots preservam esses estados; CSS usa `initial` para o modo explicitamente vazio.
