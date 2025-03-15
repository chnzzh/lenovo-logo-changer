# Lenovo UEFI Boot Logo Changer

![GitHub License](https://img.shields.io/github/license/chnzzh/lenovo-logo-changer)
![GitHub top language](https://img.shields.io/github/languages/top/chnzzh/lenovo-logo-changer)
![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/chnzzh/lenovo-logo-changer/release.yml)
![Static Badge](https://img.shields.io/badge/!!!SeeReadmeFirst!!!-orangered)

*Lenovo UEFI Boot Logo Changer* is a rust program designed to modify the Boot startup logo on Lenovo devices with UEFI firmware.
This tool allows u to customize the boot logo with different format image.

![20240128171115](https://github.com/chnzzh/lenovo-logo-changer/assets/41407837/674d7db6-e2af-4360-956d-edacf9fe5157)

**[Download](https://github.com/chnzzh/lenovo-logo-changer/releases/latest) the latest executable file compiled by GitHub Actions**

You can also refer to [How to build](#how-to-build) to compile it yourself.

## Important

+ **This program involves modifications to UEFI variables and the ESP partition. Please ensure to backup important files before usage.**
+ **This program will not check if the image files you are using comply with the correct image format. Please ensure that your images can function properly.** (Otherwise your system may be compromised: [LogoFAIL](https://binarly.io/posts/finding_logofail_the_dangers_of_image_parsing_during_system_boot/))
+ This program is intended for personal research use only.
+ **All risks are assumed by the user**.

## Usage

+ Right-click on the executable file and run it in administrator mode.
+ Click "Open Image" to upload a suitable image.
+ Click "Change Logo"

![ui](https://github.com/user-attachments/assets/b1d5112e-3bcb-44c2-8c9b-d43669285cfd)

## How it Works

Lenovo UEFI Boot Logo Changer operates by leveraging Lenovo's support for user customization of the boot logo through the ESP (EFI System Partition).
The process involves placing a custom image into the ESP partition and then configuring UEFI variables to instruct the DXE (Driver Execution Environment) program to read and display the user-defined logo during the system's boot process.

So this tool do:

1. **Read UEFI Variables** to determine whether the system supports Logo Change;
2. **Place Selected Image in ESP Partition**;
3. **Modify UEFI Variables** to enable the UEFI program to correctly set and display the customized logo.

All of the above operations need to be performed with administrator privileges.

## How to build

1. Install Rust and MinGW:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   sudo apt install mingw-w64 -y
   ```

2. Add the Windows target for Rust:
   ```bash
   rustup target add x86_64-pc-windows-gnu --toolchain nightly
   ```

3. Build the project:
   ```bash
   cargo +nightly build --verbose --target x86_64-pc-windows-gnu --release
   ```

## Support Types

+ ThinkBook 14 G4+ ARA
+ ThinkBook 16 G5+ ARP
+ IdeaPad Slim 5 14AHP9 (83DB)
+ Lenovo LOQ 15IRH8
+ Lenovo Yoga Slim 7 Aura Edition 15,3"
+ ...
