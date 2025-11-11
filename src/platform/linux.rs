// Linux平台特定实现

use std::path::Path;
use std::process::Command;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use log::{info, error, warn, debug};

use super::{PlatformOps, EspPartitionOps};

// Linux下处理EFI变量immutable属性所需的常量
const FS_IOC_GETFLAGS: libc::c_ulong = 0x80086601;
const FS_IOC_SETFLAGS: libc::c_ulong = 0x40086602;
const FS_IMMUTABLE_FL: u32 = 0x00000010;

/// Linux平台实现
pub struct LinuxPlatform;

impl PlatformOps for LinuxPlatform {
    fn is_admin() -> bool {
        // Linux下检查是否为root用户（UID == 0）
        unsafe {
            libc::geteuid() == 0
        }
    }

    fn find_available_drive() -> Option<char> {
        // Linux下不使用盘符，返回None
        // ESP分区通常挂载在 /boot/efi 或 /efi
        None
    }

    fn mount_esp(mount_point: &str) -> bool {
        // Linux下挂载ESP分区
        // 首先尝试找到ESP分区
        // 通常ESP分区类型为 vfat，标签可能是 EFI 或有特定的分区类型标识
        
        // 尝试挂载到指定的挂载点
        // mount -t vfat /dev/sdX1 /mnt/esp
        
        // 这里提供一个基本实现，实际使用时可能需要更复杂的逻辑
        let output = Command::new("mount")
            .arg("-t")
            .arg("vfat")
            .arg("-o")
            .arg("rw")
            .arg(Self::find_esp_partition().unwrap_or("/dev/sda1".to_string()))
            .arg(mount_point)
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() {
                info!("Mounted ESP partition at {}", mount_point);
                    true
                } else {
                    error!("Failed to mount ESP partition: {}", String::from_utf8_lossy(&output.stderr));
                    false
                }
            }
            Err(e) => {
                error!("Failed to execute mount command: {}", e);
                false
            }
        }
    }

    fn unmount_esp(mount_point: &str) -> bool {
        // Linux下卸载分区
        // umount /mnt/esp
        let output = Command::new("umount")
            .arg(mount_point)
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    info!("Unmounted ESP partition");
                    true
                } else {
                    error!("Failed to unmount ESP partition: {}", String::from_utf8_lossy(&output.stderr));
                    false
                }
            }
            Err(e) => {
                error!("Failed to execute umount command: {}", e);
                false
            }
        }
    }

    fn get_loading_icon() -> bool {
        // Linux下不支持Windows加载图标功能
        // 返回默认值false
    info!("Loading icon feature not supported on Linux");
        false
    }

    fn set_loading_icon(_show_loading_icon: bool) -> bool {
        // Linux下不支持Windows加载图标功能
    info!("Loading icon feature not supported on Linux");
        true // 返回true避免错误提示
    }

    fn get_system_font_path() -> Option<String> {
        // Linux系统字体路径
        // 尝试常见的中文字体位置
        let font_paths = vec![
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ];
        
        for path in font_paths {
            if Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
        
        None
    }
}

impl LinuxPlatform {
    /// 查找ESP分区设备
    fn find_esp_partition() -> Option<String> {
        // 使用 lsblk 或 blkid 查找ESP分区
        // ESP分区通常有 PARTTYPE="c12a7328-f81f-11d2-ba4b-00a0c93ec93b"
        
        let output = Command::new("lsblk")
            .args(&["-o", "NAME,PARTTYPE,MOUNTPOINT", "-n", "-l"])
            .output();
        
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                // 查找ESP分区类型的GUID
                if line.contains("c12a7328-f81f-11d2-ba4b-00a0c93ec93b") {
                    if let Some(device) = line.split_whitespace().next() {
                        return Some(format!("/dev/{}", device));
                    }
                }
            }
        }
        
        // 如果找不到，尝试查找已挂载的EFI分区
        if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
            for line in mounts.lines() {
                if line.contains("/boot/efi") || line.contains("vfat") && line.contains("efi") {
                    if let Some(device) = line.split_whitespace().next() {
                        return Some(device.to_string());
                    }
                }
            }
        }
        
        None
    }

    /// 在Linux下设置EFI变量文件的immutable属性
    pub fn set_efi_var_immutable(var_path: &Path, immutable: bool) -> Result<(), String> {
        use std::io;
        
        let file = File::open(var_path).map_err(|e| format!("Failed to open {}: {}", var_path.display(), e))?;
        let fd = file.as_raw_fd();
        
        // 获取当前flags
        let mut flags: u32 = 0;
        let ret = unsafe {
            libc::ioctl(fd, FS_IOC_GETFLAGS, &mut flags as *mut u32)
        };
        
        if ret < 0 {
            return Err(format!("Failed to get file flags: {}", io::Error::last_os_error()));
        }
        
        // 设置或清除immutable标志
        if immutable {
            flags |= FS_IMMUTABLE_FL;
        } else {
            flags &= !FS_IMMUTABLE_FL;
        }
        
        // 设置新的flags
        let ret = unsafe { libc::ioctl(fd, FS_IOC_SETFLAGS, &flags as *const u32) };
        
        if ret < 0 {
            return Err(format!("Failed to set file flags: {}", io::Error::last_os_error()));
        }
        
    info!("Set immutable={} for {}", immutable, var_path.display());
        Ok(())
    }

    /// 在Linux下，写入EFI变量前后需要处理immutable属性
    pub fn with_efi_var_writable<F>(var_name: &str, f: F) -> Result<(), String>
    where
        F: FnOnce() -> Result<(), String>,
    {
        let var_path = Path::new("/sys/firmware/efi/efivars").join(var_name);
        
        if !var_path.exists() {
            return Err(format!("EFI variable {} not found", var_name));
        }
        
        // 移除immutable属性
        Self::set_efi_var_immutable(&var_path, false)?;
        
        // 执行写入操作
        let result = f();
        
        // 恢复immutable属性
        if let Err(e) = Self::set_efi_var_immutable(&var_path, true) {
            warn!("Failed to restore immutable flag: {}", e);
        }
        
        result
    }
}

struct EspMountGuard<'a> {
    mount_point: &'a str,
    mounted: bool,
}

impl<'a> EspMountGuard<'a> {
    fn new(mount_point: &'a str) -> Result<Self, ()> {
        if LinuxPlatform::mount_esp(mount_point) {
            Ok(Self { mount_point, mounted: true })
        } else {
            Err(())
        }
    }
}

impl Drop for EspMountGuard<'_> {
    fn drop(&mut self) {
        if self.mounted {
            if !LinuxPlatform::unmount_esp(self.mount_point) {
                warn!("Auto-unmount ESP failed at {}", self.mount_point);
            }
        }
    }
}

impl EspPartitionOps for LinuxPlatform {
    fn copy_file_to_esp(src: &str, dst: &str) -> bool {
        // 判断src是否为文件
        let src_path = Path::new(src);
        if !src_path.is_file() {
            error!("Source path is not a file");
            return false;
        }

        // Linux下通常ESP分区挂载在 /boot/efi
        // 创建临时挂载点
        let mount_point = "/tmp/lenovo_esp_mount";
        
        // 创建挂载点目录
        if let Err(e) = std::fs::create_dir_all(mount_point) {
            error!("Failed to create mount point: {}", e);
            return false;
        }

        // 挂载ESP分区（RAII自动卸载）
        let _guard = match EspMountGuard::new(mount_point) { Ok(g) => g, Err(_) => return false };

        let target_path = Path::new(mount_point).join(dst.trim_start_matches('/'));
        
        // 如果目标上级路径存在，删除目标路径
        if let Some(parent) = target_path.parent() {
            if parent.exists() {
                if let Err(err) = std::fs::remove_dir_all(parent) {
                    error!("Remove directory failed: {}", err);
                    return false;
                }
            }
        }
        
        // 创建目标路径
        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                if let Err(err) = std::fs::create_dir_all(parent) {
                    error!("Create directory failed: {}", err);
                    return false;
                }
            }
        }

        // 将文件复制到目标路径
        if let Err(err) = std::fs::copy(src, &target_path) {
            error!("Copy file failed: {}", err);
            return false;
        }
        
    info!("File copied successfully: {}", target_path.display());
        
        // 同步文件系统，确保写入
        Command::new("sync").output().ok();
        
        true
    }

    fn delete_logo_path() -> bool {
        let mount_point = "/tmp/lenovo_esp_mount";
        
        // 创建挂载点目录
        if let Err(e) = std::fs::create_dir_all(mount_point) {
            error!("Failed to create mount point: {}", e);
            return false;
        }

        // 挂载ESP分区（RAII自动卸载）
        let _guard = match EspMountGuard::new(mount_point) { Ok(g) => g, Err(_) => return false };

        let target_path = Path::new(mount_point).join("EFI/Lenovo/Logo");
        
        // 如果目标路径存在，删除目标路径
        if target_path.exists() {
            if let Err(err) = std::fs::remove_dir_all(&target_path) {
                error!("Remove directory failed: {}", err);
                return false;
            }
            info!("Logo directory deleted successfully");
        } else {
            debug!("Logo directory does not exist");
        }
        
        // 同步文件系统
        Command::new("sync").output().ok();
        
        true
    }
}
