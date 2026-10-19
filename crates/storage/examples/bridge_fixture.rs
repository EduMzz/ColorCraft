//! Synthetic local fixture; never reads or edits a user's library.
use colorcraft_core::{Item, NativeColor, Project};
use colorcraft_storage as storage;
fn main() -> Result<(), String> {
    let root = std::env::args_os()
        .nth(1)
        .map(std::path::PathBuf::from)
        .ok_or("Expected a new fixture directory")?;
    if root.join("fixture.json").exists() {
        return Err("Fixture already exists".into());
    }
    let (settings, _) = storage::open_settings(Some(root.join("config")), true);
    let paths = storage::Paths::new(&settings, Some(root.join("data")))?;
    let mut p = Project {
        id: "bridge-demo".into(),
        name: "ColorCraft bridge demo".into(),
        ..Project::default()
    };
    let mut folder = Item::new("folder", None);
    folder.id = "brand".into();
    folder.name = "Brand".into();
    p.items.push(folder);
    for (id, name, rgb) in [
        ("coral", "Coral", [238.0, 77.0, 89.0]),
        ("ocean", "Ocean", [22.0, 137.0, 179.0]),
        ("leaf", "Leaf", [91.0, 166.0, 100.0]),
    ] {
        let mut item = Item::new("color", Some("brand".into()));
        item.id = id.into();
        item.name = name.into();
        item.rgb = rgb;
        item.color = Some(NativeColor::srgb(rgb));
        p.items.push(item);
    }
    let project = root.join("Bridge demo.colorcraft");
    let saved = storage::save(&p, &project, None, &paths)?;
    if let Some(e) = saved.publication_error {
        return Err(e);
    }
    let fixture = serde_json::json!({"paletteId":p.id,"colorId":"coral","folderId":"brand","project":project,"library":paths.library});
    storage::atomic(
        &root.join("fixture.json"),
        &serde_json::to_vec_pretty(&fixture).map_err(|e| e.to_string())?,
    )?;
    println!("{fixture}");
    Ok(())
}
