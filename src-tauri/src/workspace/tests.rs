use super::{ErrorCode, owner::Owner};
use std::os::unix::fs::{PermissionsExt, symlink};

#[test]
fn one_owner_retains_lock_until_drop() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("workspace");
    let owner = Owner::acquire(&root).unwrap();
    assert_eq!(
        Owner::acquire(&root).err().unwrap().code,
        ErrorCode::AlreadyOpen
    );
    drop(owner);
    assert!(Owner::acquire(&root).is_ok());
}

#[test]
fn managed_root_is_private_and_symlink_root_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("workspace");
    let owner = Owner::acquire(&root).unwrap();
    assert_eq!(
        std::fs::metadata(&root).unwrap().permissions().mode() & 0o777,
        0o700
    );
    let link = temp.path().join("alias");
    symlink(&root, &link).unwrap();
    assert!(Owner::acquire(&link).is_err());
    drop(owner);
}

#[test]
#[ignore = "private child process, launched by process_owner_releases_after_kill"]
fn lock_child() {
    let root = std::env::var_os("MEMOQUILL_TEST_LOCK_ROOT").unwrap();
    let _owner = Owner::acquire(std::path::Path::new(&root)).unwrap();
    std::fs::write(std::path::Path::new(&root).join("child-ready"), b"ready").unwrap();
    std::thread::sleep(std::time::Duration::from_secs(15));
}

#[test]
fn process_owner_releases_after_kill_without_unlinking_lock() {
    use std::{
        process::{Command, Stdio},
        time::{Duration, Instant},
    };
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("workspace");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "workspace::tests::lock_child"])
        .env("MEMOQUILL_TEST_LOCK_ROOT", &root)
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let start = Instant::now();
    while awaiting_child_ready(&root, start) {
        std::thread::sleep(Duration::from_millis(10));
    }
    let ready = root.join("child-ready").exists();
    let competing = Owner::acquire(&root).err().map(|e| e.code);
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(ready);
    assert_eq!(competing, Some(ErrorCode::AlreadyOpen));
    assert!(root.join("workspace.lock").exists());
    assert!(Owner::acquire(&root).is_ok());
}

#[test]
fn public_root_and_symlink_lock_do_not_change_outside_files() {
    let temp = tempfile::tempdir().unwrap();
    let public = temp.path().join("public");
    std::fs::create_dir(&public).unwrap();
    std::fs::set_permissions(&public, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(
        Owner::acquire(&public).err().unwrap().code,
        ErrorCode::RecoveryRequired
    );
    let outside = temp.path().join("outside");
    std::fs::write(&outside, b"preserve").unwrap();
    let private = temp.path().join("private");
    std::fs::DirBuilder::new()
        .recursive(true)
        .create(&private)
        .unwrap();
    std::fs::set_permissions(&private, std::fs::Permissions::from_mode(0o700)).unwrap();
    symlink(&outside, private.join("workspace.lock")).unwrap();
    assert!(Owner::acquire(&private).is_err());
    assert_eq!(std::fs::read(outside).unwrap(), b"preserve");
}

fn awaiting_child_ready(root: &std::path::Path, start: std::time::Instant) -> bool {
    !root.join("child-ready").exists() && start.elapsed() < std::time::Duration::from_secs(4)
}
