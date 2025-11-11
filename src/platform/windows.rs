// Windows平台特定实现

use std::path::Path;
use std::process::Command;
use windows_sys::{
    Win32::System::Threading::*,
    Win32::Security::*,
    Win32::Storage::FileSystem::GetLogicalDrives,
};

use super::{PlatformOps, EspPartitionOps};

/// Windows平台实现
pub struct WindowsPlatform;

impl PlatformOps for WindowsPlatform {
    fn is_admin() -> bool {
        unsafe {
            let mut is_admin = 0u32;
            let mut size = std::mem::size_of::<u32>() as u32;
            let mut token = CreateEventW(std::ptr::null(), 1, 0, std::ptr::null());
            OpenProcessToken(GetCurrentProcess(), TOKEN_ALL_ACCESS, &mut token);
            GetTokenInformation(
                token,
                TokenElevation,
                &mut is_admin as *mut u32 as *mut std::ffi::c_void,
                size,
                &mut size,
            );
            println!("is_admin: {}", is_admin);
            is_admin != 0
        }
    }

    fn find_available_drive() -> Option<char> {
        // 获取逻辑驱动器的位掩码
        let drive_mask = unsafe { GetLogicalDrives() };

        // 从 A 到 Z 检查每个盘符
        for drive_letter in b'A'..=b'Z' {
            let mask = 1 << (drive_letter - b'A');

            // 检查位掩码中是否存在当前盘符对应的位
            if (drive_mask & mask) == 0 {
                // 盘符可用，返回对应的字符
                return Some(drive_letter as char);
            }
        }
        // 所有盘符都已用完
        None
    }

    fn mount_esp(mount_point: &str) -> bool {
        // Windows下挂载ESP分区到指定盘符
        // mountvol X: /s
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        let mut mountvol_cmd = Command::new("mountvol");
        mountvol_cmd
            .arg(format!("{}:", mount_point))
            .arg("/s")
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        
        match mountvol_cmd.status() {
            Ok(status) => {
                if status.success() {
                    true
                } else {
                    eprintln!("[!] Mountvol failed / 挂载失败");
                    false
                }
            }
            Err(e) => {
                eprintln!("[!] Mountvol command execution failed / 挂载命令执行失败: {}", e);
                false
            }
        }
    }

    fn unmount_esp(mount_point: &str) -> bool {
        // Windows下卸载ESP分区
        // mountvol X: /d
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        let mut mountvol_cmd = Command::new("mountvol");
        mountvol_cmd
            .arg(format!("{}:", mount_point))
            .arg("/d")
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        
        match mountvol_cmd.status() {
            Ok(status) => {
                if status.success() {
                    true
                } else {
                    eprintln!("[!] Unmountvol failed / 卸载失败");
                    false
                }
            }
            Err(e) => {
                eprintln!("[!] Unmountvol command execution failed / 卸载命令执行失败: {}", e);
                false
            }
        }
    }

    fn get_loading_icon() -> bool {
        // 执行 bcdedit /enum all 命令
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        let mut cmd = Command::new("bcdedit");
        cmd.arg("/enum")
            .arg("all")
            .creation_flags(CREATE_NO_WINDOW);
        
        match cmd.output() {
            Ok(output) => {
                // 将输出转换为字符串
                let stdout = String::from_utf8_lossy(&output.stdout);
                // 检查输出中是否包含 "bootuxdisabled" 和 "Yes"
                for line in stdout.lines() {
                    if line.contains("bootuxdisabled") && line.contains("Yes") {
                        println!("Loading icon Disabled");
                        return false; // 存在且包含 "Yes"
                    }
                }
                println!("Loading icon Enabled");
                true // 不存在或不包含 "Yes"
            }
            Err(e) => {
                eprintln!("Failed to execute command: {}", e);
                true // 处理错误，返回 true
            }
        }
    }

    fn set_loading_icon(show_loading_icon: bool) -> bool {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        let args = if show_loading_icon {
            vec!["-set", "bootuxdisabled", "off"]
        } else {
            vec!["-set", "bootuxdisabled", "on"]
        };

        let mut cmd = Command::new("bcdedit.exe");
        cmd.args(&args)
            .creation_flags(CREATE_NO_WINDOW);
        
        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    println!("Command executed successfully");
                    true
                } else {
                    eprintln!(
                        "Command failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    false
                }
            }
            Err(e) => {
                eprintln!("Failed to execute command: {}", e);
                false
            }
        }
    }

    fn get_system_font_path() -> Option<String> {
        // Windows系统字体路径
        Some("C:/Windows/Fonts/msyh.ttc".to_string())
    }
}

impl EspPartitionOps for WindowsPlatform {
    fn copy_file_to_esp(src: &str, dst: &str) -> bool {
        // 判断src是否为文件
        let src_path = Path::new(src);
        if !src_path.is_file() {
            eprintln!("[!] Source path is not a file / 源路径不是文件");
            return false;
        }

        // 获取可用的盘符
        let drive_letter = match Self::find_available_drive() {
            Some(drive_letter) => drive_letter,
            None => {
                eprintln!("[!] No available drive letter / 没有可用的盘符");
                return false;
            }
        };
        println!("drive_letter: {}", drive_letter);

        // 挂载ESP分区
        let mount_point = drive_letter.to_string();
        if !Self::mount_esp(&mount_point) {
            return false;
        }

        let target_path = Path::new(&format!("{}:\\", drive_letter)).join(dst);
        
        // 如果目标上级路径存在，删除目标路径
        if let Some(parent) = target_path.parent() {
            if parent.exists() {
                if let Err(err) = std::fs::remove_dir_all(parent) {
                    eprintln!("[!] Remove directory failed / 删除目录失败: {}", err);
                    Self::unmount_esp(&mount_point);
                    return false;
                }
            }
        }
        
        // 创建目标路径
        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                if let Err(err) = std::fs::create_dir_all(parent) {
                    eprintln!("[!] Create directory failed / 创建目录失败: {}", err);
                    Self::unmount_esp(&mount_point);
                    return false;
                }
            }
        }

        // 将文件复制到目标路径
        if let Err(err) = std::fs::copy(src, &target_path) {
            eprintln!("[!] Copy file failed / 复制文件失败: {}", err);
            Self::unmount_esp(&mount_point);
            return false;
        }
        
        println!("[+] File copied successfully / 文件复制成功: {}", target_path.display());
        Self::unmount_esp(&mount_point);
        true
    }

    fn delete_logo_path() -> bool {
        let drive_letter = match Self::find_available_drive() {
            Some(drive_letter) => drive_letter,
            None => {
                eprintln!("[!] No available drive letter / 没有可用的盘符");
                return false;
            }
        };
        println!("drive_letter: {}", drive_letter);

        let mount_point = drive_letter.to_string();
        if !Self::mount_esp(&mount_point) {
            return false;
        }

        let target_path = Path::new(&format!("{}:\\", drive_letter)).join(r"EFI/Lenovo/Logo");
        
        // 如果目标路径存在，删除目标路径
        if target_path.exists() {
            if let Err(err) = std::fs::remove_dir_all(&target_path) {
                eprintln!("[!] Remove directory failed / 删除目录失败: {}", err);
                Self::unmount_esp(&mount_point);
                return false;
            }
            println!("[+] Logo directory deleted successfully / Logo目录删除成功");
        } else {
            println!("[*] Logo directory does not exist / Logo目录不存在");
        }
        
        Self::unmount_esp(&mount_point);
        true
    }
}
