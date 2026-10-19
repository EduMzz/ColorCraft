use colorcraft_core::{
    export::{self, Format},
    *,
};
use std::{collections::BTreeMap, path::PathBuf};

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let dest = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("examples"));
    std::fs::create_dir_all(&dest)?;
    let mut p = Project::demo();
    p.id = "b7930120-03da-4b36-9cb9-e541bc143d10".into();
    for (i, item) in p.items.iter_mut().enumerate() {
        item.id = format!("b7930120-03da-4b36-9cb9-e541bc143d{:02}", i + 11);
    }
    let text = p.items[3].id.clone();
    let paper = p.items[4].id.clone();
    p.items[3].usage = "Texto em superfícies claras".into();
    p.items[4].usage = "Superfície padrão".into();
    p.limits.push(Limit {
        id: "exemplo-contraste".into(),
        a: text.clone(),
        b: paper.clone(),
        minimum: 4.5,
        metric: "wcag22-srgb".into(),
    });
    p.documentary_relations.push(Documentary {
        id: "exemplo-documental".into(),
        from: text.clone(),
        to: paper.clone(),
        kind: "texto-sobre".into(),
        note: "Uso documentado; esta relação não limita os valores.".into(),
    });
    p.roles = vec![
        Role {
            id: "texto".into(),
            name: "texto.principal".into(),
            color_id: text.clone(),
            description: "Texto principal da interface".into(),
        },
        Role {
            id: "fundo".into(),
            name: "superficie".into(),
            color_id: paper.clone(),
            description: "Fundo principal".into(),
        },
    ];
    p.modes.push(Mode {
        id: "escuro".into(),
        name: "Escuro".into(),
        overrides: BTreeMap::from([("texto".into(), paper), ("fundo".into(), text)]),
    });
    p.validate()?;
    for (format, file) in [
        (Format::Project, "Horizonte.colorcraft"),
        (Format::Palette, "Horizonte.colorcraft-palette"),
        (Format::Css, "Horizonte.css"),
    ] {
        std::fs::write(dest.join(file), export::encode(&p, format, false)?)?;
    }
    Ok(())
}
