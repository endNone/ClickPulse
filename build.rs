use std::{env, path::PathBuf, process::Command};
fn main() {
    println!("cargo:rerun-if-changed=assets/windows/app.rc");
    println!("cargo:rerun-if-changed=assets/windows/ClickPulse.ico");
    println!("cargo:rerun-if-changed=assets/windows/ClickPulse.manifest");
    println!("cargo:rerun-if-env-changed=RC");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let gnu = env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu");
    let output = out.join(if gnu {
        "clickpulse-resource.o"
    } else {
        "clickpulse-resource.res"
    });
    let compiler = env::var_os("RC").map(PathBuf::from).unwrap_or_else(|| {
        if gnu {
            return "x86_64-w64-mingw32-windres".into();
        }
        // Native MSVC builds may run outside the VS developer command prompt.
        let kits = env::var_os("ProgramFiles(x86)")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("C:/Program Files (x86)"))
            .join("Windows Kits/10/bin");
        let mut versions = std::fs::read_dir(kits)
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.path().join("x64/rc.exe"))
            .filter(|path| path.is_file())
            .collect::<Vec<_>>();
        versions.sort();
        versions.pop().unwrap_or_else(|| "rc.exe".into())
    });
    let mut command = Command::new(&compiler);
    if gnu {
        command
            .args(["-i", "assets/windows/app.rc", "-O", "coff", "-o"])
            .arg(&output);
    } else {
        command
            .arg("/nologo")
            .arg(format!("/fo{}", output.display()))
            .arg("assets/windows/app.rc");
    }
    let status = command.status().unwrap_or_else(|error| {
        panic!(
            "Windows resource compiler {} unavailable: {error}",
            compiler.display()
        )
    });
    assert!(status.success(), "Windows icon/manifest compilation failed");
    println!("cargo:rustc-link-arg-bin=clickpulse={}", output.display());
}
