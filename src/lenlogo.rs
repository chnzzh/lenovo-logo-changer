use crc32fast::Hasher;
use efivar::efi::{Variable, VariableFlags};
use log::{debug, error, info};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

use crate::esp_partition::{copy_file_to_esp, delete_logo_path};

#[cfg(target_os = "linux")]
use crate::platform::linux::LinuxPlatform;

/// 跨平台的EFI变量写入包装函数
/// Linux下需要处理immutable属性，Windows下直接调用
fn with_efi_var_writable<F>(var_name: &str, f: F) -> Result<(), String>
where
    F: FnOnce() -> Result<(), String>,
{
    #[cfg(target_os = "linux")]
    {
        LinuxPlatform::with_efi_var_writable(var_name, f)
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Windows和其他平台直接执行
        let _ = var_name; // 消除未使用变量警告
        f()
    }
}

pub(crate) struct PlatformInfo {
    pub(crate) enable: u8,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) version: u32,
    pub(crate) support: Vec<&'static str>,
    pub(crate) lbldesp_var: [u8; 10],
    pub(crate) lbldvc_var: [u8; 40],
}

impl Default for PlatformInfo {
    fn default() -> Self {
        Self {
            enable: 0,
            width: 0,
            height: 0,
            version: 0,
            support: Vec::new(),
            lbldesp_var: [0u8; 10],
            lbldvc_var: [0u8; 40],
        }
    }
}

impl PlatformInfo {
    pub(crate) fn get_info(&mut self) -> bool {
        let varman = efivar::system();

        let esp_var = Variable::from_str("LBLDESP-871455D0-5576-4FB8-9865-AF0824463B9E").unwrap();

        match varman.read(&esp_var) {
            Ok((esp_buffer, _attr)) => {
                if esp_buffer.len() != 10 {
                    error!("read lbldesp_var failed: buffer length is not 10");
                    return false;
                }
                self.enable = esp_buffer[0];
                self.width = u32::from_le_bytes(esp_buffer[1..5].try_into().unwrap());
                self.height = u32::from_le_bytes(esp_buffer[5..9].try_into().unwrap());
                self.support = Self::support_format(esp_buffer[9]);
                self.lbldesp_var = <[u8; 10]>::try_from(esp_buffer).unwrap();
            }
            Err(err) => {
                error!("read lbldesp_var failed: {}", err);
                return false;
            }
        }

        let dvc_var = Variable::from_str("LBLDVC-871455D1-5576-4FB8-9865-AF0824463C9F").unwrap();
        match varman.read(&dvc_var) {
            Ok((dvc_buffer, _attr)) => {
                if dvc_buffer.len() != 40 {
                    error!("read lbldvc_var failed: buffer length is not 40");
                    return false;
                }
                self.version = u32::from_le_bytes(dvc_buffer[0..4].try_into().unwrap());
                self.lbldvc_var = <[u8; 40]>::try_from(dvc_buffer).unwrap();
            }
            Err(err) => {
                error!("read lbldvc_var failed: {}", err);
                return false;
            }
        }
        true
    }

    pub(crate) fn set_logo(&mut self, img_path: &String) -> bool {
        // 复制文件到ESP分区
        let file_path = Path::new(img_path);
        let file_extension = file_path.extension().unwrap().to_str().unwrap();
        debug!("file_extension: {}", file_extension);

        let dst_path = format!(
            r"/EFI/Lenovo/Logo/mylogo_{}x{}.{}",
            self.width, self.height, file_extension
        );
        info!("target path: {}", dst_path);

        if copy_file_to_esp(img_path, &dst_path) == false {
            error!("copy file failed");
            return false;
        }

        let mut varman = efivar::system();

        // 修改logoinfo
        let mut esp_buffer = self.lbldesp_var.clone();
        esp_buffer[0] = 1;
        let esp_var = Variable::from_str("LBLDESP-871455D0-5576-4FB8-9865-AF0824463B9E").unwrap();

        // 在Linux下需要先移除immutable属性
        let write_result = with_efi_var_writable(
            "LBLDESP-871455d0-5576-4fb8-9865-af0824463b9e",
            || match varman.write(
                &esp_var,
                VariableFlags::from_bits(0x7).unwrap(),
                &esp_buffer,
            ) {
                Ok(rt) => {
                    debug!("write lbldesp_var: {:?}", rt);
                    Ok(())
                }
                Err(err) => Err(format!("write lbldesp_var failed: {}", err)),
            },
        );

        match write_result {
            Ok(_) => {
                self.enable = 1;
                self.lbldesp_var = esp_buffer;
            }
            Err(err) => {
                error!("{}", err);
                return false;
            }
        }

        // 修改logocheck - 根据version选择SHA256或CRC32
        let mut dvc_buffer = self.lbldvc_var.clone();

        if self.version == 0x20003 {
            // version 0x20003: 使用SHA256 (32字节)
            let sha256_bytes;
            match calculate_sha256(img_path) {
                Ok(sha256) => {
                    // 将sha256十六进制字符串转化为十六进制序列
                    sha256_bytes = hex::decode(sha256).unwrap();
                }
                Err(e) => {
                    error!("read error {}: {}", img_path, e);
                    return false;
                }
            }
            dvc_buffer[4..36].clone_from_slice(&sha256_bytes);
            debug!("sha256_bytes: {:?}", sha256_bytes);
        } else if self.version == 0x20000 {
            // version 0x20000: 使用CRC32 (4字节)
            match calculate_crc32_first_512(img_path) {
                Ok(crc32) => {
                    dvc_buffer[4..8].clone_from_slice(&crc32.to_le_bytes());
                    debug!("crc32: 0x{:08x}", crc32);
                }
                Err(e) => {
                    error!("read error {}: {}", img_path, e);
                    return false;
                }
            }
        } else {
            error!("unsupported version: 0x{:x}", self.version);
            return false;
        }
        debug!("dvc_buffer: {:?}", dvc_buffer);

        let dvc_var = Variable::from_str("LBLDVC-871455D1-5576-4FB8-9865-AF0824463C9F").unwrap();

        // 在Linux下需要先移除immutable属性
        let write_result = with_efi_var_writable(
            "LBLDVC-871455d1-5576-4fb8-9865-af0824463c9f",
            || match varman.write(
                &dvc_var,
                VariableFlags::from_bits(0x7).unwrap(),
                &dvc_buffer,
            ) {
                Ok(rt) => {
                    debug!("write lbldvc_var: {:?}", rt);
                    Ok(())
                }
                Err(err) => Err(format!("write lbldvc_var failed: {}", err)),
            },
        );

        match write_result {
            Ok(_) => {
                self.lbldvc_var = dvc_buffer;
            }
            Err(err) => {
                error!("{}", err);
                return false;
            }
        }
        true
    }

    pub(crate) fn restore_logo(&mut self) -> bool {
        //
        let mut status = true;
        if !delete_logo_path() {
            error!("delete logo path failed");
            status = false;
        }

        let mut varman = efivar::system();
        // 修改logoinfo
        let mut esp_buffer = self.lbldesp_var.clone();
        if esp_buffer[0] != 0 {
            esp_buffer[0] = 0;
            let esp_var =
                Variable::from_str("LBLDESP-871455D0-5576-4FB8-9865-AF0824463B9E").unwrap();

            // 在Linux下需要先移除immutable属性
            let write_result =
                with_efi_var_writable("LBLDESP-871455d0-5576-4fb8-9865-af0824463b9e", || {
                    match varman.write(
                        &esp_var,
                        VariableFlags::from_bits(0x7).unwrap(),
                        &esp_buffer,
                    ) {
                        Ok(rt) => {
                            debug!("write lbldesp_var: {:?}", rt);
                            Ok(())
                        }
                        Err(err) => Err(format!("write lbldesp_var failed: {}", err)),
                    }
                });

            match write_result {
                Ok(_) => {
                    self.lbldesp_var = esp_buffer;
                }
                Err(err) => {
                    error!("{}", err);
                    status = false;
                }
            }
        }

        // 修改logocheck - 根据version选择清零范围
        let mut dvc_buffer = self.lbldvc_var.clone();
        let need_clear = if self.version == 0x20000 {
            // version 0x20000
            dvc_buffer[4..8] != [0u8; 4]
        } else {
            // version 0x20003...
            dvc_buffer[4..40] != [0u8; 36]
        };

        if need_clear {
            if self.version == 0x20000 {
                // version 0x20000: 只清零CRC32的4字节（offset 4-8）
                dvc_buffer[4..8].clone_from_slice(&[0u8; 4]);
            } else {
                // version 0x20003及其他: 清零整个校验区域（offset 4-40）
                dvc_buffer[4..40].clone_from_slice(&[0u8; 36]);
            }

            let dvc_var =
                Variable::from_str("LBLDVC-871455D1-5576-4FB8-9865-AF0824463C9F").unwrap();

            let write_result = with_efi_var_writable(
                "LBLDVC-871455d1-5576-4fb8-9865-af0824463c9f",
                || match varman.write(
                    &dvc_var,
                    VariableFlags::from_bits(0x7).unwrap(),
                    &dvc_buffer,
                ) {
                    Ok(rt) => {
                        debug!("write lbldvc_var: {:?}", rt);
                        Ok(())
                    }
                    Err(err) => Err(format!("write lbldvc_var failed: {}", err)),
                },
            );

            match write_result {
                Ok(_) => {
                    self.lbldvc_var = dvc_buffer;
                }
                Err(err) => {
                    error!("{}", err);
                    status = false;
                }
            }
        }
        status
    }

    fn support_format(support: u8) -> Vec<&'static str> {
        let mut support_types = Vec::new();
        if support & 0x1 == 0x1 {
            support_types.push("jpg");
        }
        if support & 0x2 == 0x2 {
            support_types.push("tga");
        }
        if support & 0x4 == 0x4 {
            support_types.push("pcx");
        }
        if support & 0x8 == 0x8 {
            support_types.push("gif");
        }
        if support & 0x10 == 0x10 {
            support_types.push("bmp");
        }
        if support & 0x20 == 0x20 {
            support_types.push("png");
        }
        support_types
    }
}

fn calculate_sha256(file_path: &str) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut sha256 = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;

        if bytes_read == 0 {
            break;
        }

        sha256.update(&buffer[..bytes_read]);
    }
    Ok(format!("{:x}", sha256.finalize()))
}

fn calculate_crc32_first_512(file_path: &str) -> io::Result<u32> {
    let mut file = File::open(file_path)?;
    let mut buffer = [0u8; 512];
    let bytes_read = file.read(&mut buffer)?;

    let mut hasher = Hasher::new();
    hasher.update(&buffer[..bytes_read]);
    Ok(hasher.finalize())
}
