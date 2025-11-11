// ESP分区操作模块
// 使用平台抽象层来实现跨平台兼容

use crate::platform::{EspPartitionOps, NativePlatform};

/// 删除ESP分区中的Logo路径
pub(crate) fn delete_logo_path() -> bool {
    NativePlatform::delete_logo_path()
}

/// 复制文件到ESP分区
pub(crate) fn copy_file_to_esp(src: &str, dst: &str) -> bool {
    NativePlatform::copy_file_to_esp(src, dst)
}