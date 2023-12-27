# Lenovo UEFI Boot Logo Changer

*Lenovo UEFI Boot Logo Changer* is a rust program designed to modify the Boot startup logo on Lenovo devices with UEFI firmware.
This tool allows u to customize the boot logo with different format image.

## Important

+ **This program involves modifications to UEFI variables and the ESP partition. Please ensure to backup important files before usage.**
+ This program is intended for personal research use only.
+ **All risks are assumed by the user**.

## Usage

+ Right-click on the executable file and run it in administrator mode.
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

## Support Types

+ ThinkBook 14 G4+ ARA
