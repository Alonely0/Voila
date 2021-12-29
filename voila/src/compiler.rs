use flate2::read::GzDecoder;
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use tar::Archive;

// Error messages
const COMPILER_ERR_MSG: &str = "Could not compile. Make sure Cargo (a wrapper over rust's compiler) is installed, it's in your PATH and the nightly toolchain is installed:\nhttps://www.rust-lang.org/tools/install";
const TEMPDIR_ERR_MSG: &str = "Can't write to temporary directory";
const ENV_READ_ERR_MSG: &str = "Can't access current directory";
const ENV_WRITE_ERR_MSG: &str = "Can't change environment";

/// Embeds a Voila Script into a binary through
/// the `compiled_voila` crate. The source is
/// statically linked into the binary.
pub fn compile(vars: [&str; 3]) -> Result<(), &str> {
    let source = include_bytes!("../../code.tar.gz").as_ref();
    let pwd = env::current_dir().map_err(|_| ENV_READ_ERR_MSG)?;
    let target_dir = &get_target_dir();
    let mut archive = Archive::new(GzDecoder::new(source));

    // prepare target dir
    if Path::new(target_dir).exists() {
        fs::remove_dir_all(target_dir).map_err(|_| TEMPDIR_ERR_MSG)?;
    }
    fs::create_dir_all(target_dir).map_err(|_| TEMPDIR_ERR_MSG)?;
    archive.unpack(target_dir).map_err(|_| TEMPDIR_ERR_MSG)?;

    // set dir so cargo knows where to run
    env::set_current_dir(target_dir).map_err(|_| ENV_WRITE_ERR_MSG)?;

    // launch compiler & get exit code
    let compiler_exit_status = process::Command::new("cargo")
        .env("v_code", vars[0])
        .env("v_path", vars[1])
        .env("v_recursive", vars[2])
        .arg("+nightly")
        .arg("build")
        .args(["-Z", "unstable-options"])
        .arg("--release")
        .args(["--bin", "compiled_voila"])
        .arg(format!("--out-dir={p}", p = pwd.display()))
        .status()
        .map_err(|_| COMPILER_ERR_MSG)?
        .code()
        .unwrap_or(0);

    // restore to the actual current directory
    env::set_current_dir(pwd).map_err(|_| ENV_WRITE_ERR_MSG)?;

    // remove temporary files
    // fs::remove_dir_all(target_dir).map_err(|_| "Can't write to temporary directory")?;

    if compiler_exit_status > 0 {
        Err(COMPILER_ERR_MSG)
    } else {
        Ok(())
    }
}

pub fn get_target_dir() -> String {
    #[cfg(unix)]
    return "/tmp/Voila".to_string();
    #[cfg(windows)]
    return format!(
        "{p}/Voila",
        p = env::var("temp").unwrap_or_else(|_| env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap())
    );
    #[cfg(not(any(unix, windows)))]
    return format!(
        "{p}/.voila",
        p = env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
    );
}
