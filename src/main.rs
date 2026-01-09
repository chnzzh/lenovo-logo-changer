#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)] // hide console window on Windows in release

mod esp_partition;
mod i18n;
mod lenlogo;
mod platform;

use eframe::egui;
use eframe::egui::Color32;
use eframe::epaint::text::FontData;
use egui::FontFamily::Proportional;
use egui::FontId;
use egui::RichText;
use egui::TextStyle::{Body, Button, Heading, Monospace, Small};
use i18n::{Lang, t};
use lenlogo::PlatformInfo;
use platform::{NativePlatform, PlatformOps};
use poll_promise::Promise;

fn main() -> Result<(), eframe::Error> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_target(false)
        .try_init();
    let icon = include_bytes!("../assets/icon.png");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 420.0])
            .with_min_inner_size([600.0, 420.0])
            .with_icon(eframe::icon_data::from_png_bytes(icon).unwrap()),
        ..Default::default()
    };
    eframe::run_native(
        "Lenovo UEFI Boot Logo Changer",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

// 定义Logo操作的结果
struct LogoOperationResult {
    success: bool,
    new_loading_icon_state: bool,
    // 返回更新后的平台信息
    enable: u8,
    lbldesp_var: [u8; 10],
    lbldvc_var: [u8; 40],
}

// 在后台线程执行设置Logo操作
fn perform_set_logo_operation(
    img_path: String,
    show_loading_icon: bool,
    lbldesp_var: [u8; 10],
    lbldvc_var: [u8; 40],
) -> LogoOperationResult {
    // 先设置加载图标
    let loading_icon_result = NativePlatform::set_loading_icon(show_loading_icon);
    let new_loading_icon_state = NativePlatform::get_loading_icon();

    if loading_icon_result {
        log::info!("Loading icon change success");
    } else {
        log::error!("Loading icon change failed");
    }

    // 执行设置Logo操作
    let mut temp_info = PlatformInfo::default();
    temp_info.lbldesp_var = lbldesp_var;
    temp_info.lbldvc_var = lbldvc_var;
    temp_info.width = u32::from_le_bytes(lbldesp_var[1..5].try_into().unwrap());
    temp_info.height = u32::from_le_bytes(lbldesp_var[5..9].try_into().unwrap());
    temp_info.version = u32::from_le_bytes(lbldvc_var[0..4].try_into().unwrap());

    let success = temp_info.set_logo(&img_path);

    if success {
        log::info!("Change logo success");
    } else {
        log::error!("Change logo failed");
    }

    // 在后台重新获取平台信息，避免在UI线程中读取
    let mut updated_info = PlatformInfo::default();
    updated_info.get_info();

    LogoOperationResult {
        success,
        new_loading_icon_state,
        enable: updated_info.enable,
        lbldesp_var: updated_info.lbldesp_var,
        lbldvc_var: updated_info.lbldvc_var,
    }
}

// 在后台线程执行恢复Logo操作
fn perform_restore_logo_operation(
    lbldesp_var: [u8; 10],
    lbldvc_var: [u8; 40],
) -> LogoOperationResult {
    // 设置加载图标为启用
    let loading_icon_result = NativePlatform::set_loading_icon(true);
    let new_loading_icon_state = NativePlatform::get_loading_icon();

    if loading_icon_result {
        log::info!("Restore loading icon success");
    } else {
        log::error!("Restore loading icon failed");
    }

    // 执行恢复Logo操作
    let mut temp_info = PlatformInfo::default();
    temp_info.lbldesp_var = lbldesp_var;
    temp_info.lbldvc_var = lbldvc_var;
    temp_info.version = u32::from_le_bytes(lbldvc_var[0..4].try_into().unwrap());

    let success = temp_info.restore_logo();

    if success {
        log::info!("Restore logo success");
    } else {
        log::error!("Restore logo failed");
    }

    // 在后台重新获取平台信息，避免在UI线程中读取
    let mut updated_info = PlatformInfo::default();
    updated_info.get_info();

    LogoOperationResult {
        success,
        new_loading_icon_state,
        enable: updated_info.enable,
        lbldesp_var: updated_info.lbldesp_var,
        lbldvc_var: updated_info.lbldvc_var,
    }
}

#[derive(Default)]
struct MyApp {
    language: String,
    is_admin: bool,
    is_support: bool,
    is_loading_icon: bool,
    platform_info: PlatformInfo,
    last_set_logo: i8,
    last_restore_logo: i8,
    set_loading_icon: bool,
    picked_path: Option<String>,
    // Promise用于异步操作
    set_logo_promise: Option<Promise<LogoOperationResult>>,
    restore_logo_promise: Option<Promise<LogoOperationResult>>,
    // 添加待处理标志，用于在下一帧启动异步操作
    pending_set_logo: bool,
    pending_restore_logo: bool,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_admin {
            self.show_main_ui(ctx);
        } else {
            self.show_admin_prompt_ui(ctx);
        }
    }
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        log::debug!("Start MyApp::new");
        setup_custom_fonts(&cc.egui_ctx);
        let is_admin = NativePlatform::is_admin();
        let mut platform_info = PlatformInfo::default();
        let mut is_support = false;
        if is_admin {
            is_support = platform_info.get_info();
        }
        let language = String::from("en");
        let is_loading_icon = NativePlatform::get_loading_icon();
        let set_loading_icon = is_loading_icon;

        Self {
            language,
            is_admin,
            is_support,
            is_loading_icon,
            set_loading_icon,
            platform_info,
            ..Default::default()
        }
    }

    fn show_main_ui(&mut self, ctx: &egui::Context) {
        let lang = Lang::from_code(&self.language);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("{} : ", t(lang, "language")));
                ui.radio_value(
                    &mut self.language,
                    String::from("en"),
                    t(lang, "english").as_ref(),
                );
                ui.radio_value(
                    &mut self.language,
                    String::from("zh"),
                    t(lang, "chinese").as_ref(),
                );
            });
            ui.separator();

            if self.is_support {
                ui.colored_label(Color32::LIGHT_GREEN, t(lang, "supported"));

                ui.separator();
                if self.platform_info.enable != 0 {
                    ui.colored_label(Color32::LIGHT_GREEN, t(lang, "logo_enabled"));
                } else {
                    ui.colored_label(Color32::LIGHT_RED, t(lang, "logo_disabled"));
                }

                ui.label(format!(
                    "{} : {}x{}",
                    t(lang, "max_image_size"),
                    self.platform_info.width,
                    self.platform_info.height
                ));
                // ui.label(format!("Support Format / 支持的图片格式 : {}", self.platform_info.support.join(" / ")));
                ui.label(format!(
                    "{} : {}",
                    t(lang, "supported_formats"),
                    self.platform_info.support.join(" / ")
                ));
                ui.label(format!(
                    "{} : {:x}",
                    t(lang, "version"),
                    self.platform_info.version
                ));

                ui.separator();

                if !self.platform_info.support.is_empty() {
                    // 只在Windows平台显示加载图标选项
                    #[cfg(target_os = "windows")]
                    ui.checkbox(
                        &mut self.set_loading_icon,
                        t(lang, "show_windows_loading").as_ref(),
                    );

                    if ui.button(t(lang, "pick_image").as_ref()).clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Image", &*self.platform_info.support)
                            .pick_file()
                        {
                            self.picked_path = Some(path.display().to_string());
                        }
                    }
                }

                if let Some(picked_path) = &self.picked_path {
                    if self.platform_info.version == 0x20003
                        || self.platform_info.version == 0x20000
                    {
                        ui.horizontal(|ui| {
                            ui.label(t(lang, "picked_image").as_ref());
                            ui.monospace(picked_path);
                        });
                        if ui
                            .button(
                                RichText::new(t(lang, "change_logo_btn").to_string())
                                    .color(Color32::RED),
                            )
                            .clicked()
                            && self.set_logo_promise.is_none()
                            && !self.pending_set_logo
                        {
                            self.last_restore_logo = 0;
                            self.last_set_logo = 0;
                            // 标记为待处理，在下一帧启动异步操作
                            self.pending_set_logo = true;
                            ctx.request_repaint();
                        }

                        // 在单独的逻辑块中启动异步操作，避免在按钮点击时立即执行
                        if self.pending_set_logo && self.set_logo_promise.is_none() {
                            // 捕获需要的数据
                            let img_path = picked_path.clone();
                            let show_loading_icon = self.set_loading_icon;
                            let lbldesp_var = self.platform_info.lbldesp_var;
                            let lbldvc_var = self.platform_info.lbldvc_var;

                            // 在后台线程执行操作
                            self.set_logo_promise =
                                Some(Promise::spawn_thread("set_logo", move || {
                                    perform_set_logo_operation(
                                        img_path,
                                        show_loading_icon,
                                        lbldesp_var,
                                        lbldvc_var,
                                    )
                                }));
                            self.pending_set_logo = false;
                        }

                        // 检查Promise是否完成
                        if let Some(promise) = &self.set_logo_promise {
                            if let Some(result) = promise.ready() {
                                // 操作完成，更新状态（从后台线程返回的结果更新，不在UI线程读取）
                                self.is_loading_icon = result.new_loading_icon_state;
                                self.set_loading_icon = result.new_loading_icon_state;
                                self.last_set_logo = if result.success { 1 } else { -1 };

                                // 使用后台线程返回的平台信息，避免在UI线程调用get_info()
                                self.platform_info.enable = result.enable;
                                self.platform_info.lbldesp_var = result.lbldesp_var;
                                self.platform_info.lbldvc_var = result.lbldvc_var;

                                // 清除Promise
                                self.set_logo_promise = None;
                            } else {
                                // 正在处理中，显示spinner
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label(t(lang, "setting_logo_wait").as_ref());
                                });
                                ctx.request_repaint(); // 继续请求重绘以更新UI
                            }
                        }
                    }
                }

                match self.last_set_logo {
                    1 => {
                        ui.colored_label(Color32::LIGHT_GREEN, t(lang, "change_logo_success"));
                    }
                    -1 => {
                        ui.colored_label(Color32::LIGHT_RED, t(lang, "change_logo_failed"));
                    }
                    _ => {}
                }

                ui.separator();
                if ui.button(t(lang, "restore_logo_btn").as_ref()).clicked()
                    && self.restore_logo_promise.is_none()
                    && !self.pending_restore_logo
                {
                    self.last_restore_logo = 0;
                    self.last_set_logo = 0;
                    // 标记为待处理，在下一帧启动异步操作
                    self.pending_restore_logo = true;
                    ctx.request_repaint();
                }

                // 在单独的逻辑块中启动异步操作，避免在按钮点击时立即执行
                if self.pending_restore_logo && self.restore_logo_promise.is_none() {
                    // 捕获需要的数据
                    let lbldesp_var = self.platform_info.lbldesp_var;
                    let lbldvc_var = self.platform_info.lbldvc_var;

                    // 在后台线程执行操作
                    self.restore_logo_promise =
                        Some(Promise::spawn_thread("restore_logo", move || {
                            perform_restore_logo_operation(lbldesp_var, lbldvc_var)
                        }));
                    self.pending_restore_logo = false;
                }

                // 检查Promise是否完成
                if let Some(promise) = &self.restore_logo_promise {
                    if let Some(result) = promise.ready() {
                        // 操作完成，更新状态（从后台线程返回的结果更新，不在UI线程读取）
                        self.is_loading_icon = result.new_loading_icon_state;
                        self.set_loading_icon = result.new_loading_icon_state;
                        self.last_restore_logo = if result.success { 1 } else { -1 };

                        // 使用后台线程返回的平台信息，避免在UI线程调用get_info()
                        self.platform_info.enable = result.enable;
                        self.platform_info.lbldesp_var = result.lbldesp_var;
                        self.platform_info.lbldvc_var = result.lbldvc_var;
                        self.is_support = result.enable != 0 || result.success;

                        // 清除Promise
                        self.restore_logo_promise = None;
                    } else {
                        // 正在处理中，显示spinner
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(t(lang, "restoring_logo_wait").as_ref());
                        });
                        ctx.request_repaint(); // 继续请求重绘以更新UI
                    }
                }
                match self.last_restore_logo {
                    1 => {
                        ui.colored_label(Color32::LIGHT_GREEN, t(lang, "restore_logo_success"));
                    }
                    -1 => {
                        ui.colored_label(Color32::LIGHT_RED, t(lang, "restore_logo_failed"));
                    }
                    _ => {}
                }

                ui.separator();

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new("Free & Open. Built with 💗.").size(12.0))
                        .on_hover_cursor(egui::CursorIcon::Help)
                        .on_hover_ui(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.hyperlink_to("@chnzzh", "https://github.com/chnzzh");
                                ui.hyperlink_to(
                                    "★ Star",
                                    "https://github.com/chnzzh/lenovo-logo-changer",
                                )
                                .on_hover_text(
                                    "If you like this tool, please give it a star on GitHub!",
                                );
                                ui.hyperlink_to(
                                    "❓ Issue",
                                    "https://github.com/chnzzh/lenovo-logo-changer/issues",
                                )
                                .on_hover_text(
                                    "Having problems? Open an issue on GitHub!",
                                );
                                ui.label("Version 0.2.0 | MIT License | © 2026 chnzzh");
                            });
                        });
                    ui.add_space(4.0);
                });
            } else {
                ui.label(t(lang, "unsupported").as_ref());
            }
        });
    }

    fn show_admin_prompt_ui(&mut self, ctx: &egui::Context) {
        let lang = Lang::from_code(&self.language);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(t(lang, "admin_required").as_ref());
        });
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own assets (maybe supporting non-latin characters).
    // .ttf and .otf files supported.

    // 尝试获取系统字体路径（Windows特定）
    if let Some(font_path) = NativePlatform::get_system_font_path() {
        if let Ok(font) = std::fs::read(&font_path) {
            fonts.font_data.insert(
                "my_font".to_owned(),
                std::sync::Arc::new(FontData::from_owned(font)),
            );

            fonts
                .families
                .get_mut(&Proportional)
                .unwrap()
                .insert(0, "my_font".to_owned());

            fonts
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .insert(0, "my_font".to_owned());
        } else {
            log::warn!("Failed to load system font from: {}", font_path);
        }
    }

    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (Heading, FontId::new(30.0, Proportional)),
        (Body, FontId::new(18.0, Proportional)),
        (Monospace, FontId::new(18.0, Proportional)),
        (Button, FontId::new(18.0, Proportional)),
        (Small, FontId::new(15.0, Proportional)),
    ]
    .into();
    ctx.set_style(style);
}
