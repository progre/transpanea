use std::{env, mem::forget, path::Path};

use anyhow::{Result, bail};
use raw_window_handle::RawWindowHandle;
use tray_icon::Icon;
use windows::{
    Win32::{
        Foundation::{COLORREF, HWND},
        UI::{
            Shell::ExtractIconExW,
            WindowsAndMessaging::{
                GWL_EXSTYLE, GetWindowLongPtrW, HICON, HWND_TOPMOST, LWA_ALPHA, SWP_NOMOVE,
                SWP_NOSIZE, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos,
                WINDOW_EX_STYLE, WS_EX_LAYERED,
            },
        },
    },
    core::{HSTRING, Owned},
};

fn to_hwnd(handle: &RawWindowHandle) -> HWND {
    let RawWindowHandle::Win32(handle) = handle else {
        unreachable!();
    };
    HWND(handle.hwnd.get() as *mut _)
}

pub fn enable_alpha(handle: &RawWindowHandle) {
    let hwnd = to_hwnd(handle);

    let ex_style = WINDOW_EX_STYLE(unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) } as u32);
    let new_ex_style = ex_style | WS_EX_LAYERED;

    unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style.0 as isize) };
}

pub fn set_opacity(handle: &RawWindowHandle, opacity: u8) {
    let hwnd = to_hwnd(handle);

    unsafe { SetLayeredWindowAttributes(hwnd, COLORREF(0), u8::MAX - opacity, LWA_ALPHA) }.unwrap();
}

pub fn always_on_top(handle: &RawWindowHandle) {
    let hwnd = to_hwnd(handle);

    let uflags = SWP_NOMOVE | SWP_NOSIZE;
    unsafe { SetWindowPos(hwnd, Some(HWND_TOPMOST), 0, 0, 0, 0, uflags) }.unwrap();
}

fn extract_icon(exe_path: &Path) -> Result<Owned<HICON>> {
    let exe_path = HSTRING::from(exe_path.as_os_str());
    let mut hicon = HICON::default();
    let result = unsafe { ExtractIconExW(&exe_path, 0, None, Some(&mut hicon), 1) };
    if result == 0 {
        bail!("Failed to extract icon");
    }
    if hicon.is_invalid() {
        bail!("Invalid icon handle");
    }
    Ok(unsafe { Owned::new(hicon) })
}

pub fn load_icon_from_exe() -> Result<Icon> {
    let exe_path = env::current_exe()?;
    let hicon = extract_icon(&exe_path)?;
    let icon = Icon::from_handle((*hicon).0 as isize);
    forget(hicon);
    Ok(icon)
}
