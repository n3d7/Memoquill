use super::error::Result;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    Write,
    FileSync,
    Publish,
    DirectorySync,
    BeforeCommit,
    MigrationBeforeCommit,
    AfterCommit,
}

/// No environment switches or failure callbacks exist in production builds.
#[cfg(not(test))]
pub(super) fn check(_: Phase) -> Result<()> {
    Ok(())
}

#[cfg(test)]
pub(super) use testing::{check, using};

#[cfg(test)]
mod testing {
    use super::*;
    use std::cell::RefCell;
    type Hook = Box<dyn Fn(Phase) -> Result<()>>;
    thread_local! { static HOOK:RefCell<Option<Hook>>=RefCell::new(None); }
    pub(crate) fn check(phase: Phase) -> Result<()> {
        HOOK.with(|h| h.borrow().as_ref().map_or(Ok(()), |hook| hook(phase)))
    }
    pub(crate) fn using<T>(
        hook: impl Fn(Phase) -> Result<()> + 'static,
        work: impl FnOnce() -> T,
    ) -> T {
        struct Reset(Option<Hook>);
        impl Drop for Reset {
            fn drop(&mut self) {
                HOOK.with(|h| *h.borrow_mut() = self.0.take());
            }
        }
        let _reset = Reset(HOOK.with(|h| h.borrow_mut().replace(Box::new(hook))));
        work()
    }
}
