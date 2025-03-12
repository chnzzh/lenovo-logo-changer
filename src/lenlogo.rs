use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use std::process::Command;
use efivar::efi::{Variable, VariableFlags};
use sha2::{Sha256, Digest};

use crate::esp_partition::{copy_file_to_esp, delete_logo_path};

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
                    eprintln!("read lbldesp_var failed: buffer length is not 10");
                    return false;
                }
                self.enable = esp_buffer[0];
                self.width = u32::from_le_bytes(esp_buffer[1..5].try_into().unwrap());
                self.height = u32::from_le_bytes(esp_buffer[5..9].try_into().unwrap());
                self.support = Self::support_format(esp_buffer[9]);
                self.lbldesp_var = <[u8; 10]>::try_from(esp_buffer).unwrap();
            },
            Err(err) => {
                eprintln!("read lbldesp_var failed: {}", err);
                return false;
            }
        }

        let dvc_var = Variable::from_str("LBLDVC-871455D1-5576-4FB8-9865-AF0824463C9F").unwrap();
        match varman.read(&dvc_var) {
            Ok((dvc_buffer, _attr)) => {
                if dvc_buffer.len() != 40 {
                    eprintln!("read lbldvc_var failed: buffer length is not 40");
                    return false;
                }
                self.version = u32::from_le_bytes(dvc_buffer[0..4].try_into().unwrap());
                self.lbldvc_var = <[u8; 40]>::try_from(dvc_buffer).unwrap();
            },
            Err(err) => {
                eprintln!("read lbldvc_var failed: {}", err);
                return false;
            }
        }
        true
    }

    pub(crate) fn set_logo(&mut self, img_path: &String) -> bool {

        // 复制文件到ESP分区
        let file_path = Path::new(img_path);
        let file_extension = file_path.extension().unwrap().to_str().unwrap();
        println!("file_extension: {}", file_extension);

        let dst_path = format!(r"/EFI/Lenovo/Logo/mylogo_{}x{}.{}", self.width, self.height, file_extension);
        println!("dst_path: {}", dst_path);

        if copy_file_to_esp(img_path, &dst_path) == false {
            eprintln!("copy file failed");
            return false;
        }

        let mut varman = efivar::system();
        // 修改logoinfo
        let mut esp_buffer = self.lbldesp_var.clone();
        esp_buffer[0] = 1;
        let esp_var = Variable::from_str("LBLDESP-871455D0-5576-4FB8-9865-AF0824463B9E").unwrap();
        match varman.write(&esp_var,VariableFlags::from_bits(0x7).unwrap(), &esp_buffer) {
            Ok(rt) => {
                self.enable = 1;
                self.lbldesp_var = esp_buffer;
                println!("{:?}", rt);
            },
            Err(err) => {
                eprintln!("write lbldesp_var failed: {}", err);
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
                eprintln!("[!] READ ERROR {}: {}", img_path, e);
                return false;
            },
        }
        let mut dvc_buffer = self.lbldvc_var.clone();
        dvc_buffer[4..36].clone_from_slice(&sha256_bytes);
        println!("sha256_bytes: {:?}", sha256_bytes);
        println!("dvc_buffer: {:?}", dvc_buffer);

        let dvc_var = Variable::from_str("LBLDVC-871455D1-5576-4FB8-9865-AF0824463C9F").unwrap();
        match varman.write(&dvc_var,VariableFlags::from_bits(0x7).unwrap(), &dvc_buffer) {
            Ok(rt) => {
                self.lbldvc_var = dvc_buffer;
                println!("{:?}", rt);
            },
            Err(err) => {
                eprintln!("write lbldvc_var failed: {}", err);
                return false;
            }
        }
        true
    }

    pub(crate) fn restore_logo(&mut self) -> bool {
        //
        let mut status = true;
        if !delete_logo_path() {
            eprintln!("delete logo path failed");
            status = false;
        }

        let mut varman = efivar::system();
        // 修改logoinfo
        let mut esp_buffer = self.lbldesp_var.clone();
        if esp_buffer[0] != 0 {
            esp_buffer[0] = 0;
            let esp_var = Variable::from_str("LBLDESP-871455D0-5576-4FB8-9865-AF0824463B9E").unwrap();
            match varman.write(&esp_var,VariableFlags::from_bits(0x7).unwrap(), &esp_buffer) {
                Ok(rt) => {
                    self.lbldesp_var = esp_buffer;
                    println!("{:?}", rt);
                },
                Err(err) => {
                    eprintln!("write lbldesp_var failed: {}", err);
                    status =  false;
                }
            }
        }

        // 修改logocheck
        let mut dvc_buffer = self.lbldvc_var.clone();
        if dvc_buffer[4..40] != [0u8; 36] {
            dvc_buffer[4..40].clone_from_slice(&[0u8; 36]);
            let dvc_var = Variable::from_str("LBLDVC-871455D1-5576-4FB8-9865-AF0824463C9F").unwrap();
            match varman.write(&dvc_var,VariableFlags::from_bits(0x7).unwrap(), &dvc_buffer) {
                Ok(rt) => {
                    self.lbldvc_var = dvc_buffer;
                    println!("{:?}", rt);
                },
                Err(err) => {
                    eprintln!("write lbldvc_var failed: {}", err);
                    status = false;
                }
            }
        }
        status
    }

    pub(crate) fn get_loading_icon(&mut self) -> bool {
        // 执行 bcdedit /enum all 命令
        let output = Command::new("cmd")
            .args(&["/C", "bcdedit", "/enum", "all"])
            .output();

        match output {
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
                true // 处理错误，返回 false
            }
        }
    }

    pub(crate) fn set_loading_icon(&mut self, show_loading_icon: bool) -> bool {
        let command = if show_loading_icon {
            "bcdedit.exe -set bootuxdisabled off"
        } else {
            "bcdedit.exe -set bootuxdisabled on"
        };

        // 使用 Command 来执行外部命令
        match Command::new("cmd")
            .args(&["/C", command])
            .output() {
            Ok(output) => {
                // 检查命令是否成功执行
                if output.status.success() {
                    println!("Command executed successfully");
                    true
                } else {
                    // 输出失败信息
                    eprintln!(
                        "Command failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    false
                }
            },
            Err(e) => {
                eprintln!("Failed to execute command: {}", e);
                false
            }
        }
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