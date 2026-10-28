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
        let lock = private_file(&root.join("workspace.lock"), false)?;
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

pub(super) fn private_file(path: &Path, exclusive: bool) -> Result<File> {
    let mut flags = OFlags::RDWR | OFlags::CREATE | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    if exclusive {
        flags |= OFlags::EXCL;
    }
    Ok(File::from(rustix::fs::open(
        path,
        flags,
        Mode::RUSR | Mode::WUSR,
    )?))
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
