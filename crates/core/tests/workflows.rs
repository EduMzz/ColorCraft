#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use colorcraft_core::{
    color,
    engine::{Command, CompositionDraft, Engine, Replacement},
    export::{self, Format},
    solver, *,
};
use std::collections::BTreeMap;
fn palette() -> Project {
    Project {
        items: vec![
            Item {
                id: "a".into(),
                name: "A".into(),
                rgb: [80.0, 100.0, 120.0],
                ..Item::default()
            },
            Item {
                id: "b".into(),
                name: "B".into(),
                rgb: [160.0, 200.0, 240.0],
                ..Item::default()
            },
        ],
        ..Project::default()
    }
}
#[test]
fn direct_lock_blocks_controls_but_not_relationship_propagation() {
    let mut e = Engine::new(palette());
    link(&mut e, "a", "b", vec![0, 1, 2]);
    e.execute(Command::DirectLock {
        id: "b".into(),
        locked: true,
    })
    .unwrap();
    let before = e.project.clone();
    for continuous in [false, true] {
        assert!(
            e.execute(Command::SetColor {
                id: "b".into(),
                rgb: [100.0; 3],
                continuous
            })
            .is_err()
        );
        assert_eq!(e.project, before);
    }
    assert!(
        e.execute(Command::SetNative {
            id: "b".into(),
            value: NativeColor::srgb([100.0; 3])
        })
        .is_err()
    );
    assert_eq!(e.project, before);
    e.execute(Command::SetColor {
        id: "a".into(),
        rgb: [40.0, 50.0, 60.0],
        continuous: false,
    })
    .unwrap();
    assert_eq!(e.project.item("b").unwrap().rgb, [80.0, 100.0, 120.0]);
    assert!(e.project.item("b").unwrap().direct_locked);
    e.undo();
    assert_eq!(e.project, before);
    e.execute(Command::Lock {
        id: "b".into(),
        locked: true,
    })
    .unwrap();
    let fixed = e.project.clone();
    assert!(
        e.execute(Command::SetColor {
            id: "a".into(),
            rgb: [40.0, 50.0, 60.0],
            continuous: false
        })
        .is_err()
    );
    assert_eq!(e.project, fixed);
    e.execute(Command::Lock {
        id: "b".into(),
        locked: false,
    })
    .unwrap();
    assert!(e.project.item("b").unwrap().direct_locked);
    e.execute(Command::DirectLock {
        id: "b".into(),
        locked: false,
    })
    .unwrap();
    e.execute(Command::SetColor {
        id: "b".into(),
        rgb: [80.0, 100.0, 120.0],
        continuous: false,
    })
    .unwrap();
    assert_eq!(e.project.item("a").unwrap().rgb, [40.0, 50.0, 60.0]);
}

#[test]
fn direct_lock_roundtrips_is_optional_and_does_not_leak_into_consumption_snapshot() {
    let mut e = Engine::new(palette());
    let legacy = serde_json::to_vec(&e.project).unwrap();
    assert!(
        !String::from_utf8(legacy.clone())
            .unwrap()
            .contains("directLocked")
    );
    assert!(
        !Project::from_json(&legacy)
            .unwrap()
            .item("a")
            .unwrap()
            .direct_locked
    );
    e.execute(Command::DirectLock {
        id: "a".into(),
        locked: true,
    })
    .unwrap();
    let saved = e.project.clone();
    let json = serde_json::to_vec(&saved).unwrap();
    assert_eq!(Project::from_json(&json).unwrap(), saved);
    let snapshot = export::palette(&saved, false).unwrap();
    assert!(snapshot["items"][0].get("directLocked").is_none());
    e.undo();
    assert!(!e.project.item("a").unwrap().direct_locked);
    e.redo();
    assert_eq!(e.project, saved);
    let mut metadata = e.project.item("a").unwrap().clone();
    metadata.name = "Renomeada".into();
    e.execute(Command::Metadata(metadata)).unwrap();
    assert!(e.project.item("a").unwrap().direct_locked);
    e.execute(Command::Duplicate(vec!["a".into()])).unwrap();
    assert!(e.project.items.last().unwrap().direct_locked);
}

#[test]
fn composition_requires_explicit_direct_unlock_and_undo_restores_it() {
    let mut e = Engine::new(palette());
    e.execute(Command::DirectLock {
        id: "b".into(),
        locked: true,
    })
    .unwrap();
    let old = e.project.clone();
    let mut draft = CompositionDraft {
        kind: "complementary".into(),
        originals: vec!["a".into()],
        ..Default::default()
    };
    let slots = colorcraft_core::engine::composition_preview(&old, &draft)
        .unwrap()
        .0;
    let position = slots.iter().position(|s| s.original.is_none()).unwrap();
    draft.replacements.insert(
        position,
        Replacement {
            id: "b".into(),
            ..Default::default()
        },
    );
    assert!(e.execute(Command::Compose(draft.clone())).is_err());
    assert_eq!(e.project, old);
    draft.replacements.get_mut(&position).unwrap().unlock = true;
    e.execute(Command::Compose(draft)).unwrap();
    assert!(!e.project.item("b").unwrap().direct_locked);
    e.undo();
    assert_eq!(e.project, old);
}

#[test]
fn directly_protected_harmony_and_gradient_colours_still_follow_origins() {
    for kind in ["gradient", "complementary"] {
        let mut e = Engine::new(palette());
        e.execute(Command::Compose(CompositionDraft {
            kind: kind.into(),
            originals: if kind == "gradient" {
                vec!["a".into(), "b".into()]
            } else {
                vec!["a".into()]
            },
            count: 3,
            ..Default::default()
        }))
        .unwrap();
        let dependent = if kind == "gradient" {
            e.project.gradients[0].ids[0].clone()
        } else {
            e.project.harmonies[0]
                .ids
                .iter()
                .find(|id| id.as_str() != "a")
                .unwrap()
                .clone()
        };
        e.execute(Command::DirectLock {
            id: dependent.clone(),
            locked: true,
        })
        .unwrap();
        let before = e.project.item(&dependent).unwrap().rgb;
        e.execute(Command::SetColor {
            id: "a".into(),
            rgb: [40.0, 50.0, 60.0],
            continuous: false,
        })
        .unwrap();
        assert_ne!(e.project.item(&dependent).unwrap().rgb, before);
        assert!(e.project.item(&dependent).unwrap().direct_locked);
        assert!(solver::problems(&e.project, &solver::state(&e.project), None).is_empty());
    }
}
fn link(engine: &mut Engine, a: &str, b: &str, channels: Vec<usize>) {
    engine
        .execute(Command::AddLink {
            a: a.into(),
            b: b.into(),
            channels,
        })
        .unwrap();
}
#[test]
fn rgb_links_are_bidirectional_and_preserve_ratios() {
    let mut e = Engine::new(palette());
    link(&mut e, "a", "b", vec![0, 1, 2]);
    e.execute(Command::SetColor {
        id: "b".into(),
        rgb: [120.0, 160.0, 200.0],
        continuous: false,
    })
    .unwrap();
    assert_eq!(e.project.item("a").unwrap().rgb, [60.0, 80.0, 100.0]);
    assert!(solver::problems(&e.project, &solver::state(&e.project), None).is_empty());
}
#[test]
fn first_channel_limit_stops_whole_edit_without_breaking_links() {
    let mut e = Engine::new(palette());
    link(&mut e, "a", "b", vec![0, 1, 2]);
    e.execute(Command::SetColor {
        id: "a".into(),
        rgb: [200.0, 220.0, 240.0],
        continuous: true,
    })
    .unwrap();
    let a = e.project.item("a").unwrap().rgb;
    let b = e.project.item("b").unwrap().rgb;
    assert!(b[2] <= 255.00001);
    assert!(a[2] < 240.0);
    for c in 0..3 {
        assert!((b[c] - a[c] * 2.0).abs() < 0.001);
    }
}
#[test]
fn zero_uses_offsets_and_hue_wraps() {
    let mut p = palette();
    p.item_mut("a").unwrap().rgb = color::from_hsv([355.0, 1.0, 0.5]);
    p.item_mut("b").unwrap().rgb = color::from_hsv([5.0, 1.0, 0.5]);
    let mut e = Engine::new(p);
    link(&mut e, "a", "b", vec![3]);
    e.execute(Command::SetColor {
        id: "a".into(),
        rgb: color::from_hsv([10.0, 1.0, 0.5]),
        continuous: false,
    })
    .unwrap();
    assert!(color::hue_delta(color::hsv(e.project.item("b").unwrap().rgb)[0], 20.0).abs() < 1e-4);
    let mut p = palette();
    p.item_mut("a").unwrap().rgb[0] = 0.0;
    let mut e = Engine::new(p);
    link(&mut e, "a", "b", vec![0]);
    assert_eq!(e.project.links[0].relations["0"].mode, "offset");
}
#[test]
fn compatible_cycles_survive_and_incompatible_edits_rollback() {
    let mut p = palette();
    p.items.push(Item {
        id: "c".into(),
        rgb: [40.0, 50.0, 60.0],
        ..Item::default()
    });
    let mut e = Engine::new(p);
    link(&mut e, "a", "b", vec![0]);
    link(&mut e, "b", "c", vec![0]);
    link(&mut e, "c", "a", vec![0]);
    e.execute(Command::SetColor {
        id: "c".into(),
        rgb: [50.0, 50.0, 60.0],
        continuous: false,
    })
    .unwrap();
    assert!((e.project.item("a").unwrap().rgb[0] - 100.0).abs() < 0.001);
    let old = e.project.clone();
    assert!(
        e.execute(Command::SetColor {
            id: "a".into(),
            rgb: [250.0, 100.0, 120.0],
            continuous: false
        })
        .is_err()
    );
    assert_eq!(e.project, old);
}
#[test]
fn locked_peer_is_not_silently_moved() {
    let mut e = Engine::new(palette());
    link(&mut e, "a", "b", vec![0]);
    e.execute(Command::Lock {
        id: "b".into(),
        locked: true,
    })
    .unwrap();
    let old = e.project.clone();
    assert!(
        e.execute(Command::SetColor {
            id: "a".into(),
            rgb: [90.0, 100.0, 120.0],
            continuous: false
        })
        .is_err()
    );
    assert_eq!(old, e.project);
}
#[test]
fn hue_harmony_moves_bidirectionally() {
    let mut e = Engine::new(palette());
    e.execute(Command::Compose(CompositionDraft {
        originals: vec!["a".into()],
        kind: "triad".into(),
        ..Default::default()
    }))
    .unwrap();
    let h = e.project.harmonies[0].clone();
    let id = h.ids[1].clone();
    let rgb = e.project.item(&id).unwrap().rgb;
    let mut hsv = color::hsv(rgb);
    hsv[0] += 35.0;
    e.execute(Command::SetColor {
        id,
        rgb: color::from_hsv(hsv),
        continuous: false,
    })
    .unwrap();
    assert!(solver::problems(&e.project, &solver::state(&e.project), None).is_empty());
    assert_eq!(e.project.harmonies[0].anchor_id, Some("a".into()));
}
#[test]
fn legacy_import_does_not_invent_harmony_anchor() {
    let mut p = palette();
    p.schema_version = 1;
    p.harmonies.push(Harmony {
        id: "old".into(),
        kind: "complementary".into(),
        ids: vec!["a".into(), "b".into()],
        ..Default::default()
    });
    let bytes = serde_json::to_vec(&p).unwrap();
    let loaded = Project::from_json(&bytes).unwrap();
    assert_eq!(loaded.schema_version, 4);
    assert!(loaded.harmonies[0].anchor_id.is_none());
    assert!(!loaded.harmonies[0].constraints.is_empty());
}
#[test]
fn gradients_update_and_can_be_detached() {
    let mut e = Engine::new(palette());
    e.execute(Command::Compose(CompositionDraft {
        kind: "gradient".into(),
        originals: vec!["a".into(), "b".into()],
        count: 3,
        space: "oklab".into(),
        ..Default::default()
    }))
    .unwrap();
    let g = e.project.gradients[0].clone();
    let middle = g.ids[1].clone();
    assert!(
        e.execute(Command::SetColor {
            id: middle.clone(),
            rgb: [1.0, 2.0, 3.0],
            continuous: false
        })
        .is_err()
    );
    e.execute(Command::Detach {
        gradient: g.id,
        color: middle.clone(),
    })
    .unwrap();
    e.execute(Command::SetColor {
        id: middle.clone(),
        rgb: [1.0, 2.0, 3.0],
        continuous: false,
    })
    .unwrap();
    e.execute(Command::SetColor {
        id: "a".into(),
        rgb: [50.0, 70.0, 80.0],
        continuous: false,
    })
    .unwrap();
    assert_eq!(e.project.item(&middle).unwrap().rgb, [1.0, 2.0, 3.0]);
    assert!(solver::problems(&e.project, &solver::state(&e.project), None).is_empty());
}
#[test]
fn replacement_preserves_id_folder_metadata_and_rolls_back_conflict() {
    let mut p = palette();
    p.item_mut("b").unwrap().description = "Não apagar".into();
    let mut e = Engine::new(p);
    link(&mut e, "a", "b", vec![0, 1, 2]);
    let before = e.project.clone();
    let mut draft = CompositionDraft {
        originals: vec!["a".into()],
        kind: "complementary".into(),
        ..Default::default()
    };
    draft.replacements.insert(
        1,
        Replacement {
            id: "b".into(),
            unlock: false,
            detach: false,
        },
    );
    assert!(e.execute(Command::Compose(draft.clone())).is_err());
    assert_eq!(e.project, before);
    e.execute(Command::RemoveRelation(e.project.links[0].id.clone()))
        .unwrap();
    e.execute(Command::Compose(draft)).unwrap();
    assert_eq!(e.project.item("b").unwrap().description, "Não apagar");
    assert_eq!(e.project.items.len(), 2);
}
#[test]
fn gesture_is_one_undo_and_redo_restores_all_colors() {
    let mut e = Engine::new(palette());
    link(&mut e, "a", "b", vec![0, 1, 2]);
    let old = e.project.clone();
    let count = e.undo.len();
    for x in 80..100 {
        e.preview_color("a", [x as f64, 100.0, 120.0], true)
            .unwrap();
    }
    e.finish_gesture();
    assert_eq!(e.undo.len(), count + 1);
    let new = e.project.clone();
    e.undo();
    assert_eq!(e.project, old);
    e.redo();
    assert_eq!(e.project, new);
}
#[test]
fn contrast_rejects_numeric_and_stops_before_crossing_disconnected_region() {
    let mut p = palette();
    p.item_mut("a").unwrap().rgb = [0.0; 3];
    p.item_mut("b").unwrap().rgb = [120.0; 3];
    let mut e = Engine::new(p);
    e.execute(Command::AddContrast {
        a: "a".into(),
        b: "b".into(),
        minimum: 4.5,
    })
    .unwrap();
    let old = e.project.clone();
    assert!(
        e.execute(Command::SetColor {
            id: "a".into(),
            rgb: [120.0; 3],
            continuous: false
        })
        .is_err()
    );
    assert_eq!(e.project, old);
    e.execute(Command::SetColor {
        id: "a".into(),
        rgb: [255.0; 3],
        continuous: true,
    })
    .unwrap();
    let a = e.project.item("a").unwrap().rgb;
    assert!(a[0] < 120.0);
    assert!(color::contrast(a, [120.0; 3]) >= 4.5 - 1e-8);
    assert_eq!(e.project.item("b").unwrap().rgb, [120.0; 3]);
}
#[test]
fn folders_reject_cycles_and_duplicates_rewrite_internal_refs() {
    let mut p = palette();
    p.items.push(Item {
        id: "folder".into(),
        kind: "folder".into(),
        ..Item::default()
    });
    p.item_mut("a").unwrap().parent_id = Some("folder".into());
    p.item_mut("b").unwrap().parent_id = Some("folder".into());
    let mut e = Engine::new(p);
    link(&mut e, "a", "b", vec![0]);
    assert!(
        e.execute(Command::Move {
            ids: vec!["folder".into()],
            parent: Some("folder".into())
        })
        .is_err()
    );
    e.execute(Command::Duplicate(vec!["folder".into()]))
        .unwrap();
    assert_eq!(e.project.items.len(), 6);
    assert_eq!(e.project.links.len(), 2);
    assert_ne!(e.project.links[0].a, e.project.links[1].a);
    e.project.validate().unwrap();
}
#[test]
fn documentary_roles_and_modes_never_move_colors() {
    let mut p = palette();
    let before = p.items.clone();
    p.roles.push(Role {
        id: "text".into(),
        name: "texto.principal".into(),
        color_id: "a".into(),
        description: "Texto".into(),
    });
    p.modes.push(Mode {
        id: "dark".into(),
        name: "Escuro".into(),
        overrides: BTreeMap::from([("text".into(), "b".into())]),
    });
    assert_eq!(p.resolve_role(&p.roles[0], Some("dark")).unwrap().id, "b");
    assert_eq!(p.resolve_role(&p.roles[0], None).unwrap().id, "a");
    assert_eq!(before, p.items);
    let v = export::palette(&p, false).unwrap();
    assert!(v.get("links").is_none());
    assert_eq!(v["roles"][0]["name"], "texto.principal");
}
#[test]
fn unassigned_role_bindings_persist_export_and_undo_without_editing_colors() {
    let p = palette();
    let before = p.items.clone();
    let mut e = Engine::new(p);
    let roles = vec![
        Role {
            id: "text".into(),
            name: "text.primary".into(),
            color_id: "a".into(),
            ..Default::default()
        },
        Role {
            id: "empty".into(),
            name: "accent".into(),
            ..Default::default()
        },
    ];
    let modes = vec![Mode {
        id: "dark".into(),
        name: "Dark".into(),
        overrides: BTreeMap::from([("text".into(), String::new())]),
    }];
    e.execute(Command::Documentation {
        relations: vec![],
        roles,
        modes,
    })
    .unwrap();
    assert_eq!(
        e.project
            .resolve_role(&e.project.roles[0], None)
            .unwrap()
            .id,
        "a"
    );
    assert!(
        e.project
            .resolve_role(&e.project.roles[0], Some("dark"))
            .is_none()
    );
    assert!(e.project.resolve_role(&e.project.roles[1], None).is_none());
    let bytes = serde_json::to_vec(&e.project).unwrap();
    let reopened = Project::from_json(&bytes).unwrap();
    assert_eq!(reopened, e.project);
    let public = export::palette(&e.project, false).unwrap();
    let imported = export::from_palette(&serde_json::to_vec(&public).unwrap()).unwrap();
    assert!(
        imported
            .resolve_role(&imported.roles[0], Some("dark"))
            .is_none()
    );
    let css = String::from_utf8(export::encode(&e.project, Format::Css, false).unwrap()).unwrap();
    assert!(css.contains("--papel-text-primary: initial;"));
    assert_eq!(e.project.items, before);
    e.undo();
    assert!(e.project.roles.is_empty());
    e.redo();
    assert_eq!(e.project.roles.len(), 2);
    e.execute(Command::Delete(vec!["a".into()])).unwrap();
    assert_eq!(e.project.roles.len(), 2);
    assert!(e.project.roles[0].color_id.is_empty());
    e.undo();
    assert_eq!(e.project.items, before);
}
#[test]
fn native_icc_and_models_roundtrip_without_flattening() {
    let mut p = palette();
    let bytes = craft_cms::Builtin::DisplayP3
        .profile()
        .to_bytes()
        .as_ref()
        .clone();
    let profile = IccProfile {
        id: "p3".into(),
        name: "Perfil de teste próprio".into(),
        bytes: bytes.clone(),
    };
    p.profiles.push(profile);
    let value = NativeColor {
        space: "display-p3".into(),
        components: vec![0.2, 0.4, 0.8],
        profile_id: Some("p3".into()),
    };
    let rgb = color::preview(&value, &p.profiles).unwrap();
    p.item_mut("a").unwrap().color = Some(value.clone());
    p.item_mut("a").unwrap().rgb = rgb.map(|v| v.clamp(0.0, 255.0));
    let loaded = Project::from_json(&serde_json::to_vec(&p).unwrap()).unwrap();
    assert_eq!(loaded.item("a").unwrap().native(), value);
    assert_eq!(loaded.profiles[0].bytes, bytes);
    let snapshot = export::palette(&loaded, false).unwrap();
    assert_eq!(snapshot["items"][0]["color"]["space"], "display-p3");
    assert!(snapshot["items"][0].get("rgb").is_none());
    assert!(snapshot["items"][0].get("previewSrgb").is_some());
}
#[test]
fn lab_and_cmyk_use_icc_transforms() {
    let srgb = NativeColor::srgb([126.0, 111.0, 239.0]);
    let lab = color::converted(
        &srgb,
        &NativeColor {
            space: "lab".into(),
            components: vec![],
            profile_id: None,
        },
        &[],
    )
    .unwrap();
    assert!(lab.components[0] > 0.0 && lab.components[0] < 100.0);
    let rgb = color::preview(&lab, &[]).unwrap();
    for c in 0..3 {
        assert!((rgb[c] - [126.0, 111.0, 239.0][c]).abs() < 0.05)
    }
    let cmyk = color::converted(
        &srgb,
        &NativeColor {
            space: "cmyk".into(),
            components: vec![],
            profile_id: None,
        },
        &[],
    )
    .unwrap();
    assert_eq!(cmyk.components.len(), 4);
    assert!(cmyk.components.iter().all(|v| v.is_finite()));
}
#[test]
fn compound_conversion_and_link_undo_together() {
    let mut p = palette();
    let n = p.item_mut("a").unwrap();
    n.color = Some(NativeColor {
        space: "display-p3".into(),
        components: vec![0.2, 0.3, 0.4],
        profile_id: None,
    });
    n.rgb = color::preview(&n.native(), &[])
        .unwrap()
        .map(|v| v.clamp(0.0, 255.0));
    let mut e = Engine::new(p);
    let before = e.project.clone();
    assert!(
        e.execute(Command::AddLink {
            a: "a".into(),
            b: "b".into(),
            channels: vec![0]
        })
        .is_err()
    );
    let rgb = e.project.item("a").unwrap().rgb;
    e.execute(Command::Batch(vec![
        Command::SetNative {
            id: "a".into(),
            value: NativeColor::srgb(rgb),
        },
        Command::AddLink {
            a: "a".into(),
            b: "b".into(),
            channels: vec![0],
        },
    ]))
    .unwrap();
    e.undo();
    assert_eq!(e.project, before);
}
#[test]
fn css_escapes_metadata_and_avoids_name_collisions() {
    let mut p = palette();
    p.items[0].name = "*/\n malicious".into();
    p.items[0].css_name = "a".into();
    p.items[1].css_name = "a".into();
    let css = String::from_utf8(export::encode(&p, Format::Css, false).unwrap()).unwrap();
    assert!(css.contains("--a:"));
    assert!(css.contains("--a-2:"));
    assert!(css.contains("* /"));
}
#[test]
fn simple_palette_is_linear_srgb_and_matches_contract() {
    let mut p = palette();
    p.item_mut("a").unwrap().rgb = [128.0; 3];
    let v: serde_json::Value =
        serde_json::from_slice(&export::encode(&p, Format::Simple, false).unwrap()).unwrap();
    let c = v["colors"][0]["components"][0].as_f64().unwrap();
    assert!((c - 0.21586).abs() < 0.00001);
    assert!(v.get("schemaVersion").is_none());
}
#[test]
fn future_versions_missing_profiles_and_folder_cycles_fail() {
    let mut p = palette();
    p.schema_version = 5;
    assert!(p.validate().is_err());
    p.schema_version = 4;
    p.items[0].color = Some(NativeColor {
        profile_id: Some("absent".into()),
        ..NativeColor::default()
    });
    assert!(p.validate().is_err());
    let mut p = palette();
    p.items[0].kind = "folder".into();
    p.items[0].parent_id = Some("a".into());
    assert!(p.validate().is_err());
}

#[test]
fn schema_v3_migrates_and_source_capture_transactions_undo_atomically() {
    let original = palette();
    let mut legacy = serde_json::to_value(&original).unwrap();
    legacy["schemaVersion"] = 3.into();
    legacy.as_object_mut().unwrap().remove("sources");
    let migrated = Project::from_json(&serde_json::to_vec(&legacy).unwrap()).unwrap();
    assert_eq!(migrated.schema_version, 4);
    assert_eq!(migrated.items, original.items);
    assert!(migrated.sources.is_empty());

    let mut engine = Engine::new(migrated);
    let capture = ColorCapture {
        id: "capture-1".into(),
        name: "Luz da janela".into(),
        color_id: Some("sample-1".into()),
        value: [20.0, 80.0, 140.0],
        ..Default::default()
    };
    let source = SourceAsset {
        id: "source-1".into(),
        name: "foto.png".into(),
        path: "foto.png".into(),
        fingerprint: "fingerprint".into(),
        captures: vec![capture],
        ..Default::default()
    };
    let color = Item {
        id: "sample-1".into(),
        name: "Luz da janela".into(),
        rgb: [20.0, 80.0, 140.0],
        color: Some(NativeColor::srgb([20.0, 80.0, 140.0])),
        source_locked: true,
        ..Item::default()
    };
    engine
        .execute(Command::ApplySource {
            source,
            colors_to_add: vec![color],
            color_values: BTreeMap::new(),
        })
        .unwrap();
    assert!(engine.project.item("sample-1").unwrap().source_locked);
    assert_eq!(engine.project.item("sample-1").unwrap().parent_id, None);
    engine.undo();
    assert!(engine.project.item("sample-1").is_none());
    assert!(engine.project.sources.is_empty());
    engine.redo();
    assert_eq!(
        engine.project.item("sample-1").unwrap().rgb,
        [20.0, 80.0, 140.0]
    );

    engine
        .execute(Command::UnlinkCapture {
            source_id: "source-1".into(),
            capture_id: "capture-1".into(),
        })
        .unwrap();
    assert!(!engine.project.item("sample-1").unwrap().source_locked);
    assert_eq!(
        engine.project.item("sample-1").unwrap().rgb,
        [20.0, 80.0, 140.0]
    );
    engine.undo();
    assert!(engine.project.item("sample-1").unwrap().source_locked);
    engine
        .execute(Command::RemoveSource("source-1".into()))
        .unwrap();
    assert!(engine.project.sources.is_empty());
    assert!(engine.project.item("sample-1").is_some());
    assert!(!engine.project.item("sample-1").unwrap().source_locked);
    engine.undo();
    assert_eq!(engine.project.sources.len(), 1);
    assert!(engine.project.item("sample-1").unwrap().source_locked);
}

#[test]
fn embedded_source_round_trips_with_capture_data() {
    let mut p = palette();
    let bytes = b"source bytes";
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes);
    p.sources.push(SourceAsset {
        id: "source".into(),
        name: "sample.png".into(),
        path: "sample.png".into(),
        storage: "embedded".into(),
        fingerprint: "hash".into(),
        embedded_data: Some(std::sync::Arc::new(encoded)),
        ..Default::default()
    });
    let loaded = Project::from_json(&serde_json::to_vec(&p).unwrap()).unwrap();
    assert_eq!(loaded.sources, p.sources);
}

#[test]
fn tokens_preserve_standard_profiles_and_warn_for_cmyk_conversion() {
    let mut e = Engine::new(palette());
    e.execute(Command::SetNative {
        id: "a".into(),
        value: NativeColor {
            space: "display-p3".into(),
            components: vec![1.0, 0.0, 0.0],
            profile_id: None,
        },
    })
    .unwrap();
    assert!(e.project.item("a").unwrap().native().profile_id.is_some());
    assert!(!e.project.profiles.is_empty());
    let tokens: serde_json::Value =
        serde_json::from_slice(&export::encode(&e.project, Format::Tokens, false).unwrap())
            .unwrap();
    assert_eq!(tokens["colors"]["a"]["$value"]["colorSpace"], "display-p3");
    assert!(
        !export::warnings(&e.project, Format::Tokens)
            .iter()
            .any(|s| s.contains("limita os canais"))
    );
    e.execute(Command::SetNative {
        id: "b".into(),
        value: NativeColor {
            space: "cmyk".into(),
            components: vec![0.3, 0.4, 0.2, 0.1],
            profile_id: None,
        },
    })
    .unwrap();
    assert!(
        export::warnings(&e.project, Format::Tokens)
            .iter()
            .any(|s| s.contains("conversão colorimétrica"))
    );
    let mut snapshot = export::palette(&e.project, false).unwrap();
    snapshot["items"][0]["previewSrgb"] = serde_json::json!([0, 0, 0]);
    let imported = export::from_palette(&serde_json::to_vec(&snapshot).unwrap()).unwrap();
    assert!(imported.item("a").unwrap().rgb[0] > 200.0);
    assert_eq!(imported.profiles, e.project.profiles);
}

#[test]
fn deleting_one_composition_member_keeps_remaining_relations_valid() {
    for kind in ["triad", "gradient"] {
        let mut e = Engine::new(palette());
        e.execute(Command::Compose(CompositionDraft {
            originals: if kind == "gradient" {
                vec!["a".into(), "b".into()]
            } else {
                vec!["a".into()]
            },
            kind: kind.into(),
            count: 4,
            ..Default::default()
        }))
        .unwrap();
        let before = e.project.clone();
        let victim = if kind == "gradient" {
            e.project.gradients[0].ids[1].clone()
        } else {
            e.project.harmonies[0].ids[1].clone()
        };
        e.execute(Command::Delete(vec![victim.clone()])).unwrap();
        assert!(e.project.item(&victim).is_none());
        assert!(solver::problems(&e.project, &solver::state(&e.project), None).is_empty());
        e.undo();
        assert_eq!(e.project, before);
    }
    assert!(export::encode(&Project::default(), Format::Simple, false).is_err());
}

#[test]
fn composition_analysis_reports_relation_ids_and_removal_is_one_transaction() {
    let mut e = Engine::new(palette());
    link(&mut e, "a", "b", vec![0, 1, 2]);
    let before = e.project.clone();
    let relation = e.project.links[0].id.clone();
    let mut draft = CompositionDraft {
        originals: vec!["a".into()],
        kind: "complementary".into(),
        ..Default::default()
    };
    draft.replacements.insert(
        1,
        Replacement {
            id: "b".into(),
            ..Default::default()
        },
    );
    let report = colorcraft_core::composition::analyze(&e.project, &draft, &Default::default());
    assert!(report.issue.unwrap().relation_ids.contains(&relation));
    assert_eq!(e.project, before);
    let removed = std::collections::BTreeSet::from([relation.clone()]);
    let report = colorcraft_core::composition::analyze(&e.project, &draft, &removed);
    assert!(report.issue.is_none());
    assert_eq!(
        (
            report.original_count,
            report.new_count,
            report.replacement_count
        ),
        (1, 0, 1)
    );
    let count = e.undo.len();
    e.execute(Command::Batch(vec![
        Command::RemoveRelation(relation.clone()),
        Command::Compose(draft.clone()),
    ]))
    .unwrap();
    assert_eq!(e.undo.len(), count + 1);
    e.undo();
    assert_eq!(e.project, before);
    e.project.item_mut("b").unwrap().locked = true;
    let locked = e.project.clone();
    assert!(
        e.execute(Command::Batch(vec![
            Command::RemoveRelation(relation),
            Command::Compose(draft)
        ]))
        .is_err()
    );
    assert_eq!(e.project, locked);
}
#[test]
fn multi_colour_link_propagates_from_any_member_and_undoes_together() {
    let mut p = palette();
    p.items.push(Item {
        id: "c".into(),
        rgb: [40.0, 50.0, 60.0],
        ..Default::default()
    });
    let before = p.clone();
    let mut e = Engine::new(p);
    e.execute(Command::LinkColors {
        ids: vec!["a".into(), "b".into(), "c".into()],
        channels: vec![0, 1, 2],
    })
    .unwrap();
    assert_eq!(e.project.links.len(), 2);
    assert_eq!(e.undo.len(), 1);
    let links = e.project.links.clone();
    e.execute(Command::LinkColors {
        ids: vec!["a".into(), "b".into(), "c".into()],
        channels: vec![0, 1, 2],
    })
    .unwrap();
    assert_eq!(e.project.links, links);
    e.execute(Command::SetColor {
        id: "c".into(),
        rgb: [30.0, 40.0, 50.0],
        continuous: false,
    })
    .unwrap();
    assert_eq!(e.project.item("a").unwrap().rgb, [60.0, 80.0, 100.0]);
    assert_eq!(e.project.item("b").unwrap().rgb, [120.0, 160.0, 200.0]);
    e.undo();
    e.undo();
    assert_eq!(e.project, before);
    let failed = e.project.clone();
    assert!(
        e.execute(Command::LinkColors {
            ids: vec!["a".into(), "b".into(), "missing".into()],
            channels: vec![0]
        })
        .is_err()
    );
    assert_eq!(e.project, failed);
}

#[test]
fn new_gradient_preserves_all_existing_colors_even_with_accepted_link_residuals() {
    let mut e = Engine::new(palette());
    link(&mut e, "a", "b", vec![0, 1, 2]);
    // A residual accepted by solver::problems (2e-4) but stricter pin solving rejects it.
    e.project.item_mut("b").unwrap().rgb[0] += 0.00005;
    let before = e.project.clone();
    assert!(solver::problems(&before, &solver::state(&before), None).is_empty());
    let pins = BTreeMap::from([
        ("a".into(), before.item("a").unwrap().rgb),
        ("b".into(), before.item("b").unwrap().rgb),
    ]);
    assert!(solver::solve(&before, &pins).is_err());
    let draft = CompositionDraft {
        kind: "gradient".into(),
        originals: vec!["a".into(), "b".into()],
        count: 9,
        ..Default::default()
    };
    let analysis = composition::analyze(&before, &draft, &Default::default());
    assert!(analysis.issue.is_none(), "{:?}", analysis.issue);
    assert_eq!(analysis.new_count, 9);
    e.execute(Command::Compose(draft)).unwrap();
    for item in &before.items {
        assert_eq!(e.project.item(&item.id), Some(item));
    }
    assert_eq!(e.project.links, before.links);
    assert_eq!(e.project.items.len(), before.items.len() + 9);
    assert!(solver::problems(&e.project, &solver::state(&e.project), None).is_empty());
    let after = e.project.clone();
    e.undo();
    assert_eq!(e.project, before);
    e.redo();
    assert_eq!(e.project, after);
}

#[test]
fn blocked_origins_can_create_harmonies_gradients_and_links_without_changing_values() {
    for lock in ["manual", "direct", "source"] {
        for kind in ["analogous", "complementary", "mono", "gradient", "link"] {
            let mut e = Engine::new(palette());
            if lock == "source" {
                let cap = ColorCapture {
                    color_id: Some("a".into()),
                    value: e.project.item("a").unwrap().rgb,
                    ..Default::default()
                };
                let source = SourceAsset {
                    captures: vec![cap],
                    ..Default::default()
                };
                e.execute(Command::ApplySource {
                    source,
                    colors_to_add: vec![],
                    color_values: Default::default(),
                })
                .unwrap();
            } else {
                e.execute(if lock == "manual" {
                    Command::Lock {
                        id: "a".into(),
                        locked: true,
                    }
                } else {
                    Command::DirectLock {
                        id: "a".into(),
                        locked: true,
                    }
                })
                .unwrap();
            }
            let before = e.project.clone();
            let command = if kind == "link" {
                Command::LinkColors {
                    ids: vec!["a".into(), "b".into()],
                    channels: vec![3, 4, 5],
                }
            } else {
                Command::Compose(CompositionDraft {
                    kind: kind.into(),
                    originals: if kind == "gradient" {
                        vec!["a".into(), "b".into()]
                    } else {
                        vec!["a".into()]
                    },
                    ..Default::default()
                })
            };
            e.execute(command)
                .unwrap_or_else(|error| panic!("{lock}/{kind}: {error}"));
            for item in &before.items {
                assert_eq!(e.project.item(&item.id), Some(item), "{lock}/{kind}");
            }
            assert_eq!(e.project.sources, before.sources);
            let after = e.project.clone();
            e.undo();
            assert_eq!(e.project, before);
            e.redo();
            assert_eq!(e.project, after);
        }
    }
}
