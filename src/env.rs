use bstr::ByteSlice;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Change {
    Added(OsString, OsString),
    Changed(OsString, OsString, OsString),
    Removed(OsString, OsString),
}

impl Change {
    pub fn name(&self) -> OsString {
        match self {
            Added(name, _) => name,
            Changed(name, _, _) => name,
            Removed(name, _) => name,
        }
        .clone()
    }
}

pub use Change::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Diff(Vec<Change>);

impl Diff {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from(changes: &[Change]) -> Self {
        Self(changes.into())
    }

    pub fn push(&mut self, change: Change) {
        self.0.push(change);
    }

    pub fn iter(&self) -> DiffIter {
        DiffIter(self.0.iter())
    }

    pub fn exclude_by_prefix(&self, prefix: &[u8]) -> Self {
        self.exclude_by(|change| change.name().as_bytes().starts_with_str(&prefix))
    }

    pub fn exclude_by<F>(&self, func: F) -> Self
    where
        F: Fn(&Change) -> bool,
    {
        Self(
            self.0
                .iter()
                .filter(|change| !func(change))
                .cloned()
                .collect(),
        )
    }
}

pub struct DiffIter<'a>(std::slice::Iter<'a, Change>);

impl<'a> Iterator for DiffIter<'a> {
    type Item = &'a Change;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> IntoIterator for &'a Diff {
    type Item = &'a Change;
    type IntoIter = DiffIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Default for Diff {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Diff {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

pub type Item = (OsString, OsString);

pub type Env = Vec<Item>;

pub fn diff(a: &[Item], b: &[Item]) -> Diff {
    let mut diff = Diff::new();

    let veca = sorted(a);
    let vecb = sorted(b);

    let mut ia = veca.iter();
    let mut ib = vecb.iter();

    let mut enta = ia.next();
    let mut entb = ib.next();

    loop {
        match (enta, entb) {
            (Some((ka, va)), Some((kb, vb))) => match ka.cmp(kb) {
                Ordering::Less => {
                    diff.push(Removed(ka.into(), va.into()));
                    enta = ia.next();
                }
                Ordering::Greater => {
                    diff.push(Added(kb.into(), vb.into()));
                    entb = ib.next();
                }
                Ordering::Equal => {
                    if va != vb {
                        diff.push(Changed(ka.into(), va.into(), vb.into()));
                    }
                    enta = ia.next();
                    entb = ib.next();
                }
            },
            (Some((ka, va)), None) => {
                diff.push(Removed(ka.into(), va.into()));
                enta = ia.next();
            }
            (None, Some((kb, vb))) => {
                diff.push(Added(kb.into(), vb.into()));
                entb = ib.next();
            }
            (None, None) => {
                break;
            }
        }
    }

    diff
}

fn sorted<T: Ord + Clone>(v: &[T]) -> Vec<T> {
    let mut result = v.to_vec();
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_diff_empty_envs() {
        assert_eq!(Diff::new(), diff(&[], &[]));
    }

    #[test]
    fn can_diff_similar_envs() {
        let ea = env(&[("FOO", "BAR")]);
        assert_eq!(Diff::new(), diff(&ea, &ea));
    }

    #[test]
    fn can_diff_dissimilar_envs() {
        let ea = env(&[("FOO", "a"), ("ALICE", "a"), ("BOB", "c")]);
        let eb = env(&[("FOO", "a"), ("ALICE", "b"), ("CAROL", "d")]);
        assert_eq!(
            Diff::from(&[
                changed("ALICE", "a", "b"),
                removed("BOB", "c"),
                added("CAROL", "d"),
            ]),
            diff(&ea, &eb)
        );
    }

    #[test]
    fn can_diff_dissimilar_envs_with_multiple_adds() {
        let ea = env(&[]);
        let eb = env(&[("ALICE", "a"), ("BOB", "b"), ("CAROL", "c")]);
        assert_eq!(
            Diff::from(&[added("ALICE", "a"), added("BOB", "b"), added("CAROL", "c")]),
            diff(&ea, &eb)
        );
    }

    #[test]
    fn can_diff_dissimilar_envs_with_multiple_removes() {
        let ea = env(&[("ALICE", "a"), ("BOB", "b"), ("CAROL", "c")]);
        let eb = env(&[]);
        assert_eq!(
            Diff::from(&[
                removed("ALICE", "a"),
                removed("BOB", "b"),
                removed("CAROL", "c"),
            ]),
            diff(&ea, &eb)
        );
    }

    #[test]
    fn can_exclude_by_prefix() {
        let ea = env(&[
            ("ALICE", "a"),
            ("BOB", "b"),
            ("BOBBY", "bb"),
            ("CAROL", "c"),
        ]);
        let eb = env(&[("ACCRINGTON", "a"), ("BOBBINGTON", "faa")]);
        assert_eq!(
            Diff::from(&[
                added("ACCRINGTON", "a"),
                removed("ALICE", "a"),
                removed("CAROL", "c"),
            ]),
            diff(&ea, &eb).exclude_by_prefix(b"BOB")
        );
        assert_eq!(
            Diff::from(&[
                added("ACCRINGTON", "a"),
                removed("ALICE", "a"),
                removed("BOB", "b"),
                removed("CAROL", "c"),
            ]),
            diff(&ea, &eb).exclude_by_prefix(b"BOBB")
        );
    }

    fn added(key: &str, vb: &str) -> Change {
        Added(key.into(), vb.into())
    }

    fn changed(key: &str, va: &str, vb: &str) -> Change {
        Changed(key.into(), va.into(), vb.into())
    }

    fn removed(key: &str, va: &str) -> Change {
        Removed(key.into(), va.into())
    }

    fn env(items: &[(&str, &str)]) -> Env {
        items
            .iter()
            .cloned()
            .map(|(k, v)| (k.into(), v.into()))
            .collect()
    }
}
