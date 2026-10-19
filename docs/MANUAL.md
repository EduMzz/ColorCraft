# Usar o ColorCraft

## Projetos e organização

A Home reúne projetos recentes, a biblioteca publicada e recuperações disponíveis. Abra um documento para entrar no editor; o botão Home retorna à página inicial mantendo os documentos abertos. Cada documento possui histórico próprio. O ponto na aba indica alterações ainda não salvas.

Em Paleta, adicione cores e pastas pelos ícones da barra. Dê duplo clique numa pasta na grade ou expanda a árvore. Use a trilha de navegação para voltar. Ctrl alterna itens da seleção; Shift seleciona um intervalo visível, incluindo filhos de pastas expandidas. Arraste para uma pasta para mover, ou entre itens para ordenar. O menu de contexto oferece ações de organização e edição. Na linha da trilha de navegação, o dropdown escolhe Mínimo, Compacto, Resumo, Detalhado ou Completo. No mínimo aparecem somente a cor ou o ícone da pasta, com ações no menu de contexto. Os aproximadamente 5 mm consideram a escala lógica padrão de 96 DPI; o tamanho físico varia com o monitor e a escala. O seletor fica oculto na árvore e o nível escolhido é preservado ao alternar.

O menu Workspaces alterna entre disposições prontas de painéis. Pelo gerenciamento, crie um workspace a partir da disposição atual, renomeie ou exclua os personalizados e redefina os padrões. O menu Janela alterna painéis individualmente e redefine a disposição do workspace ativo. Essas escolhas alteram a interface, sem editar o documento aberto.

O lápis ao lado do título da paleta e do nome da cor abre a edição no lugar. Enter ou a confirmação aplica; Escape cancela. A edição tem desfazer/refazer e preserva as demais informações. Cadeados de cards e linhas alternam o bloqueio sem mudar a seleção; nos níveis sem botões use o menu de contexto.

Arraste a aba de um painel: solte numa área livre para deixá-lo flutuante dentro da janela. Para encaixar, solte num dos pequenos alvos indicados (lados dividem; centro agrupa) ou na faixa de abas. Escape cancela o gesto. O botão no canto direito do cabeçalho e o menu de contexto da aba também oferecem **Desencaixar painel**. O menu Janela reabre um painel e restaura a disposição padrão. Editar → Preferências controla tema e escala. A biblioteca publicada usa o caminho compartilhado canônico documentado em `FORMATOS.md`.

O ícone de corrente nos cards e linhas indica relações técnicas; a dica identifica vínculos, harmonias, gradientes e limites de contraste. A indicação adicional de gradiente identifica uma intermediária dependente. Relações documentais não acionam a corrente. No modo de amostras, consulte **Ver relações técnicas** no menu de contexto.

O botão de **mão** bloqueia somente a edição direta (roda, claridade, HEX, canais e conversão), permitindo que a cor mude em consequência das outras cores relacionadas. A mão riscada indica que essa proteção está ativa. O **cadeado** fixa o valor mesmo quando outras participantes mudam. São controles independentes: retirar um não retira o outro. Ambos estão disponíveis no painel Cor, nos cards com controles, nas linhas e no menu de contexto. Nome e metadados continuam editáveis. O bloqueio direto é salvo no projeto, acompanha cópias/duplicações e tem desfazer/refazer. Uma substituição em composição precisa da confirmação **Liberar bloqueios ao aplicar**, desfeita junto com a composição.

## Valores e relações

Selecione uma cor e use a roda de matiz/saturação e os canais. A barra Claridade abaixo da roda controla V do HSV, preservando H e S; ela permanece disponível nos modelos RGB, HSV, HSL e OKLCH e respeita vínculos e bloqueios. Com foco na barra, as setas ajustam V e Home/End escolhem os extremos. HEX e cópia rápida são representações sRGB; a identificação do espaço nativo aparece separadamente. Alterações durante um gesto formam uma etapa de desfazer.

Selecione duas ou mais cores para criar um vínculo paramétrico. O diálogo mantém toda a seleção e permite adicionar ou retirar participantes. Marque canais RGB/HSV: razões ou diferenças mantêm a relação nos dois sentidos entre todos. A primeira participante define as proporções iniciais, mas qualquer uma pode ser editada. Relações diretas já existentes nos mesmos canais são preservadas e reutilizadas. A operação toda é desfeita de uma vez. Contraste e relações documentais continuam explicitando um par de cores. O programa bloqueia edições sem solução, inclusive em ciclos incompatíveis. O painel Vínculos mostra e remove as relações. Bloquear uma cor mantém seu valor fixo durante a resolução.

Harmonias criam cores relacionadas em HSV. Gradientes criam amostras discretas entre duas origens; escolha interpolação sRGB, OKLab ou OKLCH. A receita identifica a redução de cromaticidade ao gamut quando necessária. Intermediárias são dependentes até serem destacadas. A prévia de composição permite escolher cores existentes como destino: IDs e metadados dessas cores são mantidos, e conflitos precisam ser resolvidos antes de aplicar.

No contraste mínimo, ambas as cores precisam estar em sRGB opaco. O limite usa a luminância relativa WCAG 2.2. Cada cor continua livre até a condição mínima: o programa não move a outra para compensar. O arraste percorre o caminho até o primeiro limite detectado e refina a posição numericamente; uma entrada numérica inválida é recusada integralmente. Não há suporte a composição por transparência ou APCA nesta versão.

## Espaços, perfis e gamut

Atribuir perfil muda a interpretação dos componentes. Converter transforma os componentes para tentar preservar a aparência no destino. São operações diferentes, apresentadas no diálogo com valores antes/depois e prévias sRGB. Importe um ICC original quando necessário. Lab usa D50, L* de 0 a 100 e a*/b* de -128 a 127; CMYK usa quatro componentes de 0 a 1. O CMYK interno é um exemplo sintético explicitamente identificado.

Quando o destino não comporta a cor, o diálogo informa e exige confirmação da limitação dos canais. OKLCH fora do gamut oferece redução explícita de cromaticidade. O valor nativo não é substituído apenas porque a prévia de tela precisou ser limitada. Criar vínculos de canais com cores não sRGB exige conversão confirmada das próprias cores, sem mudar IDs; a conversão e o vínculo são desfeitos juntos.

## Significado e consumo

Detalhes contém nome, nome CSS, descrição, uso, tags e campos JSON próprios. Relações documentais registram, por exemplo, que uma cor serve como texto sobre outra: elas não alteram valores e não participam do solver.

No painel **Papéis semânticos**, crie nomes como `texto.principal`. Cada papel tem uma linha fixa **Sem modo · padrão** e pode receber linhas de modos. Arraste uma única cor da grade ou árvore até a linha, ou escolha pela lista. Uma linha pode ficar **Sem associação**. Remover uma associação preserva a cor; excluir uma cor preserva o papel e deixa suas associações vazias. As operações têm desfazer/refazer.

Crie, renomeie e exclua os modos no painel **Detalhes**, com a raiz da paleta selecionada. Também pode criar um modo ao adicioná-lo a um papel. Criar o modo não cria associações automaticamente. Uma linha explícita sem associação permanece vazia; quando o papel não tem uma linha para o modo solicitado, o consumidor pode usar sua associação padrão. Vários papéis podem apontar para a mesma cor. Nenhuma dessas alterações muda os valores das cores.

CSS exporta aliases dos papéis e seletores `data-colorcraft-mode`; uma associação de modo explicitamente vazia gera `initial`. DTCG preserva modos numa extensão própria. Relações documentais continuam disponíveis no painel Vínculos e em seu diálogo próprio.

O campo **Tags** usa chips. Digite para buscar tags já usadas na paleta; clique numa sugestão ou pressione Enter para usar/criar o texto digitado. O botão de fechar em cada chip remove somente a tag daquele item.

## Salvar e exportar

Salvar grava o projeto `.colorcraft` e publica um snapshot na biblioteca compartilhada em `%APPDATA%\ArtCraft\Palettes\library.json`. Esse é o caminho canônico que outros programas ArtCraft podem ler diretamente, sem iniciar o ColorCraft. A biblioteca é local ao usuário do Windows e não envia paletas à internet. Se a publicação falhar, o projeto continua salvo e a interface informa a pendência.

Com **Salvar automaticamente projetos e atualizar a paleta pública** ativado, alterações em projetos já salvos são gravadas no arquivo e publicadas a cada 15 segundos; uma publicação que falhar é tentada novamente. Projetos novos ou importados ainda sem caminho não criam arquivos nem entradas públicas automaticamente: use Salvar/Salvar como primeiro. A opção separada **Manter rascunhos de recuperação automática** controla as cópias de recuperação, que também protegem alterações quando a gravação automática encontra um conflito externo. Ao sair, o ColorCraft tenta salvar e publicar as alterações pendentes.

Exportar → Paleta ColorCraft cria o snapshot para consumo. O adendo técnico é informativo e opcional; não converte esse arquivo num projeto editável. Para continuar editando as regras, mantenha o `.colorcraft`. Salvar cópia de um projeto existente cria uma identidade de paleta separada.

Saídas opcionais podem converter espaços, limitar gamut, arredondar ou perder estrutura. Leia o aviso e confirme apenas depois de conferir o resultado. `.color-palette` usa sRGB linear e não é o formato autoritativo do ColorCraft.

O salvamento não sobrescreve um arquivo alterado externamente sem aviso. Nessa situação, salve uma cópia ou reabra a versão atual. A extensão `.colorcraft.previous` guarda o conteúdo anterior após um salvamento subsequente. Rascunhos automáticos são cópias separadas oferecidas na Home; recuperação abre uma nova identidade e não sobrescreve o original. Faça também seus backups regulares.

## Atalhos

| Atalho | Ação |
|---|---|
| Ctrl+N / Ctrl+O | Nova paleta / abrir |
| Ctrl+S / Ctrl+Shift+S | Salvar / salvar cópia como |
| Ctrl+W | Fechar documento com proteção de alterações |
| Ctrl+Z / Ctrl+Shift+Z | Desfazer / refazer |
| Ctrl+C / Ctrl+X / Ctrl+V | Copiar / recortar / colar |
| Ctrl+D / Delete | Duplicar / excluir seleção |
| F1 | Guia de uso |

Durante edição de texto, os atalhos de texto têm precedência. Colar aceita uma seleção interna completa ou listas de valores de cor reconhecidos. Recorte entre documentos é recusado; use cópia para manter a operação segura.

Na visão de árvore, adicionar ou colar usa a seleção como destino: uma pasta recebe os itens dentro dela; uma cor usa sua pasta mãe; seleções múltiplas usam a pasta mãe do item com menor profundidade na hierarquia (empates seguem a ordem da paleta). Na visão de cards, o destino continua sendo a pasta aberta.
