use super::super::*;
use std::path::Path;

const CLSCTX_INPROC_SERVER: Dword = 0x0000_0001;
const COINIT_APARTMENTTHREADED: Dword = 0x0000_0002;
const RPC_E_CHANGED_MODE: i32 = 0x8001_0106_u32 as i32;
const CLSID_SHELL_LINK: Guid = Guid {
    data1: 0x0002_1401,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};
const IID_ISHELL_LINKW: Guid = Guid {
    data1: 0x0002_14F9,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};
const IID_IPERSIST_FILE: Guid = Guid {
    data1: 0x0000_010B,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

fn startup_path() -> Option<PathBuf> {
    env::var_os("APPDATA").map(PathBuf::from).map(|base| {
        base.join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Startup")
            .join(STARTUP_SHORTCUT_NAME)
    })
}

pub(in crate::windows::app) fn startup_is_enabled() -> bool {
    startup_path().is_some_and(|path| path.exists())
}

pub(in crate::windows::app) fn set_startup_enabled(enabled: bool) {
    let Some(path) = startup_path() else {
        return;
    };

    if enabled {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(exe) = env::current_exe() {
            let _ = create_startup_shortcut(&path, &exe);
        }
    } else {
        let _ = fs::remove_file(path);
    }
}

fn create_startup_shortcut(path: &Path, exe: &Path) -> bool {
    unsafe { create_startup_shortcut_com(path, exe) }
}

unsafe fn create_startup_shortcut_com(path: &Path, exe: &Path) -> bool {
    let init_result = CoInitializeEx(null_mut(), COINIT_APARTMENTTHREADED);
    let should_uninitialize = init_result >= 0;
    if init_result < 0 && init_result != RPC_E_CHANGED_MODE {
        return false;
    }

    let mut shell_link = null_mut();
    let created = CoCreateInstance(
        &CLSID_SHELL_LINK,
        null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_ISHELL_LINKW,
        &mut shell_link,
    );

    let ok = if created >= 0 && !shell_link.is_null() {
        save_shell_link(shell_link.cast::<IShellLinkW>(), path, exe)
    } else {
        false
    };

    if should_uninitialize {
        CoUninitialize();
    }

    ok
}

unsafe fn save_shell_link(shell_link: *mut IShellLinkW, path: &Path, exe: &Path) -> bool {
    let exe_path = wide(&exe.to_string_lossy());
    let set_path = ((*(*shell_link).lp_vtbl).set_path)(shell_link, exe_path.as_ptr()) >= 0;
    if !set_path {
        ((*(*shell_link).lp_vtbl).release)(shell_link);
        return false;
    }

    if let Some(working_dir) = exe.parent() {
        let working_dir = wide(&working_dir.to_string_lossy());
        let _ = ((*(*shell_link).lp_vtbl).set_working_directory)(shell_link, working_dir.as_ptr());
    }

    let mut persist_file = null_mut();
    let queried = ((*(*shell_link).lp_vtbl).query_interface)(
        shell_link,
        &IID_IPERSIST_FILE,
        &mut persist_file,
    );

    let ok = if queried >= 0 && !persist_file.is_null() {
        let persist_file = persist_file.cast::<IPersistFile>();
        let shortcut_path = wide(&path.to_string_lossy());
        let saved = ((*(*persist_file).lp_vtbl).save)(persist_file, shortcut_path.as_ptr(), 1) >= 0;
        ((*(*persist_file).lp_vtbl).release)(persist_file);
        saved
    } else {
        false
    };

    ((*(*shell_link).lp_vtbl).release)(shell_link);
    ok
}
