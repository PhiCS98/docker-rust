use anyhow::{Context, Result};
use std::{
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    let command_path = Path::new(command);
    let program_name = command_path.file_name().unwrap();
    let execution_dir = TempDir::new()?;
    let mut tmp_command = PathBuf::new();

    tmp_command.push(execution_dir.path());
    tmp_command.push(program_name);
    let tmp_command = tmp_command.as_path();

    std::fs::copy(command_path, tmp_command)?;

    let c_path = std::ffi::CString::new(execution_dir.path().as_os_str().as_bytes())?;
    let chroot = unsafe { libc::chroot(c_path.as_ptr()) };

    if chroot != 0 {
        std::process::exit(chroot);
    }
    // Set the directory as root to avoid problems with the chroot change
    std::env::set_current_dir("/")?;
    // Create /dev/null as is expected on the container
    std::fs::create_dir_all("/dev")?;
    std::fs::File::create("/dev/null")?;

    let mut program_path = PathBuf::new();
    program_path.push("/");
    program_path.push(program_name);

    let output = std::process::Command::new(program_path)
        .args(command_args)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    if output.status.success() {
        let _std_out = std::str::from_utf8(&output.stdout)?;
        //println!("{}", std_out);
    } else {
        std::process::exit(output.status.code().unwrap_or(1));
    }
    Ok(())
}
