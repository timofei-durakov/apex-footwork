use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const APP_ICON_RESOURCE_ID: u16 = 1;

fn main() {
    println!("cargo:rerun-if-changed=apex-footwork.ico");

    if env::var_os("CARGO_CFG_WINDOWS").is_none() {
        return;
    }

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let icon_path = manifest_dir.join("apex-footwork.ico");
    if !icon_path.exists() {
        panic!("missing application icon: {}", icon_path.display());
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let rc_path = out_dir.join("apex-footwork.rc");
    let res_path = out_dir.join("apex-footwork.res");
    let out_icon_path = out_dir.join("apex-footwork.ico");

    fs::copy(&icon_path, &out_icon_path).expect("failed to copy Windows icon");
    fs::write(
        &rc_path,
        format!("{APP_ICON_RESOURCE_ID} ICON \"apex-footwork.ico\"\n"),
    )
    .expect("failed to write Windows resource file");

    let rc_exe = find_resource_compiler().expect("could not find Windows resource compiler rc.exe");
    let status = Command::new(&rc_exe)
        .current_dir(&out_dir)
        .arg("/nologo")
        .arg("/foapex-footwork.res")
        .arg("apex-footwork.rc")
        .status()
        .expect("failed to run Windows resource compiler rc.exe");

    if !status.success() {
        panic!("Windows resource compiler rc.exe failed");
    }

    println!("cargo:rustc-link-arg-bins={}", res_path.display());
}

fn find_resource_compiler() -> Option<PathBuf> {
    env::var_os("RC")
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .or_else(|| {
            env::var_os("PATH").and_then(|path| {
                env::split_paths(&path)
                    .map(|dir| dir.join("rc.exe"))
                    .find(|candidate| candidate.exists())
            })
        })
        .or_else(find_windows_kit_resource_compiler)
}

fn find_windows_kit_resource_compiler() -> Option<PathBuf> {
    let program_files_x86 = env::var_os("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"));

    let arch = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86") => "x86",
        Ok("aarch64") => "arm64",
        _ => "x64",
    };

    [r"Windows Kits\11\bin", r"Windows Kits\10\bin"]
        .into_iter()
        .map(|relative| program_files_x86.join(relative))
        .find_map(|bin_dir| find_latest_sdk_rc(&bin_dir, arch))
}

fn find_latest_sdk_rc(bin_dir: &Path, arch: &str) -> Option<PathBuf> {
    let mut candidates = fs::read_dir(bin_dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join(arch).join("rc.exe"))
        .filter(|candidate| candidate.exists())
        .collect::<Vec<_>>();

    candidates.sort();
    candidates.pop()
}
