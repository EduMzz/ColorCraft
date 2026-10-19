//! Structured conflict evidence. No localized text is used to identify a blocker.
use crate::{
    Link, Project,
    color::{self, Rgb},
    solver::State,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    Edit,
    Composition,
    CreateRelation,
    ChangeRelation,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Protection {
    Manual,
    Direct,
    Source,
    Preserved,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Cause {
    Protected {
        color: String,
        protection: Protection,
    },
    Gradient {
        color: String,
        relation: String,
    },
    ChannelBound {
        color: String,
        channel: usize,
        maximum: bool,
    },
    Contrast {
        relation: String,
        a: String,
        b: String,
        minimum: f64,
        actual: f64,
    },
    Incompatible {
        colors: Vec<String>,
        relations: Vec<String>,
        cycle: bool,
    },
    NativeSpace {
        color: String,
    },
}
impl Cause {
    pub fn colors(&self) -> Vec<String> {
        match self {
            Self::Protected { color, .. }
            | Self::Gradient { color, .. }
            | Self::ChannelBound { color, .. }
            | Self::NativeSpace { color } => vec![color.clone()],
            Self::Contrast { a, b, .. } => vec![a.clone(), b.clone()],
            Self::Incompatible { colors, .. } => colors.clone(),
        }
    }
    pub fn relations(&self) -> Vec<String> {
        match self {
            Self::Gradient { relation, .. } | Self::Contrast { relation, .. } => {
                vec![relation.clone()]
            }
            Self::Incompatible { relations, .. } => relations.clone(),
            _ => vec![],
        }
    }
    /// Excludes numerical values: dragging further into the same blocker is one conflict.
    pub fn key(&self) -> String {
        match self {
            Self::Contrast { relation, a, b, .. } => format!("contrast:{relation}:{a}:{b}"),
            _ => format!("{self:?}"),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Hop {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub channel: Option<usize>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Trace {
    pub origin: String,
    pub hops: Vec<Hop>,
    pub cause: Cause,
}
#[derive(Clone, Debug, PartialEq)]
pub enum RelationKind {
    Parametric,
    Harmony(String),
    Gradient,
    Limit,
}
#[derive(Clone, Debug, PartialEq)]
pub struct RelationEvidence {
    pub kind: RelationKind,
    pub participants: Vec<String>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostic {
    pub operation: Operation,
    pub origins: Vec<String>,
    pub channel: Option<usize>,
    pub direction: i8,
    pub requested: State,
    pub traces: Vec<Trace>,
    pub colors: BTreeMap<String, (String, Rgb)>,
    pub relations: BTreeMap<String, RelationEvidence>,
}
impl Diagnostic {
    pub fn annotate(&mut self, p: &Project) {
        let ids: Vec<_> = self
            .origins
            .iter()
            .cloned()
            .chain(self.traces.iter().flat_map(|t| {
                std::iter::once(t.origin.clone())
                    .chain(t.hops.iter().flat_map(|h| [h.from.clone(), h.to.clone()]))
                    .chain(t.cause.colors())
            }))
            .collect();
        self.colors = ids
            .into_iter()
            .filter_map(|id| p.item(&id).map(|i| (id, (i.label(), i.rgb))))
            .collect();
        let ids: Vec<_> = self
            .traces
            .iter()
            .flat_map(|t| {
                t.hops
                    .iter()
                    .map(|h| h.relation.clone())
                    .chain(t.cause.relations())
            })
            .collect();
        for id in ids {
            let evidence = if let Some(l) = p.links.iter().find(|l| l.id == id) {
                Some(RelationEvidence {
                    kind: RelationKind::Parametric,
                    participants: vec![l.a.clone(), l.b.clone()],
                })
            } else if let Some(h) = p.harmonies.iter().find(|h| h.id == id) {
                Some(RelationEvidence {
                    kind: RelationKind::Harmony(h.kind.clone()),
                    participants: h.ids.clone(),
                })
            } else if let Some(g) = p.gradients.iter().find(|g| g.id == id) {
                Some(RelationEvidence {
                    kind: RelationKind::Gradient,
                    participants: std::iter::once(&g.a)
                        .chain(g.ids.iter())
                        .chain(std::iter::once(&g.b))
                        .cloned()
                        .collect(),
                })
            } else {
                p.limits
                    .iter()
                    .find(|l| l.id == id)
                    .map(|l| RelationEvidence {
                        kind: RelationKind::Limit,
                        participants: vec![l.a.clone(), l.b.clone()],
                    })
            };
            if let Some(e) = evidence {
                for id in &e.participants {
                    if let Some(i) = p.item(id) {
                        self.colors
                            .entry(id.clone())
                            .or_insert_with(|| (i.label(), i.rgb));
                    }
                }
                self.relations.insert(id, e);
            }
        }
    }

    pub fn key(&self) -> String {
        format!(
            "{:?}:{:?}:{:?}:{}:{:?}",
            self.operation,
            self.origins,
            self.channel,
            self.direction,
            self.traces
                .iter()
                .map(|t| (&t.origin, &t.hops, t.cause.key()))
                .collect::<Vec<_>>()
        )
    }
    pub fn edit_intent(&mut self, original: Rgb, target: Rgb) {
        let rgb: Vec<_> = (0..3)
            .filter(|c| (original[*c] - target[*c]).abs() > 1e-7)
            .collect();
        let a = color::hsv(original);
        let b = color::hsv(target);
        let hsv: Vec<_> = (0..3)
            .filter(|c| {
                if *c == 0 {
                    color::hue_delta(b[*c], a[*c]).abs() > 1e-5
                } else {
                    (a[*c] - b[*c]).abs() > 1e-7
                }
            })
            .collect();
        let (channel, delta) = if rgb.len() == 1 {
            (Some(rgb[0]), target[rgb[0]] - original[rgb[0]])
        } else if hsv.len() == 1 {
            let c = hsv[0];
            (
                Some(c + 3),
                if c == 0 {
                    color::hue_delta(b[c], a[c])
                } else {
                    b[c] - a[c]
                },
            )
        } else {
            (None, 0.0)
        };
        self.channel = channel;
        self.direction = if delta > 0.0 {
            1
        } else if delta < 0.0 {
            -1
        } else {
            0
        };
        if let Some(id) = self.origins.first() {
            self.requested.insert(id.clone(), target);
        }
    }
    pub fn direct(p: &Project, id: &str, cause: Cause) -> Self {
        let requested = p
            .item(id)
            .map(|i| BTreeMap::from([(id.into(), i.rgb)]))
            .unwrap_or_default();
        let mut d = Trail::new(p, &requested).diagnostic(vec![cause], &[]);
        d.annotate(p);
        d
    }
}
#[derive(Clone, Debug)]
struct Path {
    origin: String,
    hops: Vec<Hop>,
}
/// Provenance follows components that actually propagate changed values, not all palette links.
pub(crate) struct Trail {
    paths: BTreeMap<String, Path>,
    active: BTreeMap<String, Path>,
    requested: State,
}
impl Trail {
    pub fn new(p: &Project, requested: &State) -> Self {
        let changed: Vec<_> = requested
            .iter()
            .filter(|(id, rgb)| p.item(id).is_some_and(|i| i.rgb != **rgb))
            .map(|(id, _)| id.clone())
            .collect();
        let seeds = if changed.is_empty() {
            requested.keys().cloned().collect()
        } else {
            changed
        };
        Self {
            active: BTreeMap::new(),
            paths: seeds
                .into_iter()
                .map(|id| {
                    (
                        id.clone(),
                        Path {
                            origin: id,
                            hops: vec![],
                        },
                    )
                })
                .collect(),
            requested: requested.clone(),
        }
    }
    pub fn component(
        &mut self,
        edges: &[Link],
        members: &BTreeSet<String>,
        relations: &[String],
        channel: usize,
    ) {
        let Some((source, path)) = self
            .paths
            .iter()
            .filter(|(id, _)| members.contains(*id))
            .min_by_key(|(_, p)| p.hops.len())
            .map(|(id, p)| (id.clone(), p.clone()))
        else {
            return;
        };
        let mut routes = BTreeMap::from([(source.clone(), vec![])]);
        let mut queue = VecDeque::from([source]);
        while let Some(id) = queue.pop_front() {
            for edge in edges
                .iter()
                .filter(|e| relations.contains(&e.id) && e.channels.contains(&channel))
            {
                let to = if edge.a == id {
                    &edge.b
                } else if edge.b == id {
                    &edge.a
                } else {
                    continue;
                };
                if !members.contains(to) || routes.contains_key(to) {
                    continue;
                }
                let mut route = routes[&id].clone();
                route.push(Hop {
                    from: id.clone(),
                    to: to.clone(),
                    relation: edge.id.clone(),
                    channel: Some(channel),
                });
                routes.insert(to.clone(), route);
                queue.push_back(to.clone());
            }
        }
        self.active.clear();
        for (id, route) in &routes {
            let mut hops = path.hops.clone();
            hops.extend(route.clone());
            self.active.insert(
                id.clone(),
                Path {
                    origin: path.origin.clone(),
                    hops,
                },
            );
        }
        for (id, route) in routes {
            if self.requested.contains_key(&id) && self.paths.contains_key(&id) {
                continue;
            }
            let mut hops = path.hops.clone();
            hops.extend(route);
            // Keep a short proof and do not introduce repeated cycles while iterating projections.
            if self
                .paths
                .get(&id)
                .is_none_or(|old| hops.len() < old.hops.len())
            {
                self.paths.insert(
                    id,
                    Path {
                        origin: path.origin.clone(),
                        hops,
                    },
                );
            }
        }
    }
    pub fn gradient(&mut self, a: &str, b: &str, to: &str, relation: &str) {
        if let Some((from, path)) = [a, b]
            .into_iter()
            .filter_map(|id| self.paths.get(id).map(|p| (id, p)))
            .min_by_key(|(_, p)| p.hops.len())
        {
            let mut path = path.clone();
            path.hops.push(Hop {
                from: from.into(),
                to: to.into(),
                relation: relation.into(),
                channel: None,
            });
            if self
                .paths
                .get(to)
                .is_none_or(|p| path.hops.len() < p.hops.len())
            {
                self.paths.insert(to.into(), path);
            }
        }
    }
    pub fn diagnostic(&self, causes: Vec<Cause>, relations: &[String]) -> Diagnostic {
        let origins: Vec<_> = self
            .paths
            .values()
            .filter(|p| p.hops.is_empty())
            .map(|p| p.origin.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let mut traces = vec![];
        for cause in causes {
            let colors = cause.colors();
            let candidates: Vec<_> = colors
                .iter()
                .filter_map(|id| self.active.get(id).or_else(|| self.paths.get(id)))
                .collect();
            if candidates.is_empty() {
                traces.push(Trace {
                    origin: origins.first().cloned().unwrap_or_default(),
                    hops: vec![],
                    cause,
                });
            } else {
                for path in candidates {
                    let trace = Trace {
                        origin: path.origin.clone(),
                        hops: path.hops.clone(),
                        cause: cause.clone(),
                    };
                    if !traces.contains(&trace) {
                        traces.push(trace);
                    }
                }
            }
        }
        if traces.is_empty() {
            traces.push(Trace {
                origin: origins.first().cloned().unwrap_or_default(),
                hops: vec![],
                cause: Cause::Incompatible {
                    colors: vec![],
                    relations: relations.into(),
                    cycle: false,
                },
            });
        }
        Diagnostic {
            operation: Operation::Edit,
            origins,
            channel: None,
            direction: 0,
            requested: self.requested.clone(),
            traces,
            colors: BTreeMap::new(),
            relations: BTreeMap::new(),
        }
    }
}

pub fn problems(
    p: &Project,
    requested: &State,
    relations: &[String],
    operation: Operation,
) -> Diagnostic {
    let mut trail = Trail::new(p, requested);
    let edges = crate::solver::edges(p);
    // Only the violated constraints form evidence for this fallback (e.g. a new contrast).
    let members: BTreeSet<_> = edges
        .iter()
        .filter(|e| relations.contains(&e.id))
        .flat_map(|e| [e.a.clone(), e.b.clone()])
        .collect();
    for c in 0..6 {
        trail.component(&edges, &members, relations, c);
    }
    let mut causes = vec![];
    for id in relations {
        if let Some(l) = p.limits.iter().find(|l| &l.id == id) {
            if let Some((a, b)) = p.item(&l.a).zip(p.item(&l.b)) {
                causes.push(Cause::Contrast {
                    relation: id.clone(),
                    a: l.a.clone(),
                    b: l.b.clone(),
                    minimum: l.minimum,
                    actual: color::contrast(a.rgb, b.rgb),
                });
            }
        } else if let Some(g) = p.gradients.iter().find(|g| &g.id == id) {
            for color in g.ids.iter().filter(|id| !g.detached.contains(id)) {
                trail.gradient(&g.a, &g.b, color, &g.id);
            }
            causes.push(Cause::Incompatible {
                colors: g.ids.clone(),
                relations: vec![id.clone()],
                cycle: false,
            });
        }
    }
    if causes.is_empty() {
        causes.push(Cause::Incompatible {
            colors: members.into_iter().collect(),
            relations: relations.into(),
            cycle: false,
        });
    }
    let mut diagnostic = trail.diagnostic(causes, relations);
    diagnostic.operation = operation;
    diagnostic.annotate(p);
    diagnostic
}
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use crate::{
        Constraint, Harmony, Item, Limit,
        engine::{Command, CompositionDraft, Engine},
    };
    pub fn chain_project() -> Project {
        let mut p = Project::default();
        for (id, h, s, v) in [
            ("a", 10.0, 0.5, 0.8),
            ("b", 90.0, 0.5, 0.8),
            ("c", 120.0, 0.1, 0.7),
            ("unrelated", 240.0, 0.3, 0.8),
        ] {
            let mut i = Item::new("color", None);
            i.id = id.into();
            i.name = id.into();
            i.rgb = color::from_hsv([h, s, v]);
            p.items.push(i);
        }
        let mut engine = Engine::new(p);
        engine
            .execute(Command::AddLink {
                a: "a".into(),
                b: "b".into(),
                channels: vec![4],
            })
            .unwrap();
        let mut p = engine.project;
        p.links[0].id = "saturation".into();
        p.harmonies.push(Harmony {
            id: "analogy".into(),
            kind: "analogous".into(),
            ids: vec!["b".into(), "c".into()],
            constraints: vec![Constraint {
                a: "b".into(),
                b: "c".into(),
                offset: [30.0, -0.4, -0.1],
            }],
            ..Default::default()
        });
        p.validate().unwrap();
        assert!(crate::solver::problems(&p, &crate::solver::state(&p), None).is_empty());
        p
    }
    #[test]
    fn multiple_equal_channel_boundaries_and_locked_peers_have_separate_traces() {
        let mut p = chain_project();
        let mut i = p.item("c").unwrap().clone();
        i.id = "d".into();
        p.items.push(i);
        p.harmonies[0].ids.push("d".into());
        p.harmonies[0].constraints.push(Constraint {
            a: "b".into(),
            b: "d".into(),
            offset: [30.0, -0.4, -0.1],
        });
        let requested = BTreeMap::from([("a".into(), color::from_hsv([10.0, 0.2, 0.8]))]);
        let d = crate::solver::solve(&p, &requested)
            .unwrap_err()
            .diagnostic
            .unwrap();
        let bounds: BTreeSet<_> = d
            .traces
            .iter()
            .filter_map(|t| {
                if let Cause::ChannelBound {
                    color,
                    channel: 4,
                    maximum: false,
                } = &t.cause
                {
                    Some(color.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(bounds, BTreeSet::from(["c", "d"]));
        p.item_mut("c").unwrap().locked = true;
        p.item_mut("d").unwrap().locked = true;
        let d = crate::solver::solve(&p, &requested)
            .unwrap_err()
            .diagnostic
            .unwrap();
        let locked: BTreeSet<_> = d
            .traces
            .iter()
            .filter_map(|t| {
                if let Cause::Protected { color, .. } = &t.cause {
                    Some(color.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(locked, BTreeSet::from(["c", "d"]));
    }
    #[test]
    fn gradient_propagation_to_multiple_contrast_limits_keeps_the_dependency_hop() {
        let mut p = Project::default();
        for (id, v) in [("a", 0.0), ("b", 255.0), ("white", 255.0)] {
            let mut i = Item::new("color", None);
            i.id = id.into();
            i.rgb = [v; 3];
            p.items.push(i);
        }
        let mut e = Engine::new(p);
        e.execute(Command::Compose(CompositionDraft {
            kind: "gradient".into(),
            originals: vec!["a".into(), "b".into()],
            count: 1,
            space: "srgb".into(),
            ..Default::default()
        }))
        .unwrap();
        let g = e.project.gradients[0].clone();
        let mid = g.ids[0].clone();
        e.execute(Command::AddContrast {
            a: mid.clone(),
            b: "b".into(),
            minimum: 3.0,
        })
        .unwrap();
        e.execute(Command::AddContrast {
            a: mid.clone(),
            b: "white".into(),
            minimum: 3.0,
        })
        .unwrap();
        let before = e.project.clone();
        assert!(e.preview_color("a", [120.0; 3], false).is_err());
        let d = e.last_diagnostic.unwrap();
        let ids: BTreeSet<_> = d
            .traces
            .iter()
            .filter_map(|t| {
                if let Cause::Contrast { relation, .. } = &t.cause {
                    Some(relation.clone())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(ids.len(), 2);
        assert!(
            d.traces
                .iter()
                .filter(|t| matches!(t.cause, Cause::Contrast { .. }))
                .all(|t| t.hops.iter().any(|h| h.relation == g.id && h.to == mid))
        );
        assert_eq!(e.project, before);
    }
    #[test]
    fn continuous_clipping_keeps_a_boundary_diagnostic_without_invalidating_the_result() {
        let mut e = Engine::new(chain_project());
        e.preview_color("a", color::from_hsv([10.0, 0.2, 0.8]), true)
            .unwrap();
        let d = e.last_diagnostic.unwrap();
        assert!(d.traces.iter().any(|t|matches!(&t.cause,Cause::ChannelBound{color,channel:4,maximum:false}if color=="c")),"{d:?}");
        assert!(
            crate::solver::problems(&e.project, &crate::solver::state(&e.project), None).is_empty()
        );
    }

    #[test]
    fn saturation_harmony_proof_ends_at_the_actual_minimum_and_excludes_unrelated_color() {
        let p = chain_project();
        let mut e = Engine::new(p.clone());
        assert!(
            e.preview_color("a", color::from_hsv([10.0, 0.2, 0.8]), false)
                .is_err()
        );
        let d = e.last_diagnostic.as_ref().unwrap();
        assert_eq!(d.channel, Some(4));
        assert_eq!(d.direction, -1);
        let t=d.traces.iter().find(|t|matches!(&t.cause,Cause::ChannelBound{color,channel:4,maximum:false}if color=="c")).unwrap();
        assert_eq!(t.origin, "a");
        assert_eq!(
            t.hops
                .iter()
                .map(|h| h.relation.as_str())
                .collect::<Vec<_>>(),
            vec!["saturation", "analogy"]
        );
        assert_eq!(t.hops.last().unwrap().to, "c");
        assert!(t.hops.iter().all(|h| h.channel == Some(4)));
        assert!(!d.colors.contains_key("unrelated"));
        assert_eq!(e.project, p);
    }
    #[test]
    fn locked_peer_reports_same_proof_and_distinguishes_source_protection() {
        for source in [false, true] {
            let mut p = chain_project();
            let c = p.item_mut("c").unwrap();
            c.locked = !source;
            c.source_locked = source;
            // Solver works on effective source protection; persistence consistency is tested elsewhere.
            let mut e = Engine::new(p.clone());
            assert!(
                e.preview_color("a", color::from_hsv([10.0, 0.6, 0.8]), false)
                    .is_err()
            );
            let d = e.last_diagnostic.as_ref().unwrap();
            let t = d
                .traces
                .iter()
                .find(|t| matches!(&t.cause,Cause::Protected{color,..}if color=="c"))
                .unwrap();
            assert_eq!(
                t.hops
                    .iter()
                    .map(|h| h.relation.as_str())
                    .collect::<Vec<_>>(),
                vec!["saturation", "analogy"]
            );
            assert!(
                matches!(&t.cause,Cause::Protected{protection,..}if *protection==if source{Protection::Source}else{Protection::Manual})
            );
            assert_eq!(e.project, p);
        }
    }
    #[test]
    fn direct_lock_gradient_and_maximum_have_typed_causes() {
        let mut p = chain_project();
        p.item_mut("a").unwrap().direct_locked = true;
        let mut e = Engine::new(p);
        assert!(e.preview_color("a", [80.0, 90.0, 100.0], false).is_err());
        assert!(matches!(
            e.last_diagnostic.unwrap().traces[0].cause,
            Cause::Protected {
                protection: Protection::Direct,
                ..
            }
        ));
        let mut p = Project::default();
        for (id, r) in [("a", 100.0), ("b", 200.0)] {
            let mut i = Item::new("color", None);
            i.id = id.into();
            i.rgb = [r, 80.0, 100.0];
            p.items.push(i);
        }
        let mut e = Engine::new(p);
        e.execute(Command::LinkColors {
            ids: vec!["a".into(), "b".into()],
            channels: vec![0],
        })
        .unwrap();
        assert!(e.preview_color("a", [200.0, 80.0, 100.0], false).is_err());
        assert!(e.last_diagnostic.as_ref().unwrap().traces.iter().any(
            |t| matches!(&t.cause,Cause::ChannelBound{color,channel:0,maximum:true}if color=="b")
        ));
        e.execute(Command::Compose(CompositionDraft {
            kind: "gradient".into(),
            originals: vec!["a".into(), "b".into()],
            count: 2,
            ..Default::default()
        }))
        .unwrap();
        let id = e.project.gradients[0].ids[0].clone();
        assert!(e.preview_color(&id, [80.0, 90.0, 100.0], false).is_err());
        assert!(matches!(
            e.last_diagnostic.as_ref().unwrap().traces[0].cause,
            Cause::Gradient { .. }
        ));
    }
    #[test]
    fn contrast_and_collective_cycles_retain_real_relation_ids() {
        let mut p = Project::default();
        for (id, rgb) in [("a", [0.0; 3]), ("b", [255.0; 3])] {
            let mut i = Item::new("color", None);
            i.id = id.into();
            i.rgb = rgb;
            p.items.push(i);
        }
        p.limits.push(Limit {
            id: "contrast".into(),
            a: "a".into(),
            b: "b".into(),
            minimum: 4.5,
            metric: "wcag22-srgb".into(),
        });
        let mut e = Engine::new(p);
        e.preview_color("a", [255.0; 3], true).unwrap();
        let d = e.last_diagnostic.as_ref().unwrap();
        assert!(
            d.traces
                .iter()
                .any(|t| matches!(&t.cause,Cause::Contrast{relation,..}if relation=="contrast"))
        );
        assert!(
            crate::solver::problems(&e.project, &crate::solver::state(&e.project), None).is_empty()
        );
        let mut p = chain_project();
        let mut edge = p.links[0].clone();
        edge.id = "bad-cycle".into();
        edge.relations.get_mut("4").unwrap().mode = "offset".into();
        edge.relations.get_mut("4").unwrap().value = 0.2;
        p.links.push(edge);
        let d = crate::solver::solve(
            &p,
            &BTreeMap::from([("a".into(), color::from_hsv([10.0, 0.6, 0.8]))]),
        )
        .unwrap_err()
        .diagnostic
        .unwrap();
        assert!(d.traces.iter().any(|t|matches!(&t.cause,Cause::Incompatible{cycle:true,relations,..}if relations.contains(&"bad-cycle".to_string()))));
    }
    #[test]
    fn diagnostic_metadata_is_read_only_and_nonpersistent() {
        let p = chain_project();
        let bytes = serde_json::to_vec(&p).unwrap();
        let mut e = Engine::new(p);
        assert!(
            e.preview_color("a", color::from_hsv([10.0, 0.2, 0.8]), false)
                .is_err()
        );
        assert_eq!(serde_json::to_vec(&e.project).unwrap(), bytes);
        assert!(e.undo.is_empty());
    }
}
