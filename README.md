# Lenovo UEFI Boot Logo Changer

![GitHub License](https://img.shields.io/github/license/chnzzh/lenovo-logo-changer)
![GitHub top language](https://img.shields.io/github/languages/top/chnzzh/lenovo-logo-changer)
![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/chnzzh/lenovo-logo-changer/release.yml)
![Static Badge](https://img.shields.io/badge/!!!SeeReadmeFirst!!!-orangered)

*Lenovo UEFI Boot Logo Changer* is a rust program designed to modify the Boot startup logo on Lenovo devices with UEFI firmware.
This tool allows u to customize the boot logo with different format image.

![20240128171115](https://github.com/chnzzh/lenovo-logo-changer/assets/41407837/674d7db6-e2af-4360-956d-edacf9fe5157)

## Important

+ **This program involves modifications to UEFI variables and the ESP partition. Please ensure to backup important files before usage.**
+ **This program will not check if the image files you are using comply with the correct image format. Please ensure that your images can function properly.** (Otherwise your system may be compromised: [LogoFAIL](https://binarly.io/posts/finding_logofail_the_dangers_of_image_parsing_during_system_boot/))
+ This program is intended for personal research use only.
+ **All risks are assumed by the user**.

## Usage

+ Right-click on the executable file and run it in administrator mode.
+ Click "Open Image" to upload a suitable image.
+ Click "Change Logo"

![ui](https://github.com/chnzzh/lenovo-logo-changer/assets/41407837/0dec7897-38ed-470c-afe6-825c6a56fcd1)

## How it Works

Lenovo UEFI Boot Logo Changer operates by leveraging Lenovo's support for user customization of the boot logo through the ESP (EFI System Partition).
The process involves placing a custom image into the ESP partition and then configuring UEFI variables to instruct the DXE (Driver Execution Environment) program to read and display the user-defined logo during the system's boot process.

So this tool do:

1. **Read UEFI Variables** to determine whether the system supports Logo Change;
2. **Place Selected Image in ESP Partition**;
3. **Modify UEFI Variables** to enable the UEFI program to correctly set and display the customized logo.

All of the above operations need to be performed with administrator privileges.

## Support Types

+ ThinkBook 14 G4+ ARA
