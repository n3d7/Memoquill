use super::{AppError, ErrorCode, error::Result};
use rustix::fs::{Mode, OFlags};
use std::fs::{DirBuilder, File, TryLockError};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};

pub(super) struct Owner {
    root: PathBuf,
    _lock: File,
}

impl Owner {
    pub(super) fn acquire(root: &Path) -> Result<Self> {
        if !root.is_absolute() {
            return Err(AppError::new(ErrorCode::InvalidRequest));
        }
        create_private_tree(root)?;
        private_directory(root)?;
        let lock = open_private_file(&root.join("workspace.lock"), OFlags::empty())?;
        lock.try_lock().map_err(lock_error)?;
        sync_directory(root)?;
        Ok(Self {
            root: root.to_owned(),
            _lock: lock,
        })
    }

    pub(super) fn root(&self) -> &Path {
        &self.root
    }
}

fn lock_error(error: TryLockError) -> AppError {
    match error {
        TryLockError::WouldBlock => AppError::new(ErrorCode::AlreadyOpen),
        TryLockError::Error(error) => error.into(),
    }
}

pub(super) fn create_private_file(path: &Path) -> Result<File> {
    open_private_file(path, OFlags::EXCL)
}
fn open_private_file(path: &Path, role: OFlags) -> Result<File> {
    let flags = OFlags::RDWR
        | OFlags::CREATE
        | OFlags::NOFOLLOW
        | OFlags::CLOEXEC
        | OFlags::NONBLOCK
        | role;
    let file = File::from(rustix::fs::open(path, flags, Mode::RUSR | Mode::WUSR)?);
    if !file.metadata()?.is_file() {
        return Err(AppError::new(ErrorCode::RecoveryRequired));
    }
    Ok(file)
}

pub(super) fn sync_directory(path: &Path) -> Result<()> {
    File::open(path)?.sync_all()?;
    Ok(())
}

pub(super) fn publish_no_replace(source: &Path, destination: &Path) -> Result<()> {
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        source,
        rustix::fs::CWD,
        destination,
        rustix::fs::RenameFlags::NOREPLACE,
    )?;
    Ok(())
}

pub(super) fn private_directory(path: &Path) -> Result<()> {
    let meta = std::fs::symlink_metadata(path)?;
    if !meta.is_dir() || meta.permissions().mode() & 0o077 != 0 {
        return Err(AppError::new(ErrorCode::RecoveryRequired));
    }
    Ok(())
}

pub(super) fn create_private_tree(path: &Path) -> Result<()> {
    if std::fs::symlink_metadata(path).is_ok() {
        return Ok(());
    }
    let parent = path
        .parent()
        .ok_or(AppError::new(ErrorCode::InvalidRequest))?;
    create_private_tree(parent)?;
    create_child(path, parent)
}
fn create_child(path: &Path, parent: &Path) -> Result<()> {
    match DirBuilder::new().mode(0o700).create(path) {
        Ok(()) => {
            sync_directory(path)?;
            sync_directory(parent)?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => private_directory(path)?,
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_special_lock_file_is_rejected_as_unsafe_workspace_state() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        create_private_tree(&root).unwrap();
        rustix::fs::mkfifoat(
            rustix::fs::CWD,
            root.join("workspace.lock"),
            Mode::RUSR | Mode::WUSR,
        )
        .unwrap();
        assert_eq!(
            Owner::acquire(&root).err().unwrap().code,
            ErrorCode::RecoveryRequired
        );
    }
    #[test]
    fn exclusive_files_preserve_existing_bytes_and_existing_private_directories_are_validated() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        create_private_tree(&root).unwrap();
        let path = root.join("original");
        std::fs::write(&path, b"preserve").unwrap();
        assert!(create_private_file(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"preserve");
        create_child(&root, temp.path()).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(
            create_child(&root, temp.path()).unwrap_err().code,
            ErrorCode::RecoveryRequired
        );
    }
}
