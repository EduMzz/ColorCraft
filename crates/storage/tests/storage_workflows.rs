#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use colorcraft_core::{
    Item, Project,
    export::{self, Format},
};
use colorcraft_storage::*;
use std::{fs, path::PathBuf};
fn setup() -> (PathBuf, Paths) {
    let root = std::env::temp_dir().join(format!("colorcraft-test-{}", colorcraft_core::uid()));
    fs::create_dir_all(&root).unwrap();
    let (store, _) = open_settings(None, true);
    let paths = Paths::new(&store, Some(root.join("data"))).unwrap();
    (root, paths)
}
#[test]
fn saves_publish_immutable_snapshots_and_conflicts_preserve_file() {
    let (root, paths) = setup();
    let file = root.join("palette.colorcraft");
    let mut p = Project::default();
    p.items.push(Item::default());
    let a = save(&p, &file, None, &paths).unwrap();
    assert!(a.publication_error.is_none());
    let first = library(&paths).unwrap().palettes[0].snapshot.clone();
    let old = fs::read(paths.library.join(&first)).unwrap();
    let mut edited = a.project.clone();
    edited.name = "Alterado".into();
    let b = save(&edited, &file, Some(&a.hash), &paths).unwrap();
    assert_eq!(b.project.revision, 2);
    assert_eq!(fs::read(paths.library.join(first)).unwrap(), old);
    let before = fs::read(&file).unwrap();
    assert!(save(&edited, &file, Some(&a.hash), &paths).is_err());
    assert_eq!(before, fs::read(&file).unwrap());
    assert!(file.with_extension("colorcraft.previous").exists());
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn corrupt_library_does_not_destroy_saved_project() {
    let (root, paths) = setup();
    fs::write(paths.library.join("library.json"), b"broken").unwrap();
    let p = Project::default();
    let saved = save(&p, &root.join("safe.colorcraft"), None, &paths).unwrap();
    assert!(saved.publication_error.is_some());
    assert_eq!(
        load(&root.join("safe.colorcraft")).unwrap().project.name,
        p.name
    );
    assert_eq!(
        fs::read(paths.library.join("library.json")).unwrap(),
        b"broken"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unavailable_library_does_not_block_saving_elsewhere() {
    let root =
        std::env::temp_dir().join(format!("colorcraft-unavailable-{}", colorcraft_core::uid()));
    fs::create_dir_all(&root).unwrap();
    let blocked = root.join("not-a-directory");
    fs::write(&blocked, b"preserve").unwrap();
    let (mut store, _) = open_settings(None, true);
    store.update(|p| p.general.library_dir = blocked.to_string_lossy().into());
    let paths = Paths::resolve(&store, Some(root.join("data"))).unwrap();
    assert!(paths.prepare().is_err());
    let target = root.join("saved.colorcraft");
    let saved = save(&Project::demo(), &target, None, &paths).unwrap();
    assert!(saved.publication_error.is_some());
    assert_eq!(load(&target).unwrap().project.count(), 5);
    assert_eq!(fs::read(blocked).unwrap(), b"preserve");
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn project_and_published_palettes_are_distinct() {
    let (root, paths) = setup();
    let p = Project::demo();
    let saved = save(&p, &root.join("p.colorcraft"), None, &paths).unwrap();
    let file = root.join("export.colorcraft-palette");
    export_file(&saved.project, &file, Format::Palette, false).unwrap();
    let imported = load(&file).unwrap();
    assert!(imported.imported);
    assert_ne!(imported.project.id, saved.project.id);
    assert_eq!(imported.project.items.len(), saved.project.items.len());
    assert_eq!(imported.project.items[0].id, saved.project.items[0].id);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn unsupported_json_and_transparency_never_import_as_empty_project() {
    let (root, _paths) = setup();
    let path = root.join("wrong.json");
    fs::write(&path, b"{\"random\":123}").unwrap();
    assert!(load(&path).is_err());
    let path = root.join("alpha.color-palette");
    fs::write(&path, b"{\"colors\":[{\"components\":[1,0,0,0.2]}]}").unwrap();
    assert!(load(&path).is_err());
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn simple_and_gpl_exports_import_without_name_or_color_loss() {
    let (root, _paths) = setup();
    let p = Project::demo();
    for f in [Format::Simple, Format::Gpl] {
        let file = root.join(format!("colors.{}", f.extension()));
        fs::write(&file, export::encode(&p, f, false).unwrap()).unwrap();
        let loaded = load(&file).unwrap().project;
        assert_eq!(loaded.name, p.name);
        assert_eq!(loaded.items.len(), p.items.len());
        for (a, b) in loaded.items.iter().zip(&p.items) {
            assert_eq!(a.name, b.name);
            for c in 0..3 {
                assert!((a.rgb[c] - b.rgb[c]).abs() < 0.02);
            }
        }
    }
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn recovery_is_independent_of_settings() {
    let (root, paths) = setup();
    let p = Project::demo();
    recover(&p, &paths).unwrap();
    assert_eq!(recoveries(&paths).len(), 1);
    let recovered = load(&recoveries(&paths)[0]).unwrap();
    assert_eq!(recovered.project.items, p.items);
    discard_recovery(&p.id, &paths).unwrap();
    assert!(recoveries(&paths).is_empty());
    fs::remove_dir_all(root).unwrap();
}
