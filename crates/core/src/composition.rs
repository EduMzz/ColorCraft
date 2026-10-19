//! Read-only composition analysis and the shared transactional application.
use crate::{
    color,
    engine::{CompositionDraft, Slot, composition_preview},
    solver, *,
};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug)]
pub struct CompositionIssue {
    pub message: String,
    pub relation_ids: Vec<String>,
    pub diagnostic: Option<Box<crate::diagnostics::Diagnostic>>,
}
impl From<String> for CompositionIssue {
    fn from(message: String) -> Self {
        Self {
            message,
            relation_ids: vec![],
            diagnostic: None,
        }
    }
}
impl From<&str> for CompositionIssue {
    fn from(message: &str) -> Self {
        message.to_string().into()
    }
}
#[derive(Clone, Debug, Default)]
pub struct CompositionAnalysis {
    pub slots: Vec<Slot>,
    pub original_count: usize,
    pub new_count: usize,
    pub replacement_count: usize,
    pub issue: Option<CompositionIssue>,
}
pub fn analyze(
    p: &Project,
    draft: &CompositionDraft,
    removed: &BTreeSet<String>,
) -> CompositionAnalysis {
    let mut result = CompositionAnalysis::default();
    match composition_preview(p, draft) {
        Ok((slots, _)) => {
            result.original_count = slots.iter().filter(|s| s.original.is_some()).count();
            result.replacement_count = slots
                .iter()
                .enumerate()
                .filter(|(i, s)| s.original.is_none() && draft.replacements.contains_key(i))
                .count();
            result.new_count = slots.len() - result.original_count - result.replacement_count;
            result.slots = slots;
        }
        Err(error) => {
            result.issue = Some(error.into());
            return result;
        }
    }
    let mut candidate = p.clone();
    candidate.links.retain(|r| !removed.contains(&r.id));
    candidate.harmonies.retain(|r| !removed.contains(&r.id));
    candidate.gradients.retain(|r| !removed.contains(&r.id));
    candidate.limits.retain(|r| !removed.contains(&r.id));
    result.issue = apply(&mut candidate, draft.clone()).err();
    if result.issue.is_none() {
        result.issue = candidate.validate().err().map(Into::into);
    }
    if result.issue.is_none() {
        let relation_ids = solver::problems(&candidate, &solver::state(&candidate), None);
        if !relation_ids.is_empty() {
            let requested = draft
                .originals
                .iter()
                .chain(draft.replacements.values().map(|r| &r.id))
                .filter_map(|id| candidate.item(id).map(|i| (id.clone(), i.rgb)))
                .collect();
            let diagnostic = crate::diagnostics::problems(
                &candidate,
                &requested,
                &relation_ids,
                crate::diagnostics::Operation::Composition,
            );
            result.issue = Some(CompositionIssue {
                message: "A composição conflita com estas relações.".into(),
                relation_ids,
                diagnostic: Some(Box::new(diagnostic)),
            });
        }
    }
    result
}
pub(crate) fn apply(
    p: &mut Project,
    d: CompositionDraft,
) -> std::result::Result<(), CompositionIssue> {
    let (slots, anchor) = composition_preview(p, &d)?;
    p.freeze_harmonies();
    let parent = d
        .originals
        .first()
        .and_then(|id| p.item(id))
        .and_then(|i| i.parent_id.clone());
    let mut ids = vec![];
    let mut pins = BTreeMap::new();
    for id in &d.originals {
        if let Some(i) = p.item(id) {
            pins.insert(id.clone(), i.rgb);
        }
    }
    for (i, slot) in slots.iter().enumerate() {
        let id = if let Some(id) = &slot.original {
            id.clone()
        } else if let Some(replacement) = d.replacements.get(&i) {
            let n = p
                .item(&replacement.id)
                .ok_or("Cor substituída não encontrada")?;
            if n.source_locked {
                let mut diagnostic = crate::diagnostics::Diagnostic::direct(
                    p,
                    &n.id,
                    crate::diagnostics::Cause::Protected {
                        color: n.id.clone(),
                        protection: crate::diagnostics::Protection::Source,
                    },
                );
                diagnostic.operation = crate::diagnostics::Operation::Composition;
                return Err(CompositionIssue{message:"Uma cor ligada a uma captura precisa ser desvinculada no painel Fontes antes de ser substituída.".into(),relation_ids:vec![],diagnostic:Some(Box::new(diagnostic))});
            }
            if (n.locked || n.direct_locked) && !replacement.unlock {
                let mut diagnostic = crate::diagnostics::Diagnostic::direct(
                    p,
                    &n.id,
                    crate::diagnostics::Cause::Protected {
                        color: n.id.clone(),
                        protection: if n.locked {
                            crate::diagnostics::Protection::Manual
                        } else {
                            crate::diagnostics::Protection::Direct
                        },
                    },
                );
                diagnostic.operation = crate::diagnostics::Operation::Composition;
                return Err(CompositionIssue {
                    message: "Confirme o desbloqueio da cor substituída.".into(),
                    relation_ids: vec![],
                    diagnostic: Some(Box::new(diagnostic)),
                });
            }
            if !n.is_srgb() {
                return Err("Converta explicitamente a cor substituída para sRGB.".into());
            }
            let dep = p
                .gradients
                .iter()
                .any(|g| g.ids.contains(&n.id) && !g.detached.contains(&n.id));
            if dep && !replacement.detach {
                let relation = p
                    .gradients
                    .iter()
                    .find(|g| g.ids.contains(&n.id) && !g.detached.contains(&n.id))
                    .map(|g| g.id.clone())
                    .unwrap_or_default();
                let mut diagnostic = crate::diagnostics::Diagnostic::direct(
                    p,
                    &n.id,
                    crate::diagnostics::Cause::Gradient {
                        color: n.id.clone(),
                        relation: relation.clone(),
                    },
                );
                diagnostic.operation = crate::diagnostics::Operation::Composition;
                return Err(CompositionIssue {
                    message: "Confirme destacar a cor do gradiente antes de substituir.".into(),
                    relation_ids: vec![relation],
                    diagnostic: Some(Box::new(diagnostic)),
                });
            }
            let id = n.id.clone();
            if replacement.unlock
                && let Some(n) = p.item_mut(&id)
            {
                n.locked = false;
                n.direct_locked = false;
            }
            if replacement.detach {
                for g in &mut p.gradients {
                    if g.ids.contains(&id) && !g.detached.contains(&id) {
                        g.detached.push(id.clone())
                    }
                }
            }
            pins.insert(id.clone(), slot.rgb);
            id
        } else {
            let mut n = Item::new("color", parent.clone());
            n.name = format!(
                "{} {}",
                if d.kind == "gradient" {
                    "Gradiente"
                } else {
                    color::harmony_name(&d.kind)
                },
                i + 1
            );
            n.rgb = slot.rgb;
            let id = n.id.clone();
            p.items.push(n);
            id
        };
        if ids.contains(&id) {
            return Err("Cada posição precisa usar uma cor diferente.".into());
        }
        ids.push(id);
    }
    // Adding colors does not edit existing values. Re-solving unchanged originals
    // can turn accepted floating-point residuals in old links into false pin conflicts.
    // Only replacements introduce edits that require propagation through the solver.
    if slots
        .iter()
        .enumerate()
        .any(|(index, slot)| slot.original.is_none() && d.replacements.contains_key(&index))
    {
        let s = solver::solve(p, &pins).map_err(|e| CompositionIssue {
            message: e.message,
            relation_ids: e.relations,
            diagnostic: e.diagnostic.map(|mut d| {
                d.operation = crate::diagnostics::Operation::Composition;
                d
            }),
        })?;
        solver::apply_state(p, s);
    }
    if d.kind == "gradient" {
        let a = ids.first().cloned().ok_or("Gradiente vazio")?;
        let b = ids.last().cloned().ok_or("Gradiente vazio")?;
        p.gradients.push(Gradient {
            id: uid(),
            a,
            b,
            ids: ids
                .iter()
                .skip(1)
                .take(ids.len().saturating_sub(2))
                .cloned()
                .collect(),
            count: d.count.clamp(1, 32),
            space: d.space,
            detached: vec![],
        });
    } else {
        p.harmonies.push(Harmony {
            id: uid(),
            kind: d.kind,
            anchor_id: ids.get(anchor).cloned(),
            spread: Some(d.spread),
            ids,
            constraints: vec![],
        });
        p.freeze_harmonies();
    }

    Ok(())
}
