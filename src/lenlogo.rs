use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use efivar::efi::{Variable, VariableFlags};
use sha2::{Sha256, Digest};
use log::{debug, error, info, warn};

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
    pub(crate) lbldesp_var_name: String,
    pub(crate) lbldvc_var_name: String,
}

const LBLDESP_VAR_NAME: &str = "LBLDESP-871455D0-5576-4FB8-9865-AF0824463B9E";
const LBLDVC_VAR_NAME: &str = "LBLDVC-871455D1-5576-4FB8-9865-AF0824463C9F";

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
            lbldesp_var_name: LBLDESP_VAR_NAME.to_string(),
            lbldvc_var_name: LBLDVC_VAR_NAME.to_string(),
        }
    }
}

impl PlatformInfo {
    pub(crate) fn get_info(&mut self) -> bool {
        let varman = efivar::system();

        let mut lbldesp_var_name = self.lbldesp_var_name.clone();
        let mut lbldvc_var_name = self.lbldvc_var_name.clone();
        let mut esp_var = Variable::from_str(&lbldesp_var_name).unwrap();

        let esp_read_result = varman.read(&esp_var);
        let (esp_buffer, _attr) = match esp_read_result {
            Ok((esp_buffer, _attr)) => {
                (esp_buffer, _attr)
            }
            Err(err) => {
                #[cfg(target_os = "linux")]
                {
                    if let Some((desp_name, dvc_name)) = find_lenovo_vars() {
                        lbldesp_var_name = desp_name;
                        lbldvc_var_name = dvc_name;
                        esp_var = Variable::from_str(&lbldesp_var_name).unwrap();
                        match varman.read(&esp_var) {
                            Ok((esp_buffer, _attr)) => (esp_buffer, _attr),
                            Err(err) => {
                                error!("read lbldesp_var failed: {}", err);
                                return false;
                            }
                        }
                    } else {
                        error!("read lbldesp_var failed: {}", err);
                        return false;
                    }
                }
                #[cfg(not(target_os = "linux"))]
                {
                    error!("read lbldesp_var failed: {}", err);
                    return false;
                }
            }
        };

        if esp_buffer.len() != 10 {
            error!("read lbldesp_var failed: buffer length is not 10");
            return false;
        }
        self.enable = esp_buffer[0];
        self.width = u32::from_le_bytes(esp_buffer[1..5].try_into().unwrap());
        self.height = u32::from_le_bytes(esp_buffer[5..9].try_into().unwrap());
        self.support = Self::support_format(esp_buffer[9]);
        self.lbldesp_var = <[u8; 10]>::try_from(esp_buffer).unwrap();
        self.lbldesp_var_name = lbldesp_var_name.clone();
        self.lbldvc_var_name = lbldvc_var_name.clone();

        #[cfg(target_os = "linux")]
        {
            if !efivar_exists(&lbldvc_var_name) {
                if let Some(dvc_name) = find_lenovo_var("LBLDVC") {
                    lbldvc_var_name = dvc_name;
                } else {
                    warn!("LBLDVC variable not found; proceeding without DVC support");
                    self.lbldvc_var_name = String::new();
                    return true;
                }
            }
        }

        let mut dvc_var = Variable::from_str(&lbldvc_var_name).unwrap();
        let dvc_read_result = varman.read(&dvc_var);
        let dvc_read_result = match dvc_read_result {
            Ok((dvc_buffer, _attr)) => Some((dvc_buffer, _attr)),
            Err(err) => {
                #[cfg(target_os = "linux")]
                {
                    if let Some(dvc_name) = find_lenovo_var("LBLDVC") {
                        lbldvc_var_name = dvc_name;
                        dvc_var = Variable::from_str(&lbldvc_var_name).unwrap();
                        match varman.read(&dvc_var) {
                            Ok((dvc_buffer, _attr)) => Some((dvc_buffer, _attr)),
                            Err(err) => {
                                error!("read lbldvc_var failed: {}", err);
                                return false;
                            }
                        }
                    } else {
                        warn!("LBLDVC variable not found; proceeding without DVC support");
                        self.lbldvc_var_name = String::new();
                        return true;
                    }
                }
                #[cfg(not(target_os = "linux"))]
                {
                    error!("read lbldvc_var failed: {}", err);
                    return false;
                }
            }
        };

        if let Some((dvc_buffer, _attr)) = dvc_read_result {
            if dvc_buffer.len() != 40 {
                error!("read lbldvc_var failed: buffer length is not 40");
                return false;
            }
            self.version = u32::from_le_bytes(dvc_buffer[0..4].try_into().unwrap());
            self.lbldvc_var = <[u8; 40]>::try_from(dvc_buffer).unwrap();
            self.lbldvc_var_name = lbldvc_var_name;
        }
        true
    }

    pub(crate) fn set_logo(&mut self, img_path: &String) -> bool {

        // 复制文件到ESP分区
        let file_path = Path::new(img_path);
        let file_extension = file_path.extension().unwrap().to_str().unwrap();
    debug!("file_extension: {}", file_extension);

        let dst_path = format!(r"/EFI/Lenovo/Logo/mylogo_{}x{}.{}", self.width, self.height, file_extension);
    info!("target path: {}", dst_path);

        if copy_file_to_esp(img_path, &dst_path) == false {
            error!("copy file failed");
            return false;
        }

        let mut varman = efivar::system();
        
        // 修改logoinfo
        let mut esp_buffer = self.lbldesp_var.clone();
        esp_buffer[0] = 1;
        let esp_var = Variable::from_str(&self.lbldesp_var_name).unwrap();
        
        // 在Linux下需要先移除immutable属性
        let write_result = with_efi_var_writable(&self.lbldesp_var_name, || {
            match varman.write(&esp_var, VariableFlags::from_bits(0x7).unwrap(), &esp_buffer) {
                Ok(rt) => {
                    debug!("write lbldesp_var: {:?}", rt);
                    Ok(())
                },
                Err(err) => {
                    Err(format!("write lbldesp_var failed: {}", err))
                }
            }
        });
        
        match write_result {
            Ok(_) => {
                self.enable = 1;
                self.lbldesp_var = esp_buffer;
            },
            Err(err) => {
                error!("{}", err);
                return false;
            }
        }

        // 修改logocheck
        let sha256_bytes;
        match calculate_sha256(img_path) {
            Ok(sha256) => {
                // 填充到new_logo_check中
                //let sha256_bytes = sha256.as_bytes();
                // 将sha256十六进制字符串转化为十六进制序列
                sha256_bytes = hex::decode(sha256).unwrap();
            }
            Err(e) => {
                error!("read error {}: {}", img_path, e);
                return false;
            },
        }
        let mut dvc_buffer = self.lbldvc_var.clone();
        dvc_buffer[4..36].clone_from_slice(&sha256_bytes);
    debug!("sha256_bytes: {:?}", sha256_bytes);
    debug!("dvc_buffer: {:?}", dvc_buffer);

        if self.lbldvc_var_name.is_empty() {
            return true;
        }

        let dvc_var = Variable::from_str(&self.lbldvc_var_name).unwrap();
        
        // 在Linux下需要先移除immutable属性
        let write_result = with_efi_var_writable(&self.lbldvc_var_name, || {
            match varman.write(&dvc_var, VariableFlags::from_bits(0x7).unwrap(), &dvc_buffer) {
                Ok(rt) => {
                    debug!("write lbldvc_var: {:?}", rt);
                    Ok(())
                },
                Err(err) => {
                    Err(format!("write lbldvc_var failed: {}", err))
                }
            }
        });
        
        match write_result {
            Ok(_) => {
                self.lbldvc_var = dvc_buffer;
            },
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
            let esp_var = Variable::from_str(&self.lbldesp_var_name).unwrap();
            
            // 在Linux下需要先移除immutable属性
            let write_result = with_efi_var_writable(&self.lbldesp_var_name, || {
                match varman.write(&esp_var, VariableFlags::from_bits(0x7).unwrap(), &esp_buffer) {
                    Ok(rt) => {
                        debug!("write lbldesp_var: {:?}", rt);
                        Ok(())
                    },
                    Err(err) => {
                        Err(format!("write lbldesp_var failed: {}", err))
                    }
                }
            });
            
            match write_result {
                Ok(_) => {
                    self.lbldesp_var = esp_buffer;
                },
                Err(err) => {
                    error!("{}", err);
                    status = false;
                }
            }
        }

        // 修改logocheck
        if self.lbldvc_var_name.is_empty() {
            return status;
        }

        let mut dvc_buffer = self.lbldvc_var.clone();
        if dvc_buffer[4..40] != [0u8; 36] {
            dvc_buffer[4..40].clone_from_slice(&[0u8; 36]);
            let dvc_var = Variable::from_str(&self.lbldvc_var_name).unwrap();
            
            let write_result = with_efi_var_writable(&self.lbldvc_var_name, || {
                match varman.write(&dvc_var, VariableFlags::from_bits(0x7).unwrap(), &dvc_buffer) {
                    Ok(rt) => { debug!("write lbldvc_var: {:?}", rt); Ok(()) },
                    Err(err) => Err(format!("write lbldvc_var failed: {}", err))
                }
            });
            
            match write_result {
                Ok(_) => {
                    self.lbldvc_var = dvc_buffer;
                },
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

#[cfg(target_os = "linux")]
fn find_lenovo_vars() -> Option<(String, String)> {
    let entries = std::fs::read_dir("/sys/firmware/efi/efivars").ok()?;
    let mut lbldesp_var_name = None;
    let mut lbldvc_var_name = None;

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let upper = file_name.to_ascii_uppercase();

        if lbldesp_var_name.is_none() && upper.starts_with("LBLDESP") {
            lbldesp_var_name = Some(file_name.to_string());
        } else if lbldvc_var_name.is_none() && upper.starts_with("LBLDVC") {
            lbldvc_var_name = Some(file_name.to_string());
        }

        if lbldesp_var_name.is_some() && lbldvc_var_name.is_some() {
            break;
        }
    }

    match (lbldesp_var_name, lbldvc_var_name) {
        (Some(desp), Some(dvc)) => Some((desp, dvc)),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn find_lenovo_var(prefix: &str) -> Option<String> {
    let entries = std::fs::read_dir("/sys/firmware/efi/efivars").ok()?;

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let upper = file_name.to_ascii_uppercase();
        if upper.starts_with(&prefix.to_ascii_uppercase()) {
            return Some(file_name.to_string());
        }
    }

    None
}

#[cfg(target_os = "linux")]
fn efivar_exists(var_name: &str) -> bool {
    if var_name.is_empty() {
        return false;
    }
    let path = Path::new("/sys/firmware/efi/efivars").join(var_name);
    path.exists()
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