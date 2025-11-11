// 平台抽象层模块

// Windows平台支持（包括在Linux上交叉编译Windows目标）
#[cfg(any(target_os = "windows", all(target_family = "windows")))]
mod windows;

#[cfg(any(target_os = "windows", all(target_family = "windows")))]
pub use windows::WindowsPlatform as NativePlatform;

// Linux平台支持
#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use linux::LinuxPlatform as NativePlatform;

// 为未实现的平台提供编译时错误提示
#[cfg(not(any(target_os = "windows", all(target_family = "windows"), target_os = "linux")))]
compile_error!("This platform is not yet supported. Currently only Windows and Linux are supported. Please add support for your platform in src/platform/");

/// 平台操作trait，定义所有平台特定的操作接口
pub trait PlatformOps {
    /// 检查是否具有管理员/root权限
    fn is_admin() -> bool;
    
    /// 查找可用的驱动器盘符（Windows特有，其他平台可能不需要）
    fn find_available_drive() -> Option<char>;
    
    /// 挂载ESP分区到指定路径
    /// 
    /// # 参数
    /// * `mount_point` - 挂载点（Windows下是盘符，Linux下是路径）
    /// 
    /// # 返回值
    /// 成功返回true，失败返回false
    fn mount_esp(mount_point: &str) -> bool;
    
    /// 卸载ESP分区
    /// 
    /// # 参数
    /// * `mount_point` - 挂载点
    fn unmount_esp(mount_point: &str) -> bool;
    
    /// 获取Boot加载图标状态
    /// 
    /// # 返回值
    /// true表示显示加载图标，false表示隐藏
    fn get_loading_icon() -> bool;
    
    /// 设置Boot加载图标状态
    /// 
    /// # 参数
    /// * `show_loading_icon` - true表示显示加载图标，false表示隐藏
    /// 
    /// # 返回值
    /// 成功返回true，失败返回false
    fn set_loading_icon(show_loading_icon: bool) -> bool;
    
    /// 获取系统字体路径（用于UI显示）
    fn get_system_font_path() -> Option<String>;
}

/// ESP分区操作trait
pub trait EspPartitionOps {
    /// 复制文件到ESP分区
    /// 
    /// # 参数
    /// * `src` - 源文件路径
    /// * `dst` - 目标路径（相对于ESP分区根目录）
    /// 
    /// # 返回值
    /// 成功返回true，失败返回false
    fn copy_file_to_esp(src: &str, dst: &str) -> bool;
    
    /// 删除ESP分区中的Logo路径
    /// 
    /// # 返回值
    /// 成功返回true，失败返回false
    fn delete_logo_path() -> bool;
}
