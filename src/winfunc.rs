use windows_sys::{
    Win32::System::Threading::*,Win32::Security::*,
};

pub(crate) fn is_admin() -> bool {
    unsafe {
        let mut is_admin = 0u32;
        let mut size = std::mem::size_of::<u32>() as u32;
        let mut token = CreateEventW(std::ptr::null(), 1, 0, std::ptr::null());
        OpenProcessToken(GetCurrentProcess(), TOKEN_ALL_ACCESS, &mut token);
        GetTokenInformation(
            token,
            TokenElevation,
            &mut is_admin as *mut u32 as *mut std::ffi::c_void,
            size,
            &mut size,
        );
    log::debug!("is_admin: {}", is_admin);
        if is_admin == 0 {
            false
        }
        else {
            true
        }
    }
}