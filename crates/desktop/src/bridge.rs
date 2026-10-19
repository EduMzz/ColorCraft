//! Receiver for explicitly confirmed local integration requests.
use crate::app::ColorCraftApp;
use colorcraft_bridge::{Request, Response, Snapshot};
use colorcraft_core::engine::Command;
use colorcraft_core::{IccProfile, Item, NativeColor, color, export};
use colorcraft_storage as storage;
use eframe::egui;
use std::path::PathBuf;

impl ColorCraftApp {
    pub(crate) fn bridge_tick(&mut self, ctx: &egui::Context) {
        let Some(receiver) = self.bridge.take() else {
            return;
        };
        ctx.request_repaint_after(std::time::Duration::from_millis(350));
        match receiver.pending() {
            Ok(requests) => {
                for request in requests {
                    let mut response = match self.bridge_request(&request, ctx) {
                        Ok(r) => r,
                        Err(e) => Response::error(&request, "rejected", e),
                    };
                    response.error = response
                        .error
                        .map(|e| crate::i18n::for_language(&e, &request.language));
                    if request.operation == "open" && !response.ok {
                        self.error = response.error.clone();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    }
                    if let Err(e) = receiver.finish(&response) {
                        self.error = Some(format!("Integração ColorCraft: {e}"));
                    }
                }
            }
            Err(e) => self.error = Some(format!("Integração ColorCraft: {e}")),
        }
        self.bridge = Some(receiver);
    }
    fn bridge_document(&mut self, palette: &str) -> Result<usize, String> {
        if let Some(i) = self
            .docs
            .iter()
            .position(|d| d.engine.project.id == palette)
        {
            return Ok(i);
        }
        let paths = self
            .paths
            .as_ref()
            .ok_or("Biblioteca local indisponível.")?;
        let entry = storage::library(paths)?
            .palettes
            .into_iter()
            .find(|e| e.palette_id == palette)
            .ok_or("O projeto solicitado não está na biblioteca ColorCraft.")?;
        let path = PathBuf::from(entry.source_path);
        let loaded = storage::load(&path)?;
        if loaded.imported || loaded.project.id != palette {
            return Err("O arquivo não corresponde ao projeto solicitado. Abra e publique o projeto correto no ColorCraft.".into());
        }
        let old = self.current;
        let home = self.home;
        self.add_doc(loaded.project, Some(path), Some(loaded.hash), true);
        let index = self.current;
        self.current = old;
        self.home = home;
        Ok(index)
    }
    fn bridge_state(&self, index: usize) -> Result<String, String> {
        let d = self.docs.get(index).ok_or("Projeto não encontrado.")?;
        let path = d
            .path
            .as_ref()
            .ok_or("Salve o projeto antes de receber cores.")?;
        let disk = storage::file_hash(path)?;
        if d.disk_hash.as_deref() != Some(disk.as_str()) {
            return Err("O arquivo mudou em outro processo. Reabra ou salve uma cópia no ColorCraft antes de enviar cores.".into());
        }
        Ok(native_state(&d.engine.project, &disk, path))
    }
    pub(crate) fn bridge_request(
        &mut self,
        r: &Request,
        ctx: &egui::Context,
    ) -> Result<Response, String> {
        r.validate()?;
        if self.dialog.is_some()
            || self.capture.is_some()
            || self.picking.is_some()
            || self.title_edit.is_some()
            || self.docs.iter().any(|d| d.engine.is_interacting())
        {
            return Ok(Response::error(
                r,
                "busy",
                "Conclua a edição ou o diálogo aberto no ColorCraft e tente novamente.".into(),
            ));
        }
        let index = self.bridge_document(&r.palette_id)?;
        let mut response = Response::new(r, "ready");
        if r.operation == "open" {
            self.current = index;
            self.home = false;
            if let Some(id) = &r.color_id {
                if !self
                    .docs
                    .get(index)
                    .is_some_and(|d| d.engine.project.item(id).is_some_and(|i| i.is_color()))
                {
                    return Err(
                        "A cor solicitada foi removida ou não pertence a este projeto.".into(),
                    );
                }
                self.reveal_conflict_color(id);
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            response.status = "opened".into();
            return Ok(response);
        }
        let state = self.bridge_state(index)?;
        let d = self.docs.get(index).ok_or("Projeto não encontrado.")?;
        if let Some(folder) = &r.folder_id
            && !d.engine.project.item(folder).is_some_and(|i| !i.is_color())
        {
            return Ok(Response::error(
                r,
                "changed",
                "A pasta de destino foi removida. Escolha o destino novamente.".into(),
            ));
        }
        response.dirty = d.dirty();
        response.state = Some(state.clone());
        if r.operation == "inspect" {
            let snapshot: Snapshot =
                serde_json::from_value(export::palette(&d.engine.project, false)?)
                    .map_err(|e| e.to_string())?;
            snapshot.validate()?;
            response.snapshot = Some(snapshot);
            return Ok(response);
        }
        if r.operation == "publish" {
            let path = d.path.as_ref().ok_or("Projeto sem arquivo.")?;
            let paths = self.paths.as_ref().ok_or("Biblioteca indisponível.")?;
            if d.dirty() {
                return Err("Salve as alterações pendentes antes de repetir a publicação.".into());
            }
            storage::publish(&d.engine.project, path, paths)?;
            if let Some(d) = self.docs.get_mut(index) {
                d.publication_pending = false;
            }
            response.status = "published".into();
            return Ok(response);
        }
        if let Some(receipt) = d
            .engine
            .project
            .extra
            .get("colorcraftBridgeReceipts")
            .and_then(serde_json::Value::as_array)
            .and_then(|v| {
                v.iter().find(|v| {
                    v.get("requestId").and_then(serde_json::Value::as_str)
                        == Some(r.request_id.as_str())
                })
            })
        {
            response.color_ids = receipt
                .get("colorIds")
                .and_then(serde_json::Value::as_array)
                .map(|v| {
                    v.iter()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default();
            let published = self
                .paths
                .as_ref()
                .and_then(|paths| storage::library(paths).ok())
                .is_some_and(|lib| {
                    lib.palettes.iter().any(|e| {
                        e.palette_id == r.palette_id && e.revision == d.engine.project.revision
                    })
                });
            response.status = if published {
                "alreadyApplied"
            } else {
                "savedPublicationPending"
            }
            .into();
            return Ok(response);
        }
        if r.expected_state.as_deref() != Some(&state) {
            return Ok(Response::error(
                r,
                "changed",
                "O destino mudou. Revise o formulário e confirme novamente.".into(),
            ));
        }
        let mut profiles = d.engine.project.profiles.clone();
        let mut added_profiles = vec![];
        let mut items = vec![];
        for row in &r.colors {
            let mut value: NativeColor = serde_json::from_value(
                serde_json::to_value(&row.color).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            if let Some(id) = &row.color.profile_id {
                let source = row
                    .profiles
                    .iter()
                    .chain(r.profiles.iter())
                    .find(|p| &p.id == id)
                    .ok_or("O envio não contém o perfil ICC da cor.")?;
                let profile_id = format!("bridge-{}", colorcraft_bridge::digest(&source.bytes));
                let target = IccProfile {
                    id: profile_id.clone(),
                    name: source.name.clone(),
                    bytes: source.bytes.clone(),
                };
                if !profiles.iter().any(|p| p.id == profile_id) {
                    profiles.push(target.clone());
                    added_profiles.push(target);
                }
                value.profile_id = Some(profile_id);
            }
            let rgb = color::preview(&value, &profiles)?;
            let mut item = Item::new("color", r.folder_id.clone());
            item.name = row.name.clone();
            item.rgb = rgb;
            item.color = Some(value);
            items.push(item);
        }
        response.color_ids = items.iter().map(|i| i.id.clone()).collect();
        let backup = d.engine.clone();
        let d = self.docs.get_mut(index).ok_or("Projeto não encontrado.")?;
        d.engine.execute(Command::ReceiveColors {
            request_id: r.request_id.clone(),
            items,
            profiles: added_profiles,
        })?;
        let saved = storage::save(
            &d.engine.project,
            d.path.as_ref().ok_or("Projeto sem arquivo.")?,
            d.disk_hash.as_deref(),
            self.paths.as_ref().ok_or("Biblioteca indisponível.")?,
        );
        match saved {
            Ok(saved) => {
                d.engine.project = saved.project;
                d.saved_hash = storage::content_hash(&d.engine.project);
                d.disk_hash = Some(saved.hash);
                d.publication_pending = saved.publication_error.is_some();
                d.dirty_cache.replace((d.engine.generation, false));
                response.status = if saved.publication_error.is_some() {
                    "savedPublicationPending"
                } else {
                    "savedAndPublished"
                }
                .into();
                response.error = saved.publication_error;
                response.state = self.bridge_state(index).ok();
                self.refresh_library();
                Ok(response)
            }
            Err(e) => {
                d.engine = backup;
                Err(e)
            }
        }
    }
}
fn native_state(p: &colorcraft_core::Project, disk: &str, path: &std::path::Path) -> String {
    colorcraft_bridge::digest(
        format!("{}:{disk}:{}", storage::content_hash(p), path.display()).as_bytes(),
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::Args;
    use colorcraft_core::Project;
    fn setup() -> (ColorCraftApp, egui::Context, String, PathBuf) {
        let root =
            std::env::temp_dir().join(format!("colorcraft-bridge-test-{}", colorcraft_core::uid()));
        let ctx = egui::Context::default();
        let mut app = ColorCraftApp::with_context(
            &ctx,
            Args {
                isolated: true,
                data: Some(root.clone()),
                config: Some(root.join("config")),
                ..Default::default()
            },
        );
        let mut p = Project::default();
        let mut color = Item::new("color", None);
        color.name = "Original".into();
        p.items.push(color);
        let path = root.join("project.colorcraft");
        let saved = storage::save(&p, &path, None, app.paths.as_ref().unwrap()).unwrap();
        let id = saved.project.id;
        app.open_path(path.clone());
        (app, ctx, id, path)
    }
    fn inspect(app: &mut ColorCraftApp, ctx: &egui::Context, id: &str) -> Response {
        app.bridge_request(&Request::new("inspect", id), ctx)
            .unwrap()
    }
    fn append(id: &str, state: Option<String>) -> Request {
        let mut r = Request::new("append", id);
        r.expected_state = state;
        r.colors = vec![colorcraft_bridge::NewColor {
            name: "From VectorCraft".into(),
            color: colorcraft_bridge::NativeColor {
                space: "srgb".into(),
                components: vec![0.1, 0.4, 0.8],
                profile_id: None,
            },
            profiles: vec![],
        }];
        r
    }
    #[test]
    fn append_is_one_undo_step_saved_published_and_idempotent() {
        let (mut app, ctx, id, path) = setup();
        let initial = inspect(&mut app, &ctx, &id);
        assert!(!initial.dirty);
        let r = append(&id, initial.state);
        let history = app.doc().unwrap().engine.undo.len();
        let response = app.bridge_request(&r, &ctx).unwrap();
        assert_eq!(response.status, "savedAndPublished");
        assert_eq!(app.doc().unwrap().engine.undo.len(), history + 1);
        assert!(!app.doc().unwrap().dirty());
        assert_eq!(storage::load(&path).unwrap().project.count(), 2);
        assert_eq!(
            storage::library(app.paths.as_ref().unwrap())
                .unwrap()
                .palettes[0]
                .colors,
            2
        );
        assert_eq!(
            app.bridge_request(&r, &ctx).unwrap().status,
            "alreadyApplied"
        );
        assert_eq!(app.doc().unwrap().engine.project.count(), 2);
        app.doc_mut().unwrap().engine.undo();
        assert_eq!(app.doc().unwrap().engine.project.count(), 1);
        app.doc_mut().unwrap().engine.redo();
        assert_eq!(app.doc().unwrap().engine.project.count(), 2);
        let root = app.paths.as_ref().unwrap().data.clone();
        let mut reopened = ColorCraftApp::with_context(
            &ctx,
            Args {
                isolated: true,
                data: Some(root.clone()),
                config: Some(root.join("config2")),
                ..Default::default()
            },
        );
        reopened.open_path(path);
        assert_eq!(
            reopened.bridge_request(&r, &ctx).unwrap().status,
            "alreadyApplied"
        );
    }
    #[test]
    fn preflight_detects_pending_changes_and_rejects_stale_confirmation() {
        let (mut app, ctx, id, _) = setup();
        let old = inspect(&mut app, &ctx, &id);
        app.doc_mut()
            .unwrap()
            .engine
            .execute(Command::ProjectDetails {
                name: "Pending title".into(),
                description: "".into(),
            })
            .unwrap();
        let response = app.bridge_request(&append(&id, old.state), &ctx).unwrap();
        assert_eq!(response.status, "changed");
        assert_eq!(app.doc().unwrap().engine.project.count(), 1);
        let current = inspect(&mut app, &ctx, &id);
        assert!(current.dirty);
        let response = app
            .bridge_request(&append(&id, current.state), &ctx)
            .unwrap();
        assert!(response.ok);
        assert_eq!(app.doc().unwrap().engine.project.name, "Pending title");
        assert!(!app.doc().unwrap().dirty());
    }
    #[test]
    fn busy_and_missing_folder_do_not_mutate_the_project() {
        let (mut app, ctx, id, _) = setup();
        let before = app.doc().unwrap().engine.project.clone();
        app.doc_mut().unwrap().engine.begin_gesture();
        assert_eq!(inspect(&mut app, &ctx, &id).status, "busy");
        app.doc_mut().unwrap().engine.finish_gesture();
        let current = inspect(&mut app, &ctx, &id);
        let mut r = append(&id, current.state);
        r.folder_id = Some("missing".into());
        assert_eq!(app.bridge_request(&r, &ctx).unwrap().status, "changed");
        assert_eq!(app.doc().unwrap().engine.project, before);
    }
    #[test]
    fn publication_failure_does_not_repeat_the_append() {
        let (mut app, ctx, id, path) = setup();
        let current = inspect(&mut app, &ctx, &id);
        let r = append(&id, current.state);
        let library = app.paths.as_ref().unwrap().library.join("library.json");
        std::fs::write(&library, b"{\"schemaVersion\":999,\"palettes\":[]}").unwrap();
        let result = app.bridge_request(&r, &ctx).unwrap();
        assert!(result.ok);
        assert_eq!(result.status, "savedPublicationPending");
        assert_eq!(storage::load(&path).unwrap().project.count(), 2);
        assert_eq!(
            app.bridge_request(&r, &ctx).unwrap().status,
            "savedPublicationPending"
        );
    }
    #[test]
    fn uri_reveals_the_exact_color_and_keeps_missing_targets_explicit() {
        let (mut app, ctx, id, _) = setup();
        let color = app.doc().unwrap().engine.project.items[0].id.clone();
        let request = Request::from_uri(&colorcraft_bridge::open_uri(&id, Some(&color))).unwrap();
        assert_eq!(app.bridge_request(&request, &ctx).unwrap().status, "opened");
        assert_eq!(app.doc().unwrap().selected, vec![color]);
        let request =
            Request::from_uri(&colorcraft_bridge::open_uri(&id, Some("missing"))).unwrap();
        assert!(app.bridge_request(&request, &ctx).is_err());
    }
    #[test]
    fn batch_profile_table_preserves_native_components_and_icc_bytes() {
        let (mut app, ctx, id, path) = setup();
        let initial = inspect(&mut app, &ctx, &id);
        let native = NativeColor {
            space: "display-p3".into(),
            components: vec![0.8, 0.2, 0.4],
            profile_id: None,
        };
        let bytes = color::profile(&native, &[])
            .unwrap()
            .to_bytes()
            .as_ref()
            .clone();
        let mut r = append(&id, initial.state);
        r.profiles = vec![colorcraft_bridge::Profile {
            id: "source-p3".into(),
            name: "Display P3".into(),
            bytes: bytes.clone(),
        }];
        r.colors[0].color = colorcraft_bridge::NativeColor {
            space: native.space.clone(),
            components: native.components.clone(),
            profile_id: Some("source-p3".into()),
        };
        assert!(app.bridge_request(&r, &ctx).unwrap().ok);
        let p = storage::load(&path).unwrap().project;
        let saved = p.items.last().unwrap().color.as_ref().unwrap();
        assert_eq!(saved.components, native.components);
        assert_eq!(
            p.profiles
                .iter()
                .find(|p| Some(&p.id) == saved.profile_id.as_ref())
                .unwrap()
                .bytes,
            bytes
        );
    }
    #[test]
    fn preflight_changes_when_the_destination_file_moves_without_color_changes() {
        let (mut app, ctx, id, path) = setup();
        let before = inspect(&mut app, &ctx, &id);
        let other = path.with_file_name("moved.colorcraft");
        std::fs::copy(&path, &other).unwrap();
        app.doc_mut().unwrap().path = Some(other);
        assert_eq!(
            app.bridge_request(&append(&id, before.state), &ctx)
                .unwrap()
                .status,
            "changed"
        );
        assert_eq!(app.doc().unwrap().engine.project.count(), 1);
    }
}
