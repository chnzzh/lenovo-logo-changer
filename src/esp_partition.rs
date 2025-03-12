use std::path::Path;
use windows_sys::Win32::Storage::FileSystem::GetLogicalDrives;

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

pub(crate) fn delete_logo_path() -> bool {
    let drive_letter = match find_available_drive() {
        Some(drive_letter) => drive_letter,
        None => return false,
    };
    println!("drive_letter: {}", drive_letter);

    if mountvol_mount(drive_letter) == false {
        return false;
    }

    let target_path = Path::new(&format!("{}:\\", drive_letter)).join(r"/EFI/Lenovo/Logo");
    // 如果目标路径存在，删除目标路径
    if target_path.exists() {
        if let Err(err) = std::fs::remove_dir_all(target_path) {
            eprintln!("[!] Remove directory failed / 删除目录失败: {}", err);
            mountvol_unmount(drive_letter);
            return false;
        }
    }
    mountvol_unmount(drive_letter);
    true
}

pub(crate) fn copy_file_to_esp(src: &str, dst: &str) -> bool {
    // 判断src是否为文件
    let src_path = Path::new(src);
    if !src_path.is_file() {
        eprintln!("[!] Source path is not a file / 源路径不是文件");
        return false;
    }

    // 获取可用的盘符
    let drive_letter = match find_available_drive() {
        Some(drive_letter) => drive_letter,
        None => return false,
    };
    println!("drive_letter: {}", drive_letter);

    if mountvol_mount(drive_letter) == false {
        return false;
    }

    let target_path = Path::new(&format!("{}:\\", drive_letter)).join(dst);
    // 如果目标上级路径存在，删除目标路径
    if let Some(parent) = target_path.parent() {
        if parent.exists() {
            if let Err(err) = std::fs::remove_dir_all(parent) {
                eprintln!("[!] Remove directory failed / 删除目录失败: {}", err);

                mountvol_unmount(drive_letter);
                return false;
            }
        }
    }
    // 创建目标路径
    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            if let Err(err) = std::fs::create_dir_all(parent) {
                eprintln!("[!] Create directory failed / 创建目录失败: {}", err);
                mountvol_unmount(drive_letter);
                return false;
            }
        }
    }

    // 将文件复制到目标路径
    if let Err(err) = std::fs::copy(src, target_path) {
        eprintln!("[!] Copy file failed / 复制文件失败: {}", err);
        mountvol_unmount(drive_letter);
        return false;
    }
    mountvol_unmount(drive_letter);
    true
}

fn mountvol_mount(drive_letter: char) -> bool {
    // 执行mountvol driver_letter: /s, 失败return false
    let mut mountvol_cmd = std::process::Command::new("mountvol");
    mountvol_cmd
        .arg(format!("{}:", drive_letter))
        .arg("/s")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    if !mountvol_cmd.status().unwrap().success() {
        eprintln!("[!] Mountvol failed / 挂载失败");
        return false;
    }
    true
}

fn mountvol_unmount(drive_letter: char) -> bool {
    // 执行mountvol driver_letter: /s, 失败return false
    let mut mountvol_cmd = std::process::Command::new("mountvol");
    mountvol_cmd
        .arg(format!("{}:", drive_letter))
        .arg("/d")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    if !mountvol_cmd.status().unwrap().success() {
        eprintln!("[!] Unmountvol failed / 卸载失败");
        return false;
    }
    true
}