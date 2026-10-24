//! Native Rust touch editor preview. All footage is drawn procedurally.

use eframe::egui::{self, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use filmcraft_mobile_core::editor::{DemoSource, FRAME_TICKS, Project};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const ACCENT: Color32 = Color32::from_rgb(108, 230, 180);
const MUTED: Color32 = Color32::from_rgb(154, 170, 187);
const MAX_PROJECT_BYTES: u64 = 128 * 1024;

pub struct MobileEditor {
    project: Project,
    selected: usize,
    playhead: u32,
    playing: bool,
    fractional_frame: f64,
    trim_in: u32,
    trim_out: u32,
    undo: Vec<Project>,
    redo: Vec<Project>,
    save_path: Option<PathBuf>,
    status: String,
}

impl MobileEditor {
    pub fn new(cc: &eframe::CreationContext<'_>, save_path: Option<PathBuf>) -> Self {
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        let mut style = (*cc.egui_ctx.style_of(egui::Theme::Dark)).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = Color32::from_rgb(14, 20, 28);
        style.visuals.window_fill = Color32::from_rgb(22, 30, 40);
        style.visuals.selection.bg_fill = Color32::from_rgb(32, 94, 77);
        style.visuals.selection.stroke = Stroke::new(1.0, ACCENT);
        style.spacing.interact_size = Vec2::new(44.0, 44.0);
        style.spacing.button_padding = Vec2::new(12.0, 9.0);
        style.spacing.item_spacing = Vec2::new(8.0, 9.0);
        cc.egui_ctx.set_style_of(egui::Theme::Dark, style);
        let (project, status) = match save_path.as_deref().map(read_project) {
            Some(Ok(project)) => (project, "Restored your demo project".to_owned()),
            Some(Err(error)) if save_path.as_ref().is_some_and(|path| path.exists()) => {
                (Project::default(), format!("Could not restore: {error}"))
            }
            _ => (
                Project::default(),
                "Ready. Edit the three generated demo clips.".to_owned(),
            ),
        };
        let mut editor = Self {
            project,
            selected: 0,
            playhead: 0,
            playing: false,
            fractional_frame: 0.0,
            trim_in: 0,
            trim_out: 120,
            undo: Vec::new(),
            redo: Vec::new(),
            save_path,
            status,
        };
        editor.sync_trim();
        editor
    }

    fn sync_trim(&mut self) {
        if let Some(clip) = self.project.clips.get(self.selected) {
            self.trim_in = u32::try_from(clip.start.0 / FRAME_TICKS).unwrap_or(0);
            self.trim_out = u32::try_from(clip.end.0 / FRAME_TICKS).unwrap_or(1);
        }
    }

    fn checkpoint(&mut self, before: Project) {
        if self.undo.len() >= 32 {
            self.undo.remove(0);
        }
        self.undo.push(before);
        self.redo.clear();
        self.playhead = self
            .playhead
            .min(self.project.total_frames().saturating_sub(1));
        self.persist();
    }

    fn persist(&mut self) {
        self.status = match self.save_path.as_deref() {
            Some(path) => match write_project(&self.project, path) {
                Ok(()) => "Saved on this device".to_owned(),
                Err(error) => format!("Save failed: {error}"),
            },
            None => "Edits are in memory; storage is unavailable".to_owned(),
        };
    }

    fn transport(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Start").clicked() {
                self.playhead = 0;
                self.fractional_frame = 0.0;
            }
            if ui
                .button(if self.playing { "Pause" } else { "Play" })
                .clicked()
            {
                self.playing = !self.playing;
            }
            ui.label(
                egui::RichText::new(timecode(self.playhead))
                    .monospace()
                    .color(ACCENT),
            );
            ui.label(
                egui::RichText::new(format!("/ {}", timecode(self.project.total_frames())))
                    .color(MUTED),
            );
        });
        let last = self.project.total_frames().saturating_sub(1);
        let slider = ui
            .scope(|ui| {
                ui.spacing_mut().slider_width = ui.available_width().max(44.0);
                ui.add(egui::Slider::new(&mut self.playhead, 0..=last).show_value(false))
            })
            .inner;
        if slider.dragged() {
            self.playing = false;
            self.fractional_frame = 0.0;
        }
    }

    fn viewer(&mut self, ui: &mut egui::Ui, source: DemoSource, source_frame: u32) {
        draw_preview(ui, source, source_frame);
        ui.label(
            egui::RichText::new("Generated demo footage - no video import / export yet")
                .size(11.0)
                .color(MUTED),
        );
        self.transport(ui);
    }

    fn timeline(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("TIMELINE").strong().color(MUTED));
            ui.label(format!("{} clips / 30 fps", self.project.clips.len()));
        });
        let mut choose = None;
        let mut offset = 0_u32;
        egui::ScrollArea::horizontal()
            .id_salt("timeline")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (index, clip) in self.project.clips.iter().enumerate() {
                        let selected = index == self.selected;
                        let title = format!(
                            "{}\n{:.1}s",
                            clip.source.title(),
                            f64::from(clip.frames()) / 30.0
                        );
                        let button = egui::Button::new(egui::RichText::new(title).size(13.0))
                            .fill(source_color(clip.source).gamma_multiply(0.45))
                            .stroke(Stroke::new(
                                if selected { 2.0 } else { 1.0 },
                                if selected {
                                    ACCENT
                                } else {
                                    MUTED.gamma_multiply(0.3)
                                },
                            ));
                        if ui.add_sized([116.0, 66.0], button).clicked() {
                            choose = Some((index, offset));
                        }
                        offset = offset.saturating_add(clip.frames());
                    }
                });
            });
        if let Some((index, offset)) = choose {
            self.selected = index;
            self.playhead = offset;
            self.playing = false;
            self.sync_trim();
        }
        ui.horizontal_wrapped(|ui| {
            if ui.button("Split here").clicked() {
                let before = self.project.clone();
                match self.project.split(self.selected, self.playhead) {
                    Ok(()) => {
                        self.checkpoint(before);
                        self.sync_trim();
                    }
                    Err(error) => self.status = error.to_string(),
                }
            }
            if ui
                .add_enabled(!self.undo.is_empty(), egui::Button::new("Undo"))
                .clicked()
                && let Some(previous) = self.undo.pop()
            {
                self.redo
                    .push(std::mem::replace(&mut self.project, previous));
                self.reconcile();
            }
            if ui
                .add_enabled(!self.redo.is_empty(), egui::Button::new("Redo"))
                .clicked()
                && let Some(next) = self.redo.pop()
            {
                self.undo.push(std::mem::replace(&mut self.project, next));
                self.reconcile();
            }
        });
    }

    fn reconcile(&mut self) {
        self.selected = self
            .selected
            .min(self.project.clips.len().saturating_sub(1));
        self.playhead = self
            .playhead
            .min(self.project.total_frames().saturating_sub(1));
        self.playing = false;
        self.sync_trim();
        self.persist();
    }

    fn inspector(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        let name = self
            .project
            .clips
            .get(self.selected)
            .map(|c| c.source.title())
            .unwrap_or("No clip");
        ui.label(egui::RichText::new(format!("TRIM / {name}")).strong());
        ui.add(egui::Slider::new(&mut self.trim_in, 0..=299).text("In frame"));
        ui.add(egui::Slider::new(&mut self.trim_out, 1..=300).text("Out frame"));
        ui.horizontal_wrapped(|ui| {
            if ui.button("Apply trim").clicked() {
                let before = self.project.clone();
                match self
                    .project
                    .trim(self.selected, self.trim_in, self.trim_out)
                {
                    Ok(()) => self.checkpoint(before),
                    Err(error) => self.status = error.to_string(),
                }
            }
            for (label, destination) in [
                ("Move left", self.selected.checked_sub(1)),
                (
                    "Move right",
                    self.selected
                        .checked_add(1)
                        .filter(|i| *i < self.project.clips.len()),
                ),
            ] {
                if ui
                    .add_enabled(destination.is_some(), egui::Button::new(label))
                    .clicked()
                    && let Some(destination) = destination
                {
                    let before = self.project.clone();
                    match self.project.move_clip(self.selected, destination) {
                        Ok(()) => {
                            self.selected = destination;
                            self.checkpoint(before);
                            self.sync_trim();
                        }
                        Err(error) => self.status = error.to_string(),
                    }
                }
            }
        });
    }
}

impl eframe::App for MobileEditor {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.playing {
            let elapsed = f64::from(ui.input(|i| i.stable_dt)).min(0.1);
            self.fractional_frame += elapsed.max(0.0) * 30.0;
            let frames = self.fractional_frame.floor() as u32;
            self.fractional_frame -= f64::from(frames);
            let total = self.project.total_frames().max(1);
            self.playhead = self.playhead.saturating_add(frames) % total;
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
        }
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_max_width(ui.available_width().min(1000.0));
                ui.add_space(28.0); // NativeActivity system bars; validated on emulator separately.
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("FILMCRAFT").size(25.0).strong().color(ACCENT));
                    ui.label(egui::RichText::new("MOBILE").size(12.0).color(MUTED));
                });
                ui.label(egui::RichText::new("EARLY PREVIEW  /  0.1.0-alpha.1").size(11.0).color(MUTED));
                ui.add_space(6.0);
                let (index, local) = self.project.clip_at(self.playhead).unwrap_or((0, 0));
                let source = self.project.clips.get(index).map(|c| c.source).unwrap_or(DemoSource::Dawn);
                let source_frame = self.project.clips.get(index)
                    .and_then(|clip| u32::try_from(clip.start.0 / FRAME_TICKS).ok())
                    .unwrap_or(0).saturating_add(local);
                if filmcraft_mobile_core::layout_for_width(ui.available_width() as u32)
                    == filmcraft_mobile_core::Layout::Expanded
                {
                    ui.columns(2, |columns| {
                        if let [viewer, tools] = columns {
                            self.viewer(viewer, source, source_frame);
                            self.timeline(tools);
                            self.inspector(tools);
                        }
                    });
                } else {
                    self.viewer(ui, source, source_frame);
                    ui.add_space(4.0);
                    self.timeline(ui);
                    self.inspector(ui);
                }
                ui.add_space(5.0);
                ui.label(egui::RichText::new(&self.status).color(ACCENT).size(12.0));
                ui.collapsing("About this preview", |ui| {
                    ui.label("Native Rust interface. Offline. No account or network permissions.");
                    ui.label("Editing uses FilmCraft integer Tick time. Demo state saves after every edit.");
                    ui.label("This prototype does not edit or export your device's videos.");
                    ui.label("MIT OR Apache-2.0. Third-party notices are included in the APK assets.");
                    if ui.button("Reset demo project").clicked() {
                        let before = std::mem::take(&mut self.project);
                        self.selected = 0;
                        self.playhead = 0;
                        self.playing = false;
                        self.checkpoint(before);
                        self.sync_trim();
                    }
                });
                ui.add_space(32.0);
            });
        });
    }
}

fn source_color(source: DemoSource) -> Color32 {
    match source {
        DemoSource::Dawn => Color32::from_rgb(243, 160, 113),
        DemoSource::Waves => Color32::from_rgb(87, 179, 206),
        DemoSource::Night => Color32::from_rgb(149, 139, 218),
    }
}

fn timecode(frame: u32) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        frame / 1800,
        (frame / 30) % 60,
        frame % 30
    )
}

fn draw_preview(ui: &mut egui::Ui, source: DemoSource, frame: u32) {
    let width = ui.available_width().min(960.0);
    let (response, painter) =
        ui.allocate_painter(Vec2::new(width, width * 9.0 / 16.0), Sense::hover());
    let rect = response.rect;
    painter.rect_filled(rect, 8.0, Color32::from_rgb(22, 34, 48));
    let color = source_color(source);
    for strip in 0..24 {
        let y0 = rect.top() + rect.height() * strip as f32 / 24.0;
        let y1 = rect.top() + rect.height() * (strip + 1) as f32 / 24.0;
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(rect.left(), y0), Pos2::new(rect.right(), y1)),
            0.0,
            color.gamma_multiply(0.15 + 0.45 * (1.0 - strip as f32 / 24.0)),
        );
    }
    let phase = frame as f32 / 30.0;
    let sun = Pos2::new(
        rect.left() + width * (0.55 + 0.16 * (phase * 0.6).sin()),
        rect.top() + rect.height() * 0.38,
    );
    painter.circle_filled(sun, width * 0.09, color);
    for line in 0..7 {
        let y = rect.top() + rect.height() * (0.60 + line as f32 * 0.045);
        let points: Vec<Pos2> = (0..=40)
            .map(|x| {
                let t = x as f32 / 40.0;
                Pos2::new(
                    rect.left() + width * t,
                    y + (t * 10.0 + phase + line as f32).sin() * width * 0.014,
                )
            })
            .collect();
        painter.add(egui::Shape::line(
            points,
            Stroke::new(1.3, color.gamma_multiply(0.45)),
        ));
    }
    painter.text(
        rect.left_top() + Vec2::new(14.0, 13.0),
        egui::Align2::LEFT_TOP,
        "DEMO / PROCEDURAL",
        FontId::monospace(11.0),
        Color32::WHITE,
    );
    painter.text(
        rect.left_bottom() + Vec2::new(14.0, -13.0),
        egui::Align2::LEFT_BOTTOM,
        source.title(),
        FontId::proportional(18.0),
        Color32::WHITE,
    );
}

fn read_project(path: &Path) -> Result<Project, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut text = String::new();
    file.take(MAX_PROJECT_BYTES + 1).read_to_string(&mut text)?;
    if text.len() as u64 > MAX_PROJECT_BYTES {
        return Err("Saved project is too large".into());
    }
    let project: Project = serde_json::from_str(&text)?;
    project.validate()?;
    Ok(project)
}

fn write_project(project: &Project, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    project.validate()?;
    let temporary = path.with_extension("tmp");
    let mut file = std::fs::File::create(&temporary)?;
    serde_json::to_writer(&mut file, project)?;
    file.flush()?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(temporary, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_edits_replace_previous_snapshot_and_reopen() -> Result<(), Box<dyn std::error::Error>>
    {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("project.json");
        let mut project = Project::default();
        write_project(&project, &path)?;
        project.trim(0, 30, 90)?;
        write_project(&project, &path)?;
        assert_eq!(read_project(&path)?, project);
        Ok(())
    }

    #[test]
    fn invalid_save_keeps_previous_project() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("project.json");
        let project = Project::default();
        write_project(&project, &path)?;
        let mut invalid = project.clone();
        invalid.clips.clear();
        assert!(write_project(&invalid, &path).is_err());
        assert_eq!(read_project(&path)?, project);
        Ok(())
    }

    #[test]
    fn corrupt_and_oversized_saves_are_errors() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("project.json");
        std::fs::write(&path, "{broken")?;
        assert!(read_project(&path).is_err());
        std::fs::write(&path, vec![b' '; MAX_PROJECT_BYTES as usize + 1])?;
        assert!(read_project(&path).is_err());
        Ok(())
    }
}

#[cfg(target_os = "android")]
#[allow(
    unsafe_code,
    reason = "Android's Rust entry point requires an exported no_mangle symbol; no unsafe block is used"
)]
#[unsafe(no_mangle)]
pub fn android_main(app: android_activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    let path = app
        .internal_data_path()
        .map(|path| path.join("demo-project-v1.json"));
    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eframe::run_native(
            "FilmCraft Mobile",
            options,
            Box::new(move |cc| Ok(Box::new(MobileEditor::new(cc, path)))),
        )
    }));
    match result {
        Ok(Ok(())) => log::info!("FilmCraft activity stopped"),
        Ok(Err(error)) => log::error!("FilmCraft startup failed: {error}"),
        Err(_) => log::error!("FilmCraft stopped after a runtime error"),
    }
}
