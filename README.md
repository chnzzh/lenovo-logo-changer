# Lenovo UEFI Boot Logo Changer

![GitHub License](https://img.shields.io/github/license/chnzzh/lenovo-logo-changer)
![GitHub top language](https://img.shields.io/github/languages/top/chnzzh/lenovo-logo-changer)
![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/chnzzh/lenovo-logo-changer/release.yml)
![Static Badge](https://img.shields.io/badge/contributions-welcome-blue)
![Static Badge](https://img.shields.io/badge/!!!SeeReadmeFirst!!!-orangered)

*Lenovo UEFI Boot Logo Changer* is a rust program designed to modify the Boot startup logo on Lenovo devices with UEFI firmware.
This tool allows you to customize the boot logo with different format image.

![20240128171115](https://github.com/chnzzh/lenovo-logo-changer/assets/41407837/674d7db6-e2af-4360-956d-edacf9fe5157)

**[Download](https://github.com/chnzzh/lenovo-logo-changer/releases/latest) the latest executable file compiled by GitHub Actions**

You can also refer to [How to build](#how-to-build) to compile it yourself.

## Disclaimer

+ **This program involves modifications to UEFI variables and the ESP partition. Please ensure to backup important files before usage.**
+ **This program will not check if the image files you are using comply with the correct image format. Please ensure that your images can function properly.** (Otherwise your system may be compromised: [LogoFAIL](https://binarly.io/posts/finding_logofail_the_dangers_of_image_parsing_during_system_boot/))
+ This program is intended for personal research use only.
+ The detailed list provided at the end is user contributed and **ONLY** for informational purposes. It shall provide a cleaner view of user tested devices than [#34](https://github.com/chnzzh/lenovo-logo-changer/issues/34). No responsibility can be put on the authors for any problem encountered after taking information from the list.
+ **All risks are assumed by the user**.

## Usage

![ui](https://github.com/user-attachments/assets/b1d5112e-3bcb-44c2-8c9b-d43669285cfd)

### Windows

+ Right-click on the executable file and run it in administrator mode.
+ Click "Open Image" to upload a suitable image.
+ Click "Change Logo"

### Linux

+ Run the program with root privileges:
  ```bash
  sudo ./lenovo-logo-changer
  ```
+ Click "Open Image" to upload a suitable image.
+ Click "Change Logo"

## How it Works

Lenovo UEFI Boot Logo Changer operates by leveraging Lenovo's support for user customization of the boot logo through the ESP (EFI System Partition).
The process involves placing a custom image into the ESP partition and then configuring UEFI variables to instruct the DXE (Driver Execution Environment) program to read and display the user-defined logo during the system's boot process.

So this tool do:

1. **Read UEFI Variables** to determine whether the system supports Logo Change;
2. **Place Selected Image in ESP Partition**;
3. **Modify UEFI Variables** to enable the UEFI program to correctly set and display the customized logo.

All of the above operations need to be performed with administrator privileges.

## How to build

### On Linux

1. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
   
#### For Linux

2. Build the project:
   ```bash
   cargo build --release
   ```  
#### For Windows

2. Add the Windows target for Rust:
   ```bash
   sudo apt install mingw-w64 -y
   rustup target add x86_64-pc-windows-gnu
   ```

3. Build the project:
   ```bash
   cargo build --release --target x86_64-pc-windows-gnu
   ```

## Community Contributions Welcome
If you have successfully used this tool on a Lenovo device, please report it in [#34](https://github.com/chnzzh/lenovo-logo-changer/issues/34).
Thanks to everyone who tests and shares successful devices !

## Currently Supported Devices

**🔴 ThinkPad**

+ ThinkPad E14 G6 (Intel)
+ Thinkpad T14P Gen 1
+ ThinkPad X13 Gen 4

**📕 ThinkBook**

+ ThinkBook 14p G3 ARH
+ ThinkBook 14 G4+ ARA
+ ThinkBook 14 G7+ IAH
+ ThinkBook 14 G7+ AHP
+ ThinkBook 16 G5+ ARP
+ ThinkBook 16 G7 IML

**💻 IdeaPad**

+ Ideapad 3 15ALC6 (82MF)
+ IdeaPad Pro 5 14APH8 (83AM)
+ IdeaPad Slim 3 15AMN8
+ IdeaPad Slim 3 15ARP10
+ Ideapad Slim 3i
+ IdeaPad Slim 5 14AHP9 (83DB)
+ IdeaPad Slim 5 16AHP9
+ Ideapad Gaming 3 15ACH6 (82K2)
+ IdeaPad Gaming 3 15IAH7
+ IdeaPad Pro 5 14AKP10
+ IdeaPad Pro 14IAH10

**🎮 LOQ**

+ LOQ 15ARP9
+ LOQ 15AHP9
+ LOQ 15AHP10
+ LOQ 15APH8
+ LOQ 15IAX9
+ LOQ 15IAX9E
+ LOQ 15IRH8
+ LOQ 15IRX10

**⚡ Legion**

+ Legion Go
+ Legion Pro 5 16ADR10 (83LT)
+ Legion Pro 7i Gen 10 (16IAX10H)
+ Legion y7000p 2024 IRX9
+ Legion 5 15ITH6H

**🪽 Yoga**

+ Yoga Slim 6 14IAP8 (82WU)
+ Yoga Slim 7 Aura Edition 15ILL9
+ Yoga Pro 7i Aura Edition
+ Yoga 9 2-in-1 14IMH9 (83AC)

**V series**

+ V15 G3 IAP (82TT)
+ V15 G4 AMN (82YU)

**Misc**
+ Lenovo XiaoXin Pro 13 (82DN)

## Device details

**A device name listed below but not directly followed by a table means that the device was tested but that minimal or no precisions were provided about formats or device specs.**

### 🔴 ThinkPad

**ThinkPad E14 G6 (Intel)**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Debian Trixie | untested | untested | untested | untested | yes | untested |

**Thinkpad T14P Gen 1**
**ThinkPad X13 Gen 4**

**📕 ThinkBook**

**ThinkBook 14p G3 ARH**
**ThinkBook 14 G4+ ARA**
**ThinkBook 14 G7+ IAH**
**ThinkBook 14 G7+ AHP**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Windows 11 Pro for Workstations 25H2 (build 26200.8117) | yes | untested | untested | untested | yes | yes |

**ThinkBook 16 G5+ ARP**
**ThinkBook 16 G7 IML**

**💻 IdeaPad**

**Ideapad 3 15ALC6 (82MF)**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Gentoo Linux x86_64 7.0.12-gentoo-r1 | no | untested | untested | untested | yes (1920x1080, BMP3 TrueColor, uncompressed only) | no |

**IdeaPad Pro 5 14APH8 (83AM)**
**IdeaPad Slim 3 15AMN8**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Windows 11 Home 25H2 | yes | untested | untested | untested | yes | untested |

**IdeaPad Slim 3 15AMN8**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Void Linux x86_64 7.3.0 | yes | untested | untested | untested | untested | untested |

**IdeaPad Slim 3 15ARP10**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Arch Linux 6.18.9-arch1-2 | yes | untested | untested | untested (not tried) | untested | untested |

**Ideapad Slim 3i**
**IdeaPad Slim 5 14AHP9 (83DB)**
**IdeaPad Slim 5 16AHP9**
**Ideapad Gaming 3 15ACH6 (82K2)**
**IdeaPad Gaming 3 15IAH7**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Windows 11 Home 25H2 (build 26200.8246) | untested | untested | untested | partial (plays as still image, no animation) | untested | yes |
| Windows 11 Home 25H2 | yes | untested | untested | yes (static) | yes | yes |

**IdeaPad Pro 5 14AKP10**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Arch Linux 6.18.8-3-cachyos | untested | untested | untested | untested (not tried) | untested | untested |

Note: static images confirmed working, exact image format not specified.

**IdeaPad Pro 5 14IAH10**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Windows 11 Pro 25H2 (build 26200.8968) | yes | untested | untested | untested | untested | yes |

**🎮 LOQ**

**LOQ 15ARP9**
**LOQ 15AHP9**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Windows 11 Home 24H2-25H2 (BIOS NZCN28WW) | untested | untested | untested | yes (full sized) | untested | yes |

**LOQ 15AHP10**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Windows 11 | untested | untested | untested | yes | untested | untested |

Note: static images also confirmed working, exact image format not specified.

**LOQ 15APH8**
**LOQ 15IAX9**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Arch Linux | untested | untested | untested | partial (single-frame gifs work; multi-frame gifs boot loop after latest BIOS update) | untested | untested |
| Windows 11 Home (build 26300, BIOS NECN47WW) | untested | untested | untested | yes (animated) | untested | untested |

Note: static images work flawlessly, exact image format not specified.

**LOQ 15IAX9E**
**LOQ 15IRH8**
**LOQ 15IRX10**

**⚡ Legion**

**Legion Go**
**Legion Pro 5 16ADR10 (83LT)**
**Legion Pro 7i Gen 10 (16IAX10H)**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Windows 11 Pro (build 26200.8039, BIOS Q7CN77WW) | yes | untested | untested | yes | yes | untested |

**Legion y7000p 2024 IRX9**
**Legion 5 15ITH6H**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Windows 11 Home 25H2 (build 26200.8875, BIOS H1CN58WW) | untested | untested | untested | no (2.1s clip, possibly too large) | untested | untested |

Note: other formats reported as successful, exact image formats not specified.

**🪽 Yoga**

**Yoga Slim 6 14IAP8 (82WU)**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Debian GNU/Linux 13 (trixie) x86_64 |   yes  |  untested   |  untested   |  untested   |  untested   |   yes  |

**Yoga Slim 7 Aura Edition 15ILL9**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Windows 11 Pro 25H2 (BIOS NYCN76WW) | untested | untested | untested | untested | untested | untested |

**Yoga Pro 7i Aura Edition**
**Yoga 9 2-in-1 14IMH9 (83AC)**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Windows 11 Pro 25H2 | yes | untested | untested | untested | untested (not tested yet) | untested |

**V series**

**V15 G3 IAP (82TT)**
**V15 G4 AMN (82YU)**

**Misc**
**Lenovo XiaoXin Pro 13 (82DN)**
|         | jpg | tga | pcx | gif | bmp | png |
|---------|-----|-----|-----|-----|-----|-----|
| Fedora Linux 44 (7.0.12) | untested | untested | untested | untested | yes (640x480) | untested |

Note: reported in issue #34 as model 82DM (XiaoXinPro-13ARE 2020); listed here under the existing 82DN entry pending confirmation of the exact model number.
