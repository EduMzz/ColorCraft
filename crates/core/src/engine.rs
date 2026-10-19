use crate::diagnostics::{self, Cause, Diagnostic, Operation, Protection};
use crate::{
    color::{self, Rgb},
    solver, *,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Replacement {
    pub id: String,
    pub unlock: bool,
    pub detach: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CompositionDraft {
    pub kind: String,
    pub originals: Vec<String>,
    pub count: usize,
    pub spread: f64,
    pub space: String,
    pub replacements: BTreeMap<usize, Replacement>,
}
impl Default for CompositionDraft {
    fn default() -> Self {
        Self {
            kind: "analogous".into(),
            originals: vec![],
            count: 5,
            spread: 30.0,
            space: "oklab".into(),
            replacements: BTreeMap::new(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Slot {
    pub rgb: Rgb,
    pub original: Option<String>,
}
pub fn composition_preview(p: &Project, d: &CompositionDraft) -> Result<(Vec<Slot>, usize)> {
    let originals: Vec<_> = d
        .originals
        .iter()
        .filter_map(|id| p.item(id))
        .filter(|i| i.is_color())
        .collect();
    if originals.iter().any(|i| !i.is_srgb()) {
        return Err("Converta explicitamente as cores originais para sRGB antes de criar uma relação técnica.".into());
    }
    if d.kind == "gradient" {
        if originals.len() != 2 {
            return Err("Selecione exatamente duas cores para o gradiente.".into());
        }
        let a = originals[0];
        let b = originals[1];
        let n = d.count.clamp(1, 32);
        let mut slots = vec![Slot {
            rgb: a.rgb,
            original: Some(a.id.clone()),
        }];
        slots.extend((0..n).map(|i| Slot {
            rgb: color::interpolate(a.rgb, b.rgb, (i + 1) as f64 / (n + 1) as f64, &d.space),
            original: None,
        }));
        slots.push(Slot {
            rgb: b.rgb,
            original: Some(b.id.clone()),
        });
        return Ok((slots, 0));
    }
    if originals.is_empty() {
        return Err("Selecione uma ou duas cores de origem.".into());
    }
    for base in &originals {
        let mut slots: Vec<_> = color::harmony(base.rgb, &d.kind, d.spread, d.count)
            .into_iter()
            .map(|rgb| Slot {
                rgb,
                original: None,
            })
            .collect();
        let mut fits = true;
        for item in &originals {
            if let Some(slot) = slots.iter_mut().find(|slot| {
                slot.original.is_none()
                    && slot
                        .rgb
                        .iter()
                        .zip(item.rgb)
                        .all(|(a, b)| (*a - b).abs() < 0.05)
            }) {
                slot.original = Some(item.id.clone());
                slot.rgb = item.rgb
            } else {
                fits = false;
                break;
            }
        }
        if fits {
            let anchor = slots
                .iter()
                .position(|s| s.original.as_deref() == Some(base.id.as_str()))
                .unwrap_or(0);
            return Ok((slots, anchor));
        }
    }
    Err("As cores selecionadas não pertencem a esta harmonia com estes parâmetros.".into())
}
#[derive(Clone, Debug)]
pub enum Command {
    ReceiveColors {
        request_id: String,
        items: Vec<Item>,
        profiles: Vec<IccProfile>,
    },
    Batch(Vec<Command>),
    Add(Item),
    Metadata(Item),
    LinkColors {
        ids: Vec<String>,
        channels: Vec<usize>,
    },
    ProjectDetails {
        name: String,
        description: String,
    },
    Delete(Vec<String>),
    Duplicate(Vec<String>),
    Move {
        ids: Vec<String>,
        parent: Option<String>,
    },
    Reorder {
        id: String,
        before: String,
    },
    Paste {
        source: Project,
        ids: Vec<String>,
        parent: Option<String>,
    },
    SetColor {
        id: String,
        rgb: Rgb,
        continuous: bool,
    },
    SetNative {
        id: String,
        value: NativeColor,
    },
    Lock {
        id: String,
        locked: bool,
    },
    DirectLock {
        id: String,
        locked: bool,
    },
    AddLink {
        a: String,
        b: String,
        channels: Vec<usize>,
    },
    RedefineLink(String),
    RemoveRelation(String),
    Compose(CompositionDraft),
    HarmonySpread {
        id: String,
        spread: f64,
    },
    GradientSpace {
        id: String,
        space: String,
    },
    Detach {
        gradient: String,
        color: String,
    },
    AddContrast {
        a: String,
        b: String,
        minimum: f64,
    },
    AddProfile(IccProfile),
    Documentation {
        relations: Vec<Documentary>,
        roles: Vec<Role>,
        modes: Vec<Mode>,
    },
    ApplySource {
        source: SourceAsset,
        colors_to_add: Vec<Item>,
        color_values: BTreeMap<String, Rgb>,
    },
    RemoveSource(String),
    UnlinkCapture {
        source_id: String,
        capture_id: String,
    },
}
impl Command {
    pub fn label(&self) -> &str {
        match self {
            Self::ReceiveColors { .. } => "Receber cores de outro aplicativo",
            Self::Batch(_) => "Operação composta",
            Self::Add(_) => "Adicionar item",
            Self::Metadata(_) => "Editar detalhes",
            Self::ProjectDetails { .. } => "Editar projeto",
            Self::Delete(_) => "Excluir",
            Self::Duplicate(_) => "Duplicar",
            Self::Move { .. } | Self::Reorder { .. } => "Organizar itens",
            Self::Paste { .. } => "Colar",
            Self::SetColor { .. } | Self::SetNative { .. } => "Editar cor",
            Self::Lock { .. } => "Bloqueio da cor",
            Self::DirectLock { .. } => "Bloqueio de edição direta",
            Self::AddLink { .. } | Self::RedefineLink(_) | Self::LinkColors { .. } => {
                "Editar vínculo"
            }
            Self::RemoveRelation(_) => "Remover relação",
            Self::Compose(_) => "Criar composição",
            Self::HarmonySpread { .. } | Self::GradientSpace { .. } => "Editar composição",
            Self::Detach { .. } => "Destacar cor",
            Self::AddContrast { .. } => "Definir contraste mínimo",
            Self::AddProfile(_) => "Adicionar perfil ICC",
            Self::Documentation { .. } => "Editar documentação",
            Self::ApplySource { .. } => "Editar fonte e capturas",
            Self::RemoveSource(_) => "Remover fonte",
            Self::UnlinkCapture { .. } => "Desvincular captura",
        }
    }
}
#[derive(Clone)]
pub struct HistoryEntry {
    pub project: Project,
    pub label: String,
}
#[derive(Clone)]
pub struct Engine {
    pub project: Project,
    pub undo: Vec<HistoryEntry>,
    pub redo: Vec<HistoryEntry>,
    gesture: Option<Project>,
    pub notice: String,
    pub last_diagnostic: Option<Diagnostic>,
    pub generation: u64,
    gesture_label: String,
}
impl Engine {
    pub fn new(project: Project) -> Self {
        Self {
            project,
            undo: vec![],
            redo: vec![],
            gesture: None,
            notice: String::new(),
            last_diagnostic: None,
            generation: 0,
            gesture_label: "Editar cor".into(),
        }
    }
    pub fn is_interacting(&self) -> bool {
        self.gesture.is_some()
    }
    pub fn begin_gesture(&mut self) {
        if self.gesture.is_none() {
            self.gesture = Some(self.project.clone())
        }
    }
    pub fn finish_gesture(&mut self) {
        if let Some(old) = self.gesture.take()
            && old != self.project
        {
            self.push(old, self.gesture_label.clone())
        }
    }
    fn push(&mut self, p: Project, label: String) {
        self.undo.push(HistoryEntry { project: p, label });
        if self.undo.len() > 80 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.generation = self.generation.wrapping_add(1);
    }
    pub fn undo(&mut self) {
        self.finish_gesture();
        if let Some(h) = self.undo.pop() {
            self.redo.push(HistoryEntry {
                project: std::mem::replace(&mut self.project, h.project),
                label: h.label,
            });
            self.generation = self.generation.wrapping_add(1);
        }
    }
    pub fn redo(&mut self) {
        self.finish_gesture();
        if let Some(h) = self.redo.pop() {
            self.undo.push(HistoryEntry {
                project: std::mem::replace(&mut self.project, h.project),
                label: h.label,
            });
            self.generation = self.generation.wrapping_add(1);
        }
    }
    pub fn preview_color(&mut self, id: &str, rgb: Rgb, continuous: bool) -> Result<()> {
        if self.gesture.is_none() {
            self.gesture_label = "Editar cor".into();
        }
        self.begin_gesture();
        let limited = solver::edit_diagnosed(
            &mut self.project,
            id,
            rgb,
            continuous,
            &mut self.last_diagnostic,
        )?;
        self.generation = self.generation.wrapping_add(1);
        self.notice = if limited {
            "Limite atingido. As relações e os valores originais foram preservados.".into()
        } else {
            String::new()
        };
        Ok(())
    }
    pub fn preview(&mut self, cmd: Command) -> Result<()> {
        if self.gesture.is_none() {
            self.gesture_label = cmd.label().into();
        }
        self.begin_gesture();
        self.last_diagnostic = None;
        let requested = command_origins(&self.project, &cmd);
        let operation = command_operation(&cmd);
        let mut next = self.project.clone();
        apply(&mut next, cmd, &mut self.last_diagnostic)?;
        next.validate()?;
        let bad = solver::problems(&next, &solver::state(&next), None);
        if !bad.is_empty() {
            self.last_diagnostic = Some(diagnostics::problems(&next, &requested, &bad, operation));
            return Err("Esta edição entra em conflito com as relações atuais.".into());
        }
        self.project = next;
        self.generation = self.generation.wrapping_add(1);
        Ok(())
    }
    pub fn execute(&mut self, cmd: Command) -> Result<()> {
        self.finish_gesture();
        let before = self.project.clone();
        let label = cmd.label().to_string();
        self.last_diagnostic = None;
        let requested = command_origins(&before, &cmd);
        let operation = command_operation(&cmd);
        let mut next = before.clone();
        apply(&mut next, cmd, &mut self.last_diagnostic)?;
        next.validate()?;
        let bad = solver::problems(&next, &solver::state(&next), None);
        if !bad.is_empty() {
            self.last_diagnostic = Some(diagnostics::problems(&next, &requested, &bad, operation));
            return Err(format!(
                "A operação entra em conflito com relações existentes: {}",
                bad.join(", ")
            ));
        }
        if next != before {
            self.project = next;
            self.push(before, label);
        }
        Ok(())
    }
}
fn require_srgb(p: &Project, ids: &[String]) -> Result<()> {
    for id in ids {
        let i = p.item(id).ok_or("Cor não encontrada")?;
        if !i.is_color() || !i.is_srgb() {
            return Err("Para relações técnicas, converta explicitamente as cores para sRGB, mantendo seus IDs.".into());
        }
    }
    Ok(())
}
fn make_link(p: &Project, a: String, b: String, channels: Vec<usize>) -> Result<Link> {
    require_srgb(p, &[a.clone(), b.clone()])?;
    if a == b || channels.is_empty() {
        return Err("Escolha duas cores e ao menos um canal.".into());
    }
    let aa = p.item(&a).ok_or("Cor ausente")?.rgb;
    let bb = p.item(&b).ok_or("Cor ausente")?.rgb;
    let mut rels = BTreeMap::new();
    let mut unique = BTreeSet::new();
    for c in &channels {
        if *c > 5 || !unique.insert(*c) {
            return Err("Canal inválido ou repetido.".into());
        }
        let av = color::channel(aa, *c);
        let bv = color::channel(bb, *c);
        let rel = if *c == 3 || av.abs() < 1e-12 || bv.abs() < 1e-12 {
            Relation {
                mode: "offset".into(),
                value: if *c == 3 {
                    color::hue_delta(bv, av)
                } else {
                    bv - av
                },
            }
        } else {
            Relation {
                mode: "ratio".into(),
                value: bv / av,
            }
        };
        rels.insert(c.to_string(), rel);
    }
    Ok(Link {
        id: uid(),
        a,
        b,
        channels,
        relations: rels,
    })
}
fn copy_into(
    p: &mut Project,
    source: &Project,
    ids: &[String],
    parent: Option<String>,
) -> Result<()> {
    let all = source.descendants(ids);
    let mapping: BTreeMap<_, _> = all.iter().map(|id| (id.clone(), uid())).collect();
    for profile in &source.profiles {
        if let Some(existing) = p.profiles.iter().find(|x| x.id == profile.id) {
            if existing.bytes != profile.bytes {
                return Err(
                    "Duas paletas usam o mesmo ID de perfil com conteúdo diferente.".into(),
                );
            }
        } else {
            p.profiles.push(profile.clone());
        }
    }
    for item in source.items.iter().filter(|i| all.contains(&i.id)) {
        let mut new = item.clone();
        new.id = mapping.get(&item.id).cloned().ok_or("Mapeamento ausente")?;
        // A copied captured colour is independent from its original source.
        new.source_locked = false;
        new.parent_id = item
            .parent_id
            .as_ref()
            .and_then(|id| mapping.get(id))
            .cloned()
            .or_else(|| parent.clone());
        if ids.contains(&item.id) {
            new.name = format!("{} · cópia", item.label())
        }
        p.items.push(new);
    }
    for l in source
        .links
        .iter()
        .filter(|l| all.contains(&l.a) && all.contains(&l.b))
    {
        let mut n = l.clone();
        n.id = uid();
        n.a = mapping.get(&l.a).cloned().ok_or("Origem ausente")?;
        n.b = mapping.get(&l.b).cloned().ok_or("Destino ausente")?;
        p.links.push(n);
    }
    for h in source
        .harmonies
        .iter()
        .filter(|h| h.ids.iter().all(|id| all.contains(id)))
    {
        let mut n = h.clone();
        n.id = uid();
        n.ids = h
            .ids
            .iter()
            .filter_map(|id| mapping.get(id).cloned())
            .collect();
        n.anchor_id = h.anchor_id.as_ref().and_then(|id| mapping.get(id)).cloned();
        for e in &mut n.constraints {
            e.a = mapping.get(&e.a).cloned().ok_or("Origem ausente")?;
            e.b = mapping.get(&e.b).cloned().ok_or("Destino ausente")?;
        }
        p.harmonies.push(n);
    }
    for g in source.gradients.iter().filter(|g| {
        all.contains(&g.a) && all.contains(&g.b) && g.ids.iter().all(|id| all.contains(id))
    }) {
        let mut n = g.clone();
        n.id = uid();
        n.a = mapping.get(&g.a).cloned().ok_or("Origem ausente")?;
        n.b = mapping.get(&g.b).cloned().ok_or("Destino ausente")?;
        n.ids = g
            .ids
            .iter()
            .filter_map(|id| mapping.get(id).cloned())
            .collect();
        n.detached = g
            .detached
            .iter()
            .filter_map(|id| mapping.get(id).cloned())
            .collect();
        p.gradients.push(n);
    }
    for l in source
        .limits
        .iter()
        .filter(|l| all.contains(&l.a) && all.contains(&l.b))
    {
        let mut n = l.clone();
        n.id = uid();
        n.a = mapping.get(&l.a).cloned().ok_or("Origem ausente")?;
        n.b = mapping.get(&l.b).cloned().ok_or("Destino ausente")?;
        p.limits.push(n);
    }
    for d in source
        .documentary_relations
        .iter()
        .filter(|d| all.contains(&d.from) && all.contains(&d.to))
    {
        let mut n = d.clone();
        n.id = uid();
        n.from = mapping.get(&d.from).cloned().ok_or("Origem ausente")?;
        n.to = mapping.get(&d.to).cloned().ok_or("Destino ausente")?;
        p.documentary_relations.push(n);
    }
    Ok(())
}
fn command_operation(cmd: &Command) -> Operation {
    match cmd {
        Command::Batch(commands) => commands
            .last()
            .map(command_operation)
            .unwrap_or(Operation::ChangeRelation),
        Command::Compose(_) => Operation::Composition,
        Command::LinkColors { .. } | Command::AddLink { .. } | Command::AddContrast { .. } => {
            Operation::CreateRelation
        }
        Command::SetColor { .. } | Command::SetNative { .. } => Operation::Edit,
        _ => Operation::ChangeRelation,
    }
}
fn command_origins(p: &Project, cmd: &Command) -> solver::State {
    let ids: Vec<String> = match cmd {
        Command::Batch(commands) => {
            return commands
                .iter()
                .flat_map(|c| command_origins(p, c))
                .collect();
        }
        Command::Compose(d) => d
            .originals
            .iter()
            .cloned()
            .chain(d.replacements.values().map(|r| r.id.clone()))
            .collect(),
        Command::AddLink { a, b, .. } | Command::AddContrast { a, b, .. } => {
            vec![a.clone(), b.clone()]
        }
        Command::LinkColors { ids, .. } => ids.clone(),
        Command::SetColor { id, rgb, .. } => return BTreeMap::from([(id.clone(), *rgb)]),
        Command::SetNative { id, .. } => vec![id.clone()],
        Command::HarmonySpread { id, .. } => p
            .harmonies
            .iter()
            .find(|h| &h.id == id)
            .map(|h| h.ids.clone())
            .unwrap_or_default(),
        Command::GradientSpace { id, .. } => p
            .gradients
            .iter()
            .find(|g| &g.id == id)
            .map(|g| vec![g.a.clone(), g.b.clone()])
            .unwrap_or_default(),
        _ => vec![],
    };
    ids.into_iter()
        .filter_map(|id| p.item(&id).map(|i| (id, i.rgb)))
        .collect()
}
fn apply(p: &mut Project, cmd: Command, diagnostic: &mut Option<Diagnostic>) -> Result<()> {
    match cmd {
        Command::ReceiveColors {
            request_id,
            items,
            profiles,
        } => {
            let key = "colorcraftBridgeReceipts";
            let mut receipts = p
                .extra
                .get(key)
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default();
            if receipts.iter().any(|r| {
                r.get("requestId").and_then(serde_json::Value::as_str) == Some(request_id.as_str())
            }) {
                return Ok(());
            }
            if items.is_empty() || items.len() > 1000 {
                return Err("Lote de cores inválido.".into());
            }
            let ids: Vec<_> = items.iter().map(|i| i.id.clone()).collect();
            for profile in profiles {
                if let Some(existing) = p.profiles.iter().find(|v| v.id == profile.id) {
                    if existing.bytes != profile.bytes {
                        return Err("Identificador de perfil ICC em conflito.".into());
                    }
                } else {
                    p.profiles.push(profile);
                }
            }
            p.items.extend(items);
            receipts.push(serde_json::json!({"requestId":request_id,"colorIds":ids}));
            if receipts.len() > 1024 {
                receipts.remove(0);
            }
            p.extra
                .insert(key.into(), serde_json::Value::Array(receipts));
        }
        Command::Batch(commands) => {
            if commands.len() > 100 {
                return Err("Operação composta muito grande.".into());
            }
            for cmd in commands {
                if matches!(cmd, Command::Batch(_)) {
                    return Err("Operação composta aninhada não permitida.".into());
                }
                apply(p, cmd, diagnostic)?;
            }
        }
        Command::Add(item) => p.items.push(item),
        Command::Metadata(i) => {
            let n = p.item_mut(&i.id).ok_or("Item não encontrado")?;
            n.name = i.name;
            n.css_name = i.css_name;
            n.description = i.description;
            n.usage = i.usage;
            n.tags = i.tags;
            n.custom = i.custom;
            let renamed = n.name.clone();
            for source in &mut p.sources {
                for capture in &mut source.captures {
                    if capture.color_id.as_deref() == Some(i.id.as_str()) {
                        capture.name = renamed.clone();
                    }
                }
            }
        }
        Command::ProjectDetails { name, description } => {
            p.name = name;
            p.description = description;
        }
        Command::Delete(ids) => {
            let all = p.descendants(&ids);
            p.items.retain(|i| !all.contains(&i.id));
            for source in &mut p.sources {
                for capture in &mut source.captures {
                    if capture.color_id.as_ref().is_some_and(|id| all.contains(id)) {
                        capture.color_id = None;
                    }
                }
            }
            sync_source_locks(p);
            p.links
                .retain(|l| !all.contains(&l.a) && !all.contains(&l.b));
            for h in &mut p.harmonies {
                if h.ids.iter().any(|id| all.contains(id)) {
                    h.ids.retain(|id| !all.contains(id));
                    h.constraints.clear();
                    if h.anchor_id.as_ref().is_some_and(|id| all.contains(id)) {
                        h.anchor_id = None;
                    }
                }
            }
            p.harmonies.retain(|h| h.ids.len() > 1);
            p.freeze_harmonies();
            p.gradients
                .retain(|g| !all.contains(&g.a) && !all.contains(&g.b));
            for g in &mut p.gradients {
                g.ids.retain(|id| !all.contains(id));
                g.detached.retain(|id| !all.contains(id));
                g.count = g.ids.len();
            }
            p.gradients.retain(|g| !g.ids.is_empty());
            p.limits
                .retain(|l| !all.contains(&l.a) && !all.contains(&l.b));
            p.documentary_relations
                .retain(|d| !all.contains(&d.from) && !all.contains(&d.to));
            for role in &mut p.roles {
                if all.contains(&role.color_id) {
                    role.color_id.clear();
                }
            }
            let roles: BTreeSet<_> = p.roles.iter().map(|r| r.id.clone()).collect();
            for m in &mut p.modes {
                m.overrides.retain(|role, _| roles.contains(role));
                for col in m.overrides.values_mut() {
                    if all.contains(col) {
                        col.clear();
                    }
                }
            }
            if !p.gradients.is_empty() {
                let pins = p
                    .gradients
                    .iter()
                    .flat_map(|g| [&g.a, &g.b])
                    .filter_map(|id| p.item(id).map(|i| (id.clone(), i.rgb)))
                    .collect();
                let state = solver::solve(p, &pins).map_err(|e| {
                    format!(
                        "Não foi possível reorganizar as intermediárias: {}",
                        e.message
                    )
                })?;
                solver::apply_state(p, state);
            }
        }
        Command::Duplicate(ids) => {
            let source = p.clone();
            let parent = ids
                .first()
                .and_then(|id| p.item(id))
                .and_then(|i| i.parent_id.clone());
            copy_into(p, &source, &ids, parent)?;
        }
        Command::Paste {
            source,
            ids,
            parent,
        } => copy_into(p, &source, &ids, parent)?,
        Command::Move { ids, parent } => {
            if parent
                .as_ref()
                .is_some_and(|id| p.item(id).is_none_or(|i| i.kind != "folder"))
            {
                return Err("A pasta de destino não existe.".into());
            }
            let all = p.descendants(&ids);
            if parent.as_ref().is_some_and(|id| all.contains(id)) {
                return Err("Uma pasta não pode conter a si mesma.".into());
            }
            let selected: BTreeSet<_> = ids.iter().cloned().collect();
            let mut roots = Vec::new();
            for id in ids {
                let mut par = p.item(&id).and_then(|i| i.parent_id.as_ref());
                let mut child = false;
                for _ in 0..100 {
                    let Some(pid) = par else { break };
                    if selected.contains(pid) {
                        child = true;
                        break;
                    }
                    par = p.item(pid).and_then(|i| i.parent_id.as_ref());
                }
                if !child {
                    roots.push(id)
                }
            }
            for id in roots {
                if let Some(i) = p.item_mut(&id) {
                    i.parent_id = parent.clone()
                }
            }
        }
        Command::Reorder { id, before } => {
            if id != before {
                let source = p
                    .items
                    .iter()
                    .position(|i| i.id == id)
                    .ok_or("Item ausente")?;
                let n = p.items.remove(source);
                let index = p
                    .items
                    .iter()
                    .position(|i| i.id == before)
                    .ok_or("Destino ausente")?;
                p.items.insert(index, n);
            }
        }
        Command::SetColor {
            id,
            rgb,
            continuous,
        } => {
            solver::edit_diagnosed(p, &id, rgb, continuous, diagnostic)?;
        }
        Command::SetNative { id, mut value } => {
            let before = p.item(&id).ok_or("Cor ausente")?;
            if before.locked || before.source_locked {
                *diagnostic = Some(Diagnostic::direct(
                    p,
                    &id,
                    Cause::Protected {
                        color: id.clone(),
                        protection: if before.source_locked {
                            Protection::Source
                        } else {
                            Protection::Manual
                        },
                    },
                ));
                return Err("Desbloqueie a cor antes de convertê-la. Se ela estiver ligada a uma captura, desvincule-a primeiro.".into());
            }
            if before.direct_locked {
                *diagnostic = Some(Diagnostic::direct(
                    p,
                    &id,
                    Cause::Protected {
                        color: id.clone(),
                        protection: Protection::Direct,
                    },
                ));
                return Err(
                    "Libere a edição direta da cor antes de convertê-la ou atribuir um perfil."
                        .into(),
                );
            }
            let related = p.links.iter().any(|l| l.a == id || l.b == id)
                || p.harmonies.iter().any(|h| h.ids.contains(&id))
                || p.gradients
                    .iter()
                    .any(|g| g.a == id || g.b == id || g.ids.contains(&id))
                || p.limits.iter().any(|l| l.a == id || l.b == id);
            if related && !value.is_srgb() {
                return Err(
                    "Remova os vínculos técnicos antes de alterar o espaço nativo desta cor."
                        .into(),
                );
            }
            // Embed the actual built-in bytes, especially synthetic CMYK. A future
            // library update must not reinterpret colours already saved by the user.
            if !value.is_srgb() && value.profile_id.is_none() {
                let profile = color::profile(&value, &p.profiles)?;
                let profile_id = format!("icc-{:016x}", profile.content_hash());
                if !p.profiles.iter().any(|pr| pr.id == profile_id) {
                    p.profiles.push(IccProfile {
                        id: profile_id.clone(),
                        name: profile.description.clone(),
                        bytes: profile.to_bytes().as_ref().clone(),
                    });
                }
                value.profile_id = Some(profile_id);
            }
            let rgb = color::preview(&value, &p.profiles)?;
            let n = p.item_mut(&id).ok_or("Cor ausente")?;
            n.color = Some(value);
            n.rgb = rgb.map(|x| x.clamp(0.0, 255.0));
        }
        Command::Lock { id, locked } => {
            if !locked && p.is_source_locked(&id) {
                return Err("Esta cor está bloqueada por uma captura. Desvincule-a pelo botão de cadeado para preservar o valor atual.".into());
            }
            p.item_mut(&id).ok_or("Cor ausente")?.locked = locked
        }
        Command::DirectLock { id, locked } => {
            let item = p
                .item_mut(&id)
                .filter(|i| i.is_color())
                .ok_or("Cor ausente")?;
            item.direct_locked = locked;
        }
        Command::LinkColors { ids, channels } => {
            if channels.is_empty()
                || channels.iter().any(|c| *c > 5)
                || channels.iter().collect::<BTreeSet<_>>().len() != channels.len()
            {
                return Err("Escolha canais válidos e distintos.".into());
            }
            let unique: BTreeSet<_> = ids.iter().collect();
            if ids.len() < 2 || unique.len() != ids.len() {
                return Err("Escolha pelo menos duas cores diferentes.".into());
            }
            require_srgb(p, &ids)?;
            let anchor = ids.first().cloned().ok_or("Seleção vazia")?;
            for id in ids.into_iter().skip(1) {
                let missing: Vec<_> = channels
                    .iter()
                    .copied()
                    .filter(|c| {
                        !p.links.iter().any(|l| {
                            ((l.a == anchor && l.b == id) || (l.b == anchor && l.a == id))
                                && l.channels.contains(c)
                        })
                    })
                    .collect();
                if !missing.is_empty() {
                    apply(
                        p,
                        Command::AddLink {
                            a: anchor.clone(),
                            b: id,
                            channels: missing,
                        },
                        diagnostic,
                    )?;
                }
            }
        }
        Command::AddLink { a, b, channels } => {
            if p.links.iter().any(|l| {
                ((l.a == a && l.b == b) || (l.a == b && l.b == a))
                    && l.channels.iter().any(|c| channels.contains(c))
            }) {
                return Err("Algum desses canais já está ligado diretamente.".into());
            }
            let l = make_link(p, a, b, channels)?;
            p.links.push(l);
        }
        Command::RedefineLink(id) => {
            let old = p
                .links
                .iter()
                .find(|l| l.id == id)
                .cloned()
                .ok_or("Vínculo ausente")?;
            let mut new = make_link(p, old.a, old.b, old.channels)?;
            new.id = id.clone();
            if let Some(l) = p.links.iter_mut().find(|l| l.id == id) {
                *l = new
            }
        }
        Command::RemoveRelation(id) => {
            p.links.retain(|l| l.id != id);
            p.harmonies.retain(|l| l.id != id);
            p.gradients.retain(|l| l.id != id);
            p.limits.retain(|l| l.id != id);
        }
        Command::Compose(d) => crate::composition::apply(p, d).map_err(|e| {
            *diagnostic = e.diagnostic.map(|d| *d);
            e.message
        })?,
        Command::HarmonySpread { id, spread } => {
            let h = p
                .harmonies
                .iter()
                .find(|h| h.id == id)
                .cloned()
                .ok_or("Harmonia ausente")?;
            if crate::catalog::harmony(&h.kind).is_none_or(|t| t.spread.is_none()) {
                return Err("Harmonia sem abertura ajustável.".into());
            }
            let anchor = h
                .anchor_id
                .as_deref()
                .and_then(|id| p.item(id))
                .ok_or("Harmonia antiga sem âncora registrada")?;
            let colors = color::harmony(anchor.rgb, &h.kind, spread.clamp(5.0, 90.0), h.ids.len());
            if colors.len() != h.ids.len() {
                return Err("Estrutura incompatível.".into());
            }
            p.harmonies.retain(|x| x.id != id);
            let pins = h.ids.iter().cloned().zip(colors).collect();
            let s = solver::solve(p, &pins).map_err(|e| {
                *diagnostic = e.diagnostic.map(|mut d| {
                    d.operation = Operation::ChangeRelation;
                    *d
                });
                e.message
            })?;
            solver::apply_state(p, s);
            p.harmonies.push(Harmony {
                spread: Some(spread),
                constraints: vec![],
                ..h
            });
            p.freeze_harmonies();
        }
        Command::GradientSpace { id, space } => {
            let g = p
                .gradients
                .iter_mut()
                .find(|g| g.id == id)
                .ok_or("Gradiente ausente")?;
            g.space = space;
            let a = g.a.clone();
            let b = g.b.clone();
            let pins = [a, b]
                .into_iter()
                .filter_map(|id| p.item(&id).map(|n| (id, n.rgb)))
                .collect();
            let s = solver::solve(p, &pins).map_err(|e| {
                *diagnostic = e.diagnostic.map(|mut d| {
                    d.operation = Operation::ChangeRelation;
                    *d
                });
                e.message
            })?;
            solver::apply_state(p, s);
        }
        Command::Detach { gradient, color } => {
            let g = p
                .gradients
                .iter_mut()
                .find(|g| g.id == gradient)
                .ok_or("Gradiente ausente")?;
            if !g.ids.contains(&color) {
                return Err("Cor não pertence ao gradiente.".into());
            }
            if !g.detached.contains(&color) {
                g.detached.push(color)
            }
        }
        Command::AddContrast { a, b, minimum } => {
            require_srgb(p, &[a.clone(), b.clone()])?;
            p.limits.push(Limit {
                id: uid(),
                a,
                b,
                minimum,
                metric: "wcag22-srgb".into(),
            });
        }
        Command::AddProfile(profile) => {
            if !p.profiles.iter().any(|x| x.id == profile.id) {
                p.profiles.push(profile)
            }
        }
        Command::Documentation {
            relations,
            roles,
            modes,
        } => {
            p.documentary_relations = relations;
            p.roles = roles;
            p.modes = modes;
        }
        Command::ApplySource {
            source,
            colors_to_add,
            color_values,
        } => {
            if source.id.is_empty() {
                return Err("ID da fonte inválido.".into());
            }
            let linked: BTreeSet<String> = source
                .captures
                .iter()
                .filter_map(|c| c.color_id.clone())
                .collect();
            if colors_to_add
                .iter()
                .any(|item| !linked.contains(&item.id) || !item.is_color())
            {
                return Err("Uma cor nova não corresponde a uma captura desta fonte.".into());
            }
            for mut item in colors_to_add {
                item.parent_id = None;
                item.source_locked = true;
                item.locked = false;
                p.items.push(item);
            }
            for (id, rgb) in color_values {
                if !linked.contains(&id) || !color::in_gamut(rgb) {
                    return Err("Valor de captura inválido.".into());
                }
                let item = p
                    .item_mut(&id)
                    .filter(|i| i.is_color())
                    .ok_or("Cor vinculada à captura ausente.")?;
                item.set_rgb(rgb);
                item.source_locked = true;
            }
            let labels: Vec<(String, String)> = source
                .captures
                .iter()
                .filter_map(|capture| {
                    capture
                        .color_id
                        .as_ref()
                        .map(|id| (id.clone(), capture.name.clone()))
                })
                .collect();
            if let Some(index) = p.sources.iter().position(|s| s.id == source.id) {
                p.sources[index] = source;
            } else {
                p.sources.push(source);
            }
            for (id, name) in labels {
                if let Some(item) = p.item_mut(&id) {
                    item.name = name.clone();
                    item.css_name = color::css_name(&name);
                }
            }
            sync_source_locks(p);
        }
        Command::RemoveSource(id) => {
            if !p.sources.iter().any(|s| s.id == id) {
                return Err("Fonte ausente.".into());
            }
            p.sources.retain(|s| s.id != id);
            sync_source_locks(p);
        }
        Command::UnlinkCapture {
            source_id,
            capture_id,
        } => {
            let source = p
                .sources
                .iter_mut()
                .find(|s| s.id == source_id)
                .ok_or("Fonte ausente.")?;
            let capture = source
                .captures
                .iter_mut()
                .find(|c| c.id == capture_id)
                .ok_or("Captura ausente.")?;
            if capture.color_id.take().is_none() {
                return Err("A captura já está desvinculada.".into());
            }
            sync_source_locks(p);
        }
    }
    Ok(())
}
fn sync_source_locks(p: &mut Project) {
    let linked: BTreeSet<String> = p
        .sources
        .iter()
        .flat_map(|s| s.captures.iter())
        .filter_map(|c| c.color_id.clone())
        .collect();
    for item in &mut p.items {
        item.source_locked = linked.contains(&item.id);
    }
}
