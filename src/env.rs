use anyhow::{bail, Context, Result};
use bstr::ByteSlice;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::hash_map::HashMap;
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Change {
    Added(OsString, OsString),
    Changed(OsString, OsString, OsString),
    Removed(OsString, OsString),
}

impl Change {
    pub fn name(&self) -> &OsString {
        match self {
            Added(name, _) => name,
            Changed(name, _, _) => name,
            Removed(name, _) => name,
        }
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

    pub fn extend(&mut self, diff: Diff) {
        self.0.extend(diff.0);
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

    pub fn simplify(&mut self) {
        let mut last: HashMap<OsString, Change> = HashMap::new();
        for change in self.0.drain(0..) {
            match change {
                Added(name, vnow) => match last.remove_entry(&name) {
                    None => {
                        last.insert(name.clone(), Added(name, vnow));
                    }
                    Some((key, Added(_, _))) => {
                        last.insert(key, Added(name, vnow));
                    }
                    Some((key, Changed(_, vfirst, _))) => {
                        last.insert(key, Changed(name, vfirst, vnow));
                    }
                    Some((key, Removed(_, _))) => {
                        last.insert(key, Added(name, vnow));
                    }
                },
                Changed(name, vprev, vnow) => match last.remove_entry(&name) {
                    None => {
                        last.insert(name.clone(), Changed(name, vprev, vnow));
                    }
                    Some((key, Added(_, _))) => {
                        last.insert(key, Added(name, vnow));
                    }
                    Some((key, Changed(_, vfirst, _))) => {
                        last.insert(key, Changed(name, vfirst, vnow));
                    }
                    Some((key, Removed(_, _))) => {
                        last.insert(key, Added(name, vnow));
                    }
                },
                Removed(name, vnow) => match last.remove_entry(&name) {
                    None => {
                        last.insert(name.clone(), Removed(name, vnow));
                    }
                    Some((key, Added(_, vlast))) => {
                        last.insert(key, Removed(name, vlast));
                    }
                    Some((key, Changed(_, vfirst, _))) => {
                        last.insert(key, Removed(name, vfirst));
                    }
                    Some((key, Removed(_, vlast))) => {
                        last.insert(key, Removed(name, vlast));
                    }
                },
            }
        }
        let mut changes: Vec<_> = last.drain().collect();
        changes.sort_by(|(key1, _), (key2, _)| key1.cmp(key2));
        self.0.extend(changes.drain(0..).map(|(_, change)| change))
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
    fn can_push_into_diff() {
        let mut d = Diff::new();
        d.push(added("ALICE", "alice"));
        assert_eq!(Diff::from(&[added("ALICE", "alice"),]), d,);
    }

    #[test]
    fn can_extend_diff() {
        let mut da = Diff::from(&[added("ALICE", "alice")]);
        let db = Diff::from(&[added("BOB", "bob")]);
        da.extend(db);
        let expected = Diff::from(&[added("ALICE", "alice"), added("BOB", "bob")]);
        assert_eq!(expected, da);
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

    #[test]
    fn can_simplify_diffs_1() {
        let mut da = Diff::from(&[
            added("ALICE1", "alice1"),
            added("ALICE2", "alice2"),
            added("ALICE3", "alice3"),
        ]);
        let mut env_before = eval_diff(&da);
        let db = Diff::from(&[
            added("ALICE1", "Alice1"),
            changed("ALICE2", "Alice2-before", "Alice2"),
            removed("ALICE3", "Alice3"),
        ]);
        eval_diff_into(&db, &mut env_before);
        da.extend(db);
        da.simplify();
        let env_after = eval_diff(&da);
        assert_eq!(
            Diff::from(&[
                added("ALICE1", "Alice1"),
                added("ALICE2", "Alice2"),
                removed("ALICE3", "alice3"),
            ]),
            da,
        );
        // The resulting environment is the same.
        assert_eq!(env_before, env_after);
    }

    #[test]
    fn can_simplify_diffs_2() {
        let mut da = Diff::from(&[
            changed("CAROL1", "carol1-before", "carol1"),
            changed("CAROL2", "carol2-before", "carol2"),
            changed("CAROL3", "carol3-before", "carol3"),
        ]);
        let mut env_before = eval_diff(&da);
        let db = Diff::from(&[
            added("CAROL1", "Carol1"),
            changed("CAROL2", "Carol2-before", "Carol2"),
            removed("CAROL3", "Carol3"),
        ]);
        eval_diff_into(&db, &mut env_before);
        da.extend(db);
        da.simplify();
        let env_after = eval_diff(&da);
        assert_eq!(
            Diff::from(&[
                changed("CAROL1", "carol1-before", "Carol1"),
                changed("CAROL2", "carol2-before", "Carol2"),
                removed("CAROL3", "carol3-before"),
            ]),
            da,
        );
        // The resulting environment is the same.
        assert_eq!(env_before, env_after);
    }

    #[test]
    fn can_simplify_diffs_3() {
        let mut da = Diff::from(&[
            removed("ROGER1", "roger1"),
            removed("ROGER2", "roger2"),
            removed("ROGER3", "roger3"),
        ]);
        let mut env_before = eval_diff(&da);
        let db = Diff::from(&[
            added("ROGER1", "Roger1"),
            changed("ROGER2", "Roger2-before", "Roger2"),
            removed("ROGER3", "Roger3"),
        ]);
        eval_diff_into(&db, &mut env_before);
        da.extend(db);
        da.simplify();
        let env_after = eval_diff(&da);
        assert_eq!(
            Diff::from(&[
                added("ROGER1", "Roger1"),
                added("ROGER2", "Roger2"),
                removed("ROGER3", "roger3"),
            ]),
            da,
        );
        // The resulting environment is the same.
        assert_eq!(env_before, env_after);
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

    fn eval_diff(diff: &Diff) -> HashMap<OsString, Option<OsString>> {
        let mut env = HashMap::new();
        eval_diff_into(diff, &mut env);
        env
    }

    fn eval_diff_into(diff: &Diff, env: &mut HashMap<OsString, Option<OsString>>) {
        for change in diff {
            match change {
                Added(name, value) => env.insert(name.clone(), Some(value.clone())),
                Changed(name, _, value) => env.insert(name.clone(), Some(value.clone())),
                Removed(name, _) => env.insert(name.clone(), None),
            };
        }
    }
}

pub fn capture(dump_path: &Path, mut dump_cmd: Command) -> Result<Env> {
    log::debug!("{:?}", dump_cmd);
    let mut dump_proc = dump_cmd
        .spawn()
        .context("could not spawn dumping command")?;
    if !dump_proc
        .wait()
        .context("could not wait for dumping command")?
        .success()
    {
        bail!("failed to capture environment")
    }

    match bincode::deserialize(
        &fs::read(dump_path).context("could not read dumped environment file")?,
    ) {
        Ok(env) => Ok(env),
        err => err.context("could not deserialize dumped environment"),
    }
}
