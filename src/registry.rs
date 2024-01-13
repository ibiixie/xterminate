use windows::core::PCSTR;
use windows::Win32::Foundation::{GetLastError, ERROR_SUCCESS};

use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExA, RegDeleteValueA, RegOpenKeyExA, RegQueryValueExA,
    RegSetKeyValueA, HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_CONFIG, HKEY_CURRENT_USER,
    HKEY_LOCAL_MACHINE, HKEY_USERS, KEY_READ, KEY_SET_VALUE, KEY_WRITE, REG_BINARY,
    REG_CREATE_KEY_DISPOSITION, REG_DWORD, REG_DWORD_BIG_ENDIAN, REG_EXPAND_SZ, REG_LINK,
    REG_OPTION_NON_VOLATILE, REG_QWORD, REG_SZ,
};

use crate::logf;

#[derive(Copy, Clone, Debug)]
#[repr(isize)]
pub enum HKey {
    HKeyClassesRoot = HKEY_CLASSES_ROOT.0,
    HKeyCurrentConfig = HKEY_CURRENT_CONFIG.0,
    HKeyCurrentUser = HKEY_CURRENT_USER.0,
    HKeyLocalMachine = HKEY_LOCAL_MACHINE.0,
    HKeyUsers = HKEY_USERS.0,
}

impl std::fmt::Display for HKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                HKey::HKeyClassesRoot => "HKEY_CLASSES_ROOT",
                HKey::HKeyCurrentConfig => "HKEY_CURRENT_CONFIG",
                HKey::HKeyCurrentUser => "HKEY_CURRENT_USER",
                HKey::HKeyLocalMachine => "HKEY_LOCAL_MACHINE",
                HKey::HKeyUsers => "HKEY_USERS",
            }
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
// Note: DWord and QWord are little endian by default
pub enum ValueType {
    ExpandSZ = REG_EXPAND_SZ.0,
    // MultiSZ = REG_MULTI_SZ.0, // Unsupported because it requires a double null-terminated string
    Sz = REG_SZ.0,
    DWord = REG_DWORD.0,
    DWordBigEndian = REG_DWORD_BIG_ENDIAN.0,
    QWord = REG_QWORD.0,
    Binary = REG_BINARY.0,
    Link = REG_LINK.0,
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::ExpandSZ => "ExpandSZ",
                Self::Sz => "Sz",
                Self::DWord => "DWord",
                Self::DWordBigEndian => "DWordBigEndian",
                Self::QWord => "QWord",
                Self::Binary => "Binary",
                Self::Link => "Link",
            }
        )
    }
}

/// Sets an existing key to the specified value or creates a new one if the specified key does not exist.
///
/// # Panics
///
/// This function panics if the specified registry key could not be opened
/// or created, could not be set, or could not be closed. This should only
/// occur if the given parameters are invalid. On panic, the full OS error
/// code will be printed on-screen and to stderr.
pub fn set_value(root_key: HKey, subkey: &str, name: &str, value_type: ValueType, value: &str) {
    unsafe {
        let mut hkey = HKEY(root_key as isize);

        if RegCreateKeyExA(
            hkey,
            PCSTR(std::ffi::CString::new(subkey).unwrap().as_bytes().as_ptr()),
            0,
            PCSTR(std::ptr::null()),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            std::ptr::null(),
            &mut hkey,
            std::ptr::null_mut::<REG_CREATE_KEY_DISPOSITION>(),
        )
        .is_err()
        {
            panic!(
                "failed to set registry key: could not create or open registry key 
            '{root_key}:{subkey}' for writing: RegCreateKeyExA() failed (os error {})",
                GetLastError().0
            );
        }

        logf!(
            "Writing registry value '{}\\{}\\{}' (type: {}) to '{}'",
            root_key,
            subkey,
            name,
            value_type,
            value
        );

        if RegSetKeyValueA(
            hkey,
            PCSTR(std::ptr::null()),
            PCSTR(std::ffi::CString::new(name).unwrap().as_bytes().as_ptr()),
            value_type as u32,
            std::ffi::CString::new(value)
                .unwrap()
                .as_bytes()
                .as_ptr()
                .cast::<std::ffi::c_void>(),
            match value_type {
                ValueType::Sz | ValueType::ExpandSZ => {
                    u32::try_from(value.len() + 1).unwrap()
                    // Include null-terminator
                }

                _ => u32::try_from(value.len()).unwrap(),
            },
        )
        .is_err()
        {
            panic!(
                "failed to set registry key '{root_key}\\{subkey}\\{name}' 
            to '{value}': RegSetKeyValueA() failed (os error {})",
                GetLastError().0
            );
        }

        assert!(
            RegCloseKey(hkey).is_ok(),
            "could not close registry key '{root_key}\\{subkey}': RegCloseKey() failed (os error {})",
            GetLastError().0
        );
    }
}

/// Deletes an existing registry key
///
/// # Panics
///
/// This function panics if the specified registry key could not
/// be opened, deleted or closed. This should only occur if the
/// given parameters are invalid. On panic, the full OS error code
/// will be printed on-screen and to stderr.
pub fn delete_value(root_key: HKey, subkey: &str, name: &str) {
    unsafe {
        let mut hkey = HKEY(root_key as isize);

        if RegOpenKeyExA(
            hkey,
            PCSTR(std::ffi::CString::new(subkey).unwrap().as_bytes().as_ptr()),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        )
        .is_err()
        {
            panic!(
                "could not delete registry value '{root_key}\\{subkey}\\{name}': 
            RegOpenKeyExA() (os error {})",
                GetLastError().0
            );
        }

        logf!(
            "Deleting registry value '{}::{}::{}'",
            root_key,
            subkey,
            name
        );

        if RegDeleteValueA(
            hkey,
            PCSTR(std::ffi::CString::new(name).unwrap().as_bytes().as_ptr()),
        )
        .is_err()
        {
            panic!(
                "could not delete registry value '{root_key}\\{subkey}\\{name}': 
            RegDeleteValueA() failed (os error {})",
                GetLastError().0
            );
        }

        assert!(
            RegCloseKey(hkey).is_ok(),
            "failed to close registry key '{root_key}\\{subkey}' (os error {})",
            GetLastError().0
        );
    }
}

/// Returns true of the given registry key and/or value exists, or false otherwise.
/// If `name` is not specified, the function will only check if the key exists. If
/// `name` _is_ specified, it will also check if any value by the given name exists.
///
/// # Panics
///
/// Panics if:
///  - `name` is null-terminated
///  - The underlying calls to Windows registry fails
#[must_use]
pub fn exists(root_key: HKey, subkey: &str, name: Option<&str>) -> bool {
    unsafe {
        let mut hkey = HKEY(root_key as isize);

        if RegOpenKeyExA(
            hkey,
            PCSTR(std::ffi::CString::new(subkey).unwrap().as_bytes().as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        )
        .is_err()
        {
            return false; // Key was not found or could not be opened.
        }

        // Do not check value if `name` was not supplied
        if name.is_none() {
            assert!(
                RegCloseKey(hkey).is_ok(),
                "failed to close registry key '{root_key}\\{subkey}' (os error {})",
                GetLastError().0
            );

            return true;
        }

        let result = RegQueryValueExA(
            hkey,
            Some(PCSTR(
                std::ffi::CString::new(name.unwrap())
                    .unwrap()
                    .as_bytes()
                    .as_ptr(),
            )),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        assert!(
            RegCloseKey(hkey).is_ok(),
            "failed to close registry key '{root_key}\\{subkey}' (os error {})",
            GetLastError().0
        );

        result == ERROR_SUCCESS
    }
}
