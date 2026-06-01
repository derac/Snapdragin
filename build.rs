use std::{env, fs, io, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=assets/icon.ico");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set"));
    let res_path = out_dir.join("Snapdragin.res");
    let rc_path = PathBuf::from("app.rc");

    let rc = find_resource_compiler().expect("Windows resource compiler rc.exe was not found");
    let status = Command::new(&rc)
        .arg("/nologo")
        .arg("/fo")
        .arg(&res_path)
        .arg(&rc_path)
        .status()
        .expect("failed to run rc.exe");

    if !status.success() {
        panic!("rc.exe failed with status {status}");
    }

    println!("cargo:rustc-link-arg-bin=Snapdragin={}", res_path.display());
}

fn find_resource_compiler() -> io::Result<PathBuf> {
    if let Ok(path) = which_in_path("rc.exe") {
        return Ok(path);
    }

    let Some(base) = env::var_os("ProgramFiles(x86)") else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "ProgramFiles(x86) is not set",
        ));
    };

    let arch = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86") => "x86",
        Ok("aarch64") => "arm64",
        _ => "x64",
    };

    let kits_bin = PathBuf::from(base)
        .join("Windows Kits")
        .join("10")
        .join("bin");
    let mut candidates = Vec::new();
    for entry in fs::read_dir(kits_bin)? {
        let entry = entry?;
        let path = entry.path().join(arch).join("rc.exe");
        if path.exists() {
            candidates.push(path);
        }
    }

    candidates.sort();
    candidates.pop().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "rc.exe was not found in Windows Kits",
        )
    })
}

fn which_in_path(name: &str) -> io::Result<PathBuf> {
    let Some(paths) = env::var_os("PATH") else {
        return Err(io::Error::new(io::ErrorKind::NotFound, "PATH is not set"));
    };

    for path in env::split_paths(&paths) {
        let candidate = path.join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("{name} was not found in PATH"),
    ))
}
