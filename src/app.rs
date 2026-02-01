use eframe::egui;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use serde::Deserialize;

use crate::icons::{icon, icon_text};
use crate::image_splitter::{ImageSplitter, SplitConfig};

#[derive(Clone, Copy, PartialEq, Debug)]
enum LineType {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq)]
enum UpdateStatus {
    Idle,
    Checking,
    NewVersion(String, String), // version, download_url
    UpToDate,
    Error(String),
}

#[derive(Deserialize, Debug)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
}

pub struct BatchImageSplitterApp {
    // 图片列表
    image_paths: Vec<PathBuf>,
    current_index: usize,
    
    // 当前显示的图片
    current_texture: Option<egui::TextureHandle>,
    current_image: Option<image::DynamicImage>,
    
    // 分割配置
    config: SplitConfig,
    saved_config: Option<SplitConfig>,
    
    // 交互状态
    selected_lines: Vec<(LineType, usize)>, // (类型, 索引)
    dragging_line: Option<(LineType, usize)>,
    is_selecting: bool,
    selection_start: Option<egui::Pos2>,
    selection_end: Option<egui::Pos2>,
    
    // 图片显示区域
    image_rect: Option<egui::Rect>,
    // 图片实际显示尺寸（用于坐标转换）
    image_display_scale: f32,
    
    // 状态信息
    status_message: String,
    show_progress: bool,
    progress: f32,
    
    // 关于窗口
    show_about: bool,
    about_icon: Option<egui::TextureHandle>,
    // 混淆的版权信息
    obfuscated_info_label: String,
    obfuscated_info_url: String,
    obfuscated_repo_label: String,
    obfuscated_repo_url: String,
    
    // 更新状态
    update_status: Arc<Mutex<UpdateStatus>>,
}

// 简单的 XOR 混淆/解密函数
fn xor_cipher(data: &[u8], key: u8) -> String {
    let xored: Vec<u8> = data.iter().map(|&b| b ^ key).collect();
    String::from_utf8_lossy(&xored).to_string()
}

// 混淆后的数据字节数组 (Key: 0x5A)
// "如果您想发现更多有趣好玩的项目、软件，欢迎访问：" -> [232, 139, 133, ...] (xored)
const INFO_PART1: &[u8] = &[
    191, 252, 216, 188, 196, 198, 188, 216, 242, 188, 217, 233, 191, 213, 203, 189, 212, 234, 188, 193, 
    238, 191, 254, 192, 188, 198, 211, 178, 236, 249, 191, 255, 231, 189, 212, 243, 189, 192, 222, 179, 
    251, 227, 189, 193, 244, 185, 218, 219, 178, 231, 245, 190, 225, 236, 181, 230, 214, 188, 246, 248, 
    178, 229, 212, 178, 244, 229, 179, 205, 244, 181, 230, 192
];
// "sevencn.com" -> (xored)
const INFO_PART2: &[u8] = &[41, 63, 44, 63, 52, 57, 52, 116, 57, 53, 55];

// "开源地址：" -> (xored)
const REPO_LABEL: &[u8] = &[191, 230, 218, 188, 224, 202, 191, 198, 234, 191, 199, 218, 181, 230, 192];
// "https://github.com/75110/openstar-Image-cropping" -> (xored)
const REPO_URL: &[u8] = &[
    50, 46, 46, 42, 41, 96, 117, 117, 61, 51, 46, 50, 47, 56, 116, 57, 53, 55, 117, 109, 111, 107, 107, 
    106, 117, 53, 42, 63, 52, 41, 46, 59, 40, 119, 19, 55, 59, 61, 63, 119, 57, 40, 53, 42, 42, 51, 52, 61
];

impl BatchImageSplitterApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // 在初始化时解密
        let info1 = xor_cipher(INFO_PART1, 0x5A);
        let info2 = xor_cipher(INFO_PART2, 0x5A);
        let repo_label = xor_cipher(REPO_LABEL, 0x5A);
        let repo_url = xor_cipher(REPO_URL, 0x5A);
        
        Self {
            image_paths: Vec::new(),
            current_index: 0,
            current_texture: None,
            current_image: None,
            config: SplitConfig::new(1, 1),
            saved_config: None,
            selected_lines: Vec::new(),
            dragging_line: None,
            is_selecting: false,
            selection_start: None,
            selection_end: None,
            image_rect: None,
            image_display_scale: 1.0,
            status_message: "请选择图片文件".to_string(),
            show_progress: false,
            progress: 0.0,
            show_about: false,
            about_icon: None,
            obfuscated_info_label: info1,
            obfuscated_info_url: info2,
            obfuscated_repo_label: repo_label,
            obfuscated_repo_url: repo_url,
            update_status: Arc::new(Mutex::new(UpdateStatus::Idle)),
        }
    }

    fn load_about_icon(&mut self, ctx: &egui::Context) {
        if self.about_icon.is_none() {
            // 尝试多个可能的图标路径
            let icon_names = ["icon.ico", "LOGO.png"];
            let mut search_dirs = vec![PathBuf::from(".")];
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(parent) = exe_path.parent() {
                    search_dirs.push(parent.to_path_buf());
                }
            }

            for name in &icon_names {
                for dir in &search_dirs {
                    let path = dir.join(name);
                    if path.exists() {
                        if let Ok(img) = image::open(&path) {
                            let size = [img.width() as usize, img.height() as usize];
                            let rgba = img.to_rgba8();
                            let pixels = rgba.as_raw();
                            
                            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels);
                            let texture = ctx.load_texture(
                                "about_icon",
                                color_image,
                                egui::TextureOptions::default(),
                            );
                            
                            self.about_icon = Some(texture);
                            return; // 找到并加载成功后退出
                        }
                    }
                }
            }
        }
    }

    fn load_image(&mut self, ctx: &egui::Context, path: &PathBuf) {
        match ImageSplitter::open_image(path) {
            Ok(img) => {
                let size = [img.width() as usize, img.height() as usize];
                let rgba = img.to_rgba8();
                let pixels = rgba.as_raw();
                
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels);
                let texture = ctx.load_texture(
                    "current_image",
                    color_image,
                    egui::TextureOptions::default(),
                );
                
                self.current_texture = Some(texture);
                self.current_image = Some(img);
                self.status_message = format!("已加载: {}", path.file_name().unwrap_or_default().to_string_lossy());
            }
            Err(e) => {
                self.status_message = format!("加载失败: {}", e);
            }
        }
    }

    fn show_previous_image(&mut self, ctx: &egui::Context) {
        if self.current_index > 0 {
            self.current_index -= 1;
            let path = self.image_paths.get(self.current_index).cloned();
            if let Some(path) = path {
                self.load_image(ctx, &path);
            }
        }
    }

    fn show_next_image(&mut self, ctx: &egui::Context) {
        if self.current_index + 1 < self.image_paths.len() {
            self.current_index += 1;
            let path = self.image_paths.get(self.current_index).cloned();
            if let Some(path) = path {
                self.load_image(ctx, &path);
            }
        }
    }

    fn save_config(&mut self) {
        self.saved_config = Some(self.config.clone());
        self.status_message = format!("已保存: {}行 x {}列", self.config.rows, self.config.cols);
    }

    fn start_batch_process(&mut self) {
        if self.image_paths.is_empty() {
            return;
        }

        // 在主线程中打开文件对话框
        if let Some(output_dir) = rfd::FileDialog::new().pick_folder() {
            let config = self.saved_config.clone().unwrap_or_else(|| self.config.clone());
            let paths = self.image_paths.clone();

            std::thread::spawn(move || {
                match ImageSplitter::batch_process(&paths, &config, &output_dir, |current, total| {
                    let progress = current as f32 / total as f32;
                    println!("进度: {:.1}%", progress * 100.0);
                }) {
                    Ok((processed, failed)) => {
                        println!("处理完成: {} 成功, {} 失败", processed, failed);
                    }
                    Err(e) => {
                        eprintln!("批量处理失败: {}", e);
                    }
                }
            });
        }
    }

    fn check_for_updates(&self, ctx: egui::Context) {
        let repo_url = self.obfuscated_repo_url.clone();
        let current_version = env!("CARGO_PKG_VERSION").to_string();
        let update_status = self.update_status.clone();
        
        // 设置状态为正在检查
        {
            if let Ok(mut status) = update_status.lock() {
                *status = UpdateStatus::Checking;
            }
        }

        // 转换 GitHub URL 到 API URL
        let api_url = if repo_url.starts_with("http") {
            repo_url.replace("github.com", "api.github.com/repos")
        } else {
            format!("https://{}", repo_url.replace("github.com", "api.github.com/repos"))
        } + "/releases/latest";

        std::thread::spawn(move || {
            let agent = ureq::AgentBuilder::new()
                .timeout(std::time::Duration::from_secs(10))
                .build();
            
            let result = (|| -> Result<UpdateStatus, String> {
                let response = match agent.get(&api_url)
                    .set("User-Agent", "BatchImageSplitter-UpdateChecker")
                    .call() {
                        Ok(resp) => resp,
                        Err(ureq::Error::Status(404, _)) => {
                            // 404 通常意味着没有 release
                            return Ok(UpdateStatus::UpToDate);
                        }
                        Err(e) => return Err(format!("网络请求失败: {}", e)),
                    };
                
                let release = response.into_json::<GithubRelease>()
                    .map_err(|e| format!("解析响应失败: {}", e))?;
                
                let latest_tag = release.tag_name.trim_start_matches('v');
                let current_tag = current_version.trim_start_matches('v');
                
                match (semver::Version::parse(latest_tag), semver::Version::parse(current_tag)) {
                    (Ok(latest), Ok(current)) => {
                        if latest > current {
                            Ok(UpdateStatus::NewVersion(release.tag_name, release.html_url))
                        } else {
                            Ok(UpdateStatus::UpToDate)
                        }
                    }
                    _ => Err(format!("版本解析失败: {} vs {}", latest_tag, current_tag)),
                }
            })();

            if let Ok(mut status) = update_status.lock() {
                match result {
                    Ok(new_status) => *status = new_status,
                    Err(e) => *status = UpdateStatus::Error(e),
                }
            }
            ctx.request_repaint();
        });
    }
}

/// 绘制卡片风格的容器
fn draw_card<R>(
    ui: &mut egui::Ui,
    title: &str,
    icon: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R
) -> R {
    egui::Frame::none()
        .fill(egui::Color32::WHITE)
        .rounding(12.0)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(229, 231, 235))) // Gray 200
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(icon_text(icon, 18.0).color(egui::Color32::from_rgb(19, 78, 74))); // #134e4a
                ui.add_space(4.0);
                ui.label(egui::RichText::new(title).strong().size(15.0).color(egui::Color32::from_rgb(31, 41, 55))); // Dark text
            });
            ui.add_space(12.0);
            add_contents(ui)
        }).inner
}

impl eframe::App for BatchImageSplitterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 快捷键处理
        let mut should_prev = false;
        let mut should_next = false;
        let mut should_open = false;
        let mut should_save = false;
        let mut should_process = false;
        let mut h_adjust: Vec<(usize, f32)> = Vec::new();
        let mut v_adjust: Vec<(usize, f32)> = Vec::new();
        
        ctx.input(|i| {
            if i.modifiers.ctrl {
                if i.key_pressed(egui::Key::ArrowLeft) { should_prev = true; }
                if i.key_pressed(egui::Key::ArrowRight) { should_next = true; }
                if i.key_pressed(egui::Key::O) { should_open = true; }
                if i.key_pressed(egui::Key::S) { should_save = true; }
                if i.key_pressed(egui::Key::Enter) { should_process = true; }
            } else if !self.selected_lines.is_empty() && !i.modifiers.ctrl {
                let step = if i.modifiers.shift { 0.005 } else { 0.001 };
                for (line_type, index) in &self.selected_lines {
                    match line_type {
                        LineType::Horizontal => {
                            if i.key_pressed(egui::Key::ArrowUp) { h_adjust.push((*index, -step)); }
                            if i.key_pressed(egui::Key::ArrowDown) { h_adjust.push((*index, step)); }
                        }
                        LineType::Vertical => {
                            if i.key_pressed(egui::Key::ArrowLeft) { v_adjust.push((*index, -step)); }
                            if i.key_pressed(egui::Key::ArrowRight) { v_adjust.push((*index, step)); }
                        }
                    }
                }
            }
        });
        
        if should_prev { self.show_previous_image(ctx); }
        if should_next { self.show_next_image(ctx); }
        if should_open {
            if let Some(paths) = rfd::FileDialog::new()
                .add_filter("图片", &["jpg", "jpeg", "png", "bmp", "gif"])
                .pick_files()
            {
                for path in paths { self.image_paths.push(path); }
                if self.current_texture.is_none() && !self.image_paths.is_empty() {
                    self.load_image(ctx, &self.image_paths[0].clone());
                }
            }
        }
        if should_save { self.save_config(); }
        if should_process { self.start_batch_process(); }
        
        for (index, delta) in h_adjust {
            if let Some(line) = self.config.h_lines.get_mut(index) {
                *line = (*line + delta).max(0.0).min(1.0);
            }
        }
        for (index, delta) in v_adjust {
            if let Some(line) = self.config.v_lines.get_mut(index) {
                *line = (*line + delta).max(0.0).min(1.0);
            }
        }

        // 1. 右侧控制面板
        egui::SidePanel::right("control_panel")
            .resizable(false)
            .exact_width(320.0)
            .frame(egui::Frame::side_top_panel(ctx.style().as_ref())
                .fill(egui::Color32::from_rgb(249, 250, 251))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(19, 78, 74)))) // #134e4a
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(8.0);

                    // 文件选择卡片
                    draw_card(ui, "文件操作", icon::FOLDER_OPEN, |ui| {
                        // 选择文件按钮
                        let file_btn = ui.add_sized(
                            [ui.available_width(), 40.0],
                            egui::Button::new(
                                egui::RichText::new(format!("{} 选择文件", icon::INSERT_DRIVE_FILE))
                                    .size(14.0)
                                    .strong()
                                    .color(egui::Color32::WHITE) // 显式设置为白色
                            )
                            .fill(egui::Color32::from_rgb(19, 78, 74)) // #134e4a
                            .rounding(8.0)
                        );
                        if file_btn.clicked() {
                            if let Some(paths) = rfd::FileDialog::new()
                                .add_filter("图片", &["jpg", "jpeg", "png", "bmp", "gif"])
                                .pick_files()
                            {
                                for path in paths { self.image_paths.push(path); }
                                if self.current_texture.is_none() && !self.image_paths.is_empty() {
                                    self.load_image(ctx, &self.image_paths[0].clone());
                                }
                            }
                        }
                        
                        ui.add_space(8.0);
                        
                        // 选择文件夹按钮
                        let folder_btn = ui.add_sized(
                            [ui.available_width(), 40.0],
                            egui::Button::new(
                                egui::RichText::new(format!("{} 选择文件夹", icon::FOLDER)).size(13.0).color(egui::Color32::from_rgb(55, 65, 81))
                            )
                            .fill(egui::Color32::WHITE)
                            .rounding(8.0)
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(209, 213, 219)))
                        );
                        if folder_btn.clicked() {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                if let Ok(entries) = std::fs::read_dir(&folder) {
                                    for entry in entries.flatten() {
                                        let path = entry.path();
                                        if let Some(ext) = path.extension() {
                                            let ext = ext.to_string_lossy().to_lowercase();
                                            if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "bmp" | "gif") {
                                                self.image_paths.push(path);
                                            }
                                        }
                                    }
                                }
                                if self.current_texture.is_none() && !self.image_paths.is_empty() {
                                    self.load_image(ctx, &self.image_paths[0].clone());
                                }
                            }
                        }
                    });

                    ui.add_space(12.0);

                    // 分割设置卡片
                    draw_card(ui, "分割设置", icon::SETTINGS, |ui| {
                         // 行数设置
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("分割行数:").size(13.0).color(egui::Color32::from_rgb(75, 85, 99)));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let mut rows = self.config.rows;
                                if ui.add(egui::DragValue::new(&mut rows).range(1..=10).speed(1)).changed() {
                                    self.config.rows = rows;
                                    self.config.reset_to_default();
                                }
                            });
                        });
                        
                        ui.add_space(8.0);
                        
                        // 列数设置
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("分割列数:").size(13.0).color(egui::Color32::from_rgb(75, 85, 99)));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let mut cols = self.config.cols;
                                if ui.add(egui::DragValue::new(&mut cols).range(1..=10).speed(1)).changed() {
                                    self.config.cols = cols;
                                    self.config.reset_to_default();
                                }
                            });
                        });
                        
                        ui.add_space(12.0);
                        
                        // 保存分割线位置按钮
                        let save_btn = ui.add_sized(
                            [ui.available_width(), 40.0],
                            egui::Button::new(
                                egui::RichText::new(format!("{} 保存分割线位置", icon::SAVE)).size(13.0).strong().color(egui::Color32::WHITE)
                            )
                            .fill(egui::Color32::from_rgb(19, 78, 74)) // #134e4a
                            .rounding(8.0)
                        );
                        if save_btn.clicked() {
                            self.save_config();
                        }
                        
                        // 保存状态
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                             if let Some(ref cfg) = self.saved_config {
                                ui.label(egui::RichText::new(format!("{} 已保存: {}行 x {}列", icon::CHECK, cfg.rows, cfg.cols))
                                    .size(12.0).color(egui::Color32::from_rgb(34, 197, 94)));
                            } else {
                                ui.label(egui::RichText::new(format!("{} 未保存分割线位置", icon::WARNING))
                                    .size(12.0).color(egui::Color32::from_rgb(251, 146, 60)));
                            };
                        });
                    });

                    ui.add_space(12.0);

                    // 图片列表卡片
                    draw_card(ui, "图片列表", icon::PHOTO_LIBRARY, |ui| {
                        // 图片列表
                        let paths_to_load: Vec<_> = self.image_paths.clone();
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(249, 250, 251))
                            .rounding(6.0)
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(229, 231, 235)))
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                                    for (idx, path) in paths_to_load.iter().enumerate() {
                                        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                                        let is_selected = idx == self.current_index;
                                        let response = ui.selectable_label(is_selected, &name);
                                        if response.clicked() {
                                            self.current_index = idx;
                                            self.load_image(ctx, path);
                                        }
                                    }
                                });
                            });
                        
                        ui.add_space(8.0);
                        
                        // 导航按钮
                        ui.horizontal(|ui| {
                            if ui.add_sized([ui.available_width() / 2.0 - 4.0, 32.0], egui::Button::new(icon::ARROW_BACK)).clicked() {
                                self.show_previous_image(ctx);
                            }
                            if ui.add_sized([ui.available_width() / 2.0 - 4.0, 32.0], egui::Button::new(icon::ARROW_FORWARD)).clicked() {
                                self.show_next_image(ctx);
                            }
                        });

                        ui.add_space(8.0);

                        // 清除按钮和计数
                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new(format!("{} 清除", icon::DELETE)).small()).clicked() {
                                self.image_paths.clear();
                                self.current_index = 0;
                                self.current_texture = None;
                                self.current_image = None;
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(format!("{} 张图片", self.image_paths.len())).size(12.0).color(egui::Color32::GRAY));
                            });
                        });
                    });

                    ui.add_space(12.0);
                    
                    // 开始处理按钮
                    let process_btn = ui.add_sized(
                        [ui.available_width(), 48.0],
                        egui::Button::new(
                            egui::RichText::new(format!("{} 开始批量处理", icon::PLAY_ARROW)).size(16.0).strong().color(egui::Color32::WHITE)
                        )
                        .fill(egui::Color32::from_rgb(19, 78, 74)) // #134e4a
                        .rounding(10.0)
                    );
                    if process_btn.clicked() {
                        self.start_batch_process();
                    }
                    
                    ui.add_space(12.0);

                    // 快捷键提示
                    ui.label(egui::RichText::new("快捷键提示:").strong().size(13.0).color(egui::Color32::from_rgb(31, 41, 55)));
                    ui.add_space(4.0);
                    
                    let hint_color = egui::Color32::from_rgb(107, 114, 128);
                    ui.label(egui::RichText::new("• Ctrl + O: 打开图片文件").size(11.5).color(hint_color));
                    ui.label(egui::RichText::new("• Ctrl + S: 保存当前分割线配置").size(11.5).color(hint_color));
                    ui.label(egui::RichText::new("• Ctrl + Enter: 开始批量处理").size(11.5).color(hint_color));
                    ui.label(egui::RichText::new("• Ctrl + ← / →: 上一张 / 下一张").size(11.5).color(hint_color));
                    ui.label(egui::RichText::new("• 方向键: 微调选中分割线 (加Shift加速)").size(11.5).color(hint_color));
                    
                    ui.add_space(12.0);
                    
                    // 状态信息 (整合到侧边栏底部)
                    ui.separator();
                    ui.add_space(8.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new(format!("{} 状态:", icon::INFO)).size(12.0).color(egui::Color32::from_rgb(19, 78, 74)));
                        ui.label(egui::RichText::new(&self.status_message).size(12.0).color(egui::Color32::from_rgb(75, 85, 99)));
                    });
                    
                    ui.add_space(12.0);
                    
                    // 关于按钮
                    if ui.button(format!("{} 关于软件", icon::INFO)).clicked() {
                        self.show_about = true;
                    }
                });
            });

        // 2. 中央图片区域
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ctx.style().as_ref()).fill(egui::Color32::from_rgb(243, 244, 246))) // 浅色背景
            .show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    if let Some(texture) = &self.current_texture {
                        let available_size = ui.available_size();
                        let texture_size = texture.size_vec2();
                        
                        let scale = (available_size.x / texture_size.x)
                            .min(available_size.y / texture_size.y);
                        let display_size = texture_size * scale;
                        self.image_display_scale = scale;

                        let image_rect = egui::Rect::from_center_size(
                            ui.available_rect_before_wrap().center(),
                            display_size,
                        );

                        let response = ui.add(
                            egui::Image::new(texture)
                                .fit_to_exact_size(display_size)
                                .sense(egui::Sense::click_and_drag()),
                        );

                        self.image_rect = Some(image_rect);

                        // 处理框选
                        if let Some(rect) = self.image_rect {
                            if response.drag_started() {
                                if let Some(pointer_pos) = response.interact_pointer_pos() {
                                    self.is_selecting = true;
                                    self.selection_start = Some(pointer_pos);
                                    self.selection_end = self.selection_start;
                                    if !ui.input(|i| i.modifiers.shift) {
                                        self.selected_lines.clear();
                                    }
                                }
                            }
                            
                            if self.is_selecting {
                                if let Some(pointer_pos) = response.interact_pointer_pos() {
                                    self.selection_end = Some(pointer_pos);
                                }
                            }
                            
                            if response.drag_stopped() && self.is_selecting {
                                if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                                    let selection_rect = egui::Rect::from_two_pos(start, end);
                                    
                                    // 检查水平分割线
                                    for (i, &pos) in self.config.h_lines.iter().enumerate() {
                                        let y = rect.top() + rect.height() * pos;
                                        let line_rect = egui::Rect::from_min_max(
                                            egui::pos2(rect.left(), y - 3.0),
                                            egui::pos2(rect.right(), y + 3.0)
                                        );
                                        if selection_rect.intersects(line_rect) {
                                            let line_key = (LineType::Horizontal, i);
                                            if !self.selected_lines.contains(&line_key) {
                                                self.selected_lines.push(line_key);
                                            }
                                        }
                                    }
                                    
                                    // 检查垂直分割线
                                    for (i, &pos) in self.config.v_lines.iter().enumerate() {
                                        let x = rect.left() + rect.width() * pos;
                                        let line_rect = egui::Rect::from_min_max(
                                            egui::pos2(x - 3.0, rect.top()),
                                            egui::pos2(x + 3.0, rect.bottom())
                                        );
                                        if selection_rect.intersects(line_rect) {
                                            let line_key = (LineType::Vertical, i);
                                            if !self.selected_lines.contains(&line_key) {
                                                self.selected_lines.push(line_key);
                                            }
                                        }
                                    }
                                }
                                self.is_selecting = false;
                                self.selection_start = None;
                                self.selection_end = None;
                            }
                        }

                        // 绘制分割线
                        if let Some(rect) = self.image_rect {
                            let painter = ui.painter();
                            
                            // 水平分割线
                            for (i, &pos) in self.config.h_lines.iter().enumerate() {
                                let y = rect.top() + rect.height() * pos;
                                let is_selected = self.selected_lines.contains(&(LineType::Horizontal, i));
                                let is_dragging = self.dragging_line == Some((LineType::Horizontal, i));
                                
                                let color = if is_selected || is_dragging {
                                    egui::Color32::from_rgb(34, 197, 94) // 绿色
                                } else {
                                    egui::Color32::from_rgb(239, 68, 68) // 红色
                                };
                                
                                let stroke = if is_selected || is_dragging {
                                    egui::Stroke::new(4.0, color)
                                } else {
                                    egui::Stroke::new(2.0, color)
                                };
                                
                                painter.line_segment(
                                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                                    stroke,
                                );
                            }

                            // 垂直分割线
                            for (i, &pos) in self.config.v_lines.iter().enumerate() {
                                let x = rect.left() + rect.width() * pos;
                                let is_selected = self.selected_lines.contains(&(LineType::Vertical, i));
                                let is_dragging = self.dragging_line == Some((LineType::Vertical, i));
                                
                                let color = if is_selected || is_dragging {
                                    egui::Color32::from_rgb(34, 197, 94) // 绿色
                                } else {
                                    egui::Color32::from_rgb(239, 68, 68) // 红色
                                };
                                
                                let stroke = if is_selected || is_dragging {
                                    egui::Stroke::new(3.0, color)
                                } else {
                                    egui::Stroke::new(2.0, color)
                                };
                                
                                painter.line_segment(
                                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                                    stroke,
                                );
                            }
                            
                            // 绘制选择框
                            if self.is_selecting {
                                if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                                    let selection_rect = egui::Rect::from_two_pos(start, end);
                                    painter.rect_stroke(
                                        selection_rect,
                                        0.0,
                                        egui::Stroke::new(1.0, egui::Color32::from_rgb(19, 78, 74)), // #134e4a
                                    );
                                    painter.rect_filled(
                                        selection_rect,
                                        0.0,
                                        egui::Color32::from_rgba_premultiplied(19, 78, 74, 30), // #134e4a with alpha
                                    );
                                }
                            }
                        }
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(100.0);
                            ui.label(icon_text(icon::IMAGE, 64.0).color(egui::Color32::from_rgb(209, 213, 219)));
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("请点击右侧「选择文件」按钮").size(20.0).color(egui::Color32::from_rgb(107, 114, 128)));
                            ui.label(egui::RichText::new("或使用 Ctrl+O 快捷键").size(14.0).color(egui::Color32::from_rgb(156, 163, 175)));
                        });
                    }
                });
            });
        
        // 关于窗口
        if self.show_about {
            self.load_about_icon(ctx);
            egui::Window::new("关于")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .frame(egui::Frame::window(ctx.style().as_ref())
                    .rounding(16.0)
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(19, 78, 74)))) // #134e4a 边框
                .show(ctx, |ui| {
                    ui.set_min_width(360.0);
                    ui.vertical_centered(|ui| {
                        ui.add_space(24.0);
                        if let Some(icon) = &self.about_icon {
                            ui.add(egui::Image::new(icon).fit_to_exact_size(egui::vec2(80.0, 80.0)));
                        } else {
                            ui.label(egui::RichText::new("📷").size(48.0));
                        }
                        ui.add_space(16.0);
                        ui.label(egui::RichText::new("批量图片分割工具").size(22.0).strong().color(egui::Color32::from_rgb(19, 78, 74))); // #134e4a
                        ui.label(egui::RichText::new("v1.2").size(13.0).color(egui::Color32::GRAY));
                        ui.add_space(20.0);
                        
                        // 自定义颜色的分割线
                        let rect = ui.available_rect_before_wrap();
                        ui.painter().line_segment(
                            [egui::pos2(rect.left() + 20.0, ui.cursor().top()), egui::pos2(rect.right() - 20.0, ui.cursor().top())],
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(19, 78, 74).linear_multiply(0.3)), // 30% 不透明度的主题色线条
                        );
                        ui.add_space(20.0);

                        ui.label("简洁高效的图片批量分割工具");
                        ui.add_space(4.0);
                        ui.label("支持自定义分割线位置，批量处理多张图片");
                        ui.add_space(12.0);
                        
                        // 添加开发初衷
                        ui.scope(|ui| {
                            ui.set_max_width(300.0); // 限制宽度以便自动换行
                            ui.label(
                                egui::RichText::new("为什么开发此软件？一些软件都是在日常生活中需要用到的但是找了很久找到的可能不是收费就是各种限制，或者是没有自己想要实现的功能等，那为何不自己做呢？就这么简单~")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(107, 114, 128)) // 灰色
                                    .italics()
                            );
                        });
                        
                        ui.add_space(20.0);
                         ui.horizontal(|ui| {
                             ui.label(egui::RichText::new(&self.obfuscated_info_label).size(12.0));
                             ui.hyperlink_to(
                                 egui::RichText::new(&self.obfuscated_info_url).size(12.0).color(egui::Color32::from_rgb(59, 130, 246)),
                                 format!("https://{}", self.obfuscated_info_url)
                             );
                         });
                         ui.add_space(8.0);
                         ui.horizontal(|ui| {
                             ui.label(egui::RichText::new(&self.obfuscated_repo_label).size(12.0));
                             ui.hyperlink_to(
                                 egui::RichText::new(&self.obfuscated_repo_url).size(12.0).color(egui::Color32::from_rgb(59, 130, 246)),
                                 &self.obfuscated_repo_url
                             );
                         });
                         ui.add_space(24.0);
                         ui.horizontal(|ui| {
                             ui.style_mut().spacing.item_spacing.x = 12.0;
                             
                             let status = if let Ok(s) = self.update_status.lock() {
                                 s.clone()
                             } else {
                                 UpdateStatus::Idle
                             };

                             match status {
                                 UpdateStatus::Idle => {
                                     let check_btn = ui.add_sized(
                                         [120.0, 32.0],
                                         egui::Button::new(egui::RichText::new("检查更新").strong())
                                             .fill(egui::Color32::WHITE)
                                             .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(19, 78, 74)))
                                             .rounding(6.0)
                                     );
                                     if check_btn.clicked() {
                                         self.check_for_updates(ui.ctx().clone());
                                     }
                                 }
                                 UpdateStatus::Checking => {
                                     ui.add_sized([120.0, 32.0], egui::Spinner::new());
                                     ui.label("正在检查...");
                                 }
                                 UpdateStatus::NewVersion(version, url) => {
                                     let download_btn = ui.add_sized(
                                         [120.0, 32.0],
                                         egui::Button::new(egui::RichText::new(format!("下载 {}", version)).strong())
                                             .fill(egui::Color32::from_rgb(19, 78, 74))
                                             .rounding(6.0)
                                     );
                                     if download_btn.clicked() {
                                         ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                                     }
                                     ui.label(egui::RichText::new("发现新版本！").color(egui::Color32::from_rgb(19, 78, 74)));
                                 }
                                 UpdateStatus::UpToDate => {
                                     ui.add_sized([120.0, 32.0], egui::Button::new("已是最新").sense(egui::Sense::hover()));
                                     if ui.button("重新检查").clicked() {
                                         self.check_for_updates(ui.ctx().clone());
                                     }
                                 }
                                 UpdateStatus::Error(e) => {
                                     if ui.add_sized([120.0, 32.0], egui::Button::new("检查失败").rounding(6.0)).clicked() {
                                         self.check_for_updates(ui.ctx().clone());
                                     }
                                     ui.label(egui::RichText::new(e).size(10.0).color(egui::Color32::RED));
                                 }
                             }

                             ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                 if ui.add_sized([80.0, 32.0], egui::Button::new(egui::RichText::new("知道了").strong()).rounding(6.0)).clicked() {
                                     self.show_about = false;
                                 }
                             });
                         });
                        ui.add_space(16.0);
                    });
                });
        }
    }
}
