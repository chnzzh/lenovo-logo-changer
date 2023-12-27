#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod winfunc;
mod lenlogo;
mod esp_partition;
use egui::FontId;
use egui::RichText;
use eframe::egui;
use eframe::egui::Color32;
use egui::FontFamily::Proportional;
use egui::TextStyle::{Body, Button, Heading, Monospace, Small};
use lenlogo::PlatformInfo;


fn main() -> Result<(), eframe::Error> {

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Lenovo UEFI Boot Logo Changer",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc)))
    )
}

#[derive(Default)]
struct MyApp {
    is_admin:bool,
    is_support:bool,
    platform_info: PlatformInfo,
    last_set_logo:i8,
    last_restore_logo:i8,
    picked_path: Option<String>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_admin {
            self.show_main_ui(ctx);
        }
        else {
            self.show_admin_prompt_ui(ctx);
        }
    }
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        println!("Start MyApp::new");
        setup_custom_fonts(&cc.egui_ctx);
        let is_admin = winfunc::is_admin();
        let mut platform_info = PlatformInfo::default();
        let mut is_support = false;
        if is_admin {
            is_support = platform_info.get_info();
        }
        Self {
            is_admin,
            is_support,
            platform_info,
            ..Default::default()
        }
    }

    fn show_main_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_support {

                ui.label(RichText::new("Your device is supported ! / 您的设备是支持的！").color(Color32::GREEN));
                ui.separator();
                if self.platform_info.enable != 0 {
                    ui.colored_label(Color32::LIGHT_GREEN, "UEFI Logo DIY Enabled / 自定义UEFI Logo已启用");
                }
                else {
                    ui.colored_label(Color32::LIGHT_RED, "UEFI Logo DIY Disabled / 自定义UEFI Logo未启用");
                }

                ui.label(format!("Max Image Size / 图片最大分辨率: {}x{}", self.platform_info.width, self.platform_info.height));
                ui.label(format!("Support Format / 支持的图片格式 : {}", self.platform_info.support.join(" / ")));
                ui.label(format!("Version : {:x}", self.platform_info.version));
                ui.separator();

                if !self.platform_info.support.is_empty() {
                    if ui.button("Open Image / 打开图片").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Image", &*self.platform_info.support)
                            .pick_file() {
                            self.picked_path = Some(path.display().to_string());
                        }
                    }
                }

                if let Some(picked_path) = &self.picked_path  {
                    if self.platform_info.version == 0x20003 {
                        ui.horizontal(|ui| {
                            ui.label("Selected Image Path :");
                            ui.monospace(picked_path);
                        });
                        ui.colored_label(Color32::LIGHT_RED, "Please confirm the Size and format of the uploaded images");
                        ui.colored_label(Color32::LIGHT_RED, "请最后确认分辨率和上传的图片格式");

                        if ui.button(RichText::new("!!! Change Logo / 设置Logo !!!").color(Color32::RED)).clicked() {
                            self.last_restore_logo = 0;
                            self.last_set_logo = 0;
                            if self.platform_info.set_logo(picked_path) {
                                self.last_set_logo = 1;
                                println!("Change Logo Success !")
                            }
                            else {
                                self.last_set_logo = -1;
                                println!("Change Logo Failed !")
                            }
                        }
                    }
                }

                match self.last_set_logo {
                    1 => {
                        ui.colored_label(Color32::GREEN, "Change Logo Succeed / 设置Logo成功");
                    },
                    -1 => {
                        ui.colored_label(Color32::RED, "Change Logo Failed / 设置Logo失败");
                    },
                    _ => {}
                }

                ui.separator();
                if ui.button("Restore Logo / 恢复Logo").clicked() {
                    self.last_restore_logo = 0;
                    self.last_set_logo = 0;

                    if self.platform_info.restore_logo() {
                        self.last_restore_logo = 1;
                        println!("Restore Logo Success!")
                    }
                    else {
                        self.last_restore_logo = -1;
                        println!("Restore Logo Failed!")
                    }
                    self.is_support = self.platform_info.get_info();
                }
                match self.last_restore_logo {
                    1 => {
                        ui.colored_label(Color32::GREEN, "Restore Logo Succeed / 恢复Logo成功");
                    },
                    -1 => {
                        ui.colored_label(Color32::RED, "Restore Logo Failed / 恢复Logo失败");
                    },
                    _ => {}
                }

                ui.separator();
            }
            else {
                ui.label("Your device is not supported !");
                ui.label("不支持您的设备！");
            }


        });
    }
    fn show_admin_prompt_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("You need to run this program as Administrator !");
            ui.label("需要以管理员权限运行程序！");
        });
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "./font/HarmonyOS_Sans_SC_Regular.ttf"
        )),
    );

    fonts.families.get_mut(&Proportional).unwrap()
        .insert(0, "my_font".to_owned());

    fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap()
        .insert(0, "my_font".to_owned());

    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (Heading, FontId::new(30.0, Proportional)),
        (Body, FontId::new(18.0, Proportional)),
        (Monospace, FontId::new(18.0, Proportional)),
        (Button, FontId::new(18.0, Proportional)),
        (Small, FontId::new(10.0, Proportional)),
    ].into();
    ctx.set_style(style);
}
