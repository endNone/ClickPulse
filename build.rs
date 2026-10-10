use std::{env, path::PathBuf, process::Command};

fn plist_string<'a>(contents: &'a str, key: &str) -> &'a str {
    let marker = format!("<key>{key}</key>");
    contents
        .split_once(marker.as_str())
        .and_then(|(_, value)| value.trim_start().strip_prefix("<string>"))
        .and_then(|value| value.split_once("</string>"))
        .map(|(value, _)| value)
        .unwrap_or_else(|| panic!("Missing string {key} in assets/Info.plist"))
}

fn main() {
    println!("cargo:rerun-if-changed=assets/windows/app.rc");
    println!("cargo:rerun-if-changed=assets/Info.plist");
    println!("cargo:rerun-if-changed=assets/windows/ClickPulse.ico");
    println!("cargo:rerun-if-changed=assets/windows/ClickPulse.manifest");
    println!("cargo:rerun-if-env-changed=RC");

    // Compile the package's internal build into every target. Public preview
    // versions alone cannot distinguish an old download from a fixed build.
    let resource =
        std::fs::read_to_string("assets/windows/app.rc").expect("Read Windows package version");
    let numbers = resource
        .lines()
        .find_map(|line| line.trim().strip_prefix("FILEVERSION "))
        .expect("Missing Windows FILEVERSION")
        .split(',')
        .map(|part| part.trim().parse::<u16>().expect("Invalid FILEVERSION"))
        .collect::<Vec<_>>();
    assert_eq!(numbers.len(), 4, "FILEVERSION needs four components");
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    assert_eq!(
        format!("{}.{}.{}", numbers[0], numbers[1], numbers[2]),
        version,
        "Cargo and Windows public versions must match"
    );
    let build = numbers[3].to_string();
    let plist = std::fs::read_to_string("assets/Info.plist").expect("Read macOS package version");
    assert_eq!(
        plist_string(&plist, "CFBundleShortVersionString"),
        version,
        "Cargo and macOS public versions must match"
    );
    assert_eq!(
        plist_string(&plist, "CFBundleVersion"),
        build,
        "Windows and macOS internal build numbers must match"
    );
    println!("cargo:rustc-env=CLICKPULSE_INTERNAL_BUILD={build}");

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
