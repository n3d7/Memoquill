use std::ffi::OsStr;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

const DISABLE_DMABUF: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";

fn needs_workaround(nvidia_loaded: bool, override_value: Option<&OsStr>) -> bool {
    nvidia_loaded && override_value.is_none()
}

pub fn bootstrap() -> Result<(), String> {
    bootstrap_for_driver(Path::new("/sys/module/nvidia").exists())
}

fn bootstrap_for_driver(nvidia_loaded: bool) -> Result<(), String> {
    if !needs_workaround(nvidia_loaded, std::env::var_os(DISABLE_DMABUF).as_deref()) {
        return Ok(());
    }
    let executable = std::env::current_exe().map_err(|error| {
        format!("Cannot locate Memoquill executable for WebKit startup: {error}")
    })?;
    reexec(&executable)
}

fn reexec(executable: &Path) -> Result<(), String> {
    // WebKit's DMA-BUF renderer can fail with NVIDIA/GBM and leave a white window.
    // Disable it before GUI initialization; this may cost rendering performance.
    // Re-exec sets only this process's environment without unsafe global mutation.
    // Any existing value is an explicit override and prevents another re-exec.
    let error = Command::new(executable)
        .args(std::env::args_os().skip(1))
        .env(DISABLE_DMABUF, "1")
        .exec();
    Err(format!(
        "Cannot re-execute Memoquill ({}) for WebKit startup: {error}",
        executable.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::process::Command;
    use std::time::{Duration, Instant};

    const CHILD: &str = "MEMOQUILL_STARTUP_TEST_CHILD";
    const PAYLOAD: &str = "MEMOQUILL_STARTUP_TEST_PAYLOAD";
    const CHILD_TEST: &str = "linux_startup::tests::startup_child";

    #[test]
    fn failed_reexec_reports_the_executable_and_error() {
        let directory = tempfile::tempdir().unwrap();
        let executable = directory.path().join("missing-executable");
        let error = reexec(&executable).expect_err("missing executable must fail clearly");
        assert!(error.contains(&executable.display().to_string()));
        assert!(error.contains("No such file"));
    }

    #[test]
    fn only_defaults_loaded_nvidia_without_an_explicit_override() {
        assert!(needs_workaround(true, None));
        assert!(!needs_workaround(false, None));
        for value in [
            OsString::from(""),
            OsString::from("0"),
            OsString::from("1"),
            OsString::from_vec(vec![0xff]),
        ] {
            assert!(!needs_workaround(true, Some(&value)));
        }
    }

    #[test]
    fn reexec_preserves_process_context_and_respects_overrides() {
        for override_value in [
            None,
            Some(OsString::from("")),
            Some(OsString::from("0")),
            Some(OsString::from_vec(vec![0xff])),
        ] {
            let directory = tempfile::tempdir().unwrap();
            let payload = OsString::from_vec(vec![b'p', 0xff]);
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
                .current_dir(directory.path())
                .env(CHILD, "1")
                .env(PAYLOAD, &payload)
                .env("MEMOQUILL_STARTUP_TEST_CWD", directory.path())
                .env_remove(DISABLE_DMABUF);
            if let Some(value) = &override_value {
                command
                    .env(DISABLE_DMABUF, value)
                    .env("MEMOQUILL_STARTUP_TEST_OVERRIDE", value);
            } else {
                command.env_remove("MEMOQUILL_STARTUP_TEST_OVERRIDE");
            }
            let mut child = command.spawn().unwrap();
            let deadline = Instant::now() + Duration::from_secs(15);
            loop {
                if let Some(status) = child.try_wait().unwrap() {
                    assert!(status.success(), "startup child failed: {status}");
                    break;
                }
                if Instant::now() >= deadline {
                    child.kill().unwrap();
                    child.wait().unwrap();
                    panic!("startup child did not exit; possible re-exec loop");
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            assert_eq!(
                std::fs::read_to_string(directory.path().join("finished")).unwrap(),
                child.id().to_string()
            );
        }
    }

    #[test]
    fn startup_child() {
        if std::env::var_os(CHILD).is_none() {
            return;
        }
        let pid = std::process::id().to_string();
        let expected_override = std::env::var_os("MEMOQUILL_STARTUP_TEST_OVERRIDE");
        if std::env::var_os(DISABLE_DMABUF).is_none() {
            std::fs::write("initial-pid", &pid).unwrap();
            bootstrap_for_driver(true).unwrap();
            panic!("NVIDIA startup without an override must re-exec");
        }
        bootstrap_for_driver(true).unwrap();
        assert_eq!(
            std::env::var_os(DISABLE_DMABUF),
            Some(expected_override.clone().unwrap_or_else(|| "1".into()))
        );
        if expected_override.is_none() {
            assert_eq!(std::fs::read_to_string("initial-pid").unwrap(), pid);
        }
        assert_eq!(
            std::env::current_dir().unwrap(),
            std::path::PathBuf::from(std::env::var_os("MEMOQUILL_STARTUP_TEST_CWD").unwrap())
        );
        assert_eq!(
            std::env::var_os(PAYLOAD),
            Some(OsString::from_vec(vec![b'p', 0xff]))
        );
        assert_eq!(
            std::env::args_os().skip(1).collect::<Vec<_>>(),
            ["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"].map(OsString::from)
        );
        std::fs::write("finished", pid).unwrap();
    }
}
