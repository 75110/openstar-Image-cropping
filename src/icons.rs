//! Material Icons 工具模块

use eframe::egui;

/// Material Icons 图标字符映射
pub mod icon {
    // 文件相关
    pub const FOLDER: &str = "\u{e2c7}";           // folder
    pub const FOLDER_OPEN: &str = "\u{e2c8}";      // folder_open
    pub const INSERT_DRIVE_FILE: &str = "\u{e24d}"; // insert_drive_file
    pub const FILE_UPLOAD: &str = "\u{e2c6}";      // file_upload
    pub const FILE_DOWNLOAD: &str = "\u{e2c4}";    // file_download
    
    // 图片相关
    pub const IMAGE: &str = "\u{e3f4}";            // image
    pub const PHOTO_LIBRARY: &str = "\u{e413}";    // photo_library
    pub const CAMERA_ALT: &str = "\u{e3b0}";       // camera_alt
    
    // 操作相关
    pub const SAVE: &str = "\u{e161}";             // save
    pub const DELETE: &str = "\u{e872}";           // delete
    pub const CLEAR: &str = "\u{e0b8}";            // clear
    pub const SETTINGS: &str = "\u{e8b8}";         // settings
    pub const REFRESH: &str = "\u{e5d5}";          // refresh
    
    // 导航相关
    pub const ARROW_BACK: &str = "\u{e5c4}";       // arrow_back
    pub const ARROW_FORWARD: &str = "\u{e5c8}";    // arrow_forward
    pub const ARROW_UPWARD: &str = "\u{e5d8}";     // arrow_upward
    pub const ARROW_DOWNWARD: &str = "\u{e5db}";   // arrow_downward
    pub const FIRST_PAGE: &str = "\u{e5dc}";       // first_page
    pub const LAST_PAGE: &str = "\u{e5dd}";        // last_page
    
    // 播放控制
    pub const PLAY_ARROW: &str = "\u{e037}";       // play_arrow
    pub const PAUSE: &str = "\u{e034}";            // pause
    pub const STOP: &str = "\u{e047}";             // stop
    
    // 选择相关
    pub const CHECK: &str = "\u{e5ca}";            // check
    pub const CLOSE: &str = "\u{e5cd}";            // close
    pub const CANCEL: &str = "\u{e5c9}";           // cancel
    pub const RADIO_BUTTON_UNCHECKED: &str = "\u{e836}"; // radio_button_unchecked
    pub const RADIO_BUTTON_CHECKED: &str = "\u{e837}";   // radio_button_checked
    
    // 信息相关
    pub const INFO: &str = "\u{e88e}";             // info
    pub const HELP: &str = "\u{e887}";             // help
    pub const WARNING: &str = "\u{e002}";          // warning
    pub const ERROR: &str = "\u{e000}";            // error
    
    // 编辑相关
    pub const EDIT: &str = "\u{e3c9}";             // edit
    pub const CUT: &str = "\u{e08b}";              // content_cut
    pub const COPY: &str = "\u{e14d}";             // content_copy
    pub const PASTE: &str = "\u{e14f}";            // content_paste
    
    // 键盘相关
    pub const KEYBOARD: &str = "\u{e312}";         // keyboard
    pub const KEYBOARD_ARROW_UP: &str = "\u{e316}";    // keyboard_arrow_up
    pub const KEYBOARD_ARROW_DOWN: &str = "\u{e313}";  // keyboard_arrow_down
    pub const KEYBOARD_ARROW_LEFT: &str = "\u{e314}";  // keyboard_arrow_left
    pub const KEYBOARD_ARROW_RIGHT: &str = "\u{e315}"; // keyboard_arrow_right
    
    // 其他
    pub const MENU: &str = "\u{e5d2}";             // menu
    pub const MORE_VERT: &str = "\u{e5d4}";        // more_vert
    pub const MORE_HORIZ: &str = "\u{e5d3}";       // more_horiz
    pub const SEARCH: &str = "\u{e8b6}";           // search
    pub const ZOOM_IN: &str = "\u{e8ff}";          // zoom_in
    pub const ZOOM_OUT: &str = "\u{e900}";         // zoom_out
    pub const FULLSCREEN: &str = "\u{e5d0}";       // fullscreen
    pub const FULLSCREEN_EXIT: &str = "\u{e5d1}";  // fullscreen_exit
    pub const GRID_ON: &str = "\u{e3ec}";          // grid_on
    pub const GRID_OFF: &str = "\u{e3eb}";         // grid_off
    pub const CROP: &str = "\u{e3be}";             // crop
    pub const STRAIGHTEN: &str = "\u{e41c}";       // straighten
    pub const FLIP: &str = "\u{e3e8}";             // flip
    pub const ROTATE_LEFT: &str = "\u{e419}";      // rotate_left
    pub const ROTATE_RIGHT: &str = "\u{e41a}";     // rotate_right
}

/// 获取图标字体 ID（现在使用 Proportional 字体家族，让 fallback 机制工作）
pub fn icon_font_id(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Proportional)
}

/// 创建一个图标文本
pub fn icon_text(icon: &str, size: f32) -> egui::RichText {
    egui::RichText::new(icon).font(icon_font_id(size))
}

/// 创建一个带图标和文字的文本
pub fn icon_with_text(icon: &str, text: &str, size: f32) -> egui::RichText {
    egui::RichText::new(format!("{} {}", icon, text))
        .font(icon_font_id(size))
}

/// 创建一个带图标的按钮
pub fn icon_button(ui: &mut egui::Ui, icon: &str, size: f32) -> egui::Response {
    ui.button(icon_text(icon, size))
}
