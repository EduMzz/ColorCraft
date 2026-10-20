use std::sync::atomic::{AtomicBool, Ordering};

static ENGLISH: AtomicBool = AtomicBool::new(false);

pub fn set_language(language: &str) {
    ENGLISH.store(language == "en-US", Ordering::Relaxed);
}

pub fn is_english() -> bool {
    ENGLISH.load(Ordering::Relaxed)
}

pub fn for_language(text: &str, language: &str) -> String {
    translate(text, language == "en" || language == "en-US")
}

pub fn tr(text: impl AsRef<str>) -> String {
    translate(text.as_ref(), is_english())
}

fn translate(text: &str, english: bool) -> String {
    if !english {
        return text.to_owned();
    }
    if let Some(translation) = translate_exact(text) {
        return translation.to_owned();
    }
    translate_pattern(text).unwrap_or_else(|| text.to_owned())
}

fn translate_exact(text: &str) -> Option<&'static str> {
    EXACT
        .iter()
        .find_map(|(source, translated)| (*source == text).then_some(*translated))
}

const EXACT: &[(&str, &str)] = &[
    (
        "Conclua a edição ou o diálogo aberto no ColorCraft e tente novamente.",
        "Finish the active edit or dialog in ColorCraft and try again.",
    ),
    (
        "O destino mudou. Revise o formulário e confirme novamente.",
        "The destination changed. Review the form and confirm again.",
    ),
    (
        "A pasta de destino foi removida. Escolha o destino novamente.",
        "The destination folder was removed. Choose the destination again.",
    ),
    (
        "O projeto solicitado não está na biblioteca ColorCraft.",
        "The requested project is not in the ColorCraft library.",
    ),
    (
        "O arquivo não corresponde ao projeto solicitado. Abra e publique o projeto correto no ColorCraft.",
        "The file does not match the requested project. Open and publish the correct project in ColorCraft.",
    ),
    (
        "O arquivo mudou em outro processo. Reabra ou salve uma cópia no ColorCraft antes de enviar cores.",
        "The file changed in another process. Reopen it or save a copy in ColorCraft before sending colors.",
    ),
    (
        "A cor solicitada foi removida ou não pertence a este projeto.",
        "The requested color was removed or does not belong to this project.",
    ),
    (
        "Salve as alterações pendentes antes de repetir a publicação.",
        "Save pending changes before retrying publication.",
    ),
    (
        "O envio não contém o perfil ICC da cor.",
        "The request does not include the source ICC profile.",
    ),
    (
        "Receber cores de outro aplicativo",
        "Receive Colors from Another Application",
    ),
    (
        "Conflito registrado. Consulte o painel Conflitos.",
        "Conflict recorded. See the Conflicts panel.",
    ),
    ("Não aplicada", "Not Applied"),
    ("Novos", "New"),
    ("No histórico", "In History"),
    ("Conflitos", "Conflicts"),
    (
        "Tentativas limitadas e conflitos desta paleta, durante esta sessão.",
        "Limited edits and conflicts for this palette during this session.",
    ),
    ("Limpar histórico", "Clear History"),
    ("Nenhum conflito registrado.", "No conflicts recorded."),
    ("Composição em conflito", "Composition Conflict"),
    (
        "Não foi possível criar a relação",
        "Could Not Create Relationship",
    ),
    (
        "Não foi possível alterar a relação",
        "Could Not Change Relationship",
    ),
    ("Não foi possível reduzir", "Could Not Decrease"),
    ("Não foi possível aumentar", "Could Not Increase"),
    ("Não foi possível alterar", "Could Not Change"),
    ("Não foi possível alterar a cor", "Could Not Change Color"),
    ("Cor bloqueada", "Color Locked"),
    ("Edição direta bloqueada", "Direct Editing Locked"),
    ("Cor vinculada a uma fonte", "Color Linked to a Source"),
    ("Valor original preservado", "Original Value Preserved"),
    ("Atingiu o máximo do canal", "Reached Channel Maximum:"),
    ("Atingiu o mínimo do canal", "Reached Channel Minimum:"),
    (
        "Cor controlada por um gradiente",
        "Color Controlled by a Gradient",
    ),
    ("Tentativa", "Attempt"),
    (
        "Ciclo de relações incompatíveis",
        "Incompatible Relationship Cycle",
    ),
    (
        "Valores fixos ou relações incompatíveis",
        "Incompatible Fixed Values or Relationships",
    ),
    (
        "Conversão explícita para sRGB necessária",
        "Explicit Conversion to sRGB Required",
    ),
    ("Removida", "Removed"),
    ("Outra cadeia", "Another Chain"),
    ("Desfazer vínculo", "Unlink Relationship"),
    (
        "A relação inteira será removida; todas as cores serão preservadas.",
        "The entire relationship will be removed; all colors will be preserved.",
    ),
    ("Contraste", "Contrast"),
    ("Contraste atual", "Current Contrast"),
    ("Canais RGB / HSV", "RGB / HSV Channels"),
    ("Vínculo limitante", "Limiting Link"),
    ("Cores participantes", "Participating Colors"),
    (
        "Selecione duas ou mais cores para compartilhar os canais escolhidos.",
        "Select two or more colors to share the chosen channels.",
    ),
    ("Vermelho", "Red"),
    ("Verde", "Green"),
    ("Azul", "Blue"),
    ("Matiz", "Hue"),
    ("Saturação", "Saturation"),
    ("Valor / brilho", "Value / brightness"),
    (
        "Qualquer participante pode ser editado. Os canais selecionados preservam as proporções atuais; a matiz preserva a distância no círculo.",
        "Any participant can be edited. Selected channels preserve their current ratios; hue preserves the distance on the color wheel.",
    ),
    ("Tipo de limite", "Limit Type"),
    (
        "Impõe uma razão mínima de contraste entre duas cores.",
        "Enforces a minimum contrast ratio between two colors.",
    ),
    ("Primeira cor", "First Color"),
    ("Segunda cor", "Second Color"),
    ("Borracha por traço", "Stroke Eraser"),
    (
        "Borracha por traço · apaga traços completos da máscara selecionada",
        "Stroke Eraser · erases entire strokes from the selected mask",
    ),
    (
        "Passe sobre um traço para apagá-lo inteiro. Somente a máscara selecionada é afetada.",
        "Pass over a stroke to erase it entirely. Only the selected mask is affected.",
    ),
    ("Mover para grupo", "Move to Group"),
    ("Painéis", "Panels"),
    ("Redimensionar painéis", "Resize Panels"),
    ("Redimensionar painel flutuante", "Resize Floating Panel"),
    ("Papéis semânticos", "Semantic Roles"),
    ("Novo papel", "New Role"),
    ("Excluir papel", "Delete Role"),
    (
        "Associe funções às cores. Cada modo pode usar uma cor diferente.",
        "Assign roles to colors. Each mode can use a different color.",
    ),
    ("Sem modo · padrão", "No Mode · Default"),
    ("Sem associação", "Unassigned"),
    ("Remover associação", "Remove Association"),
    (
        "Arraste uma cor da paleta ou escolha na lista",
        "Drag one color from the palette or choose from the list",
    ),
    (
        "Crie um papel, como texto.principal, e arraste uma cor até sua associação.",
        "Create a role, such as text.primary, then drag a color to its binding.",
    ),
    ("Adicionar modo…", "Add Mode…"),
    ("Remover modo deste papel", "Remove Mode from This Role"),
    ("Nome do novo modo", "New Mode Name"),
    ("Criar modo", "Create Mode"),
    ("Modos da paleta", "Palette Modes"),
    (
        "Crie os modos aqui e associe suas cores no painel Papéis semânticos.",
        "Create modes here and assign their colors in the Semantic Roles panel.",
    ),
    (
        "Excluir modo e suas associações",
        "Delete Mode and Its Bindings",
    ),
    ("Abrir papéis semânticos", "Open Semantic Roles"),
    ("Relações documentais…", "Documentary Relationships…"),
    (
        "Personalize sua interface e o salvamento das paletas.",
        "Customize your interface and palette saving.",
    ),
    ("Arquivos e salvamento", "Files and Saving"),
    ("Tema", "Theme"),
    ("Salvamento automático", "Automatic Saving"),
    ("Disposição dos painéis", "Panel Layout"),
    ("Trocar workspace", "Switch Workspace"),
    ("Adicionar fonte", "Add Source"),
    ("Opções da fonte", "Source Options"),
    ("Remover tag", "Remove Tag"),
    ("Adicionar tag…", "Add Tag…"),
    (
        "Escolha o tipo de relação e confira as cores participantes.",
        "Choose a relationship type and review the participating colors.",
    ),
    ("Vincular cores", "Link Colors"),
    ("VÍNCULO RGB / HSV", "RGB / HSV LINK"),
    ("LIMITE DE CONTRASTE", "CONTRAST LIMIT"),
    ("GRADIENTE DINÂMICO", "DYNAMIC GRADIENT"),
    (
        "WCAG 2.2 · cores opacas sRGB · limite duro",
        "WCAG 2.2 · opaque sRGB colors · hard limit",
    ),
    ("VALOR sRGB", "sRGB VALUE"),
    ("PRÉVIA sRGB", "sRGB PREVIEW"),
    ("Monocromática", "Monochromatic"),
    ("Composta", "Compound"),
    ("Sombras", "Shades"),
    ("Tags", "Tags"),
    (
        "Remover somente esta relação, preservando as cores",
        "Remove only this relationship, keeping the colors",
    ),
    ("Escala da interface", "Interface Scale"),
    ("Interface", "Interface"),
    ("Vinculada", "Linked"),
    ("Incorporada", "Embedded"),
    ("Home", "Home"),
    ("Arquivo", "File"),
    ("Editar", "Edit"),
    ("Paleta", "Palette"),
    ("Janela", "Window"),
    ("Ajuda", "Help"),
    ("Preferências", "Preferences"),
    ("Idioma da interface", "Interface language"),
    ("Português (Brasil)", "Portuguese (Brazil)"),
    ("Inglês", "English"),
    ("Concluir", "Done"),
    ("Cancelar", "Cancel"),
    ("Aplicar", "Apply"),
    ("Salvar", "Save"),
    ("Salvar como…", "Save As…"),
    ("Abrir", "Open"),
    ("Novo", "New"),
    ("Nova paleta", "New Palette"),
    ("Criar paleta", "Create Palette"),
    ("Abrir paleta", "Open Palette"),
    ("Exportar", "Export"),
    ("Exportar paleta", "Export Palette"),
    (
        "Salvar cópia como…     Ctrl+Shift+S",
        "Save Copy As…     Ctrl+Shift+S",
    ),
    ("Salvar cópia do projeto", "Save Project Copy"),
    ("Salvar alterações?", "Save changes?"),
    ("Descartar alterações", "Discard changes"),
    ("Manter alterações", "Keep changes"),
    ("Fechar", "Close"),
    ("Fechar paleta", "Close Palette"),
    ("Fechar ColorCraft", "Close ColorCraft"),
    ("Minimizar janela", "Minimize window"),
    ("Maximizar janela", "Maximize window"),
    ("Paleta Principal", "Main Palette"),
    ("Seu laboratório de cores.", "Your color laboratory."),
    (
        "Crie paletas que guardam valores, relações e significado.",
        "Create palettes that preserve values, relationships, and meaning.",
    ),
    (
        "Uma página em branco para suas próximas combinações.",
        "A blank canvas for your next combinations.",
    ),
    (
        "Suas paletas salvas aparecerão aqui automaticamente.",
        "Your saved palettes will appear here automatically.",
    ),
    (
        "Rascunhos de recuperação disponíveis",
        "Recovery drafts available",
    ),
    ("Recuperar uma cópia", "Recover a Copy"),
    (
        "Uma cor, muitas possibilidades.",
        "One color, many possibilities.",
    ),
    (
        "Selecione uma cor da paleta para editar.",
        "Select a palette color to edit.",
    ),
    (
        "Selecione uma cor; os detalhes da pasta estão no painel Detalhes.",
        "Select a color; folder details are in the Details panel.",
    ),
    (
        "Selecione cores e crie vínculos ou composições para começar.",
        "Select colors and create links or compositions to get started.",
    ),
    ("Criar nova cor", "Create Color"),
    ("+ Cor", "+ Color"),
    ("+ Pasta", "+ Folder"),
    ("Adicionar cor", "Add Color"),
    ("Adicionar pasta", "Add Folder"),
    (
        "Buscar cores, pastas ou tags",
        "Search colors, folders, or tags",
    ),
    ("Visualização em cards", "Card View"),
    ("Visualização em árvore", "Tree View"),
    ("Cards", "Cards"),
    ("Árvore", "Tree"),
    ("Mínimo", "Minimal"),
    ("Compacto", "Compact"),
    ("Resumo", "Summary"),
    ("Detalhado", "Detailed"),
    ("Completo", "Full"),
    (
        "Mínimo · só amostras (~5 mm)",
        "Minimal · swatches only (~5 mm)",
    ),
    ("Compacto · só amostras", "Compact · swatches only"),
    ("Zoom", "Zoom"),
    ("Limpar seleção", "Clear Selection"),
    ("Desmarcar", "Deselect"),
    ("Mover seleção", "Move Selection"),
    ("Mover…", "Move…"),
    ("Duplicar", "Duplicate"),
    ("Excluir…", "Delete…"),
    ("Editar detalhes", "Edit Details"),
    ("Abrir pasta", "Open Folder"),
    ("Copiar HEX da prévia sRGB", "Copy sRGB Preview HEX"),
    (
        "Copiar HEX da representação sRGB",
        "Copy sRGB Representation HEX",
    ),
    (
        "Copiar representação da prévia sRGB",
        "Copy sRGB Preview Representation",
    ),
    (
        "Bloquear edição direta · permitir alterações pelas relações",
        "Lock direct editing · allow changes through relationships",
    ),
    (
        "Liberar edição direta · relações continuam ativas",
        "Unlock direct editing · relationships remain active",
    ),
    ("Bloquear apenas edição direta", "Lock direct editing only"),
    ("Liberar edição direta", "Unlock direct editing"),
    ("Bloqueada", "Locked"),
    ("Desbloqueada", "Unlocked"),
    ("Com relações técnicas", "Has technical relationships"),
    ("Criar harmonia ou gradiente", "Create Harmony or Gradient"),
    (
        "Criar vínculo, limite ou relação documental",
        "Create Link, Limit, or Documentary Relationship",
    ),
    ("Criar vínculo / limite…", "Create Link / Limit…"),
    ("Criar vínculo ou limite…", "Create Link or Limit…"),
    ("Criar relação", "Create Relationship"),
    ("Relacionar cores", "Relate Colors"),
    ("Vínculos", "Links"),
    ("Relações", "Relationships"),
    ("Relações da paleta", "Palette Relationships"),
    ("Relação", "Relationship"),
    ("Relação documental", "Documentary Relationship"),
    (
        "Relação documental · não restringe valores",
        "Documentary relationship · does not constrain values",
    ),
    ("Vínculo paramétrico", "Parametric Link"),
    ("Limite de contraste", "Contrast Limit"),
    ("Contraste mínimo", "Minimum Contrast"),
    ("Abertura °", "Angle °"),
    ("Interpolação", "Interpolation"),
    ("Explorar composições", "Explore Compositions"),
    ("Aplicar composição", "Apply Composition"),
    ("Cancelar composição", "Cancel Composition"),
    ("Voltar à composição", "Back to Composition"),
    ("PRÉVIA", "PREVIEW"),
    ("RECEITA", "RECIPE"),
    ("Original", "Original"),
    ("Nova", "New"),
    ("Substitui", "Replaces"),
    ("Complementar", "Complementary"),
    ("Complementar dividida", "Split Complementary"),
    ("Análoga", "Analogous"),
    ("Tríade", "Triad"),
    ("Quadrado", "Square"),
    ("Geometria", "Geometry"),
    ("Tétrade retangular", "Rectangular Tetrad"),
    ("Gradiente", "Gradient"),
    ("Cores intermediárias", "Intermediate Colors"),
    ("Edição de cor", "Color Editing"),
    ("Cor", "Color"),
    ("Nome", "Name"),
    ("Descrição", "Description"),
    ("Descrição de uso", "Usage Description"),
    ("Tags · separadas por vírgula", "Tags · comma-separated"),
    ("Editar título da paleta", "Edit Palette Title"),
    ("Editar nome e descrição…", "Edit Name and Description…"),
    ("Detalhes", "Details"),
    ("ORGANIZAÇÃO E SIGNIFICADO", "ORGANIZATION AND MEANING"),
    ("PAPÉIS ASSOCIADOS", "ASSOCIATED ROLES"),
    ("PAPÉIS SEMÂNTICOS", "SEMANTIC ROLES"),
    (
        "Papéis, modos e documentação…",
        "Roles, Modes, and Documentation…",
    ),
    (
        "Papéis, modos e relações documentais",
        "Roles, Modes, and Documentary Relationships",
    ),
    (
        "Papéis, modos e relações documentais…",
        "Roles, Modes, and Documentary Relationships…",
    ),
    ("Herdar padrão", "Inherit Default"),
    ("Adicionar papel…", "Add Role…"),
    ("Adicionar modo…", "Add Mode…"),
    (
        "Vínculos técnicos controlam valores. Relações documentais registram significado.",
        "Technical links control values. Documentary relationships record meaning.",
    ),
    ("Painéis", "Panels"),
    ("Desencaixar painel", "Detach Panel"),
    ("Restaurar disposição dos painéis", "Restore Panel Layout"),
    (
        "Redefinir disposição deste workspace",
        "Reset This Workspace Layout",
    ),
    (
        "Criar a partir da disposição atual",
        "Create from Current Layout",
    ),
    ("Novo espaço de trabalho…", "New Workspace…"),
    ("Gerenciar espaços de trabalho…", "Manage Workspaces…"),
    (
        "Salvar a disposição atual como um novo workspace",
        "Save Current Layout as a New Workspace",
    ),
    ("Workspace", "Workspace"),
    ("Workspaces", "Workspaces"),
    ("Essentials", "Essentials"),
    ("Essentials Classic", "Essentials Classic"),
    ("Automação", "Automation"),
    ("Layout", "Layout"),
    ("Pintura", "Painting"),
    ("Impressão e prova", "Printing and Proofing"),
    ("Traçado", "Tracing"),
    ("Tipografia", "Typography"),
    ("Web", "Web"),
    ("Fontes", "Sources"),
    ("Adicionar fonte", "Add Source"),
    ("+ Adicionar", "+ Add"),
    (
        "Imagens, PDF e SVG · cores capturadas são adicionadas à raiz da paleta.",
        "Images, PDF, and SVG · captured colors are added to the palette root.",
    ),
    (
        "Adicione um arquivo para começar a capturar cores.",
        "Add a file to start capturing colors.",
    ),
    ("Capturar cores da fonte", "Capture Colors from Source"),
    ("Capturas", "Samples"),
    ("+ Amostra", "+ Sample"),
    ("Excluir", "Delete"),
    ("Opções", "Options"),
    ("Página", "Page"),
    ("de", "of"),
    ("Conta-gotas", "Eyedropper"),
    ("Pincel de máscara", "Mask Brush"),
    (
        "Mão · arraste para deslocar a imagem",
        "Hand · drag to pan the image",
    ),
    (
        "Lupa · clique para ampliar; Alt ou botão direito para reduzir",
        "Magnifier · click to zoom in; Alt or right-click to zoom out",
    ),
    ("Ajustar", "Fit"),
    ("Método", "Method"),
    ("Pixel exato", "Exact Pixel"),
    ("Área próxima", "Nearby Area"),
    ("Máscara de pincel", "Brush Mask"),
    ("Círculo", "Circle"),
    ("Raio (px)", "Radius (px)"),
    ("Média linear sRGB", "Linear sRGB Mean"),
    ("Mediana OKLab", "OKLab Median"),
    ("Limpar máscara", "Clear Mask"),
    ("Nova máscara", "New Mask"),
    (
        "Cancelar para converter uma cópia",
        "Cancel to Convert a Copy",
    ),
    ("Usar cores da prévia PDFium", "Use PDFium Preview Colors"),
    (
        "Usar prévia PDFium e atualizar",
        "Use PDFium Preview and Update",
    ),
    (
        "Perfil ICC · captura dos pixels da prévia autorizada",
        "ICC Profile · sampling of the authorized preview pixels",
    ),
    ("Espaço de cor / perfil ICC…", "Color Space / ICC Profile…"),
    ("Espaço de cor e perfil", "Color Space and Profile"),
    (
        "Converter explicitamente para sRGB…",
        "Convert Explicitly to sRGB…",
    ),
    ("Confirmar conversão", "Confirm Conversion"),
    ("Confirmar atribuição", "Confirm Assignment"),
    (
        "Atribuir perfil aos mesmos números",
        "Assign Profile to the Same Numbers",
    ),
    (
        "Confira as diferenças desta saída",
        "Review the Differences in This Output",
    ),
    (
        "Entendi as conversões e perdas desta exportação",
        "I Understand the Conversions and Losses in This Export",
    ),
    (
        "Incluir adendo informativo das relações técnicas",
        "Include an Informational Appendix of Technical Relationships",
    ),
    ("Licenças e créditos", "Licenses and Credits"),
    ("Guia", "Guide"),
    ("Comece pelo ColorCraft", "Getting Started with ColorCraft"),
    ("Biblioteca local de paletas", "Local Palette Library"),
    ("Abrir pasta da biblioteca", "Open Library Folder"),
    ("Configuração:", "Configuration:"),
    (
        "Esta sessão usa preferências em memória.",
        "This session uses in-memory preferences.",
    ),
    ("Escuro", "Dark"),
    ("Claro", "Light"),
    ("Escala da interface", "Interface Scale"),
    (
        "Salvar automaticamente projetos e atualizar a paleta pública",
        "Automatically Save Projects and Update the Public Palette",
    ),
    (
        "Manter rascunhos de recuperação automática",
        "Keep Automatic Recovery Drafts",
    ),
    (
        "A cada 15 segundos, salva projetos já nomeados. Projetos novos ou importados aguardam Salvar/Salvar como.",
        "Saves named projects every 15 seconds. New or imported projects must be saved with Save/Save As first.",
    ),
    ("Alterações não salvas", "Unsaved Changes"),
    ("Alteração aplicada.", "Change applied."),
    ("Alterações pendentes", "Pending Changes"),
    (
        "Salvo e disponível na biblioteca local.",
        "Saved and available in the local library.",
    ),
    ("Salvo · publicação pendente", "Saved · publication pending"),
    (
        "Projeto e paleta pública salvos automaticamente.",
        "Project and public palette saved automatically.",
    ),
    (
        "Paleta pública publicada automaticamente.",
        "Public palette published automatically.",
    ),
    (
        "Rascunhos de recuperação são gravados separadamente. Eles não substituem o salvamento nem seus backups.",
        "Recovery drafts are saved separately. They do not replace saving or your backups.",
    ),
    (
        "Uma captura pontual é preservada ao começar uma máscara. Selecione uma máscara para continuar pintando nela.",
        "A point sample is preserved when you start a mask. Select a mask to continue painting on it.",
    ),
    (
        "Arraste para pintar uma máscara. Capturas pontuais são preservadas.",
        "Drag to paint a mask. Point samples are preserved.",
    ),
    (
        "Clique na imagem ou pinte uma máscara para começar.",
        "Click the image or paint a mask to get started.",
    ),
    (
        "Pinte a máscara antes de aplicar.",
        "Paint the mask before applying.",
    ),
    (
        "Pincel de máscara · pinte uma nova captura ou continue a máscara selecionada",
        "Mask brush · paint a new sample or continue the selected mask",
    ),
    (
        "Clique para ampliar. Alt + clique ou botão direito reduz. A roda do mouse também ajusta o zoom.",
        "Click to zoom in. Alt-click or right-click zooms out. The mouse wheel also adjusts zoom.",
    ),
    ("Claridade", "Lightness"),
    (
        "Valor V do HSV: preserva matiz e saturação. Os vínculos e limites continuam ativos.",
        "HSV Value (V): preserves hue and saturation. Links and limits remain active.",
    ),
    (
        "Fora do gamut sRGB · a prévia é uma aproximação",
        "Outside sRGB gamut · preview is an approximation",
    ),
    (
        "Fora do gamut: esta conversão limita canais e altera a aparência.",
        "Out of gamut: this conversion clips channels and changes appearance.",
    ),
    ("PRÉVIA sRGB", "sRGB PREVIEW"),
    (
        "DOCUMENTAL · NÃO RESTRINGE VALORES",
        "DOCUMENTARY · DOES NOT CONSTRAIN VALUES",
    ),
    ("Relações documentais", "Documentary Relationships"),
    (
        "RGB/S/V conservam a proporção atual, ou a diferença quando há zero. H conserva o deslocamento circular. Não é necessário vincular cada par manualmente.",
        "RGB/S/V preserve the current ratio, or the difference when a value is zero. H preserves the circular offset. You do not need to link every pair manually.",
    ),
    (
        "Todas usam os canais abaixo. A primeira cor define as proporções iniciais; qualquer participante pode ser editado depois.",
        "All participants use the channels below. The first color sets the initial ratios; any participant can be edited later.",
    ),
    ("Canais compartilhados", "Shared Channels"),
    ("Adicionar cor…", "Add Color…"),
    (
        "Retirar desta seleção de vínculo",
        "Remove from This Link Selection",
    ),
    (
        "WCAG 2.2 · cores opacas em sRGB. Uma cor pode variar até o limite; a outra não é ajustada para compensar.",
        "WCAG 2.2 · opaque sRGB colors. One color may change up to the limit; the other is not adjusted to compensate.",
    ),
    (
        "Estas cores precisam de conversão explícita para sRGB.",
        "These colors require explicit conversion to sRGB.",
    ),
    (
        "Confirmo converter mantendo os IDs. Conversão e vínculo serão desfeitos juntos.",
        "I confirm conversion while keeping the IDs. Conversion and linking can be undone together.",
    ),
    ("Aplicar documentação", "Apply Documentation"),
    ("Editar documentação…", "Edit Documentation…"),
    (
        "Aplicar aproximação e preservar desfazer",
        "Apply Approximation and Keep Undo History",
    ),
    ("Manter esta relação", "Keep This Relationship"),
    (
        "Remover somente esta relação, preservando as cores",
        "Remove Only This Relationship, Keeping the Colors",
    ),
    ("Relação não identificada", "Unknown Relationship"),
    ("Redefinir com os valores atuais", "Reset to Current Values"),
    (
        "Destacar esta cor do gradiente",
        "Detach This Color from the Gradient",
    ),
    ("Desfazer", "Undo"),
    ("Refazer", "Redo"),
    ("Copiar", "Copy"),
    ("Colar", "Paste"),
    ("Recortar", "Cut"),
    ("Selecionar tudo", "Select All"),
    ("Limpar", "Clear"),
    ("Sim", "Yes"),
    ("Não", "No"),
    ("Ok", "OK"),
    ("Nome CSS", "CSS Name"),
    ("Uso", "Usage"),
    ("Detalhes da cor", "Color Details"),
    ("Detalhes da pasta", "Folder Details"),
    ("Clique duas vezes para abrir", "Double-click to open"),
    ("ID:", "ID:"),
    ("Pasta", "Folder"),
    ("Nova pasta", "New Folder"),
    ("Raiz da paleta", "Palette Root"),
    ("Padrão", "Default"),
    ("Modo", "Mode"),
    ("Papéis", "Roles"),
    ("Aplicar modo", "Apply Mode"),
    ("Harmonia / gradiente", "Harmony / Gradient"),
    ("Vínculo / limite", "Link / Limit"),
    ("Relações técnicas", "Technical Relationships"),
    ("Vínculo RGB / HSV", "RGB / HSV Link"),
    ("Amostras", "Swatches"),
    ("Área", "Area"),
    ("Canal", "Channel"),
    ("Perfil", "Profile"),
    ("Aplicar perfil", "Apply Profile"),
    ("Atribuir", "Assign"),
    ("Converter", "Convert"),
    ("Revisar", "Review"),
    ("Arquivo de origem", "Source File"),
    ("Atualizar", "Update"),
    ("Relincar", "Relink"),
    ("Incorporar", "Embed"),
    ("Vinculada", "Linked"),
    ("Incorporada", "Embedded"),
    ("Originais", "Originals"),
    ("Nova cor", "New Color"),
    ("Bloqueada por captura", "Locked by Sample"),
    ("Selecionado", "Selected"),
    ("Sobre o ColorCraft", "About ColorCraft"),
    ("Licenças", "Licenses"),
    ("Atalhos", "Shortcuts"),
    ("Versão", "Version"),
    ("Arquivo .colorcraft", ".colorcraft File"),
    ("Paleta pública", "Public Palette"),
    ("Salvar projeto", "Save Project"),
    ("Abrir projeto", "Open Project"),
    ("Criar nova pasta", "Create Folder"),
    ("Editar cor", "Edit Color"),
    ("Cores", "Colors"),
    ("Pastas", "Folders"),
    ("Desfazer/refazer", "Undo/Redo"),
    ("Exibir", "View"),
    ("Selecionar", "Select"),
    ("Objeto", "Object"),
    ("Texto", "Text"),
    ("Efeito", "Effect"),
    ("Composição", "Composition"),
    ("Aplicar atualização", "Apply Update"),
    ("Recalcular", "Recalculate"),
    ("Preservar", "Preserve"),
    ("Bloquear", "Lock"),
    ("Desbloquear", "Unlock"),
    ("Selecionar cores", "Select Colors"),
    ("Lista", "List"),
    ("Visualizar", "View"),
    ("Tema", "Theme"),
    ("Gamut", "Gamut"),
    ("Saída", "Output"),
    ("Histórico", "History"),
    ("Documento", "Document"),
    ("Projeto", "Project"),
    ("Arquivo alterado", "File Changed"),
    ("Pendente", "Pending"),
    ("Rascunho", "Draft"),
    ("Alterações", "Changes"),
    ("Aparência", "Appearance"),
    ("Criar composições", "Create Compositions"),
    (
        "Escolha uma receita, ajuste seus parâmetros e confira as cores antes de aplicar.",
        "Choose a recipe, adjust its settings, and review the colors before applying.",
    ),
    (
        "Estas originais não cabem nesta receita. Selecione outras cores de origem para utilizá-la.",
        "These originals are not compatible with this recipe. Select different source colors to use it.",
    ),
    ("Sobre {}", "About {}"),
    ("Separação da tétrade", "Tetrad spacing"),
    ("Abertura", "Spread"),
    ("Quantidade", "Count"),
    ("ORIGINAIS PRESERVADAS", "PRESERVED ORIGINALS"),
    ("Capturas · {}", "Samples · {}"),
    ("Nova amostra", "New sample"),
    (
        "Criar captura independente, preservando a selecionada",
        "Create an independent sample and keep the current selection",
    ),
    ("Remover captura selecionada", "Remove selected sample"),
    (
        "Remover a captura preservando a cor na paleta?",
        "Remove the sample and keep its color in the palette?",
    ),
    (
        "Remover esta captura do rascunho?",
        "Remove this sample from the draft?",
    ),
    ("Manter captura", "Keep Sample"),
    ("Remover captura", "Remove Sample"),
    (
        "Pixel indicado pelo marcador.",
        "Pixel at the marker position.",
    ),
    (
        "Cor vinculada · será atualizada ao aplicar.",
        "Linked color · will update when applied.",
    ),
    (
        "Nova cor na raiz ao aplicar.",
        "A new color will be added to the root when applied.",
    ),
    (
        "Clique no marcador para selecionar; arraste para mover. Clique fora para criar.",
        "Click a marker to select it; drag to move it. Click elsewhere to add a sample.",
    ),
    (
        "Arraste a imagem para reposicionar. Ajustar centraliza novamente.",
        "Drag the image to reposition it. Fit recenters it.",
    ),
    (
        "Concepção, ícone e direção do produto: Eduardo.",
        "Product concept, icon, and direction: Eduardo.",
    ),
    (
        "Aplicativo independente, desenvolvido em Rust. Integração com os aplicativos Craft é uma direção de projeto; esta versão não é um produto oficial ArtCraft.",
        "Independent application built in Rust. Integration with Craft applications is a project direction; this version is not an official ArtCraft product.",
    ),
    (
        "Bibliotecas Craft utilizadas com seus créditos e licenças preservados.",
        "Craft libraries are used with their credits and licenses preserved.",
    ),
    (
        "Paletas salvas são publicadas automaticamente. A biblioteca é independente de preferências e caches.",
        "Saved palettes are published automatically. The library is independent of preferences and caches.",
    ),
    (
        "Cada workspace guarda a disposição dos painéis. As paletas e seus dados não são alterados.",
        "Each workspace stores a panel layout. Palettes and their data are not changed.",
    ),
    (
        "1. Crie uma paleta na Home ou importe o JSON do Colorator.",
        "1. Create a palette from Home or import Colorator JSON.",
    ),
    (
        "2. Organize cores e pastas em cards ou árvore. Ctrl seleciona várias cores; arraste para mover e ordenar.",
        "2. Organize colors and folders in cards or tree view. Ctrl selects multiple colors; drag to move and reorder.",
    ),
    (
        "3. Selecione uma cor para usar a roda e os canais. Um gesto é uma etapa de desfazer.",
        "3. Select a color to use the wheel and channels. Each gesture is one undo step.",
    ),
    (
        "4. Em Vínculos, mantenha razões RGB/HSV, crie harmonias, gradientes e contraste mínimo WCAG 2.2.",
        "4. In Relationships, maintain RGB/HSV ratios, create harmonies and gradients, and set minimum WCAG 2.2 contrast.",
    ),
    (
        "5. Gradientes controlam as intermediárias até você destacá-las. Bloqueios e limites nunca descartam as relações.",
        "5. Gradients control intermediate colors until you detach them. Locks and limits never discard relationships.",
    ),
    (
        "6. Em Detalhes, registre nomes, tags, campos próprios e uso. Papéis e modos associam significados às cores sem alterar valores.",
        "6. In Details, record names, tags, custom fields, and usage. Roles and modes associate meaning with colors without changing their values.",
    ),
    (
        "7. Perfis e valores nativos são preservados. Trocar espaço ou perfil exige confirmar a comparação; a roda usa sRGB.",
        "7. Profiles and native values are preserved. Changing a color space or profile requires reviewing and confirming the comparison; the wheel uses sRGB.",
    ),
    (
        "8. Salvar publica a paleta na biblioteca local. Exportar oferece um snapshot para consumo ou formatos de intercâmbio com perdas identificadas.",
        "8. Saving publishes the palette to the local library. Export creates a snapshot or an interchange format with identified losses.",
    ),
    (
        "9. Arraste os títulos dos painéis para reorganizar, agrupar ou desencaixar. Janela restaura o espaço padrão.",
        "9. Drag panel titles to rearrange, group, or detach them. Use Window to restore the default workspace.",
    ),
    (
        "Ctrl+N · nova  |  Ctrl+O · abrir  |  Ctrl+S · salvar  |  Ctrl+Shift+S · salvar cópia",
        "Ctrl+N · new  |  Ctrl+O · open  |  Ctrl+S · save  |  Ctrl+Shift+S · save a copy",
    ),
    (
        "Ctrl+Z / Ctrl+Shift+Z · desfazer/refazer  |  Ctrl+C/X/V · copiar/recortar/colar  |  Ctrl+D · duplicar",
        "Ctrl+Z / Ctrl+Shift+Z · undo/redo  |  Ctrl+C/X/V · copy/cut/paste  |  Ctrl+D · duplicate",
    ),
    ("Excluir itens?", "Delete Items?"),
    (
        "Você poderá desfazer esta operação.",
        "You can undo this operation.",
    ),
    ("Remover fonte?", "Remove Source?"),
    (
        "O nome identifica o arquivo no painel Fontes; o caminho vinculado não muda.",
        "This name identifies the file in the Sources panel; the linked path will not change.",
    ),
    ("Aplicar nome", "Apply Name"),
    ("Atualizar capturas da fonte?", "Update Source Samples?"),
    (
        "Atualizar usando as cores da prévia?",
        "Update Using Preview Colors?",
    ),
    (
        "Somente a confirmação abaixo recalcula as cores vinculadas. Você poderá desfazer toda a atualização.",
        "Only confirming below recalculates linked colors. You can undo the entire update.",
    ),
    (
        "Remover fonte e preservar cores",
        "Remove Source and Keep Colors",
    ),
    ("Atualizar e recalcular", "Update and Recalculate"),
    (
        "A cor não está mais disponível.",
        "This color is no longer available.",
    ),
    (
        "A cor solicitada está fora de sRGB",
        "The requested color is outside sRGB",
    ),
    (
        "É possível reduzir sua cromaticidade mantendo luminosidade e matiz. Confira a aproximação antes de aplicar.",
        "You can reduce its chroma while preserving lightness and hue. Review the approximation before applying.",
    ),
    ("Valor anterior", "Previous value"),
    ("Depois · prévia sRGB", "After · sRGB preview"),
    ("Cromaticidade ajustada", "Adjusted chroma"),
    ("Converter aparência", "Convert Appearance"),
    (
        "Atribuir muda a interpretação dos valores sem convertê-los. Só é permitido entre modelos compatíveis.",
        "Assigning changes how values are interpreted without converting them. This is only allowed between compatible models.",
    ),
    (
        "Converter recalcula os componentes para o destino: relativo colorimétrico com compensação de ponto preto.",
        "Convert recalculates the components for the destination using relative colorimetric rendering with black-point compensation.",
    ),
    ("MODOS", "MODES"),
    (
        "Sem substituição, o modo usa a associação padrão do papel.",
        "Without an override, the mode uses the role’s default association.",
    ),
    (
        "Nota sobre significado ou uso",
        "Note about meaning or usage",
    ),
    ("Nota", "Note"),
    ("＋ Adicionar", "＋ Add"),
    ("Adicionar", "Add"),
    ("Sobre ColorCraft", "About ColorCraft"),
    ("Guia do ColorCraft     F1", "ColorCraft Guide     F1"),
    ("Projetos recentes", "Recent Projects"),
    ("Biblioteca local", "Local Library"),
    (
        "Ícone de Pavão com Paleta Colorida.png",
        "Colorful Peacock Palette Icon.png",
    ),
    ("texto.principal", "primary.text"),
    ("Captura", "Sample"),
    (
        "Remover captura preservando a cor na paleta?",
        "Remove the sample and keep its color in the palette?",
    ),
    ("Abrir captura", "Open Capture"),
    ("Renomear fonte…", "Rename Source…"),
    ("Atualizar capturas…", "Update Samples…"),
    ("Relincar…", "Relink…"),
    ("Remover fonte…", "Remove Source…"),
    ("Apagar", "Delete"),
    ("Duplicar cor", "Duplicate Color"),
    ("Novo arquivo", "New File"),
    (
        "Paletas salvas aparecerão aqui automaticamente.",
        "Saved palettes will appear here automatically.",
    ),
    ("Nenhuma paleta encontrada.", "No palettes found."),
    ("Sem resultados.", "No results."),
    (
        "Não foi possível abrir o arquivo.",
        "Could not open the file.",
    ),
    ("Sair", "Quit"),
    ("Preferências…", "Preferences…"),
    (
        "Redefinir disposição deste workspace",
        "Reset This Workspace Layout",
    ),
    (
        "ArtCraft · projeto de referência ↗",
        "ArtCraft · reference project ↗",
    ),
    ("COLORCRAFT", "COLORCRAFT"),
    ("+  Nova paleta", "+  New Palette"),
    (
        "Escolha na paleta a cor que será substituída. Você pode abrir pastas. Escape retorna à composição.",
        "Choose the color to replace in the palette. You can open folders. Press Escape to return to the composition.",
    ),
    ("Fechar aviso", "Dismiss Notice"),
    ("Escolha como continuar", "Choose how to continue"),
    (
        "Cancelar para converter uma cópia",
        "Cancel to Convert a Copy",
    ),
    (
        "Uma captura pontual é preservada ao começar uma máscara. Selecione uma máscara para continuar pintando nela.",
        "A point sample is preserved when you start a mask. Select a mask to keep painting it.",
    ),
    (
        "Clique na imagem ou pinte uma máscara para começar.",
        "Click the image or paint a mask to get started.",
    ),
    (
        "Pinte a máscara antes de aplicar.",
        "Paint the mask before applying.",
    ),
    (
        "Pixel indicado pelo marcador.",
        "Pixel at the marker position.",
    ),
    ("Nome do workspace", "Workspace Name"),
    (
        "Criar a partir da disposição atual",
        "Create from Current Layout",
    ),
    ("Workspace selecionado", "Selected Workspace"),
    ("Renomear", "Rename"),
    ("Redefinir", "Reset"),
    (
        "Paletas com valores, relações e significado.",
        "Palettes with values, relationships, and meaning.",
    ),
    (
        "Código ColorCraft: MIT OU Apache-2.0.",
        "ColorCraft code: MIT OR Apache-2.0.",
    ),
    (
        "craft-ui, craft-cms, craft-settings e craft-atomic: equipe ArtCraft e contribuidores, MIT OU Apache-2.0.",
        "craft-ui, craft-cms, craft-settings, and craft-atomic: ArtCraft team and contributors, MIT OR Apache-2.0.",
    ),
    (
        "Interface: egui/eframe; diálogos nativos: rfd; imagens: image. As dependências mantêm seus próprios avisos e licenças, incluídos no pacote.",
        "Interface: egui/eframe; native dialogs: rfd; images: image. Dependencies retain their own notices and licenses, included in the package.",
    ),
    (
        "Ícone: fornecido por Eduardo para uso no ColorCraft. A licença geral de reutilização da imagem não foi declarada.",
        "Icon: provided by Eduardo for use in ColorCraft. No general reuse license for the image has been declared.",
    ),
    (
        "Matemática: especificações W3C CSS Color, WCAG 2.2, ICC e publicação original de OKLab. Exemplos e implementação de relações próprios, portados do Colorator.",
        "Color science: W3C CSS Color, WCAG 2.2, and ICC specifications, and the original OKLab publication. Relationship examples and implementation are original, ported from Colorator.",
    ),
    (
        "Salvar automaticamente projetos e atualizar a paleta pública",
        "Automatically Save Projects and Update the Public Palette",
    ),
    (
        "A cada 15 segundos, salva projetos já nomeados. Projetos novos ou importados aguardam Salvar/Salvar como.",
        "Saves named projects every 15 seconds. New or imported projects must be saved with Save/Save As first.",
    ),
    (
        "Manter rascunhos de recuperação automática",
        "Keep Automatic Recovery Drafts",
    ),
    (
        "Esta sessão usa preferências em memória.",
        "This session uses in-memory preferences.",
    ),
    ("Detalhes do projeto", "Project Details"),
    (
        "Você poderá desfazer esta operação.",
        "You can undo this operation.",
    ),
    ("Excluir itens", "Delete Items"),
    (
        "Remover fonte e preservar cores",
        "Remove Source and Keep Colors",
    ),
    ("Renomear fonte", "Rename Source"),
    (
        "O nome identifica o arquivo no painel Fontes; o caminho vinculado não muda.",
        "This name identifies the file in the Sources panel; the linked path will not change.",
    ),
    (
        "Atualizar usando as cores da prévia?",
        "Update Using Preview Colors?",
    ),
    (
        "Somente a confirmação abaixo recalcula as cores vinculadas. Você poderá desfazer toda a atualização.",
        "Only confirming below recalculates linked colors. You can undo the entire update.",
    ),
    (
        "Usar prévia PDFium e atualizar",
        "Use PDFium Preview and Update",
    ),
    ("Desvincular cor da fonte?", "Unlink Color from Source?"),
    ("Desvincular e desbloquear", "Unlink and Unlock"),
    ("Mover para pasta", "Move to Folder"),
    ("Mover", "Move"),
    (
        "Preserva cores nativas, perfis, pastas, IDs, metadados, relações documentais, papéis e modos.",
        "Preserves native colors, profiles, folders, IDs, metadata, documentary relationships, roles, and modes.",
    ),
    (
        "O consumidor usa os valores resolvidos. As regras de edição ficam no projeto .colorcraft.",
        "The consuming application uses the resolved values. Editing rules remain in the .colorcraft project.",
    ),
    (
        "Incluir adendo informativo das relações técnicas",
        "Include an Informational Appendix of Technical Relationships",
    ),
    (
        "Entendi as conversões e perdas desta exportação",
        "I Understand the Conversions and Losses in This Export",
    ),
    ("Carregar outro perfil ICC…", "Load Another ICC Profile…"),
    (
        "Atribuir perfil aos mesmos números",
        "Assign Profile to the Same Values",
    ),
    (
        "O destino é preservado nativamente; sua prévia pode exceder o gamut sRGB.",
        "The destination is preserved natively; its preview may exceed the sRGB gamut.",
    ),
    (
        "Confirmo limitar os canais ao espaço destino; o projeto poderá ser desfeito para recuperar a origem",
        "I confirm clipping channels to the destination space; undo can restore the original values.",
    ),
    (
        "A cor solicitada está fora de sRGB",
        "The requested color is outside sRGB",
    ),
    (
        "É possível reduzir sua cromaticidade mantendo luminosidade e matiz. Confira a aproximação antes de aplicar.",
        "You can reduce chroma while preserving lightness and hue. Review the approximation before applying.",
    ),
    (
        "Aplicar aproximação e preservar desfazer",
        "Apply Approximation and Preserve Undo",
    ),
    (
        "Papéis, modos e relações documentais",
        "Roles, Modes, and Documentary Relationships",
    ),
    (
        "Esta camada descreve significado e uso. Mudar uma associação não modifica os valores das cores.",
        "This layer describes meaning and usage. Changing an association does not change color values.",
    ),
    ("+ Papel", "+ Role"),
    (
        "Sem substituição, o modo usa a associação padrão do papel.",
        "Without an override, a mode uses the role’s default association.",
    ),
    ("Remover modo", "Remove Mode"),
    ("+ Modo", "+ Mode"),
    ("RELAÇÕES DOCUMENTAIS", "DOCUMENTARY RELATIONSHIPS"),
    (
        "Nota sobre significado ou uso",
        "Note about meaning or usage",
    ),
    ("+ Relação documental", "+ Documentary Relationship"),
    ("Arquivo", "File"),
    ("Nova paleta     Ctrl+N", "New Palette     Ctrl+N"),
    ("Abrir / importar…     Ctrl+O", "Open / Import…     Ctrl+O"),
    ("Salvar     Ctrl+S", "Save     Ctrl+S"),
    (
        "Salvar cópia como…     Ctrl+Shift+S",
        "Save Copy As…     Ctrl+Shift+S",
    ),
    ("Exportar…", "Export…"),
    ("Detalhes do projeto…", "Project Details…"),
    ("Fechar paleta     Ctrl+W", "Close Palette     Ctrl+W"),
    ("Desfazer     Ctrl+Z", "Undo     Ctrl+Z"),
    ("Refazer     Ctrl+Shift+Z", "Redo     Ctrl+Shift+Z"),
    ("Copiar     Ctrl+C", "Copy     Ctrl+C"),
    ("Recortar     Ctrl+X", "Cut     Ctrl+X"),
    ("Colar     Ctrl+V", "Paste     Ctrl+V"),
    ("Duplicar     Ctrl+D", "Duplicate     Ctrl+D"),
    ("Excluir…     Delete", "Delete…     Delete"),
    ("Nova pasta", "New Folder"),
    (
        "Explorar harmonias / gradiente…",
        "Explore Harmonies / Gradient…",
    ),
    ("Criar vínculo ou limite…", "Create Link or Limit…"),
    ("Carregar perfil ICC…", "Load ICC Profile…"),
    ("Salvar alterações?", "Save Changes?"),
    ("Novo projeto", "New Project"),
    ("Aparência", "Appearance"),
    ("Converter aparência", "Convert Appearance"),
    ("Confirmar atribuição", "Confirm Assignment"),
    ("Confirmar conversão", "Confirm Conversion"),
    ("Aplicar capturas", "Apply Samples"),
    ("Alterações pendentes", "Pending Changes"),
    ("Rascunho de captura", "Capture Draft"),
    ("Página", "Page"),
    (
        "O arquivo {} será renderizado novamente. As cores vinculadas serão recalculadas pela posição e método salvos; nada muda até confirmar.",
        "File {} will be rendered again. Linked colors will be recalculated using their saved positions and methods; nothing changes until you confirm.",
    ),
    (
        "A fonte e suas capturas serão removidas. {linked_colors} cor(es) permanecerão na raiz e serão desvinculadas.",
        "The source and its samples will be removed. {linked_colors} color(s) will remain at the root and be unlinked.",
    ),
    (
        "“{name}” está bloqueada porque acompanha uma captura. O valor atual será preservado e a cor ficará livre para edição.",
        "“{name}” is locked because it is linked to a sample. The current value will be preserved and the color will become editable.",
    ),
    (
        "“{name}” tem alterações não salvas.",
        "“{name}” has unsaved changes.",
    ),
    (
        "{} itens, incluindo conteúdo das pastas selecionadas. Relações que dependem desses itens também serão removidas.",
        "{} items, including the contents of selected folders. Relationships that depend on these items will also be removed.",
    ),
    ("Contraste atual: {:.4}:1", "Current contrast: {:.4}:1"),
    (
        "Atual {:.3}:1 · mínimo {:.3}:1",
        "Current {:.3}:1 · minimum {:.3}:1",
    ),
    ("{} cores participantes", "{} participating colors"),
    (
        "Versão {} · Windows standalone",
        "Version {} · standalone for Windows",
    ),
    ("Desfazer {}     Ctrl+Z", "Undo {}     Ctrl+Z"),
    ("Redefinir {}", "Reset {}"),
    ("Exportado: {}", "Exported: {}"),
    (
        "Projeto salvo. Publicação local pendente: {}",
        "Project saved. Local publishing pending: {}",
    ),
    (
        "Falha ao salvar preferências: {}",
        "Could not save preferences: {}",
    ),
    (
        "Não foi possível colar a lista de cores: {}",
        "Could not paste the color list: {}",
    ),
    (
        "Informe um nome para o workspace (limite de 64 espaços).",
        "Enter a workspace name (up to 64 workspaces).",
    ),
    (
        "Conteúdo da área de transferência maior que 32 MiB.",
        "Clipboard content exceeds 32 MiB.",
    ),
    (
        "Importado com regras e metadados preservados. Salve como projeto ColorCraft.",
        "Imported with rules and metadata preserved. Save as a ColorCraft project.",
    ),
    ("Projeto aberto.", "Project opened."),
    (
        "A biblioteca local não está disponível. Confira o diretório de dados.",
        "The local library is unavailable. Check the data directory.",
    ),
    (
        "Salvo e disponível na biblioteca local.",
        "Saved and available in the local library.",
    ),
    ("Salvo · publicação pendente", "Saved · publishing pending"),
    (
        "Projeto e paleta pública salvos automaticamente.",
        "Project and public palette saved automatically.",
    ),
    (
        "Paleta pública publicada automaticamente.",
        "Public palette published automatically.",
    ),
    ("Alteração aplicada.", "Change applied."),
    ("Exportar paleta", "Export Palette"),
    ("Abrir ou importar paleta", "Open or Import Palette"),
    ("Paletas e projetos", "Palettes and Projects"),
    (
        "Recorte preparado. Cole no mesmo projeto para mover os itens.",
        "Cut prepared. Paste into the same project to move the items.",
    ),
    (
        "Cores, pastas e relações internas copiadas.",
        "Colors, folders, and internal relationships copied.",
    ),
    (
        "Recortar entre projetos não é permitido; use Copiar.",
        "Cutting between projects is not allowed; use Copy.",
    ),
    ("Paleta", "Palette"),
    ("Tema", "Theme"),
    ("Claro", "Light"),
    ("Escuro", "Dark"),
    ("Lightness", "Lightness"),
    ("Explorar exemplo", "Explore Example"),
    ("Abrir / importar…", "Open / Import…"),
    ("Sobre este ColorCraft", "About ColorCraft"),
    ("Escolha um workspace", "Choose a workspace"),
    ("Arquivo", "File"),
    ("Nome do workspace", "Workspace Name"),
    ("Nova paleta", "New Palette"),
    ("ColorCraft", "ColorCraft"),
    ("Paleta ColorCraft", "ColorCraft Palette"),
    ("Projeto editável ColorCraft", "Editable ColorCraft Project"),
    ("CSS · variáveis", "CSS · variables"),
    ("JSON · paleta resolvida", "JSON · resolved palette"),
    (
        "Design Tokens · DTCG 2025.10",
        "Design Tokens · DTCG 2025.10",
    ),
    ("GIMP Palette · GPL", "GIMP Palette · GPL"),
    ("CSV · tabela de cores", "CSV · color table"),
    ("SVG · folha de amostras", "SVG · swatch sheet"),
    ("Simple Color Palette", "Simple Color Palette"),
    ("Complementar", "Complementary"),
    ("Complementar dividida", "Split Complementary"),
    ("Análoga", "Analogous"),
    ("Tríade", "Triad"),
    ("Quadrado", "Square"),
    ("Tétrade retangular", "Rectangular Tetrad"),
    ("Gradiente", "Gradient"),
    (
        "Duas cores opostas na roda. Contraste direto e equilíbrio.",
        "Two colors opposite each other on the wheel. Direct contrast and balance.",
    ),
    (
        "A base e dois matizes próximos ao seu oposto.",
        "A base color and two hues near its opposite.",
    ),
    (
        "Matizes vizinhos, com abertura angular ajustável.",
        "Neighboring hues with an adjustable angular spread.",
    ),
    (
        "Três matizes separados por 120°, com variedade equilibrada.",
        "Three hues spaced 120° apart for balanced variety.",
    ),
    (
        "Quatro matizes em intervalos de 90°.",
        "Four hues spaced at 90° intervals.",
    ),
    (
        "Distribui as cores igualmente pelos vértices de um polígono regular na roda de matizes.",
        "Distributes colors evenly across the vertices of a regular polygon on the hue wheel.",
    ),
    (
        "Dois pares complementares; ajuste a separação entre eles.",
        "Two complementary pairs; adjust the spacing between them.",
    ),
    (
        "Vizinhos da base combinados com cores próximas ao oposto.",
        "Neighbors of the base combined with colors near its opposite.",
    ),
    (
        "Um matiz e saturação constantes, com valores de brilho diferentes.",
        "One hue and saturation with varying brightness values.",
    ),
    (
        "Um matiz, com variações de saturação e brilho.",
        "One hue with variations in saturation and brightness.",
    ),
    (
        "Amostras intermediárias entre duas origens, no espaço escolhido.",
        "Intermediate samples between two source colors in the selected color space.",
    ),
    (
        "Comece em 30°. Reduza a abertura para aproximar as cores ou aumente até 60° para ampliar a variedade. A receita usa HSV; as cores originais são preservadas.",
        "Start at 30°. Reduce the spread to bring colors closer, or increase it up to 60° for more variety. This recipe uses HSV; original colors are preserved.",
    ),
    (
        "Comece em 45° ou 60°. A abertura separa os pares complementares. Em 90° a disposição equivale ao quadrado.",
        "Start at 45° or 60°. The spread separates the complementary pairs. At 90°, the arrangement is equivalent to a square.",
    ),
    (
        "Escolha o número de cores. A primeira fica na cor de origem; as demais ocupam vértices igualmente espaçados na roda, com separação de 360° dividida pelo total.",
        "Choose the number of colors. The first remains the source color; the others occupy evenly spaced vertices on the wheel, separated by 360° divided by the total.",
    ),
    (
        "OKLab distribui as etapas de forma perceptual; OKLCH interpola matiz e cromaticidade; sRGB interpola os canais. A receita reduz cromaticidade ao gamut quando necessário. As intermediárias permanecem dependentes até serem destacadas.",
        "OKLab spaces steps perceptually; OKLCH interpolates hue and chroma; sRGB interpolates channels. The recipe reduces chroma when needed to fit the gamut. Intermediate colors remain dependent until detached.",
    ),
    (
        "A quantidade controla quantas amostras serão geradas. Use o resultado para hierarquia visual; os papéis e significados podem ser documentados depois, sem criar novas restrições.",
        "The count controls how many samples are generated. Use them for visual hierarchy; roles and meanings can be documented later without adding constraints.",
    ),
    (
        "Escolha uma cor dominante e use as demais como apoio. A harmonia é uma relação HSV, não uma garantia de contraste de leitura; para isso, crie também um limite de contraste.",
        "Choose a dominant color and use the others for support. Harmony is an HSV relationship, not a guarantee of readable contrast; create a contrast limit for that.",
    ),
    ("Separação da tétrade", "Tetrad spacing"),
    ("Abertura", "Spread"),
    ("Quantidade", "Count"),
    ("Quantidade de cores", "Number of Colors"),
    ("Matiz atual", "Current hue"),
    (
        "Digite um valor, + ou − para ajustar, * ou / para multiplicar ou dividir, e % para reduzir ou aumentar proporcionalmente. H usa graus de 0 a 360 e aceita ° ou º.",
        "Enter a value, + or − to adjust, * or / to multiply or divide, and % to reduce or increase proportionally. H uses 0–360 degrees and accepts ° or º.",
    ),
    (
        "Use um número, +/− para somar ou subtrair, * ou / para multiplicar ou dividir, e % para uma fração do valor atual. H usa a escala de 0 a 360°.",
        "Use a number, +/− to add or subtract, * or / to multiply or divide, and % for a proportion of the current value. H uses the 0–360° scale.",
    ),
    ("Cores intermediárias", "Intermediate Colors"),
    ("RECEITA", "RECIPE"),
    ("PRÉVIA", "PREVIEW"),
    ("ORIGINAIS PRESERVADAS", "PRESERVED ORIGINALS"),
    (
        "Redução de cromaticidade ao gamut sRGB quando necessária; as originais permanecem intactas.",
        "Chroma is reduced to fit the sRGB gamut when necessary; original colors remain unchanged.",
    ),
    ("ORIGINAIS", "ORIGINALS"),
    ("Resumo", "Summary"),
    ("ORIGINAL", "ORIGINAL"),
    ("Nova", "New"),
    ("Substitui", "Replaces"),
    ("Criar uma nova cor", "Create a New Color"),
    (
        "Explorar harmonias / gradiente…",
        "Explore Harmonies / Gradient…",
    ),
    ("Criar vínculo / limite…", "Create Link / Limit…"),
    ("Ver relações técnicas…", "View Technical Relationships…"),
    ("Criar harmonia…", "Create Harmony…"),
    (
        "Uma cor, muitas possibilidades.",
        "One color, many possibilities.",
    ),
    (
        "Selecione uma cor da paleta para editar.",
        "Select a palette color to edit.",
    ),
    (
        "Intermediária dependente de gradiente. Destaque-a no painel Vínculos para editar livremente.",
        "Gradient-dependent intermediate color. Detach it in the Relationships panel to edit it freely.",
    ),
    (
        "Editar diretamente está bloqueado · acompanha as cores relacionadas",
        "Direct editing is locked · follows related colors",
    ),
    (
        "Edição direta bloqueada · acompanha as cores relacionadas",
        "Direct editing is locked · follows related colors",
    ),
    (
        "DOCUMENTAL · NÃO RESTRINGE VALORES",
        "DOCUMENTARY · DOES NOT RESTRICT VALUES",
    ),
    ("Cores que se movem juntas.", "Colors that move together."),
    (
        "Estas cores precisam de conversão explícita para sRGB.",
        "These colors require explicit conversion to sRGB.",
    ),
    (
        "Descreve uso ou significado, sem alterar os valores.",
        "Describes usage or meaning without changing values.",
    ),
    ("com", "with"),
    (
        "Copiar HEX da representação sRGB",
        "Copy sRGB Representation HEX",
    ),
    (
        "Copiar representação da prévia sRGB",
        "Copy sRGB Preview Representation",
    ),
    (
        "A roda usa sRGB. Os valores nativos abaixo permanecem preservados.",
        "The color wheel uses sRGB. Native values below are preserved.",
    ),
    ("Contraste mínimo", "Minimum Contrast"),
    ("Claridade", "Lightness"),
    (
        "Valor V do HSV: preserva matiz e saturação. Os vínculos e limites continuam ativos.",
        "HSV Value (V) preserves hue and saturation. Links and limits remain active.",
    ),
    ("COMPONENTES NATIVOS", "NATIVE COMPONENTS"),
    ("Editar documentação…", "Edit Documentation…"),
    (
        "Destacar esta cor do gradiente",
        "Detach This Color from the Gradient",
    ),
    ("CAMPOS PERSONALIZADOS", "CUSTOM FIELDS"),
    ("PAPÉIS ASSOCIADOS", "ASSOCIATED ROLES"),
    ("ORGANIZAÇÃO E SIGNIFICADO", "ORGANIZATION AND MEANING"),
    ("Tags · separadas por vírgula", "Tags · comma-separated"),
    ("Abrir captura", "Open Capture"),
    ("Renomear fonte…", "Rename Source…"),
    ("Atualizar capturas…", "Update Samples…"),
    ("Relincar…", "Relink…"),
    ("Remover fonte…", "Remove Source…"),
    (
        "O perfil ICC deste PDF não pôde ser interpretado.",
        "The ICC profile in this PDF could not be interpreted.",
    ),
    ("Escolha como continuar", "Choose how to continue"),
    ("Usar cores da prévia PDFium", "Use PDFium Preview Colors"),
    (
        "Cancelar para converter uma cópia",
        "Cancel to Convert a Copy",
    ),
    ("Perfil ausente", "Missing Profile"),
    ("Antes", "Before"),
    ("Depois · prévia sRGB", "After · sRGB Preview"),
    ("Novo valor: {}", "New value: {}"),
    ("Cor: {} · origem: {}", "Color: {} · source: {}"),
    ("Origem: {}", "Source: {}"),
    ("Destino: {}", "Destination: {}"),
    ("{}: {} para sRGB {}", "{}: {} to sRGB {}"),
    (
        "Importado com regras e metadados preservados. Salve como projeto ColorCraft.",
        "Imported with rules and metadata preserved. Save it as a ColorCraft project.",
    ),
    ("Paletas e projetos", "Palettes and Projects"),
    ("Nova paleta     Ctrl+N", "New Palette     Ctrl+N"),
    ("Abrir / importar…     Ctrl+O", "Open / Import…     Ctrl+O"),
    ("Salvar     Ctrl+S", "Save     Ctrl+S"),
    (
        "Salvar cópia como…     Ctrl+Shift+S",
        "Save Copy As…     Ctrl+Shift+S",
    ),
    ("Exportar…", "Export…"),
    ("Detalhes do projeto…", "Project Details…"),
    ("Fechar paleta     Ctrl+W", "Close Palette     Ctrl+W"),
    ("Desfazer     Ctrl+Z", "Undo     Ctrl+Z"),
    ("Refazer     Ctrl+Shift+Z", "Redo     Ctrl+Shift+Z"),
    ("Copiar     Ctrl+C", "Copy     Ctrl+C"),
    ("Recortar     Ctrl+X", "Cut     Ctrl+X"),
    ("Colar     Ctrl+V", "Paste     Ctrl+V"),
    ("Duplicar     Ctrl+D", "Duplicate     Ctrl+D"),
    ("Excluir…     Delete", "Delete…     Delete"),
    ("Carregar outro perfil ICC…", "Load Another ICC Profile…"),
    (
        "Atribuir muda a interpretação dos valores sem convertê-los. Só é permitido entre modelos compatíveis.",
        "Assign changes how values are interpreted without converting them. It is only allowed between compatible models.",
    ),
    (
        "Converter recalcula os componentes para o destino: relativo colorimétrico com compensação de ponto preto.",
        "Convert recalculates components for the destination using relative colorimetric rendering with black-point compensation.",
    ),
    (
        "A conversão gera valores fora do gamut destino: {}",
        "The conversion produces values outside the destination gamut: {}",
    ),
    ("Modo", "Mode"),
    ("Herdar padrão", "Inherit Default"),
    ("Cor ausente", "Missing Color"),
    ("Usada com", "Used with"),
    ("Nome", "Name"),
    ("Descrição", "Description"),
    ("Papel", "Role"),
    ("Relação", "Relationship"),
    ("Nota", "Note"),
    ("Aplicar documentação", "Apply Documentation"),
    ("Nova cor", "New Color"),
    ("Nova pasta", "New Folder"),
    ("Paleta", "Palette"),
    ("Cor", "Color"),
    ("Detalhes", "Details"),
    ("Links", "Links"),
    ("Relações", "Relationships"),
    ("Vínculos", "Links"),
    ("Raiz da paleta", "Palette Root"),
    ("Nova amostra", "New sample"),
    ("Nova máscara", "New mask"),
    ("Capturas · {}", "Samples · {}"),
    ("Captura {}", "Sample {}"),
    (
        "Remover a captura preservando a cor na paleta?",
        "Remove the sample and keep its color in the palette?",
    ),
    (
        "Remover esta captura do rascunho?",
        "Remove this sample from the draft?",
    ),
    ("Manter captura", "Keep Sample"),
    ("Remover captura", "Remove Sample"),
    ("Pixel exato", "Exact Pixel"),
    ("Área próxima", "Nearby Area"),
    ("Máscara de pincel", "Brush Mask"),
    ("Círculo", "Circle"),
    ("Raio (px)", "Radius (px)"),
    ("Média linear sRGB", "Linear sRGB Mean"),
    ("Mediana OKLab", "OKLab Median"),
    (
        "Clique no marcador para selecionar; arraste para mover. Clique fora para criar.",
        "Click a marker to select it; drag to move it. Click elsewhere to add a sample.",
    ),
    (
        "Arraste para pintar uma máscara. Capturas pontuais são preservadas.",
        "Drag to paint a mask. Point samples are preserved.",
    ),
    (
        "Arraste a imagem para reposicionar. Ajustar centraliza novamente.",
        "Drag the image to reposition it. Fit recenters it.",
    ),
    (
        "Clique para ampliar. Alt + clique ou botão direito reduz. A roda do mouse também ajusta o zoom.",
        "Click to zoom in. Alt-click or right-click to zoom out. The mouse wheel also adjusts zoom.",
    ),
    (
        "Criar captura independente, preservando a selecionada",
        "Create an independent sample and keep the current selection",
    ),
    (
        "Capturas pontuais são preservadas.",
        "Point samples are preserved.",
    ),
    ("Contraste atual: {:.4}:1", "Current contrast: {:.4}:1"),
    (
        "Atual {:.3}:1 · mínimo {:.3}:1",
        "Current {:.3}:1 · minimum {:.3}:1",
    ),
    ("{} selecionado(s)", "{} selected"),
    ("{} cores · {} pastas", "{} colors · {} folders"),
    ("{} cores", "{} colors"),
    ("{} pastas", "{} folders"),
    ("{} participantes", "{} participants"),
    ("{} cores participantes", "{} participating colors"),
    (
        "Versão {} · Windows standalone",
        "Version {} · Windows standalone",
    ),
    ("Sobre {}", "About {}"),
    ("de {}", "of {}"),
    ("Alterações pendentes", "Pending Changes"),
    ("Rascunho de captura", "Capture Draft"),
    ("Aplicar capturas", "Apply Samples"),
    ("Substituir…", "Replace…"),
    ("Aplicar nome", "Apply Name"),
    ("Escolher arquivo e exportar…", "Choose File and Export…"),
    ("Incorporar ao projeto", "Embed in Project"),
    (
        "Duplo clique para abrir a captura",
        "Double-click to open capture",
    ),
    ("Incorporada", "Embedded"),
    ("Vinculada", "Linked"),
    (
        "Limite atingido. As relações e os valores originais foram preservados.",
        "Limit reached. Relationships and original values were preserved.",
    ),
    ("Editar cor", "Edit Color"),
    (
        "Importado com regras e metadados preservados. Salve como projeto ColorCraft.",
        "Imported with rules and metadata preserved. Save it as a ColorCraft project.",
    ),
    ("Projeto aberto.", "Project opened."),
    (
        "Recorte preparado. Cole no mesmo projeto para mover os itens.",
        "Cut prepared. Paste into the same project to move the items.",
    ),
    (
        "Cores, pastas e relações internas copiadas.",
        "Colors, folders, and internal relationships copied.",
    ),
    (
        "Recortar entre projetos não é permitido; use Copiar.",
        "Cutting between projects is not allowed; use Copy.",
    ),
    ("Armazenamento indisponível: {}", "Storage unavailable: {}"),
    (
        "Biblioteca/recuperação indisponível: {}. Ainda é possível salvar projetos em outro destino.",
        "Library/recovery unavailable: {}. You can still save projects elsewhere.",
    ),
    (
        "Salvamento automático pendente: {}",
        "Automatic save pending: {}",
    ),
    (
        "Publicação automática pendente: {}",
        "Automatic publication pending: {}",
    ),
    (
        "Rascunho de recuperação pendente: {}",
        "Recovery draft pending: {}",
    ),
    (
        "Falha ao salvar preferências: {}",
        "Could not save preferences: {}",
    ),
    (
        "Projeto salvo. Publicação local pendente: {}",
        "Project saved. Local publishing pending: {}",
    ),
    ("Exportado: {}", "Exported: {}"),
    (
        "Não foi possível colar a lista de cores: {}",
        "Could not paste the color list: {}",
    ),
    (
        "Informe um nome para o workspace (limite de 64 espaços).",
        "Enter a workspace name (up to 64 workspaces).",
    ),
    (
        "Conteúdo da área de transferência maior que 32 MiB.",
        "Clipboard content exceeds 32 MiB.",
    ),
    (
        "Paleta local indisponível.",
        "Local palette library unavailable.",
    ),
    (
        "Limite atingido. As relações e valores originais foram preservados.",
        "Limit reached. Relationships and original values were preserved.",
    ),
    ("Edição de cor", "Color Editing"),
    ("Relações", "Relationships"),
    ("Essentials", "Essentials"),
    ("Essentials Classic", "Essentials Classic"),
    ("Escolha um workspace", "Choose a workspace"),
    ("Aplicar", "Apply"),
    ("Aplicar cópias", "Apply Copies"),
    ("Nova paleta", "New Palette"),
    ("Aparência", "Appearance"),
    ("Temas", "Themes"),
    ("Paleta Principal", "Main Palette"),
    (
        "Um nome, muitas possibilidades.",
        "One name, many possibilities.",
    ),
    (
        "Paletas com valores, relações e significado.",
        "Palettes with values, relationships, and meaning.",
    ),
    ("Inglês", "English"),
    ("Português (Brasil)", "Portuguese (Brazil)"),
    (
        "Mudar idioma reinicia algumas janelas.",
        "Changing the language updates open windows.",
    ),
    ("+ Criar uma cor", "+ Create Color"),
    ("Explorar harmonias…", "Explore Harmonies…"),
    (
        "Revisão {} · biblioteca local",
        "Revision {} · local library",
    ),
    ("Redefinir disposição dos painéis", "Reset Panel Layout"),
    (
        "Você poderá desfazer esta operação.",
        "You can undo this operation.",
    ),
    ("ID: {}", "ID: {}"),
    ("{} → {} · {}", "{} → {} · {}"),
    ("{} · padrão", "{} · default"),
    ("{} · {}", "{} · {}"),
    ("{} · {} · página {}", "{} · {} · page {}"),
    ("{} cores · {} pastas", "{} colors · {} folders"),
    ("{} cores", "{} colors"),
    ("{} pastas", "{} folders"),
    ("{} selecionado(s)", "{} selected"),
    (
        "O arquivo foi modificado. Atualize para recalcular as capturas.",
        "The file has changed. Update it to recalculate samples.",
    ),
    (
        "Cor vinculada · será atualizada ao aplicar.",
        "Linked color · will update when applied.",
    ),
    (
        "Nova cor na raiz ao aplicar.",
        "A new color will be added to the root when applied.",
    ),
    ("Perfil ICC", "ICC Profile"),
    ("Incorporar ao projeto", "Embed in Project"),
    ("Incorporação", "Embedding"),
    ("Idioma da interface", "Interface language"),
    ("Aparência", "Appearance"),
    ("Escuro", "Dark"),
    ("Claro", "Light"),
    ("Atualizações", "Updates"),
    ("Sem dados", "No data"),
    ("Origens", "Sources"),
    ("Relações", "Relationships"),
    ("Árvore", "Tree"),
    ("Cards", "Cards"),
    ("Lista", "List"),
    ("Home", "Home"),
    ("Preferências", "Preferences"),
    ("Arquivo", "File"),
    ("Editar", "Edit"),
    ("Janela", "Window"),
    ("Ajuda", "Help"),
];

fn translate_pattern(text: &str) -> Option<String> {
    const PATTERNS: &[(&str, &str)] = &[
        ("Criar tag: {}", "Create tag: {}"),
        ("Contraste atual: {:.4}:1", "Current contrast: {:.4}:1"),
        (
            "Contraste mínimo {:.2}:1: {}",
            "Minimum contrast {:.2}:1: {}",
        ),
        (
            "Atual {:.3}:1 · mínimo {:.3}:1",
            "Current {:.3}:1 · minimum {:.3}:1",
        ),
        (
            "Revisão {} · biblioteca local",
            "Revision {} · local library",
        ),
        (
            "{} originais · {} novas · {} substituições",
            "{} originals · {} new · {} replacements",
        ),
        (
            "{} itens, incluindo conteúdo das pastas selecionadas. Relações que dependem desses itens também serão removidas.",
            "{} items, including the contents of selected folders. Relationships that depend on these items will also be removed.",
        ),
        (
            "{} intermediárias · gamut: redução de cromaticidade",
            "{} intermediate colors · gamut: chroma reduction",
        ),
        ("Relações técnicas: {}", "Technical relationships: {}"),
        ("{} vínculo de canais", "{} channel link"),
        ("{} vínculos de canais", "{} channel links"),
        ("{} harmonia", "{} harmony"),
        ("{} harmonias", "{} harmonies"),
        ("{} gradiente", "{} gradient"),
        ("{} gradientes", "{} gradients"),
        ("{} limite de contraste", "{} contrast limit"),
        ("{} limites de contraste", "{} contrast limits"),
        (
            "Esta saída contém cores resolvidas; não guarda as regras técnicas de edição.",
            "This export contains resolved colors; it does not store technical editing rules.",
        ),
        (
            "Pastas, IDs, papéis, modos e metadados completos não são preservados neste formato.",
            "Folders, IDs, roles, modes, and full metadata are not preserved in this format.",
        ),
        (
            "GPL usa apenas sRGB de 8 bits. Valores serão convertidos quando necessário e arredondados; perfis ICC não acompanham o arquivo.",
            "GPL uses 8-bit sRGB only. Values are converted when needed and rounded; ICC profiles are not included.",
        ),
        (
            "Há cores nativas fora de sRGB. Esta saída usa conversão colorimétrica para sRGB; a origem permanece no projeto.",
            "Some native colors are outside sRGB. This export converts them colorimetrically to sRGB; the originals remain in the project.",
        ),
        (
            "Há cores fora do gamut sRGB. A prévia/saída sRGB limita os canais; confira o resultado antes de exportar.",
            "Some colors are outside the sRGB gamut. The sRGB preview/export clips channels; review the result before exporting.",
        ),
        (
            "Simple Color Palette escreve sRGB linear com precisão de 0,00001, sem as estruturas adicionais do ColorCraft.",
            "Simple Color Palette writes linear sRGB to 0.00001 precision without ColorCraft's additional structures.",
        ),
        ("{} · padrão", "{} · default"),
        ("{} · prévia sRGB", "{} · sRGB preview"),
        ("{} · {} · página {}", "{} · {} · page {}"),
        (
            "A captura {} não encontra pixels visíveis na nova fonte.",
            "Sample {} has no visible pixels in the new source.",
        ),
        (
            "Captura {} inválida: {error}",
            "Sample {} is invalid: {error}",
        ),
        (
            "Não foi possível recalcular {}: {e}",
            "Could not recalculate {}: {e}",
        ),
        (
            "Não foi possível validar a página da captura {}: {error}",
            "Could not validate the page for sample {}: {error}",
        ),
        ("{} selecionado(s)", "{} selected"),
        ("{} cores · {} pastas", "{} colors · {} folders"),
        ("{} cores", "{} colors"),
        ("{} pastas", "{} folders"),
        ("{} cores participantes", "{} participating colors"),
        ("{} cores · {} relações", "{} colors · {} relationships"),
        ("de {}", "of {}"),
        ("Sobre {}", "About {}"),
        ("Redefinir {}", "Reset {}"),
        ("Serão removidas: {}", "Will be removed: {}"),
        ("Será removido: {}", "Will be removed: {}"),
        ("Vínculo: {}", "Link: {}"),
        ("{}: {} para sRGB {}", "{}: {} to sRGB {}"),
        (
            "O arquivo {} será renderizado novamente. As cores vinculadas serão recalculadas pela posição e método salvos; nada muda até confirmar.",
            "File {} will be rendered again. Linked colors will be recalculated using their saved position and method; nothing changes until you confirm.",
        ),
        (
            "A fonte e suas capturas serão removidas. {linked_colors} cor(es) permanecerão na raiz e serão desvinculadas.",
            "The source and its samples will be removed. {linked_colors} color(s) will remain at the root and be unlinked.",
        ),
        (
            "“{name}” está bloqueada porque acompanha uma captura. O valor atual será preservado e a cor ficará livre para edição.",
            "“{name}” is locked because it is linked to a sample. Its current value will be preserved and the color will become editable.",
        ),
        (
            "“{name}” tem alterações não salvas.",
            "“{name}” has unsaved changes.",
        ),
        (
            "Salvamento automático pendente: {e}",
            "Automatic save pending: {e}",
        ),
        (
            "Projeto salvo automaticamente; publicação pendente: {e}",
            "Project saved automatically; publication pending: {e}",
        ),
        (
            "Publicação automática pendente: {e}",
            "Automatic publication pending: {e}",
        ),
        (
            "Rascunho de recuperação pendente: {e}",
            "Recovery draft pending: {e}",
        ),
        (
            "Biblioteca/recuperação indisponível: {e}. Ainda é possível salvar projetos em outro destino.",
            "Library/recovery unavailable: {e}. You can still save projects elsewhere.",
        ),
        (
            "Armazenamento indisponível: {e}",
            "Storage unavailable: {e}",
        ),
        ("Disposição de painéis: {error}", "Panel layout: {error}"),
        ("Painéis: {e}", "Panels: {e}"),
        ("Fonte indisponível: {e}", "Source unavailable: {e}"),
        (
            "Não foi possível abrir o PDF: {e}",
            "Could not open PDF: {e}",
        ),
        (
            "Não foi possível ler a fonte: {e}",
            "Could not read source: {e}",
        ),
        (
            "Não foi possível decodificar a imagem: {e}",
            "Could not decode image: {e}",
        ),
        (
            "Falha ao decodificar a prévia do SVG.",
            "Failed to decode the SVG preview.",
        ),
        (
            "Memória insuficiente para renderizar o SVG.",
            "Not enough memory to render the SVG.",
        ),
        (
            "SVG inválido ou não suportado: {e}",
            "Invalid or unsupported SVG: {e}",
        ),
        (
            "Formato não suportado. Use PNG, JPEG, WebP, TIFF, PDF ou SVG.",
            "Unsupported format. Use PNG, JPEG, WebP, TIFF, PDF, or SVG.",
        ),
        (
            "Não foi possível carregar PDFium ({local_error}; biblioteca do sistema: {system_error}). Reinstale o ColorCraft ou coloque pdfium.dll compatível ao lado do executável.",
            "Could not load PDFium ({local_error}; system library: {system_error}). Reinstall ColorCraft or place a compatible pdfium.dll next to the executable.",
        ),
        (
            "Não foi possível abrir ColorCraft: {error}",
            "Could not open ColorCraft: {error}",
        ),
        ("Opção desconhecida: {a}", "Unknown option: {a}"),
        (
            "Versão {} · Windows standalone",
            "Version {} · standalone for Windows",
        ),
    ];
    PATTERNS
        .iter()
        .find_map(|(pattern, translated)| replace_pattern(text, pattern, translated))
}

fn replace_pattern(text: &str, pattern: &str, translated: &str) -> Option<String> {
    let mut literals = Vec::new();
    let mut rest = pattern;
    while let Some(start) = rest.find('{') {
        let end = rest[start..].find('}')? + start;
        literals.push(&rest[..start]);
        rest = &rest[end + 1..];
    }
    literals.push(rest);
    if !text.starts_with(literals[0]) {
        return None;
    }
    let mut captures = vec![];
    let mut cursor = literals[0].len();
    for next in literals.iter().skip(1) {
        if next.is_empty() {
            captures.push(&text[cursor..]);
            cursor = text.len();
        } else {
            let end = text[cursor..].find(next)? + cursor;
            captures.push(&text[cursor..end]);
            cursor = end + next.len();
        }
    }
    if cursor != text.len() {
        return None;
    }
    let mut out = String::new();
    let mut rest = translated;
    let mut index = 0;
    while let Some(start) = rest.find('{') {
        let end = rest[start..].find('}')? + start;
        out.push_str(&rest[..start]);
        out.push_str(captures.get(index)?);
        index += 1;
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    (index == captures.len()).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_static_and_formatted_ui_text() {
        assert_eq!(translate("Arquivo", true), "File");
        assert_eq!(
            translate("Fonte indisponível: arquivo ausente", true),
            "Source unavailable: arquivo ausente"
        );
        assert_eq!(translate("de 3", true), "of 3");
        assert_eq!(translate("Detalhes da cor", true), "Color Details");
        assert_eq!(translate("Nome", true), "Name");
        assert_eq!(translate("Uso", true), "Usage");
        assert_eq!(
            translate("33 cores · 7 pastas", true),
            "33 colors · 7 folders"
        );
        assert_eq!(translate("1 selecionado(s)", true), "1 selected");
        assert_eq!(translate("3 vínculos de canais", true), "3 channel links");
        assert_eq!(translate("1 harmonia", true), "1 harmony");
        assert_eq!(
            translate("2 limites de contraste", true),
            "2 contrast limits"
        );
        assert_eq!(
            translate("Contraste atual: 4.5000:1", true),
            "Current contrast: 4.5000:1"
        );
        assert_eq!(translate("Redefinir Essentials", true), "Reset Essentials");
        assert_eq!(translate("Arquivo", false), "Arquivo");
    }
}
