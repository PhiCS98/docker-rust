use anyhow::{Context, Result};
use docker_starter_rust::DOCKER;
use flate2::read::GzDecoder;
use std::{
    fs,
    io::{Seek, SeekFrom},
    path::{self}, process::Stdio,
};
use tar::Archive;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let image = &args[2];
    let command = &args[3];
    let command_args = &args[4..];
    let image: Vec<&str> = image.split(':').collect();
    let image_name = image[0];
    let image_reference = if image.len() == 2 { image[1] } else { "latest" };
    // Create temporary directory
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path();
    // Copy the binary
    let target_command = temp_path.join(
        path::Path::new(&command)
            .to_str()
            .unwrap()
            .trim_start_matches('/'),
    );
    fs::create_dir_all(target_command.parent().unwrap())?;
    fs::copy(command, &target_command)?;
    // Create empty /dev/null
    let devnull_path = temp_dir.path().join("dev/null");
    fs::create_dir_all(devnull_path.parent().unwrap())?;
    fs::File::create(devnull_path)?;
    // Copy layers
    let manifest = DOCKER::get_manifest(image_name, image_reference)?;
    for layer in manifest.layers.iter() {
        let mut file = DOCKER::get_layer(image_name, layer)?;
        file.seek(SeekFrom::Start(0))?;
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);
        archive.unpack(temp_path)?;
    }
    // chroot into the temp dir
    std::os::unix::fs::chroot(temp_path)?;
    std::env::set_current_dir("/")?;
    // Unshare to create a new process namespace
    unsafe {
        if libc::unshare(libc::CLONE_NEWPID) != 0 {
            panic!("Failed to unshare: {}", std::io::Error::last_os_error());
        }
    };
    let output = std::process::Command::new(command)
        .args(command_args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;
    std::process::exit(output.status.code().unwrap_or(1));
}
