use crate::errors::Result;
use std::{
    fs,
    hash::Hash,
    path::{Component, Path, PathBuf},
};

pub trait HasDuplicate<T: Eq + Hash> {
    fn has_duplicate(&self) -> Option<&T>;
}

impl<T> HasDuplicate<T> for [T]
where
    T: Eq + Hash,
{
    fn has_duplicate(&self) -> Option<&T> {
        use rustc_hash::FxHashSet;

        let mut hs = FxHashSet::default();

        let mut c = 0usize;
        let mut duplicate = None;
        self.iter().any(|i| {
            hs.insert(i);
            c += 1;

            // stop processing when an insert gets missed
            let dup_found = hs.len() != c;
            if dup_found {
                duplicate = Some(i);
            }

            dup_found
        });

        duplicate
    }
}

pub trait PathPush
where
    Self: Sized,
{
    fn push_path<P: AsRef<Path>>(self, path: P) -> Self;
}

impl PathPush for PathBuf {
    fn push_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.push(path.as_ref());
        self
    }
}

pub fn copy_dir(source: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    let source = source.as_ref();
    let dest = dest.as_ref();
    assert!(dest.is_dir());
    let dest = dest.join(source.file_name().unwrap());
    assert!(source.is_dir());
    fn copy_inner(source: &Path, dest: &Path) -> Result<()> {
        fs::create_dir_all(dest)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let new_dest = dest.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                copy_inner(&entry.path(), &new_dest)?;
            } else {
                fs::copy(&entry.path(), &new_dest)?;
            }
        }
        Ok(())
    }
    copy_inner(source, &dest)
}

trait ResolveFrom {
    fn resolve_from<P: AsRef<Path>>(&self, path2: P) -> PathBuf;
}

impl ResolveFrom for &Path {
    // https://github.com/rust-lang/rfcs/issues/2208#issuecomment-342679694
    fn resolve_from<P: AsRef<Path>>(&self, path2: P) -> PathBuf {
        let path = self.join(path2);

        let mut stack: Vec<Component> = vec![];

        // We assume .components() removes redundant consecutive path separators.
        // Note that .components() also does some normalization of '.' on its own anyways.
        // This '.' normalization happens to be compatible with the approach below.
        for component in path.components() {
            match component {
                // Drop CurDir components, do not even push onto the stack.
                Component::CurDir => (),
                // For ParentDir components, we need to use the contents of the stack.
                Component::ParentDir => {
                    // Look at the top element of stack, if any.
                    let top = stack.last().cloned();
                    match top {
                        // A component is on the stack, need more pattern matching.
                        Some(c) => {
                            match c {
                                // Push the ParentDir on the stack.
                                Component::Prefix(_) => {
                                    stack.push(component);
                                }
                                // The parent of a RootDir is itself, so drop the ParentDir (no-op).
                                Component::RootDir => {}
                                // A CurDir should never be found on the stack, since they are dropped when seen.
                                Component::CurDir => {
                                    unreachable!();
                                }
                                // If a ParentDir is found, it must be due to it piling up at the start of a path.
                                // Push the new ParentDir onto the stack.
                                Component::ParentDir => {
                                    stack.push(component);
                                }
                                // If a Normal is found, pop it off.
                                Component::Normal(_) => {
                                    let _ = stack.pop();
                                }
                            }
                        }
                        // Stack is empty, so path is empty, just push.
                        None => {
                            stack.push(component);
                        }
                    }
                }
                // All others, simply push onto the stack.
                _ => {
                    stack.push(component);
                }
            }
        }

        // If an empty PathBuf would be return, instead return CurDir ('.').
        if stack.is_empty() {
            return PathBuf::from(Component::CurDir.as_os_str());
        }

        let mut norm_path = PathBuf::new();

        for item in &stack {
            norm_path.push(item.as_os_str());
        }

        norm_path
    }
}

pub trait SliceExt {
    fn trim_end(&mut self);
}

impl SliceExt for Vec<u8> {
    fn trim_end(&mut self) {
        fn is_whitespace(c: &u8) -> bool {
            *c == b'\t' || *c == b'\n' || *c == b'\r' || *c == b' '
        }

        fn is_not_whitespace(c: &u8) -> bool {
            !is_whitespace(c)
        }

        if let Some(last) = self.iter().rposition(is_not_whitespace) {
            self.truncate(last + 1);
        }
    }
}
