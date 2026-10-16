//! 公共包索引：版本号、版本范围，以及按名字查包的索引仓库。
//!
//! 索引是一个 git 仓库（默认 [`DEFAULT_INDEX`]，可用环境变量 `ZAMAK_INDEX` 换成别的地址或本地目录），
//! 每个包一个文件 `packages/<名字>.toml`，列出各个版本对应的源码仓库提交。
use crate::package;
use crate::project::{identifier, Result};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_INDEX: &str = "https://github.com/VeyraLux/zamak-index";

/// 版本号 `主.次.修订`，三段都是不带前导零的十进制数。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(pub u64, pub u64, pub u64);

impl Version {
    pub fn parse(text: &str) -> Result<Version> {
        let parts: Vec<&str> = text.split('.').collect();
        let numbers = parts
            .iter()
            .map(|part| number(part))
            .collect::<Option<Vec<u64>>>();
        match numbers.as_deref() {
            Some([major, minor, patch]) => Ok(Version(*major, *minor, *patch)),
            _ => Err(format!(
                "invalid version {text:?}; write three numbers such as 1.2.0"
            )),
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

fn number(part: &str) -> Option<u64> {
    if part.is_empty()
        || !part.bytes().all(|b| b.is_ascii_digit())
        || (part.len() > 1 && part.starts_with('0'))
    {
        return None;
    }
    part.parse().ok()
}

/// 版本范围：一组 `>= 下界`、`< 上界` 条件，全部成立才算满足。写法同 Cargo：
/// `^1.2`（默认，不跨主版本）、`~1.2`（不跨次版本）、`=1.2.3`、`>=1.0, <2.0`、`*`。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Requirement {
    pub text: String,
    lower: Vec<Version>,
    upper: Vec<Version>,
}

impl fmt::Display for Requirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// 写了几段的版本：`1`、`1.2`、`1.2.3`。
type Partial = (u64, Option<u64>, Option<u64>);

fn floor((major, minor, patch): Partial) -> Version {
    Version(major, minor.unwrap_or(0), patch.unwrap_or(0))
}

/// 在写出的最后一段上加一：`1` → 2.0.0，`1.2` → 1.3.0，`1.2.3` → 1.2.4。
fn bump((major, minor, patch): Partial) -> Version {
    match (minor, patch) {
        (None, _) => Version(major + 1, 0, 0),
        (Some(minor), None) => Version(major, minor + 1, 0),
        (Some(minor), Some(patch)) => Version(major, minor, patch + 1),
    }
}

impl Requirement {
    pub fn parse(text: &str) -> Result<Requirement> {
        let invalid = || {
            format!(
                "invalid version requirement {text:?}; write it like \"^1.2\", \"~1.2.3\", \">=1.0, <2.0\" or \"=1.2.3\""
            )
        };
        let mut requirement = Requirement {
            text: text.trim().to_owned(),
            lower: Vec::new(),
            upper: Vec::new(),
        };
        if requirement.text.is_empty() {
            return Err(invalid());
        }
        for item in text.split(',') {
            let item = item.trim();
            if item == "*" {
                continue;
            }
            let (op, rest) = ["^", "~", ">=", "<=", ">", "<", "="]
                .iter()
                .find_map(|op| item.strip_prefix(op).map(|rest| (*op, rest.trim())))
                .unwrap_or(("^", item));
            let parts: Vec<&str> = rest.split('.').collect();
            if parts.is_empty() || parts.len() > 3 {
                return Err(invalid());
            }
            let numbers = parts
                .iter()
                .map(|part| number(part))
                .collect::<Option<Vec<u64>>>()
                .ok_or_else(invalid)?;
            let partial: Partial = (numbers[0], numbers.get(1).copied(), numbers.get(2).copied());
            let (major, minor, patch) = partial;
            let (lower, upper) = match op {
                "^" => {
                    let upper = match (major, minor, patch) {
                        (0, None, _) => Version(1, 0, 0),
                        (0, Some(0), None) => Version(0, 1, 0),
                        (0, Some(0), Some(patch)) => Version(0, 0, patch + 1),
                        (0, Some(minor), _) => Version(0, minor + 1, 0),
                        (major, _, _) => Version(major + 1, 0, 0),
                    };
                    (Some(floor(partial)), Some(upper))
                }
                "~" => {
                    let upper = match minor {
                        None => Version(major + 1, 0, 0),
                        Some(minor) => Version(major, minor + 1, 0),
                    };
                    (Some(floor(partial)), Some(upper))
                }
                "=" => (Some(floor(partial)), Some(bump(partial))),
                ">=" => (Some(floor(partial)), None),
                ">" => (Some(bump(partial)), None),
                "<" => (None, Some(floor(partial))),
                "<=" => (None, Some(bump(partial))),
                _ => unreachable!("listed operators"),
            };
            requirement.lower.extend(lower);
            requirement.upper.extend(upper);
        }
        Ok(requirement)
    }

    pub fn matches(&self, version: Version) -> bool {
        self.lower.iter().all(|bound| version >= *bound)
            && self.upper.iter().all(|bound| version < *bound)
    }
}

/// 索引里一个包的一个版本。
#[derive(Clone, Debug)]
pub struct Release {
    pub version: Version,
    pub commit: String,
    pub yanked: bool,
}

/// 索引里的一个包：`packages/<名字>.toml`。
#[derive(Clone, Debug)]
pub struct Entry {
    pub description: String,
    pub repository: String,
    pub versions: Vec<Release>,
}

impl Entry {
    pub fn parse(name: &str, text: &str) -> Result<Entry> {
        let fail = |message: String| format!("package index entry {name}: {message}");
        let table: toml::Table = text.parse().map_err(|e| fail(format!("{e}")))?;
        if let Some(key) = table.keys().find(|key| {
            !matches!(
                key.as_str(),
                "name" | "description" | "repository" | "versions"
            )
        }) {
            return Err(fail(format!("unknown key {key}")));
        }
        let text_field = |key: &str| {
            table
                .get(key)
                .and_then(toml::Value::as_str)
                .map(str::to_owned)
        };
        if text_field("name").as_deref() != Some(name) {
            return Err(fail(format!("name must be {name:?}")));
        }
        let repository = text_field("repository")
            .filter(|url| !url.is_empty())
            .ok_or_else(|| fail("missing repository".into()))?;
        let mut versions: Vec<Release> = Vec::new();
        for item in table
            .get("versions")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
        {
            let item = item
                .as_table()
                .ok_or_else(|| fail("each [[versions]] must be a table".into()))?;
            if let Some(key) = item
                .keys()
                .find(|key| !matches!(key.as_str(), "version" | "commit" | "yanked"))
            {
                return Err(fail(format!("unknown version key {key}")));
            }
            let version = Version::parse(
                item.get("version")
                    .and_then(toml::Value::as_str)
                    .ok_or_else(|| fail("a version without version".into()))?,
            )
            .map_err(fail)?;
            let commit = item
                .get("commit")
                .and_then(toml::Value::as_str)
                .filter(|commit| {
                    commit.len() == 40 && commit.chars().all(|c| c.is_ascii_hexdigit())
                })
                .ok_or_else(|| fail(format!("version {version} needs a 40-character commit")))?
                .to_owned();
            let yanked = match item.get("yanked") {
                None => false,
                Some(value) => value
                    .as_bool()
                    .ok_or_else(|| fail("yanked must be true or false".into()))?,
            };
            if versions.iter().any(|release| release.version == version) {
                return Err(fail(format!("version {version} is listed twice")));
            }
            versions.push(Release {
                version,
                commit,
                yanked,
            });
        }
        Ok(Entry {
            description: text_field("description").unwrap_or_default(),
            repository,
            versions,
        })
    }

    /// 满足条件的最高版本（不选已撤回的）。
    pub fn best(&self, accept: impl Fn(Version) -> bool) -> Option<&Release> {
        self.versions
            .iter()
            .filter(|release| !release.yanked && accept(release.version))
            .max_by_key(|release| release.version)
    }

    pub fn listing(&self) -> String {
        self.versions
            .iter()
            .map(|release| {
                if release.yanked {
                    format!("{} (yanked)", release.version)
                } else {
                    release.version.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// 索引仓库：git 地址时在缓存里保留一份镜像，第一次读的时候拉取最新内容；本地目录直接读。
pub struct Index {
    pub location: String,
    store: PathBuf,
    mirror: Option<PathBuf>,
    entries: BTreeMap<String, Entry>,
}

impl Index {
    pub fn new(store: &Path) -> Index {
        Index {
            location: std::env::var("ZAMAK_INDEX").unwrap_or_else(|_| DEFAULT_INDEX.into()),
            store: store.to_owned(),
            mirror: None,
            entries: BTreeMap::new(),
        }
    }

    /// 本地目录形式的索引（自建索引或测试）：`zam publish` 直接把条目写进去。
    pub fn local(&self) -> Option<PathBuf> {
        let path = Path::new(&self.location);
        path.is_dir().then(|| path.to_owned())
    }

    fn mirror(&mut self) -> Result<PathBuf> {
        if let Some(mirror) = &self.mirror {
            return Ok(mirror.clone());
        }
        let mirror = package::mirror(&self.store, &self.location, true)
            .map_err(|e| format!("cannot read the package index {}: {e}", self.location))?;
        self.mirror = Some(mirror.clone());
        Ok(mirror)
    }

    /// 索引里的一个文件；不存在时是 None。
    fn read(&mut self, file: &str) -> Result<Option<String>> {
        if let Some(dir) = self.local() {
            return Ok(fs::read_to_string(dir.join(file)).ok());
        }
        let mirror = self.mirror()?;
        Ok(package::git(&["show", &format!("HEAD:{file}")], Some(&mirror)).ok())
    }

    /// 条目文件的原文（`zam publish` 在后面追加新版本）。
    pub fn raw(&mut self, name: &str) -> Result<Option<String>> {
        self.read(&format!("packages/{name}.toml"))
    }

    pub fn find(&mut self, name: &str) -> Result<Option<Entry>> {
        if let Some(entry) = self.entries.get(name) {
            return Ok(Some(entry.clone()));
        }
        if !identifier(name) {
            return Ok(None);
        }
        let Some(text) = self.read(&format!("packages/{name}.toml"))? else {
            return Ok(None);
        };
        let entry = Entry::parse(name, &text)?;
        self.entries.insert(name.to_owned(), entry.clone());
        Ok(Some(entry))
    }

    pub fn entry(&mut self, name: &str) -> Result<Entry> {
        self.find(name)?.ok_or_else(|| {
            format!(
                "package {name} is not in the package index {}; look for it with zam search, or add it from git with zam add {name} <url>",
                self.location
            )
        })
    }

    /// 索引里所有包的名字。
    pub fn names(&mut self) -> Result<Vec<String>> {
        let files: Vec<String> = if let Some(dir) = self.local() {
            fs::read_dir(dir.join("packages"))
                .map(|items| {
                    items
                        .filter_map(|item| item.ok())
                        .map(|item| item.file_name().to_string_lossy().into_owned())
                        .collect()
                })
                .unwrap_or_default()
        } else {
            let mirror = self.mirror()?;
            package::git(
                &["ls-tree", "--name-only", "HEAD", "packages/"],
                Some(&mirror),
            )
            .map(|text| {
                text.lines()
                    .map(|line| line.trim_start_matches("packages/").to_owned())
                    .collect()
            })
            .unwrap_or_default()
        };
        let mut names: Vec<String> = files
            .iter()
            .filter_map(|file| file.strip_suffix(".toml"))
            .filter(|name| identifier(name))
            .map(str::to_owned)
            .collect();
        names.sort();
        Ok(names)
    }
}

/// 给人看的版本选择冲突说明。
pub fn conflict(name: &str, requirements: &[(String, Requirement)], entry: &Entry) -> String {
    let wanted = requirements
        .iter()
        .map(|(by, requirement)| format!("{by} requires {name} {requirement}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "no version of {name} satisfies every requirement: {wanted}; available versions: {}",
        if entry.versions.is_empty() {
            "none".to_owned()
        } else {
            entry.listing()
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(requirement: &str, version: &str) -> bool {
        Requirement::parse(requirement)
            .unwrap()
            .matches(Version::parse(version).unwrap())
    }

    #[test]
    fn requirements_follow_cargo_semantics() {
        for (requirement, yes, no) in [
            ("^1.2.3", &["1.2.3", "1.9.0"][..], &["1.2.2", "2.0.0"][..]),
            ("1.2", &["1.2.0", "1.99.1"], &["1.1.9", "2.0.0"]),
            ("^0.2.3", &["0.2.3", "0.2.9"], &["0.3.0", "0.2.2"]),
            ("^0.0.3", &["0.0.3"], &["0.0.4", "0.0.2"]),
            ("^0", &["0.0.0", "0.9.9"], &["1.0.0"]),
            ("~1.2.3", &["1.2.3", "1.2.9"], &["1.3.0"]),
            ("~1", &["1.0.0", "1.9.9"], &["2.0.0"]),
            ("=1.2.3", &["1.2.3"], &["1.2.4", "1.2.2"]),
            ("=1.2", &["1.2.0", "1.2.7"], &["1.3.0"]),
            (">1.2", &["1.3.0"], &["1.2.9"]),
            ("<=1.2", &["1.2.9"], &["1.3.0"]),
            (">=1.0, <2.0", &["1.0.0", "1.5.0"], &["2.0.0", "0.9.0"]),
            ("*", &["0.0.1", "9.9.9"], &[]),
        ] {
            for version in yes {
                assert!(matches(requirement, version), "{requirement} {version}");
            }
            for version in no {
                assert!(!matches(requirement, version), "{requirement} !{version}");
            }
        }
        for bad in ["", "1.2.3.4", "v1.0", "^", ">=1.x", "01.2", "1.0.0-beta"] {
            assert!(Requirement::parse(bad).is_err(), "{bad:?}");
        }
        assert!(Version::parse("1.2").is_err());
        assert!(Version::parse("1.02.0").is_err());
        assert!(Version(1, 10, 0) > Version(1, 9, 9));
    }

    #[test]
    fn index_entries_are_validated() {
        let commit = "a".repeat(40);
        let text = format!(
            "name = \"mathx\"\nrepository = \"https://example.com/mathx\"\n\n[[versions]]\nversion = \"1.0.0\"\ncommit = \"{commit}\"\n\n[[versions]]\nversion = \"1.1.0\"\ncommit = \"{commit}\"\nyanked = true\n"
        );
        let entry = Entry::parse("mathx", &text).unwrap();
        assert_eq!(entry.best(|_| true).unwrap().version, Version(1, 0, 0));
        assert_eq!(entry.listing(), "1.0.0, 1.1.0 (yanked)");
        assert!(Entry::parse("other", &text).is_err());
        assert!(Entry::parse("mathx", &text.replace(&commit, "abc")).is_err());
        assert!(Entry::parse("mathx", &text.replace("1.1.0", "1.0.0")).is_err());
    }
}
