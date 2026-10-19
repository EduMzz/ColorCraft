# ColorCraft 0.1.1 — revisão de interface

- Barra superior compacta, menus planos, guias integradas e tema compartilhado entre controles/painéis.
- Ícones vetoriais do Colorator com tooltips, nomes acessíveis e foco visível.
- Composição em duas colunas, receitas com diagramas/miniaturas e ajuda offline, sugestões de abertura/quantidade e interpolação por pílulas.
- Substituição por clique na paleta com rascunho e rolagem preservados. Escape retorna ao modal; outras edições ficam bloqueadas durante a escolha.
- Diagnósticos estruturados de conflito. Remoções confirmadas e aplicação compõem uma transação, sem alterar o projeto se houver falha.
- Árvore com seleção da linha completa, clique na amostra e em área vazia, cadeados de ação direta e intervalos em pastas expandidas.
- Dropdown no rodapé com cinco apresentações fixas. Mínimo: amostra de cor de ~5 mm ou apenas ícone de pasta, sem texto/botões no item; ações pelo menu de contexto. O dropdown não aparece na árvore.
- Lápis ao lado do título da paleta e do nome da cor para renomeação no lugar.
- Barra de claridade V (HSV) sob a roda; preserva H/S e respeita as relações existentes.
- Vínculos paramétricos com vários participantes, propagação bidirecional e desfazer único. Reutilização de canais já vinculados, sem duplicação.

Os formatos `.colorcraft` v3 e `.colorcraft-palette` v1 continuam iguais. Os vínculos de vários participantes usam as relações de pares já existentes no formato; não dependem de um novo tipo de arquivo.

## Atualizar a versão portátil

O pacote 0.1.1 é separado da 0.1.0. Salve alterações na versão antiga antes de abrir os arquivos na nova. Para manter preferências, biblioteca e recuperação na nova pasta portátil, copie (não mova) `ColorCraftData` da pasta anterior com os aplicativos fechados, ou configure em Preferências o diretório da biblioteca já existente. Nenhuma instância antiga é encerrada pelo empacotamento.
