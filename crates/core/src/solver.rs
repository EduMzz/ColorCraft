//! Affine channel constraints, compatible cycles and alternating RGB/HSV projections.
use crate::diagnostics::{Cause, Diagnostic, Protection, Trail};
use crate::{
    Link, Project, Relation,
    color::{self, Rgb},
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
pub type State = BTreeMap<String, Rgb>;
#[derive(Debug, Clone)]
pub struct Blocked {
    pub relations: Vec<String>,
    pub message: String,
    pub diagnostic: Option<Box<Diagnostic>>,
    causes: Vec<Cause>,
}
impl Blocked {
    fn because(mut self, cause: Cause) -> Self {
        self.causes.push(cause);
        self
    }
    fn new(relations: Vec<String>, message: &str) -> Self {
        Self {
            relations,
            message: message.into(),
            diagnostic: None,
            causes: vec![],
        }
    }
}
pub fn edges(p: &Project) -> Vec<Link> {
    let mut out = p.links.clone();
    for h in &p.harmonies {
        for e in &h.constraints {
            out.push(Link {
                id: h.id.clone(),
                a: e.a.clone(),
                b: e.b.clone(),
                channels: vec![3, 4, 5],
                relations: e
                    .offset
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        (
                            (i + 3).to_string(),
                            Relation {
                                mode: "offset".into(),
                                value: *v,
                            },
                        )
                    })
                    .collect(),
            })
        }
    }
    out
}
pub fn state(p: &Project) -> State {
    p.colors().map(|i| (i.id.clone(), i.rgb)).collect()
}
pub fn problems(p: &Project, s: &State, scope: Option<&BTreeSet<String>>) -> Vec<String> {
    let mut out = BTreeSet::new();
    for e in edges(p) {
        if scope.is_some_and(|sc| !sc.contains(&e.a) && !sc.contains(&e.b)) {
            continue;
        }
        if let (Some(a), Some(b)) = (s.get(&e.a), s.get(&e.b)) {
            for c in &e.channels {
                if let Some(r) = e.relations.get(&c.to_string()) {
                    let av = color::channel(*a, *c);
                    let expected = if r.mode == "ratio" {
                        av * r.value
                    } else {
                        av + r.value
                    };
                    if !expected.is_finite()
                        || color::channel_distance(expected, color::channel(*b, *c), *c)
                            > if *c < 4 { 0.0002 } else { 0.000002 }
                    {
                        out.insert(e.id.clone());
                    }
                }
            }
        }
    }
    for g in &p.gradients {
        if scope.is_some_and(|sc| {
            !sc.contains(&g.a)
                && !sc.contains(&g.b)
                && !g
                    .ids
                    .iter()
                    .any(|id| sc.contains(id) && !g.detached.contains(id))
        }) {
            continue;
        }
        if let (Some(a), Some(b)) = (s.get(&g.a), s.get(&g.b)) {
            for (i, id) in g.ids.iter().enumerate() {
                if g.detached.contains(id) {
                    continue;
                }
                if let Some(rgb) = s.get(id) {
                    let expected = color::interpolate(
                        *a,
                        *b,
                        (i + 1) as f64 / (g.ids.len() + 1) as f64,
                        &g.space,
                    );
                    if expected
                        .iter()
                        .zip(rgb)
                        .any(|(a, b)| (a - b).abs() > 0.0002)
                    {
                        out.insert(g.id.clone());
                    }
                }
            }
        }
    }
    for l in &p.limits {
        if scope.is_some_and(|sc| !sc.contains(&l.a) && !sc.contains(&l.b)) {
            continue;
        }
        if let (Some(a), Some(b)) = (s.get(&l.a), s.get(&l.b))
            && color::contrast(*a, *b) + 1e-10 < l.minimum
        {
            out.insert(l.id.clone());
        }
    }
    out.into_iter().collect()
}
#[derive(Clone)]
struct ArcEdge {
    relation: String,
    to: String,
    m: f64,
    k: f64,
}
struct Component {
    channel: usize,
    root: String,
    transforms: BTreeMap<String, (f64, f64)>,
    fixed: Option<f64>,
    lo: f64,
    hi: f64,
    lo_owners: Vec<(String, bool)>,
    hi_owners: Vec<(String, bool)>,
    relations: Vec<String>,
}
pub fn solve(p: &Project, requested: &State) -> std::result::Result<State, Blocked> {
    let mut trail = Trail::new(p, requested);
    solve_impl(p, requested, &mut trail).map_err(|mut error| {
        let mut d = trail.diagnostic(error.causes.clone(), &error.relations);
        d.annotate(p);
        error.diagnostic = Some(Box::new(d));
        error
    })
}
fn solve_impl(
    p: &Project,
    requested: &State,
    trail: &mut Trail,
) -> std::result::Result<State, Blocked> {
    let mut s = state(p);
    let edges = edges(p);
    let mut touched: BTreeSet<String> = requested.keys().cloned().collect();
    for _ in 0..p.items.len() + 1 {
        let n = touched.len();
        for e in &edges {
            if touched.contains(&e.a) || touched.contains(&e.b) {
                touched.insert(e.a.clone());
                touched.insert(e.b.clone());
            }
        }
        for g in &p.gradients {
            let ids = std::iter::once(&g.a)
                .chain(std::iter::once(&g.b))
                .chain(g.ids.iter().filter(|id| !g.detached.contains(id)));
            let ids: Vec<_> = ids.cloned().collect();
            if ids.iter().any(|id| touched.contains(id)) {
                touched.extend(ids);
            }
        }
        if touched.len() == n {
            break;
        }
    }
    let mut pins = requested.clone();
    for i in p.colors().filter(|i| touched.contains(&i.id)) {
        if !i.is_srgb() {
            return Err(Blocked::new(
                vec![],
                "Uma relação técnica envolve cor fora de sRGB. Converta explicitamente a cor para sRGB antes de vinculá-la.",
            ).because(Cause::NativeSpace { color: i.id.clone() }));
        }
        if i.locked || i.source_locked {
            if requested.get(&i.id).is_some_and(|rgb| rgb != &i.rgb) {
                return Err(Blocked::new(vec![], "A cor está bloqueada.").because(
                    Cause::Protected {
                        color: i.id.clone(),
                        protection: if i.source_locked {
                            Protection::Source
                        } else {
                            Protection::Manual
                        },
                    },
                ));
            }
            pins.insert(i.id.clone(), i.rgb);
        }
    }
    for (id, rgb) in &pins {
        if !s.contains_key(id) || !color::in_gamut(*rgb) {
            let mut error = Blocked::new(vec![], "Valor RGB fora do intervalo permitido.");
            for (channel, value) in rgb.iter().enumerate() {
                if value.is_finite() && (*value < 0.0 || *value > 255.0) {
                    error = error.because(Cause::ChannelBound {
                        color: id.clone(),
                        channel,
                        maximum: *value > 255.0,
                    });
                }
            }
            return Err(error);
        }
    }
    let mut components = Vec::new();
    for c in 0..6 {
        let mut graph: BTreeMap<String, Vec<ArcEdge>> = BTreeMap::new();
        for e in &edges {
            if !touched.contains(&e.a) || !e.channels.contains(&c) {
                continue;
            }
            if let Some(r) = e.relations.get(&c.to_string()) {
                let m = if r.mode == "ratio" { r.value } else { 1.0 };
                let k = if r.mode == "ratio" { 0.0 } else { r.value };
                if !m.is_finite() || m.abs() < 1e-12 {
                    return Err(Blocked::new(
                        vec![e.id.clone()],
                        "Razão de vínculo inválida.",
                    ));
                }
                graph.entry(e.a.clone()).or_default().push(ArcEdge {
                    relation: e.id.clone(),
                    to: e.b.clone(),
                    m,
                    k,
                });
                graph.entry(e.b.clone()).or_default().push(ArcEdge {
                    relation: e.id.clone(),
                    to: e.a.clone(),
                    m: 1.0 / m,
                    k: -k / m,
                });
            }
        }
        let mut visited = BTreeSet::new();
        for root in graph.keys() {
            if visited.contains(root) {
                continue;
            }
            let mut tr: BTreeMap<String, (f64, f64)> = BTreeMap::from([(root.clone(), (1.0, 0.0))]);
            let mut queue = VecDeque::from([root.clone()]);
            let mut fixed: Option<f64> = None;
            let mut lo = f64::NEG_INFINITY;
            let mut hi = f64::INFINITY;
            let mut lo_owners = vec![];
            let mut hi_owners = vec![];
            let mut tree_paths: BTreeMap<String, Vec<String>> =
                BTreeMap::from([(root.clone(), vec![])]);
            let mut cycle_relations = BTreeSet::new();
            let mut bad = false;
            while let Some(id) = queue.pop_front() {
                visited.insert(id.clone());
                let (m, k) = tr.get(&id).copied().unwrap_or((1.0, 0.0));
                if c != 3 {
                    let max = if c < 3 { 255.0 } else { 1.0 };
                    let lower = (-k / m).min((max - k) / m);
                    let upper = (-k / m).max((max - k) / m);
                    if lower > lo {
                        lo_owners.clear();
                        lo_owners.push((id.clone(), m < 0.0));
                    } else if (lower - lo).abs() < 1e-10 {
                        lo_owners.push((id.clone(), m < 0.0));
                    }
                    if upper < hi {
                        hi_owners.clear();
                        hi_owners.push((id.clone(), m > 0.0));
                    } else if (upper - hi).abs() < 1e-10 {
                        hi_owners.push((id.clone(), m > 0.0));
                    }
                    lo = lo.max(lower);
                    hi = hi.min(upper);
                }
                for e in graph.get(&id).into_iter().flatten() {
                    let next = (e.m * m, e.m * k + e.k);
                    if let Some(prev) = tr.get(&e.to) {
                        let coef = prev.0 - next.0;
                        let constant = next.1 - prev.1;
                        if coef.abs() >= 1e-9 || color::channel_distance(constant, 0.0, c) > 1e-6 {
                            let a = &tree_paths[&id];
                            let b = &tree_paths[&e.to];
                            let prefix = a.iter().zip(b).take_while(|(x, y)| x == y).count();
                            cycle_relations.extend(a[prefix..].iter().cloned());
                            cycle_relations.extend(b[prefix..].iter().cloned());
                            cycle_relations.insert(e.relation.clone());
                        }
                        if coef.abs() < 1e-9 {
                            if color::channel_distance(constant, 0.0, c) > 1e-6 {
                                bad = true
                            }
                        } else {
                            let candidate = constant / coef;
                            if fixed.is_some_and(|f| (f - candidate).abs() > 1e-6) {
                                bad = true
                            }
                            fixed = Some(candidate);
                        }
                    } else {
                        tr.insert(e.to.clone(), next);
                        let mut path = tree_paths[&id].clone();
                        path.push(e.relation.clone());
                        tree_paths.insert(e.to.clone(), path);
                        queue.push_back(e.to.clone());
                    }
                }
            }
            let rels: Vec<_> = edges
                .iter()
                .filter(|e| e.channels.contains(&c) && tr.contains_key(&e.a))
                .map(|e| e.id.clone())
                .collect();
            let members = tr.keys().cloned().collect();
            trail.component(&edges, &members, &rels, c);
            if bad {
                return Err(
                    Blocked::new(rels.clone(), "Ciclo incompatível entre relações.").because(
                        Cause::Incompatible {
                            colors: edges
                                .iter()
                                .filter(|e| cycle_relations.contains(&e.id))
                                .flat_map(|e| [e.a.clone(), e.b.clone()])
                                .collect::<BTreeSet<_>>()
                                .into_iter()
                                .collect(),
                            relations: cycle_relations.into_iter().collect(),
                            cycle: true,
                        },
                    ),
                );
            }
            components.push(Component {
                channel: c,
                root: root.clone(),
                transforms: tr,
                fixed,
                lo,
                hi,
                lo_owners,
                hi_owners,
                relations: rels,
            });
        }
    }
    for (id, rgb) in &pins {
        s.insert(id.clone(), *rgb);
    }
    for _ in 0..180 {
        for comp in &components {
            let c = comp.channel;
            let reference = s
                .get(&comp.root)
                .map(|rgb| color::channel(*rgb, c))
                .unwrap_or(0.0);
            let mut pin = None;
            let mut pin_id: Option<String> = None;
            trail.component(
                &edges,
                &comp.transforms.keys().cloned().collect(),
                &comp.relations,
                c,
            );
            let mut num = 0.0;
            let mut den = 0.0;
            for (id, (m, k)) in &comp.transforms {
                if let Some(rgb) = s.get(id) {
                    let mut v = (color::channel(*rgb, c) - k) / m;
                    if c == 3 {
                        v = reference + color::hue_delta(v, reference)
                    }
                    num += m * m * v;
                    den += m * m;
                }
                if let Some(rgb) = pins.get(id) {
                    let desired = (color::channel(*rgb, c) - k) / m;
                    if pin.is_some_and(|pv| color::channel_distance(pv, desired, c) > 1e-5) {
                        let fixed = [pin_id.clone().unwrap_or_default(), id.clone()];
                        let requested_pin = comp.transforms.iter().find_map(|(id, (m, k))| {
                            requested
                                .get(id)
                                .filter(|rgb| p.item(id).is_some_and(|i| i.rgb != **rgb))
                                .map(|rgb| (color::channel(*rgb, c) - k) / m)
                        });
                        let causes: Vec<_> = comp
                            .transforms
                            .iter()
                            .filter_map(|(id, (m, k))| {
                                let i = p.item(id)?;
                                let rgb = pins.get(id)?;
                                if (i.locked || i.source_locked)
                                    && requested_pin.is_some_and(|v| {
                                        color::channel_distance(
                                            v,
                                            (color::channel(*rgb, c) - k) / m,
                                            c,
                                        ) > 1e-5
                                    })
                                {
                                    Some(Cause::Protected {
                                        color: id.clone(),
                                        protection: if i.source_locked {
                                            Protection::Source
                                        } else {
                                            Protection::Manual
                                        },
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect();
                        let mut error = Blocked::new(
                            comp.relations.clone(),
                            "Os valores fixos entram em conflito.",
                        );
                        if causes.is_empty() {
                            error = error.because(Cause::Incompatible {
                                colors: fixed.into(),
                                relations: comp.relations.clone(),
                                cycle: false,
                            });
                        } else {
                            for cause in causes {
                                error = error.because(cause);
                            }
                        }
                        return Err(error);
                    }
                    pin = Some(desired);
                    pin_id = Some(id.clone());
                }
            }
            let mut value = pin
                .or(comp.fixed)
                .unwrap_or(if den == 0.0 { 0.0 } else { num / den });
            if let (Some(pv), Some(f)) = (pin, comp.fixed)
                && color::channel_distance(pv, f, c) > 1e-5
            {
                return Err(Blocked::new(
                    comp.relations.clone(),
                    "O ciclo fixa um valor diferente do solicitado.",
                )
                .because(Cause::Incompatible {
                    colors: comp.transforms.keys().cloned().collect(),
                    relations: comp.relations.clone(),
                    cycle: true,
                }));
            }
            if c != 3 {
                if comp.lo > comp.hi + 1e-8
                    || pin.is_some() && (value < comp.lo - 1e-8 || value > comp.hi + 1e-8)
                {
                    let mut error = Blocked::new(
                        comp.relations.clone(),
                        "O vínculo atingiu o limite de um canal.",
                    );
                    if comp.lo > comp.hi + 1e-8 || value < comp.lo - 1e-8 {
                        for (id, maximum) in &comp.lo_owners {
                            error = error.because(Cause::ChannelBound {
                                color: id.clone(),
                                channel: c,
                                maximum: *maximum,
                            });
                        }
                    }
                    if comp.lo > comp.hi + 1e-8 || value > comp.hi + 1e-8 {
                        for (id, maximum) in &comp.hi_owners {
                            error = error.because(Cause::ChannelBound {
                                color: id.clone(),
                                channel: c,
                                maximum: *maximum,
                            });
                        }
                    }
                    return Err(error);
                }
                value = value.clamp(comp.lo, comp.hi);
            }
            for (id, (m, k)) in &comp.transforms {
                let next = m * value + k;
                if let Some(rgb) = s.get_mut(id) {
                    if c < 3 {
                        if let Some(ch) = rgb.get_mut(c) {
                            *ch = next.clamp(0.0, 255.0)
                        }
                    } else {
                        let mut hv = color::hsv(*rgb);
                        if let Some(ch) = hv.get_mut(c - 3) {
                            *ch = if c == 3 {
                                color::wrap(next)
                            } else {
                                next.clamp(0.0, 1.0)
                            }
                        }
                        *rgb = color::from_hsv(hv);
                    }
                }
            }
        }
        for g in &p.gradients {
            if !touched.contains(&g.a) && !touched.contains(&g.b) {
                continue;
            }
            if let (Some(a), Some(b)) = (s.get(&g.a).copied(), s.get(&g.b).copied()) {
                for (i, id) in g.ids.iter().enumerate() {
                    if !g.detached.contains(id) {
                        trail.gradient(&g.a, &g.b, id, &g.id);
                        s.insert(
                            id.clone(),
                            color::interpolate(
                                a,
                                b,
                                (i + 1) as f64 / (g.ids.len() + 1) as f64,
                                &g.space,
                            ),
                        );
                    }
                }
            }
        }
        for (id, rgb) in &pins {
            s.insert(id.clone(), *rgb);
        }
        let bad = problems(p, &s, Some(&touched));
        if bad.is_empty() {
            return Ok(s);
        }
        // Inequalities do not move either participant to compensate.
        if bad.iter().any(|id| p.limits.iter().any(|l| &l.id == id)) {
            let mut error = Blocked::new(bad.clone(), "O contraste mínimo foi atingido.");
            for l in p.limits.iter().filter(|l| bad.contains(&l.id)) {
                if let Some((a, b)) = s.get(&l.a).zip(s.get(&l.b)) {
                    error = error.because(Cause::Contrast {
                        relation: l.id.clone(),
                        a: l.a.clone(),
                        b: l.b.clone(),
                        minimum: l.minimum,
                        actual: color::contrast(*a, *b),
                    });
                }
            }
            return Err(error);
        }
    }
    Err(Blocked::new(
        problems(p, &s, Some(&touched)),
        "As relações não admitem esta alteração.",
    ))
}
pub fn apply_state(p: &mut Project, s: State) {
    for i in &mut p.items {
        if let Some(rgb) = s.get(&i.id)
            && i.is_srgb()
        {
            i.set_rgb(*rgb)
        }
    }
}
pub fn edit(p: &mut Project, id: &str, target: Rgb, continuous: bool) -> crate::Result<bool> {
    edit_diagnosed(p, id, target, continuous, &mut None)
}
pub fn edit_diagnosed(
    p: &mut Project,
    id: &str,
    target: Rgb,
    continuous: bool,
    diagnostic: &mut Option<Diagnostic>,
) -> crate::Result<bool> {
    *diagnostic = None;
    let item = p.item(id).ok_or("Cor ausente")?;
    if item.locked || item.source_locked {
        *diagnostic = Some(Diagnostic::direct(
            p,
            id,
            Cause::Protected {
                color: id.into(),
                protection: if item.source_locked {
                    Protection::Source
                } else {
                    Protection::Manual
                },
            },
        ));
        if let Some(d) = diagnostic {
            d.edit_intent(item.rgb, target);
        }
        return Err("Esta cor está bloqueada.".into());
    }
    if item.direct_locked {
        *diagnostic = Some(Diagnostic::direct(
            p,
            id,
            Cause::Protected {
                color: id.into(),
                protection: Protection::Direct,
            },
        ));
        if let Some(d) = diagnostic {
            d.edit_intent(item.rgb, target);
        }
        return Err("A edição direta desta cor está bloqueada. Ela ainda pode acompanhar as cores relacionadas.".into());
    }
    if !item.is_srgb() {
        *diagnostic = Some(Diagnostic::direct(
            p,
            id,
            Cause::NativeSpace { color: id.into() },
        ));
        return Err(
            "Converta a cor explicitamente para sRGB para usar a roda e os canais RGB/HSV.".into(),
        );
    }
    if let Some(g) = p
        .gradients
        .iter()
        .find(|g| g.ids.iter().any(|x| x == id) && !g.detached.iter().any(|x| x == id))
    {
        *diagnostic = Some(Diagnostic::direct(
            p,
            id,
            Cause::Gradient {
                color: id.into(),
                relation: g.id.clone(),
            },
        ));
        if let Some(d) = diagnostic {
            d.edit_intent(item.rgb, target);
        }
        return Err("Esta cor é controlada por um gradiente. Destaque-a antes de editar.".into());
    }
    let original = item.rgb;
    let pins = |rgb| BTreeMap::from([(id.to_string(), rgb)]);
    if !continuous {
        let s = solve(p, &pins(target)).map_err(|e| {
            *diagnostic = e.diagnostic.map(|d| *d);
            if let Some(d) = diagnostic {
                d.edit_intent(original, target);
            }
            format!("{} {}", e.message, e.relations.join(", "))
        })?;
        apply_state(p, s);
        return Ok(false);
    }
    // Follow the edited path rather than accepting another disconnected valid region.
    let steps = if p.limits.is_empty() { 1 } else { 128 };
    let mut good = 0.0;
    let mut best = None;
    let mut bad_at = None;
    let mut rejected = None;
    for step in 1..=steps {
        let t = step as f64 / steps as f64;
        let rgb = std::array::from_fn(|c| original[c] + (target[c] - original[c]) * t);
        match solve(p, &pins(rgb)) {
            Ok(s) => {
                best = Some(s);
                good = t
            }
            Err(error) => {
                rejected = error.diagnostic.map(|d| *d);
                bad_at = Some(t);
                break;
            }
        }
    }
    if let Some(mut hi) = bad_at {
        let mut lo = good;
        for _ in 0..27 {
            let t = (lo + hi) / 2.0;
            let rgb = std::array::from_fn(|c| original[c] + (target[c] - original[c]) * t);
            match solve(p, &pins(rgb)) {
                Ok(s) => {
                    lo = t;
                    best = Some(s)
                }
                Err(error) => {
                    rejected = error.diagnostic.map(|d| *d);
                    hi = t;
                }
            }
        }
        if let Some(s) = best {
            apply_state(p, s)
        }
        *diagnostic = rejected;
        if let Some(d) = diagnostic {
            d.edit_intent(original, target);
        }
        return Ok(true);
    }
    if let Some(s) = best {
        apply_state(p, s)
    }
    Ok(false)
}
