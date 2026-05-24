//! Launch compiled offspring in a separate OS process.

use std::path::Path;
use std::process::{Child, Command, Stdio};

use super::PropagationError;

/// Spawns `binary` detached from the parent console (when the OS allows it).
pub fn spawn_offspring(
    binary: &Path,
    child_args: &[String],
) -> Result<Child, PropagationError> {
    let mut cmd = Command::new(binary);
    cmd.arg("--propagated-child");
    cmd.args(child_args);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }

    cmd.spawn().map_err(PropagationError::spawn)
}
