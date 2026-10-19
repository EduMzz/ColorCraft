//! Session-only conflict history. It never participates in project serialization or undo.
use colorcraft_core::{Project, diagnostics::Diagnostic};
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Debug)]
pub struct ColorSnapshot {
    pub existed: bool,
    pub name: String,
    pub rgb: [f64; 3],
}
#[derive(Clone, Debug)]
pub struct RelationSnapshot {
    pub existed: bool,
    pub title: String,
    pub icon: &'static str,
    pub participants: Vec<String>,
    pub group: bool,
}
pub fn relation(p: &Project, id: &str) -> Option<RelationSnapshot> {
    let (title, icon, ids, group) = if let Some(l) = p.links.iter().find(|l| l.id == id) {
        (
            "Vínculo paramétrico".into(),
            "link",
            vec![l.a.clone(), l.b.clone()],
            false,
        )
    } else if let Some(h) = p.harmonies.iter().find(|h| h.id == id) {
        (
            colorcraft_core::color::harmony_name(&h.kind).into(),
            "spark",
            h.ids.clone(),
            true,
        )
    } else if let Some(g) = p.gradients.iter().find(|g| g.id == id) {
        (
            "Gradiente".into(),
            "gradient",
            std::iter::once(&g.a)
                .chain(g.ids.iter())
                .chain(std::iter::once(&g.b))
                .cloned()
                .collect(),
            true,
        )
    } else {
        let l = p.limits.iter().find(|l| l.id == id)?;
        (
            "Vínculo limitante".into(),
            "contrast",
            vec![l.a.clone(), l.b.clone()],
            false,
        )
    };
    Some(RelationSnapshot {
        existed: true,
        title,
        icon,
        participants: ids
            .iter()
            .filter_map(|id| p.item(id).map(|i| i.label()))
            .collect(),
        group,
    })
}
#[derive(Clone, Debug)]
pub struct Entry {
    pub id: u64,
    pub diagnostic: Diagnostic,
    pub colors: BTreeMap<String, ColorSnapshot>,
    pub relations: BTreeMap<String, RelationSnapshot>,
    pub first: String,
    pub last: String,
    pub repetitions: u64,
    pub unread: bool,
    key: String,
    last_attempt: u64,
}
#[derive(Default)]
pub struct ConflictLog {
    pub entries: VecDeque<Entry>,
    next: u64,
    attempt: u64,
    gesture: Option<u64>,
}
impl ConflictLog {
    pub fn unread(&self) -> usize {
        self.entries.iter().filter(|e| e.unread).count()
    }
    pub fn finish(&mut self) {
        self.gesture = None;
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.gesture = None;
    }
    pub fn record(&mut self, p: &Project, diagnostic: Diagnostic, continuous: bool) {
        let attempt = if continuous {
            if let Some(token) = self.gesture {
                token
            } else {
                self.attempt += 1;
                self.gesture = Some(self.attempt);
                self.attempt
            }
        } else {
            self.attempt += 1;
            self.attempt
        };
        // Rejected creations get fresh IDs on every attempt. Their semantic identity
        // must be stable too, while existing relationships retain their actual IDs.
        let pending: BTreeMap<_, _> = diagnostic
            .relations
            .iter()
            .filter(|(id, _)| relation(p, id).is_none())
            .map(|(id, r)| {
                (
                    id.clone(),
                    format!("pending:{:?}:{:?}", r.kind, r.participants),
                )
            })
            .collect();
        let mut identity = diagnostic.clone();
        for trace in &mut identity.traces {
            for hop in &mut trace.hops {
                if let Some(id) = pending.get(&hop.relation) {
                    hop.relation = id.clone();
                }
            }
            use colorcraft_core::diagnostics::Cause;
            match &mut trace.cause {
                Cause::Gradient { relation, .. } | Cause::Contrast { relation, .. } => {
                    if let Some(id) = pending.get(relation) {
                        *relation = id.clone();
                    }
                }
                Cause::Incompatible { relations, .. } => {
                    for id in relations {
                        if let Some(stable) = pending.get(id) {
                            *id = stable.clone();
                        }
                    }
                }
                _ => {}
            }
        }
        let key = identity.key();
        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        let color_ids =
            diagnostic
                .origins
                .iter()
                .cloned()
                .chain(diagnostic.traces.iter().flat_map(|t| {
                    std::iter::once(t.origin.clone())
                        .chain(t.hops.iter().flat_map(|h| [h.from.clone(), h.to.clone()]))
                        .chain(t.cause.colors())
                }));
        let mut colors: BTreeMap<_, _> = color_ids
            .filter_map(|id| {
                p.item(&id).map(|i| {
                    (
                        id,
                        ColorSnapshot {
                            existed: true,
                            name: i.label(),
                            rgb: i.rgb,
                        },
                    )
                })
            })
            .collect();
        for (id, (name, rgb)) in &diagnostic.colors {
            colors.insert(
                id.clone(),
                ColorSnapshot {
                    existed: p.item(id).is_some(),
                    name: name.clone(),
                    rgb: *rgb,
                },
            );
        }
        let relation_ids = diagnostic.traces.iter().flat_map(|t| {
            t.hops
                .iter()
                .map(|h| h.relation.clone())
                .chain(t.cause.relations())
        });
        let mut relations: BTreeMap<_, _> = relation_ids
            .filter_map(|id| relation(p, &id).map(|r| (id, r)))
            .collect();
        for (id, evidence) in &diagnostic.relations {
            use colorcraft_core::diagnostics::RelationKind;
            let (title, icon, group) = match &evidence.kind {
                RelationKind::Parametric => ("Vínculo paramétrico".into(), "link", false),
                RelationKind::Harmony(kind) => (
                    colorcraft_core::color::harmony_name(kind).into(),
                    "spark",
                    true,
                ),
                RelationKind::Gradient => ("Gradiente".into(), "gradient", true),
                RelationKind::Limit => ("Vínculo limitante".into(), "contrast", false),
            };
            relations.entry(id.clone()).or_insert(RelationSnapshot {
                existed: false,
                title,
                icon,
                group,
                participants: evidence
                    .participants
                    .iter()
                    .filter_map(|id| colors.get(id).map(|c| c.name.clone()))
                    .collect(),
            });
        }
        if let Some(last) = self.entries.front_mut().filter(|e| e.key == key) {
            if last.last_attempt != attempt {
                last.repetitions += 1;
                last.unread = true;
                last.last_attempt = attempt;
            }
            last.last = now;
            last.diagnostic = diagnostic;
            last.colors = colors;
            last.relations = relations;
            return;
        }
        self.next += 1;
        self.entries.push_front(Entry {
            id: self.next,
            diagnostic,
            colors,
            relations,
            first: now.clone(),
            last: now,
            repetitions: 1,
            unread: true,
            key,
            last_attempt: attempt,
        });
        self.entries.truncate(500);
    }
}
/// Deterministic isolated QA palette, including a preserved legacy harmony with HSV offsets.
pub fn demo_project() -> Project {
    use colorcraft_core::{
        Constraint, Harmony, Item, color,
        engine::{Command, Engine},
    };
    let mut p = Project {
        name: "Conflict chains".into(),
        ..Default::default()
    };
    for (id, name, h, s, v) in [
        ("a", "Primary", 165.0, 0.5, 0.6),
        ("b", "Secondary", 40.0, 0.5, 0.8),
        ("c", "Other", 80.0, 0.1, 0.7),
        ("unrelated", "Independent", 240.0, 0.3, 0.8),
    ] {
        let mut i = Item::new("color", None);
        i.id = id.into();
        i.name = name.into();
        i.rgb = color::from_hsv([h, s, v]);
        p.items.push(i);
    }
    let mut e = Engine::new(p);
    if e.execute(Command::AddLink {
        a: "a".into(),
        b: "b".into(),
        channels: vec![4],
    })
    .is_err()
    {
        return e.project;
    }
    let mut p = e.project;
    p.links[0].id = "saturation".into();
    p.harmonies.push(Harmony {
        id: "analogy".into(),
        kind: "analogous".into(),
        ids: vec!["b".into(), "c".into()],
        constraints: vec![Constraint {
            a: "b".into(),
            b: "c".into(),
            offset: [40.0, -0.4, -0.1],
        }],
        ..Default::default()
    });
    p
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use colorcraft_core::{
        Item,
        diagnostics::{Cause, Protection},
    };
    fn fixture() -> (Project, Diagnostic) {
        let mut p = Project::default();
        let mut i = Item::new("color", None);
        i.id = "a".into();
        p.items.push(i);
        let d = Diagnostic::direct(
            &p,
            "a",
            Cause::Protected {
                color: "a".into(),
                protection: Protection::Manual,
            },
        );
        (p, d)
    }
    #[test]
    fn repeated_rejected_creations_with_fresh_ids_are_the_same_conflict() {
        use colorcraft_core::engine::{Command, Engine};
        let (mut p, _) = fixture();
        let mut b = p.items[0].clone();
        b.id = "b".into();
        p.items.push(b);
        let mut e = Engine::new(p.clone());
        let mut log = ConflictLog::default();
        for _ in 0..3 {
            assert!(
                e.execute(Command::AddContrast {
                    a: "a".into(),
                    b: "b".into(),
                    minimum: 4.5
                })
                .is_err()
            );
            log.record(&p, e.last_diagnostic.take().unwrap(), false);
        }
        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.entries[0].repetitions, 3);
        assert!(log.entries[0].relations.values().all(|r| !r.existed));
        assert_eq!(e.project, p);
    }
    #[test]
    fn repeated_frames_are_one_gesture_and_adjacent_attempts_coalesce() {
        let (p, d) = fixture();
        let mut log = ConflictLog::default();
        for _ in 0..100 {
            log.record(&p, d.clone(), true);
        }
        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.entries[0].repetitions, 1);
        log.entries[0].unread = false;
        log.record(&p, d.clone(), true);
        assert_eq!(log.unread(), 0);
        log.finish();
        log.record(&p, d.clone(), true);
        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.entries[0].repetitions, 2);
        assert_eq!(log.unread(), 1);
        let mut other = d.clone();
        other.direction = 1;
        log.record(&p, other, false);
        log.record(&p, d, false);
        assert_eq!(log.entries.len(), 3);
    }
    #[test]
    fn bounded_log_and_snapshots_survive_deletion_without_affecting_project() {
        let (p, mut d) = fixture();
        let mut log = ConflictLog::default();
        let before = p.clone();
        for n in 0..510 {
            d.direction = if n % 2 == 0 { 1 } else { -1 };
            log.record(&p, d.clone(), false);
        }
        assert_eq!(log.entries.len(), 500);
        assert_eq!(p, before);
        let empty = Project::default();
        assert!(empty.item("a").is_none());
        assert!(log.entries[0].colors.contains_key("a"));
        log.clear();
        assert_eq!(log.unread(), 0);
        assert!(log.entries.is_empty());
    }
}
