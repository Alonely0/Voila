use std::env;
use std::fs;
use std::path::Path;
use std::process;

/// Compiles with cargo nightly a crate in a given path
/// providing variables needed to run voila (e.g script's code)
/// leaving the compiled binary in the current directory.
pub fn compile(vars: [&str; 3]) -> Result<(), &str> {
    // save current dir
    let pwd = env::current_dir().unwrap();

    // set git clone path
    let target_dir = &get_target_dir();

    // delete target dir (if exists)
    if Path::new(target_dir).exists() {
        fs::remove_dir_all(target_dir).unwrap();
    }

    // clone repo
    const GIT_ERR_MSG: &str = "Could not download required files to compile the code. Make sure git is installed and in your PATH:\nhttps://git-scm.com/";
    let git_exit_status = process::Command::new("git")
        .arg("clone")
        .arg("https://github.com/Alonely0/Voila.git")
        .arg(target_dir)
        .status()
        .map_err(|_| GIT_ERR_MSG)?
        .code()
        .unwrap_or(0);

    if git_exit_status > 0 {
        return Err(GIT_ERR_MSG);
    }

    // set dir so cargo knows where to run
    env::set_current_dir(target_dir).unwrap();

    // launch compiler & get exit code
    const COMPILER_ERR_MSG: &str = "Could not compile. Make sure Cargo (a wrapper over rust's compiler) is installed, it's in your PATH and the nightly toolchain is installed:\nhttps://www.rust-lang.org/tools/install";
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
    env::set_current_dir(pwd).unwrap();

    // remove temporary files
    fs::remove_dir_all(target_dir).unwrap();

    if compiler_exit_status > 0 {
        Err(COMPILER_ERR_MSG)
    } else {
        Ok(())
    }
}

fn get_target_dir() -> String {
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
    return format!("{p}/.voila", p = env::current_dir());
}
