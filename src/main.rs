#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod winfunc;
mod lenlogo;
mod esp_partition;

use egui::FontId;
use egui::RichText;
use eframe::egui;
use eframe::egui::Color32;
use eframe::epaint::text::FontData;
use egui::FontFamily::Proportional;
use egui::TextStyle::{Body, Button, Heading, Monospace, Small};
use lenlogo::PlatformInfo;

fn main() -> Result<(), eframe::Error> {
    let icon = include_bytes!("../assets/icon.png");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 400.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(eframe::icon_data::from_png_bytes(icon).unwrap()),
        ..Default::default()
    };
    eframe::run_native(
        "Lenovo UEFI Boot Logo Changer",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc))))
    )
}

#[derive(Default)]
struct MyApp {
    language: String,
    is_admin:bool,
    is_support:bool,
    is_loading_icon: bool,
    platform_info: PlatformInfo,
    last_set_logo:i8,
    last_restore_logo:i8,
    set_loading_icon: bool,
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
        let language = String::from("en");
        let is_loading_icon = platform_info.get_loading_icon();
        let set_loading_icon = is_loading_icon;

        fn xor_encrypt_decrypt(input: &str, key: char) -> String {
            input.chars()
                .map(|c| (c as u8 ^ key as u8) as char) // 对每个字符用密钥进行异或
                .collect()
        }
        
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Language : ");
                ui.radio_value(&mut self.language, String::from("en"), "English");
                ui.radio_value(&mut self.language, String::from("zh"), "中文");
            });
            ui.separator();

            if self.is_support {

                ui.colored_label(Color32::LIGHT_GREEN, if self.language == "zh" {
                    "您的设备是支持的！"
                }
                else {
                    "Your device is supported !"
                });

                ui.separator();
                if self.platform_info.enable != 0 {
                    ui.colored_label(Color32::LIGHT_GREEN, if self.language == "zh" {
                        "自定义UEFI Logo已启用"
                    }
                    else {
                        "UEFI Logo DIY Enabled"
                    });
                }
                else {
                    ui.colored_label(Color32::LIGHT_RED, if self.language == "zh" {
                        "自定义UEFI Logo未启用"
                    }
                    else {
                        "UEFI Logo DIY Disabled"
                    });
                }

                ui.label(format!("{} : {}x{}", if self.language == "zh" {
                    "图片最大分辨率"
                }
                else {
                    "Max Image Size"
                }, self.platform_info.width, self.platform_info.height));
                // ui.label(format!("Support Format / 支持的图片格式 : {}", self.platform_info.support.join(" / ")));
                ui.label(format!("{} : {}", if self.language == "zh" {
                    "支持的图片格式"
                }
                else {
                    "Support Format"
                }, self.platform_info.support.join(" / ")));
                ui.label(format!("{} : {:x}", if self.language == "zh" {
                    "协议版本"
                }
                else {
                    "Version"
                },self.platform_info.version));

                ui.separator();

                if !self.platform_info.support.is_empty() {
                    ui.checkbox(&mut self.set_loading_icon, if self.language == "zh" {
                        "显示Windows加载图标"
                    }
                    else {
                        "show Windows loading circle"
                    });
                    if ui.button(if self.language == "zh" {
                        "选择图片"
                    }
                    else {
                        "Pick Image"
                    }).clicked() {
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
                            ui.label(if self.language == "zh" {
                                "已选择的图片："
                            }
                            else {
                                "Picked Image: "
                            });
                            ui.monospace(picked_path);
                        });
                        /*
                        ui.colored_label(Color32::LIGHT_RED, if self.language == "zh" {
                            "请最后确认分辨率和上传的图片格式！"
                        }
                        else {
                            "Please confirm the SIZE and FORMAT of the uploaded images!"
                        });
                        */
                        if ui.button(RichText::new(if self.language == "zh" {
                            "!!! 设置Logo !!! "
                        }
                        else {
                            "!!! Change Logo !!!"
                        }).color(Color32::RED)).clicked() {
                            self.last_restore_logo = 0;
                            self.last_set_logo = 0;

                            self.platform_info.set_loading_icon(self.set_loading_icon);
                            self.is_loading_icon = self.platform_info.get_loading_icon();
                            if self.is_loading_icon == self.set_loading_icon {
                                println!("Loading icon Change Success!");
                            }
                            else {
                                eprintln!("Loading icon Change Fail!");
                            }
                            self.set_loading_icon = self.is_loading_icon;

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
                        ui.colored_label(Color32::LIGHT_GREEN, if self.language == "zh" {
                            "设置Logo成功，重新启动以查看效果"
                        }
                        else {
                            "Change logo succeed, reboot to see the effect"
                        });
                    },
                    -1 => {
                        ui.colored_label(Color32::LIGHT_RED, if self.language == "zh" {
                            "设置Logo失败"
                        }
                        else {
                            "Change logo failed"
                        });
                    },
                    _ => {}
                }

                ui.separator();
                if ui.button(if self.language == "zh" {
                    "恢复Logo"
                }
                else {
                    "Restore Logo"
                }).clicked() {
                    self.last_restore_logo = 0;
                    self.last_set_logo = 0;

                    self.platform_info.set_loading_icon(true);
                    self.is_loading_icon = self.platform_info.get_loading_icon();
                    if self.is_loading_icon {
                        print!("Restore Loading icon Success!");
                    }
                    else {
                        eprintln!("Restore Loading icon Failed !")
                    }
                    self.set_loading_icon = self.is_loading_icon;

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
                        ui.colored_label(Color32::LIGHT_GREEN, if self.language == "zh" {
                            "恢复Logo成功"
                        }
                        else {
                            "Restore Logo Success"
                        });
                    },
                    -1 => {
                        ui.colored_label(Color32::LIGHT_RED, if self.language == "zh" {
                            "恢复Logo失败"
                        }
                        else {
                            "Restore Logo Failed"
                        });
                    },
                    _ => {}
                }

                ui.separator();
                
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    use egui::special_emojis::GITHUB;
                    ui.label(
                        format!("{GITHUB} | MIT License"),
                    ).on_hover_text("https://github.com/chnzzh/lenovo-logo-changer");
                });
            }
            else {
                ui.label(if self.language == "zh" {
                    "不支持您的设备！"
                }
                else {
                    "Your device is not supported !"
                });
            }
        });
    }
    fn show_admin_prompt_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("You need to run this program as Administrator !");
            ui.add_space(10.0);
            ui.label("您需要以管理员权限运行此程序！");
        });
    }

}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own assets (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    let font = std::fs::read("C:/Windows/Fonts/msyh.ttc").unwrap();

    fonts.font_data.insert(
        "my_font".to_owned(),
        std::sync::Arc::new(FontData::from_owned(font)),
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
    //style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    ctx.set_style(style);
}
