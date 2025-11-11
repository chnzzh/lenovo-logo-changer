// Simple i18n module for UI strings

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Zh,
}

impl Lang {
    pub fn from_code(code: &str) -> Self {
        match code {
            "zh" | "zh-CN" | "zh_CN" => Lang::Zh,
            _ => Lang::En,
        }
    }
}

use std::borrow::Cow;

pub fn t(lang: Lang, key: &str) -> Cow<'static, str> {
    match lang {
        Lang::En => match key {
            // General
            "language" => Cow::Borrowed("Language"),
            "english" => Cow::Borrowed("English"),
            "chinese" => Cow::Borrowed("Chinese"),
            // Support status
            "supported" => Cow::Borrowed("Your device is supported !"),
            "unsupported" => Cow::Borrowed("Your device is not supported !"),
            // UEFI logo state
            "logo_enabled" => Cow::Borrowed("UEFI Logo DIY Enabled"),
            "logo_disabled" => Cow::Borrowed("UEFI Logo DIY Disabled"),
            // Info labels
            "max_image_size" => Cow::Borrowed("Max Image Size"),
            "supported_formats" => Cow::Borrowed("Support Format"),
            "version" => Cow::Borrowed("Version"),
            // Actions
            "show_windows_loading" => Cow::Borrowed("Show Windows loading circle"),
            "pick_image" => Cow::Borrowed("Pick Image"),
            "picked_image" => Cow::Borrowed("Picked Image:"),
            "change_logo_btn" => Cow::Borrowed("!!! Change Logo !!!"),
            "restore_logo_btn" => Cow::Borrowed("Restore Logo"),
            // Progress
            "setting_logo_wait" => Cow::Borrowed("Setting logo, please wait..."),
            "restoring_logo_wait" => Cow::Borrowed("Restoring logo, please wait..."),
            // Results
            "change_logo_success" => Cow::Borrowed("Change logo succeeded, reboot to see the effect"),
            "change_logo_failed" => Cow::Borrowed("Change logo failed"),
            "restore_logo_success" => Cow::Borrowed("Restore Logo Success"),
            "restore_logo_failed" => Cow::Borrowed("Restore Logo Failed"),
            // Admin prompt
            "admin_required" => Cow::Borrowed("You need to run this program as Administrator !"),
            _ => Cow::Owned(key.to_string()),
        },
        Lang::Zh => match key {
            // General
            "language" => Cow::Borrowed("语言"),
            "english" => Cow::Borrowed("英语"),
            "chinese" => Cow::Borrowed("中文"),
            // Support status
            "supported" => Cow::Borrowed("您的设备是支持的！"),
            "unsupported" => Cow::Borrowed("不支持您的设备！"),
            // UEFI logo state
            "logo_enabled" => Cow::Borrowed("自定义UEFI Logo已启用"),
            "logo_disabled" => Cow::Borrowed("自定义UEFI Logo未启用"),
            // Info labels
            "max_image_size" => Cow::Borrowed("图片最大分辨率"),
            "supported_formats" => Cow::Borrowed("支持的图片格式"),
            "version" => Cow::Borrowed("协议版本"),
            // Actions
            "show_windows_loading" => Cow::Borrowed("显示Windows加载图标"),
            "pick_image" => Cow::Borrowed("选择图片"),
            "picked_image" => Cow::Borrowed("已选择的图片："),
            "change_logo_btn" => Cow::Borrowed("!!! 设置Logo !!! "),
            "restore_logo_btn" => Cow::Borrowed("恢复Logo"),
            // Progress
            "setting_logo_wait" => Cow::Borrowed("正在设置Logo，请稍候..."),
            "restoring_logo_wait" => Cow::Borrowed("正在恢复Logo，请稍候..."),
            // Results
            "change_logo_success" => Cow::Borrowed("设置Logo成功，重新启动以查看效果"),
            "change_logo_failed" => Cow::Borrowed("设置Logo失败"),
            "restore_logo_success" => Cow::Borrowed("恢复Logo成功"),
            "restore_logo_failed" => Cow::Borrowed("恢复Logo失败"),
            // Admin prompt
            "admin_required" => Cow::Borrowed("您需要以管理员权限运行此程序！"),
            _ => Cow::Owned(key.to_string()),
        },
    }
}
