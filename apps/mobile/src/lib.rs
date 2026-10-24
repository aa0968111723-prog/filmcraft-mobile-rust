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
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "traditional-chinese".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../assets/fonts/NotoSansCJKtc-Regular.otf"
            ))
            .into(),
        );
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            fonts
                .families
                .entry(family)
                .or_default()
                .push("traditional-chinese".to_owned());
        }
        cc.egui_ctx.set_fonts(fonts);
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
            Some(Ok(project)) => (project, "已載入上次儲存的示範專案".to_owned()),
            Some(Err(error)) if save_path.as_ref().is_some_and(|path| path.exists()) => {
                (Project::default(), format!("無法還原專案：{error}"))
            }
            _ => (
                Project::default(),
                "準備就緒，請試著剪輯三段生成的示範片段。".to_owned(),
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
                Ok(()) => "已儲存至這部裝置".to_owned(),
                Err(error) => format!("儲存失敗：{error}"),
            },
            None => "目前無法儲存，編輯內容僅暫存於記憶體。".to_owned(),
        };
    }

    fn transport(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("回到開頭").clicked() {
                self.playhead = 0;
                self.fractional_frame = 0.0;
            }
            if ui
                .button(if self.playing { "暫停" } else { "播放" })
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
            egui::RichText::new("程式生成的示範畫面，目前尚不支援影片匯入與匯出。")
                .size(11.0)
                .color(MUTED),
        );
        self.transport(ui);
    }

    fn timeline(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("時間軸").strong().color(MUTED));
            ui.label(format!("{} 個片段／每秒 30 格", self.project.clips.len()));
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
                            "{}\n{:.1} 秒",
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
            if ui.button("在此分割").clicked() {
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
                .add_enabled(!self.undo.is_empty(), egui::Button::new("復原"))
                .clicked()
                && let Some(previous) = self.undo.pop()
            {
                self.redo
                    .push(std::mem::replace(&mut self.project, previous));
                self.reconcile();
            }
            if ui
                .add_enabled(!self.redo.is_empty(), egui::Button::new("重做"))
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
            .unwrap_or("未選取片段");
        ui.label(egui::RichText::new(format!("裁切／{name}")).strong());
        ui.add(egui::Slider::new(&mut self.trim_in, 0..=299).text("起點影格"));
        ui.add(egui::Slider::new(&mut self.trim_out, 1..=300).text("終點影格"));
        ui.horizontal_wrapped(|ui| {
            if ui.button("套用裁切").clicked() {
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
                ("向左移動", self.selected.checked_sub(1)),
                (
                    "向右移動",
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
                    ui.label(
                        egui::RichText::new("影片工坊")
                            .size(25.0)
                            .strong()
                            .color(ACCENT),
                    );
                    ui.label(egui::RichText::new("行動版").size(12.0).color(MUTED));
                });
                ui.label(
                    egui::RichText::new("早期預覽版／0.1.0 第 2 版")
                        .size(11.0)
                        .color(MUTED),
                );
                ui.add_space(6.0);
                let (index, local) = self.project.clip_at(self.playhead).unwrap_or((0, 0));
                let source = self
                    .project
                    .clips
                    .get(index)
                    .map(|c| c.source)
                    .unwrap_or(DemoSource::Dawn);
                let source_frame = self
                    .project
                    .clips
                    .get(index)
                    .and_then(|clip| u32::try_from(clip.start.0 / FRAME_TICKS).ok())
                    .unwrap_or(0)
                    .saturating_add(local);
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
                ui.collapsing("關於此預覽版", |ui| {
                    ui.label("採用 Rust 原生介面，可離線使用，不需要帳號或網路權限。");
                    ui.label("使用整數時間刻度精確剪輯，每次編輯後自動儲存示範專案。");
                    ui.label("此版本尚不能匯入、編輯或匯出裝置裡的真實影片，也不包含音訊。");
                    ui.label("開源授權：MIT 或 Apache-2.0。安裝檔內附第三方授權原文。");
                    if ui.button("重設示範專案").clicked() {
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
        "示範素材／程式生成",
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

#[derive(Debug)]
enum StorageError {
    Read,
    TooLarge,
    Format,
    Project(filmcraft_mobile_core::editor::ProjectError),
    Write,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => f.write_str("無法讀取專案檔案，請確認儲存空間與存取權限。"),
            Self::TooLarge => f.write_str("專案檔案超過大小上限。"),
            Self::Format => f.write_str("專案檔案格式不正確或已損毀。"),
            Self::Project(error) => error.fmt(f),
            Self::Write => f.write_str("無法寫入專案檔案，請確認儲存空間與存取權限。"),
        }
    }
}

impl std::error::Error for StorageError {}

fn read_project(path: &Path) -> Result<Project, StorageError> {
    let file = std::fs::File::open(path).map_err(|_| StorageError::Read)?;
    let mut text = String::new();
    file.take(MAX_PROJECT_BYTES + 1)
        .read_to_string(&mut text)
        .map_err(|_| StorageError::Read)?;
    if text.len() as u64 > MAX_PROJECT_BYTES {
        return Err(StorageError::TooLarge);
    }
    let project: Project = serde_json::from_str(&text).map_err(|_| StorageError::Format)?;
    project.validate().map_err(StorageError::Project)?;
    Ok(project)
}

fn write_project(project: &Project, path: &Path) -> Result<(), StorageError> {
    project.validate().map_err(StorageError::Project)?;
    let temporary = path.with_extension("tmp");
    let mut file = std::fs::File::create(&temporary).map_err(|_| StorageError::Write)?;
    serde_json::to_writer(&mut file, project).map_err(|_| StorageError::Write)?;
    file.flush().map_err(|_| StorageError::Write)?;
    file.sync_all().map_err(|_| StorageError::Write)?;
    drop(file);
    std::fs::rename(temporary, path).map_err(|_| StorageError::Write)?;
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
            "影片工坊行動版",
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
