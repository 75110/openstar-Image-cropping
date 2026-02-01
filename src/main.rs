use eframe::egui;

mod app;
mod icons;
mod image_splitter;

use app::BatchImageSplitterApp;

/// 加载图标
fn load_icon() -> Option<egui::IconData> {
    // 优先使用嵌入的图标数据
    let icon_data = include_bytes!("../icon.ico");
    if let Ok(image) = image::load_from_memory(icon_data) {
        let image = image.to_rgba8();
        let (width, height) = image.dimensions();
        return Some(egui::IconData {
            rgba: image.into_raw(),
            width,
            height,
        });
    }
    
    // 如果嵌入失败（不应该发生），尝试从文件加载
    let icon_names = ["icon.ico", "LOGO.png"];
    let mut search_dirs = vec![
        std::env::current_dir().unwrap_or_default(),
    ];
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            search_dirs.push(parent.to_path_buf());
        }
    }

    for name in &icon_names {
        for dir in &search_dirs {
            let path = dir.join(name);
            if path.exists() {
                if let Ok(data) = std::fs::read(&path) {
                    if let Ok(image) = image::load_from_memory(&data) {
                        let image = image.to_rgba8();
                        let (width, height) = image.dimensions();
                        return Some(egui::IconData {
                            rgba: image.into_raw(),
                            width,
                            height,
                        });
                    }
                }
            }
        }
    }
    None
}

fn main() -> eframe::Result<()> {
    // 图标加载很快，直接在主线程加载以确保 ViewportBuilder 能立即使用它
    let icon = load_icon();
    
    // 异步加载最耗时的中文字体
    let font_paths = [
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\msyhbd.ttc",
        "C:\\Windows\\Fonts\\simhei.ttf",
        "C:\\Windows\\Fonts\\simsun.ttc",
    ];
    
    let font_handle = std::thread::spawn(move || {
        for path in font_paths {
            if let Ok(data) = std::fs::read(path) {
                return Some(data);
            }
        }
        None
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_drag_and_drop(true)
            .with_icon(icon.unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native(
        "Batch Image Splitter",
        options,
        Box::new(move |cc| {
            // 配置字体
            let mut fonts = egui::FontDefinitions::default();
            
            // 使用异步加载好的中文字体数据
            if let Ok(Some(font_data)) = font_handle.join() {
                fonts.font_data.insert(
                    "chinese".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                
                for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                    fonts.families.entry(family).or_default().insert(0, "chinese".to_owned());
                }
            }
            
            // 加载图标字体（直接嵌入到二进制文件中，确保便携性）
            let icon_font_data = include_bytes!("../MaterialIcons-Regular.ttf");
            fonts.font_data.insert(
                "material_icons".to_owned(),
                egui::FontData::from_static(icon_font_data),
            );
            
            // 插入到 Proportional 和 Monospace 家族
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                let family_fonts = fonts.families.entry(family).or_default();
                // 紧跟在中文字体后面，或者放在第一位
                if family_fonts.contains(&"chinese".to_owned()) {
                    family_fonts.insert(1, "material_icons".to_owned());
                } else {
                    family_fonts.insert(0, "material_icons".to_owned());
                }
            }
            
            cc.egui_ctx.set_fonts(fonts);
            
            // 应用现代化全局样式
            configure_custom_style(&cc.egui_ctx);
            
            Ok(Box::new(BatchImageSplitterApp::new(cc)))
        }),
    )
}

/// 配置自定义现代化样式
fn configure_custom_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // 1. 字体排版优化
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(18.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(11.0, egui::FontFamily::Proportional),
    );
    
    // 2. 间距优化
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.window_margin = egui::Margin::same(16.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.indent = 20.0;
    
    // 3. 视觉效果优化 (Light Theme)
    let mut visuals = egui::Visuals::light();
    
    // 圆角
    visuals.window_rounding = egui::Rounding::same(12.0);
    visuals.menu_rounding = egui::Rounding::same(8.0);
    
    // 控件样式
    visuals.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
    visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
    visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
    visuals.widgets.active.rounding = egui::Rounding::same(6.0);
    visuals.widgets.open.rounding = egui::Rounding::same(6.0);
    
    // 颜色微调
    // 主背景色 - 极淡的灰
    visuals.panel_fill = egui::Color32::from_rgb(249, 250, 251); 
    
    // 控件背景
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(255, 255, 255);
    visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(255, 255, 255);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(229, 231, 235));
    
    // 交互状态 - 使用用户指定的深青绿色 (#134e4a)
    let primary_color = egui::Color32::from_rgb(19, 78, 74); 
    let primary_hover = egui::Color32::from_rgb(20, 95, 90);  
    
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, primary_color);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(240, 253, 250); // 非常浅的青绿色背景
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5, primary_hover);
    
    // 选区颜色
    visuals.selection.bg_fill = egui::Color32::from_rgb(204, 251, 241); // Teal 100
    visuals.selection.stroke = egui::Stroke::new(1.0, primary_hover);
    
    style.visuals = visuals;
    ctx.set_style(style);
}
