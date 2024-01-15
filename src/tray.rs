use windows::core::PCSTR;

use windows::Win32::Foundation::{GetLastError, HMODULE, HWND, LPARAM, LRESULT, POINT, WPARAM};

use windows::Win32::UI::Shell::{
    Shell_NotifyIconA, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAA,
};

use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, CreateWindowExA, DefWindowProcA, DestroyWindow, DispatchMessageA,
    GetCursorPos, GetWindowLongPtrW, InsertMenuA, LoadImageA, PeekMessageA, RegisterClassExA,
    SetForegroundWindow, SetWindowLongPtrW, TrackPopupMenu, TranslateMessage, GWLP_USERDATA, HICON,
    HMENU, IMAGE_ICON, LR_LOADFROMFILE, MF_BYPOSITION, MF_DISABLED, MF_SEPARATOR, MSG, PM_REMOVE,
    TPM_BOTTOMALIGN, WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_LBUTTONDOWN, WM_RBUTTONDOWN,
    WM_USER, WNDCLASSEXA,
};

use windows::Win32::System::LibraryLoader::GetModuleHandleA;

const TRAYICON_ID: u32 = 1;
const WM_USER_TRAYICON: u32 = WM_USER + TRAYICON_ID;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::rc::Rc;

use crate::{input::Keybind, logf, registry};

#[repr(usize)]
pub enum TrayEvent {
    OnMenuSelectExit = 0,
    OnMenuSelectStartWithWindows = 1,
    OnMenuSelectResetCursor = 2,
    OnMenuSelectOpenConfig = 3,
    OnMenuSelectEnterTerminationMode = 4,
}

pub trait TrayEventHandler {
    fn handle(&mut self, event: TrayEvent);
}

impl From<u16> for TrayEvent {
    fn from(v: u16) -> Self {
        match v {
            0 => Self::OnMenuSelectExit,
            1 => Self::OnMenuSelectStartWithWindows,
            2 => Self::OnMenuSelectResetCursor,
            3 => Self::OnMenuSelectOpenConfig,
            4 => Self::OnMenuSelectEnterTerminationMode,
            _ => panic!("Invalid enum value '{v}'"),
        }
    }
}

pub struct Tray {
    hwnd: HWND,
    nid: NOTIFYICONDATAA,

    event_handler: Rc<RefCell<dyn TrayEventHandler>>,
    keybinds: HashMap<String, Keybind>,
}

impl Tray {
    pub fn create(
        icon_filename: &str,
        event_handler: Rc<RefCell<dyn TrayEventHandler>>,
        keybinds: HashMap<String, Keybind>,
    ) -> Rc<RefCell<Self>> {
        let hwnd = Self::create_window();
        let nid = Self::create_trayicon(hwnd, icon_filename);

        let tray = Rc::new(RefCell::new(Self {
            hwnd,
            nid,
            event_handler,
            keybinds,
        }));

        // Todo: Move this into create_window()?
        unsafe {
            SetWindowLongPtrW(
                hwnd,
                GWLP_USERDATA,
                std::ptr::addr_of_mut!(*tray.borrow_mut()) as isize,
            );
        }

        tray
    }

    /// # Panics
    ///
    /// Panics if [`Shell_NotifyIconA`] fails.
    pub fn delete(&self) {
        unsafe {
            assert!(
                Shell_NotifyIconA(NIM_DELETE, &self.nid).as_bool(),
                "tray icon could not be deleted"
            );

            DestroyWindow(self.hwnd);
        }
    }

    fn create_window() -> HWND {
        unsafe {
            let class_name =
                std::ffi::CString::new("xterminatortrayiconwcname".as_bytes()).unwrap();

            let wndclass = WNDCLASSEXA {
                cbSize: u32::try_from(std::mem::size_of::<WNDCLASSEXA>()).unwrap(),
                hInstance: GetModuleHandleA(PCSTR(std::ptr::null())).unwrap(),
                lpfnWndProc: Some(trayicon_input_callback),
                lpszClassName: PCSTR(class_name.as_ptr().cast::<u8>()),
                ..Default::default()
            };

            assert!(
                RegisterClassExA(&wndclass) > 0,
                "tray-icon window class registration failed: RegisterClassA() returned NULL (os error code {})", GetLastError().0
            );

            logf!("Creating system tray window");
            let hwnd = CreateWindowExA(
                WINDOW_EX_STYLE::default(),
                wndclass.lpszClassName,
                PCSTR(std::ptr::null()),
                WINDOW_STYLE::default(),
                0,
                0,
                0,
                0,
                HWND(0),
                HMENU(0),
                wndclass.hInstance,
                None,
            );

            assert!(
                hwnd.0 > 0,
                "trayicon window creation failed: CreateWindowExA() returned NULL (os error code {})",
                GetLastError().0
            );

            hwnd
        }
    }

    fn create_trayicon(hwnd: HWND, icon_filename: &str) -> NOTIFYICONDATAA {
        unsafe {
            let mut nid = NOTIFYICONDATAA {
                cbSize: u32::try_from(std::mem::size_of::<NOTIFYICONDATAA>()).unwrap(),
                hWnd: hwnd,
                uID: TRAYICON_ID,
                uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
                uCallbackMessage: WM_USER_TRAYICON,
                hIcon: Self::load_icon_from_file(icon_filename),
                ..Default::default()
            };

            // NIF_TIP
            // str to CHAR array conversion
            let tooltip_str = "xterminate says hi! :)";
            assert!(
                tooltip_str.len() < 128,
                "tooltip cannot be more than 127 characters!"
            );

            let mut tooltip_array = [0u8; 128];
            tooltip_array[..tooltip_str.len()].copy_from_slice(tooltip_str.as_bytes());

            nid.szTip = tooltip_array;

            logf!("Creating system tray icon");
            Shell_NotifyIconA(NIM_ADD, &nid);

            nid
        }
    }

    // Todo: look into shortening this method
    #[allow(clippy::too_many_lines)]
    fn show_menu(&mut self) {
        unsafe {
            let mut cursor_pos = POINT::default();
            GetCursorPos(&mut cursor_pos);

            logf!("Creating and populating system tray menu");
            let menu_handle = CreatePopupMenu().unwrap();

            InsertMenuA(
                menu_handle,
                1,
                MF_BYPOSITION,
                TrayEvent::OnMenuSelectResetCursor as usize,
                PCSTR("Reset cursor\0".as_ptr()),
            );
            InsertMenuA(
                menu_handle,
                2,
                MF_BYPOSITION,
                TrayEvent::OnMenuSelectOpenConfig as usize,
                PCSTR("Open config...\0".as_ptr()),
            );

            InsertMenuA(
                menu_handle,
                3,
                MF_BYPOSITION | MF_SEPARATOR,
                0,
                PCSTR::null(),
            );

            let terminate_click_keybind = self.keybinds.get("terminate_click").unwrap().to_string();
            let terminate_immediate_keybind = self
                .keybinds
                .get("terminate_immediate")
                .unwrap()
                .to_string();

            InsertMenuA(
                menu_handle,
                4,
                MF_BYPOSITION,
                TrayEvent::OnMenuSelectEnterTerminationMode as usize,
                PCSTR(
                    CString::new(format!(
                        "Enter termination mode ({terminate_click_keybind})"
                    ))
                    .unwrap()
                    .as_bytes()
                    .as_ptr(),
                ),
            );
            InsertMenuA(
                menu_handle,
                5,
                MF_BYPOSITION | MF_DISABLED,
                0,
                PCSTR(
                    CString::new(format!(
                        "Terminate active window ({terminate_immediate_keybind})"
                    ))
                    .unwrap()
                    .as_bytes()
                    .as_ptr(),
                ),
            );

            InsertMenuA(
                menu_handle,
                6,
                MF_BYPOSITION | MF_SEPARATOR,
                0,
                PCSTR::null(),
            );

            let enabled_str = if registry::exists(
                registry::HKey::HKeyCurrentUser,
                "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
                Some("xterminate"),
            ) {
                "ON"
            } else {
                "OFF"
            };

            InsertMenuA(
                menu_handle,
                7,
                MF_BYPOSITION,
                TrayEvent::OnMenuSelectStartWithWindows as usize,
                PCSTR(
                    CString::new(format!("Start with Windows ({enabled_str})"))
                        .unwrap()
                        .as_bytes()
                        .as_ptr(),
                ),
            );

            InsertMenuA(
                menu_handle,
                8,
                MF_BYPOSITION,
                TrayEvent::OnMenuSelectExit as usize,
                PCSTR("Exit\0".as_ptr()),
            );

            // Required or the popup menu won't close properly
            SetForegroundWindow(self.hwnd);

            TrackPopupMenu(
                menu_handle,
                TPM_BOTTOMALIGN,
                cursor_pos.x,
                cursor_pos.y,
                0,
                self.hwnd,
                None,
            );
        }
    }

    pub fn poll(&self) {
        unsafe {
            let mut message = MSG::default();
            while PeekMessageA(&mut message, self.hwnd, 0, 0, PM_REMOVE).as_bool() {
                TranslateMessage(&message);
                DispatchMessageA(&message);
            }
        }
    }

    #[must_use]
    /// # Panics
    ///
    /// Panics if Windows fails to load the specified icon file (see [`LoadImageA`])
    /// or if `filename` could not be turned into a valid [`CString`].
    pub fn load_icon_from_file(filename: &str) -> HICON {
        let hicon = unsafe {
            LoadImageA(
                HMODULE(0),
                PCSTR(
                    std::ffi::CString::new(filename)
                        .unwrap()
                        .as_bytes()
                        .as_ptr(),
                ),
                IMAGE_ICON,
                0,
                0,
                LR_LOADFROMFILE,
            )
        }
        .unwrap_or_else(|_| {
            panic!(
                "failed to load icon '{}': is the file missing or corrupt? (os error {})",
                filename,
                unsafe { GetLastError().0 }
            )
        });

        HICON(hicon.0)
    }
}

unsafe extern "system" fn trayicon_input_callback(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let instance = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Tray;

    match msg {
        WM_USER_TRAYICON => match u32::try_from(lparam.0).unwrap() {
            WM_LBUTTONDOWN | WM_RBUTTONDOWN => {
                instance.as_mut().unwrap().show_menu();
                LRESULT(0)
            }

            _ => DefWindowProcA(hwnd, msg, wparam, lparam),
        },

        WM_COMMAND => {
            // Separate the first and last 2 bytes (4 bits) of wparam, equivalent to LOWORD()/HIWORD()
            // The low bytes tell us which of the popup menu's items were clicked ('command'), and
            // correspond to the WMU_XXX events defined at the top of this file.

            // let id = (wparam.0 & 0b11110000) as u16;
            let cmd = u16::try_from(wparam.0 & 0b0000_1111).unwrap();

            let handler = &mut instance.as_mut().unwrap().event_handler;
            handler.borrow_mut().handle(TrayEvent::from(cmd));

            LRESULT(0)
        }

        _ => {
            // Any other messages can be ignored as we only care about the trayicon related ones
            DefWindowProcA(hwnd, msg, wparam, lparam)
        }
    }
}
