//! Native-selected file publication; renderer never owns filesystem authority.
use crate::workspace::{AppError, ErrorCode};
use std::{fs::File, path::Path};
type Result<T> = std::result::Result<T, AppError>;
#[cfg(test)]
pub(crate) fn publish(
    managed: &Path,
    selected: &Path,
    write: impl FnOnce(&mut File) -> Result<()>,
) -> Result<()> {
    publish_for_operation(managed, selected, uuid::Uuid::new_v4(), write)
}
pub(crate) fn publish_selection(
    managed: &Path,
    selected: Option<std::path::PathBuf>,
    operation: uuid::Uuid,
    write: impl FnOnce(&mut File) -> Result<()>,
) -> Result<Option<bool>> {
    let Some(selected) = selected else {
        return Ok(None);
    };
    publish_for_operation(managed, &selected, operation, write)?;
    Ok(Some(true))
}
fn publish_for_operation(
    managed: &Path,
    selected: &Path,
    operation: uuid::Uuid,
    write: impl FnOnce(&mut File) -> Result<()>,
) -> Result<()> {
    use rustix::fs::{
        AtFlags, CWD, Mode, OFlags, ResolveFlags, openat, openat2, renameat, unlinkat,
    };
    use std::os::fd::AsRawFd;
    let invalid = || AppError::new(ErrorCode::InvalidRequest);
    if !selected.is_absolute() {
        return Err(invalid());
    }
    let parent = selected.parent().ok_or_else(invalid)?;
    let name = selected.file_name().ok_or_else(invalid)?;
    if name == "." || name == ".." {
        return Err(invalid());
    }
    let dir = openat2(
        CWD,
        parent,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map_err(|_| invalid())?;
    // A held directory capability pins publication even if picker parents are renamed.
    let actual = std::fs::read_link(format!("/proc/self/fd/{}", dir.as_raw_fd()))?;
    let protected = std::fs::canonicalize(managed)?;
    if actual.starts_with(&protected) {
        return Err(AppError::new(ErrorCode::PermissionDenied));
    }
    if let Ok(metadata) = std::fs::symlink_metadata(actual.join(name))
        && (!metadata.is_file() || metadata.file_type().is_symlink())
    {
        return Err(invalid());
    }
    let staging = format!(".memoquill-{operation}.tmp");
    let fd = openat(
        &dir,
        staging.as_str(),
        OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::RUSR | Mode::WUSR,
    )?;
    let mut file = File::from(fd);
    let result = (|| {
        write(&mut file)?;
        if file.metadata()?.len() > 512 * 1024 * 1024 {
            return Err(AppError::new(ErrorCode::BudgetExceeded));
        }
        file.sync_all()?;
        rustix::fs::fsync(&dir)?;
        renameat(&dir, staging.as_str(), &dir, name)?;
        // Publication is committed. A later sync failure cannot honestly undo the rename.
        let _ = rustix::fs::fsync(&dir);
        Ok(())
    })();
    if result.is_err() {
        let _ = unlinkat(&dir, staging.as_str(), AtFlags::empty());
    }
    result
}
pub(crate) fn open_package(path: &Path) -> Result<File> {
    use rustix::fs::{CWD, Mode, OFlags, ResolveFlags, openat2};
    let invalid = || AppError::new(ErrorCode::InvalidRequest);
    if !path.is_absolute() {
        return Err(invalid());
    }
    let fd = openat2(
        CWD,
        path,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NONBLOCK,
        Mode::empty(),
        ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map_err(|_| invalid())?;
    let file = File::from(fd);
    let m = file.metadata()?;
    if !m.is_file() || m.len() > 512 * 1024 * 1024 {
        return Err(invalid());
    }
    Ok(file)
}
pub(crate) fn suggested_name(name: &str) -> String {
    let clean: String = name
        .chars()
        .take(128)
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | ':') {
                '_'
            } else {
                c
            }
        })
        .collect();
    if clean.trim_matches('.').trim().is_empty() {
        "original.bin".into()
    } else {
        clean
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn native_publication_is_atomic_and_refuses_managed_paths_and_aliases() {
        let temp = tempfile::tempdir().unwrap();
        let managed = temp.path().join("managed");
        std::fs::create_dir(&managed).unwrap();
        let out = temp.path().join("out");
        std::fs::write(&out, b"previous").unwrap();
        assert!(
            publish(&managed, &out, |f| {
                f.write_all(b"partial")?;
                Err(AppError::new(ErrorCode::StorageFailure))
            })
            .is_err()
        );
        assert_eq!(std::fs::read(&out).unwrap(), b"previous");
        publish(&managed, &out, |f| {
            f.write_all(b"new exact bytes")?;
            Ok(())
        })
        .unwrap();
        assert_eq!(std::fs::read(&out).unwrap(), b"new exact bytes");
        assert!(publish(&managed, &managed.join("catalogue.sqlite3"), |_| Ok(())).is_err());
        let alias = temp.path().join("alias");
        std::os::unix::fs::symlink(&managed, &alias).unwrap();
        assert!(publish(&managed, &alias.join("catalogue.sqlite3"), |_| Ok(())).is_err());
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 3);
    }
}

#[cfg(test)]
mod name_tests {
    use super::*;
    #[test]
    fn untrusted_suggested_names_never_supply_path_authority() {
        assert_eq!(
            suggested_name("../private\\secret:\n.pdf"),
            ".._private_secret__.pdf"
        );
        assert_eq!(suggested_name("..."), "original.bin");
        assert_eq!(suggested_name(&"x".repeat(300)).len(), 128);
    }
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    #[test]
    fn picker_cancellation_never_invokes_writer_or_touches_output() {
        let temp = tempfile::tempdir().unwrap();
        let out = temp.path().join("existing");
        std::fs::write(&out, b"unchanged").unwrap();
        assert_eq!(
            publish_selection(temp.path(), None, uuid::Uuid::new_v4(), |_| panic!(
                "cancelled writer invoked"
            ))
            .unwrap(),
            None
        );
        assert_eq!(std::fs::read(&out).unwrap(), b"unchanged");
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }
}
