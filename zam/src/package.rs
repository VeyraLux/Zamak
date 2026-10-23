//! 依赖包：`zam.toml` 的 `[dependencies]`、git 拉取与 `zam.lock`。
//!
//! git 依赖按 `rev`（分支、标签或提交）解析成具体提交并写进 `zam.lock`，之后的构建只看锁定的
//! 提交，源码树已经在缓存里时不再访问网络。路径依赖直接读本地目录，不进锁文件。
use crate::project::{hash, identifier, Result};
use crate::registry::{self, Index, Requirement, Version};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 依赖的来源：git 仓库、本地目录，或公共包索引里的一个版本范围（`mathx = "^1.2"`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    Git { url: String, rev: String },
    Path(PathBuf),
    Registry(Requirement),
}

/// 一个包的 `zam.toml`：名字、模块目录、入口（库可以没有）、依赖与 C 代码。
pub struct Manifest {
    pub name: String,
    pub modules: PathBuf,
    pub entry: Option<PathBuf>,
    pub dependencies: BTreeMap<String, Source>,
    pub native: Native,
    // 发布到公共包索引时要写的版本号与一句话介绍。
    pub version: Option<Version>,
    pub description: Option<String>,
}

/// `[native]`：和包一起编译的 C 源码、头文件目录、要链接的库与库目录。路径相对包的根目录；
/// `include` 与 `library_paths` 也可以是绝对路径（系统里装好的库）。
#[derive(Clone, Debug, Default)]
pub struct Native {
    pub sources: Vec<PathBuf>,
    pub include: Vec<PathBuf>,
    // C 库的头文件（`"sqlite3.h"`）：用到 extern fn 或 C 句柄的模块包含它们，按真实原型调用。
    pub headers: Vec<String>,
    pub libraries: Vec<String>,
    pub library_paths: Vec<PathBuf>,
    // 下面两项只给标准库自带的 C 代码用（std/ui 的 pkg-config 参数、macOS 的 framework、
    // Windows 的清单），zam.toml 里不能写。
    pub flags: Vec<String>,
    pub link_args: Vec<String>,
}

fn native(config: &toml::Table) -> Result<Native> {
    let Some(table) = config.get("native") else {
        return Ok(Native::default());
    };
    let table = table.as_table().ok_or("[native] must be a table")?;
    if let Some(key) = table.keys().find(|key| {
        !matches!(
            key.as_str(),
            "sources" | "include" | "headers" | "libraries" | "library_paths" | "frameworks"
        )
    }) {
        return Err(format!(
            "unknown native setting {key}; [native] accepts sources, include, headers, libraries, library_paths and frameworks"
        ));
    }
    let list = |key: &str| -> Result<Vec<String>> {
        match table.get(key) {
            None => Ok(Vec::new()),
            Some(value) => value
                .as_array()
                .and_then(|items| {
                    items
                        .iter()
                        .map(|item| item.as_str().map(str::to_owned))
                        .collect::<Option<Vec<_>>>()
                })
                .ok_or_else(|| format!("native.{key} must be a list of strings")),
        }
    };
    let mut sources = Vec::new();
    for source in list("sources")? {
        let path = relative(&source)?;
        if crate::native::Language::of(&path).is_none() {
            return Err(format!(
                "native source {source} must be a C, C++ or Objective-C file: .c, .cpp, .cc, .cxx or .m"
            ));
        }
        sources.push(path);
    }
    let libraries = list("libraries")?;
    if let Some(bad) = libraries.iter().find(|name| {
        name.is_empty()
            || name.starts_with('-')
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+'))
    }) {
        return Err(format!(
            "invalid library name {bad:?}; write the bare name, such as \"sqlite3\" for libsqlite3 or sqlite3.lib"
        ));
    }
    let headers = list("headers")?;
    if let Some(bad) = headers.iter().find(|header| {
        header.is_empty()
            || !header
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/'))
    }) {
        return Err(format!(
            "invalid header name {bad:?}; write it as in #include, such as \"sqlite3.h\" or \"SDL2/SDL.h\""
        ));
    }
    Ok(Native {
        sources,
        include: list("include")?.into_iter().map(PathBuf::from).collect(),
        headers,
        libraries,
        library_paths: list("library_paths")?
            .into_iter()
            .map(PathBuf::from)
            .collect(),
        // macOS 的 framework（`frameworks = ["Foundation"]`）：链接时写成 `-framework Foundation`，
        // 其他系统上没有 framework，忽略。
        link_args: if cfg!(target_os = "macos") {
            list("frameworks")?
                .into_iter()
                .flat_map(|name| ["-framework".to_owned(), name])
                .collect()
        } else {
            list("frameworks")?;
            Vec::new()
        },
        ..Default::default()
    })
}

/// 项目内的相对路径：不能为空，不能有 `..` 或绝对路径。
pub fn relative(path: &str) -> Result<PathBuf> {
    let path = PathBuf::from(path);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err("project paths must be nonempty relative paths without '..'".into());
    }
    Ok(path)
}

/// 读 `dir/zam.toml`（`dir` 也可以直接是清单文件）。`library` 为真时 `entry` 可以省略。
pub fn manifest(path: &Path, library: bool) -> Result<Manifest> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let config: toml::Table = text
        .parse()
        .map_err(|e| format!("{}: invalid zam.toml: {e}", path.display()))?;
    if config
        .keys()
        .any(|key| !matches!(key.as_str(), "project" | "dependencies" | "native"))
    {
        return Err("zam.toml accepts only [project], [dependencies] and [native]".into());
    }
    let table = config
        .get("project")
        .and_then(toml::Value::as_table)
        .ok_or("missing [project]")?;
    if table.keys().any(|key| {
        !matches!(
            key.as_str(),
            "name" | "entry" | "modules" | "version" | "description"
        )
    }) {
        return Err("unknown project setting".into());
    }
    let setting = |key: &str| -> Result<Option<&str>> {
        match table.get(key) {
            None => Ok(None),
            Some(value) => value
                .as_str()
                .map(Some)
                .ok_or_else(|| format!("project.{key} must be a string")),
        }
    };
    let name = setting("name")?
        .ok_or("missing string project.name")?
        .to_owned();
    if !identifier(&name) {
        return Err("project.name must be an identifier".into());
    }
    let modules = relative(setting("modules")?.ok_or("missing string project.modules")?)?;
    let entry = match setting("entry")? {
        Some(entry) => Some(relative(entry)?),
        None if library => None,
        None => return Err("missing string project.entry".into()),
    };
    let mut dependencies = BTreeMap::new();
    if let Some(table) = config.get("dependencies") {
        let table = table.as_table().ok_or("[dependencies] must be a table")?;
        for (dependency, value) in table {
            if !identifier(dependency) || dependency == "std" {
                return Err(format!("invalid dependency name {dependency}"));
            }
            // `mathx = "^1.2"`：公共包索引里的版本范围。
            if let Some(text) = value.as_str() {
                let requirement = Requirement::parse(text)
                    .map_err(|e| format!("dependency {dependency}: {e}"))?;
                dependencies.insert(dependency.clone(), Source::Registry(requirement));
                continue;
            }
            let spec = value.as_table().ok_or_else(|| {
                format!("dependency {dependency} must be written as \"^1.0\", {{ git = \"...\", rev = \"...\" }} or {{ path = \"...\" }}")
            })?;
            let text = |key: &str| -> Result<Option<String>> {
                match spec.get(key) {
                    None => Ok(None),
                    Some(value) => value
                        .as_str()
                        .map(|s| Some(s.to_owned()))
                        .ok_or_else(|| format!("dependencies.{dependency}.{key} must be a string")),
                }
            };
            if spec
                .keys()
                .any(|key| !matches!(key.as_str(), "git" | "rev" | "path" | "version"))
            {
                return Err(format!(
                    "dependency {dependency} accepts only git, rev, path and version"
                ));
            }
            let source = match (text("git")?, text("path")?, text("version")?) {
                (Some(url), None, None) => Source::Git {
                    url,
                    rev: text("rev")?.unwrap_or_else(|| "HEAD".into()),
                },
                (None, Some(path), None) if text("rev")?.is_none() => {
                    Source::Path(PathBuf::from(path))
                }
                (None, None, Some(version)) if text("rev")?.is_none() => Source::Registry(
                    Requirement::parse(&version)
                        .map_err(|e| format!("dependency {dependency}: {e}"))?,
                ),
                _ => {
                    return Err(format!(
                        "dependency {dependency} needs exactly one of version, git (with an optional rev) or path"
                    ))
                }
            };
            dependencies.insert(dependency.clone(), source);
        }
    }
    let version = setting("version")?.map(Version::parse).transpose()?;
    let description = setting("description")?.map(str::to_owned);
    Ok(Manifest {
        name,
        modules,
        entry,
        dependencies,
        native: native(&config)?,
        version,
        description,
    })
}

/// 锁文件里的一项：git 依赖（记 rev）或索引里的包（记 version）解析到的提交。
#[derive(Clone, PartialEq, Eq)]
struct Locked {
    url: String,
    rev: Option<String>,
    version: Option<String>,
    commit: String,
}

fn read_lock(path: &Path) -> Result<BTreeMap<String, Locked>> {
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(BTreeMap::new());
    };
    let table: toml::Table = text
        .parse()
        .map_err(|e| format!("{}: invalid zam.lock: {e}", path.display()))?;
    let mut locked = BTreeMap::new();
    for entry in table
        .get("package")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
    {
        let optional = |key: &str| {
            entry
                .get(key)
                .and_then(toml::Value::as_str)
                .map(str::to_owned)
        };
        let field = |key: &str| {
            optional(key).ok_or_else(|| format!("{}: package entry without {key}", path.display()))
        };
        locked.insert(
            field("name")?,
            Locked {
                url: field("git")?,
                rev: optional("rev"),
                version: optional("version"),
                commit: field("commit")?,
            },
        );
    }
    Ok(locked)
}

fn write_lock(path: &Path, locked: &BTreeMap<String, Locked>) -> Result<()> {
    let mut text = String::from(
        "# Generated by zam. Commit this file so every build uses the same dependency commits.\n",
    );
    for (name, entry) in locked {
        let (key, value) = match (&entry.version, &entry.rev) {
            (Some(version), _) => ("version", version),
            (None, Some(rev)) => ("rev", rev),
            (None, None) => unreachable!("locked entries carry a version or a rev"),
        };
        text.push_str(&format!(
            "\n[[package]]\nname = {}\n{key} = {}\ngit = {}\ncommit = {}\n",
            toml::Value::from(name.as_str()),
            toml::Value::from(value.as_str()),
            toml::Value::from(entry.url.as_str()),
            toml::Value::from(entry.commit.as_str()),
        ));
    }
    if fs::read_to_string(path).ok().as_deref() == Some(text.as_str()) {
        return Ok(());
    }
    fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn git(args: &[&str], dir: Option<&Path>) -> Result<String> {
    let mut command = Command::new("git");
    if let Some(dir) = dir {
        command.arg("-C").arg(dir);
    }
    let output = command
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| format!("git dependencies need git on PATH: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// 一个依赖包在本机的目录。
pub struct Package {
    pub dir: PathBuf,
    pub manifest: Manifest,
}

/// 索引里的包选定的版本。
#[derive(Clone, PartialEq, Eq)]
struct Choice {
    version: Version,
    url: String,
    commit: String,
}

/// 按当前选定的版本走一遍依赖树的结果。
struct Walk {
    packages: BTreeMap<String, Package>,
    locked: BTreeMap<String, Locked>,
    // 索引里的包：（谁要求的, 版本范围）。
    requirements: BTreeMap<String, Vec<(String, Requirement)>>,
    // 每个索引里的包都已经选定版本、走到了底。
    complete: bool,
}

/// 解析根项目的全部依赖（含传递依赖），需要时拉取 git 仓库、读公共包索引，并更新 `zam.lock`。
/// `update` 为真时忽略锁文件：git 依赖重新解析 rev，索引里的包重新选最高的版本。
///
/// 索引里的包按“所有依赖方的版本范围都满足的最高版本”选；锁文件里的版本仍然满足时沿用，
/// 不读索引，所以锁定后构建不访问网络。选定的版本带来新的依赖与范围时再选一轮，直到不变。
pub fn resolve(
    root: &Path,
    dependencies: &BTreeMap<String, Source>,
    store: &Path,
    update: bool,
) -> Result<BTreeMap<String, Package>> {
    let lock_path = root.join("zam.lock");
    if dependencies.is_empty() {
        if lock_path.exists() {
            fs::remove_file(&lock_path).map_err(|e| e.to_string())?;
        }
        return Ok(BTreeMap::new());
    }
    let old = if update {
        BTreeMap::new()
    } else {
        read_lock(&lock_path)?
    };
    let mut index: Option<Index> = None;
    let mut choices: BTreeMap<String, Choice> = BTreeMap::new();
    for _ in 0..64 {
        let walk = walk(root, dependencies, store, &old, &choices)?;
        let mut next = BTreeMap::new();
        for (name, requirements) in &walk.requirements {
            let accept = |version: Version| {
                requirements
                    .iter()
                    .all(|(_, requirement)| requirement.matches(version))
            };
            let kept = choices
                .get(name)
                .filter(|choice| accept(choice.version))
                .cloned()
                .or_else(|| {
                    old.get(name)
                        .and_then(|entry| {
                            let version = Version::parse(entry.version.as_deref()?).ok()?;
                            Some(Choice {
                                version,
                                url: entry.url.clone(),
                                commit: entry.commit.clone(),
                            })
                        })
                        .filter(|choice| accept(choice.version))
                });
            let choice = match kept {
                Some(choice) => choice,
                None => {
                    let index = index.get_or_insert_with(|| Index::new(store));
                    let entry = index.entry(name)?;
                    let release = entry
                        .best(accept)
                        .ok_or_else(|| registry::conflict(name, requirements, &entry))?;
                    Choice {
                        version: release.version,
                        url: entry.repository.clone(),
                        commit: release.commit.clone(),
                    }
                }
            };
            next.insert(name.clone(), choice);
        }
        if walk.complete && next == choices {
            if walk.locked.is_empty() {
                if lock_path.exists() {
                    fs::remove_file(&lock_path).map_err(|e| e.to_string())?;
                }
            } else {
                write_lock(&lock_path, &walk.locked)?;
            }
            return Ok(walk.packages);
        }
        choices = next;
    }
    Err(
        "dependency versions did not settle; pin the packages that keep changing with \"=1.2.3\""
            .into(),
    )
}

fn walk(
    root: &Path,
    dependencies: &BTreeMap<String, Source>,
    store: &Path,
    old: &BTreeMap<String, Locked>,
    choices: &BTreeMap<String, Choice>,
) -> Result<Walk> {
    let mut result = Walk {
        packages: BTreeMap::new(),
        locked: BTreeMap::new(),
        requirements: BTreeMap::new(),
        complete: true,
    };
    let mut sources: BTreeMap<String, Source> = BTreeMap::new();
    // （名字, 来源, 声明它的目录, 谁要求的）：路径依赖相对声明它的包。
    let mut pending: Vec<(String, Source, PathBuf, String)> = dependencies
        .iter()
        .map(|(name, source)| {
            (
                name.clone(),
                source.clone(),
                root.to_owned(),
                "the project".to_owned(),
            )
        })
        .collect();
    while let Some((name, source, base, by)) = pending.pop() {
        let source = match source {
            Source::Path(path) => Source::Path(
                fs::canonicalize(base.join(&path)).map_err(|_| {
                    format!(
                        "dependency {name}: directory {} not found (a path dependency inside a git package must stay inside that repository)",
                        base.join(&path).display()
                    )
                })?,
            ),
            other => other,
        };
        if let Source::Registry(requirement) = &source {
            result
                .requirements
                .entry(name.clone())
                .or_default()
                .push((by.clone(), requirement.clone()));
        }
        if let Some(existing) = sources.get(&name) {
            let same = match (existing, &source) {
                (Source::Registry(_), Source::Registry(_)) => true,
                (a, b) => a == b,
            };
            if !same {
                return Err(format!(
                    "dependency {name} is required from two different sources; use the same version, git and rev, or path everywhere"
                ));
            }
            continue;
        }
        sources.insert(name.clone(), source.clone());
        let (dir, label, chosen) = match &source {
            Source::Path(dir) => (dir.clone(), name.clone(), None),
            Source::Git { url, rev } => {
                let commit = match old.get(&name) {
                    Some(entry) if entry.url == *url && entry.rev.as_ref() == Some(rev) => {
                        entry.commit.clone()
                    }
                    _ => fetch_commit(store, url, rev)?,
                };
                let dir = checkout(store, &name, url, &commit)?;
                result.locked.insert(
                    name.clone(),
                    Locked {
                        url: url.clone(),
                        rev: Some(rev.clone()),
                        version: None,
                        commit,
                    },
                );
                (dir, name.clone(), None)
            }
            Source::Registry(_) => {
                let Some(choice) = choices.get(&name) else {
                    result.complete = false;
                    continue;
                };
                let dir = checkout(store, &name, &choice.url, &choice.commit)?;
                result.locked.insert(
                    name.clone(),
                    Locked {
                        url: choice.url.clone(),
                        rev: None,
                        version: Some(choice.version.to_string()),
                        commit: choice.commit.clone(),
                    },
                );
                (
                    dir,
                    format!("{name} {}", choice.version),
                    Some(choice.version),
                )
            }
        };
        let manifest =
            manifest(&dir.join("zam.toml"), true).map_err(|e| format!("dependency {name}: {e}"))?;
        if manifest.name != name {
            return Err(format!(
                "dependency {name} points to a package named {}; use the same name",
                manifest.name
            ));
        }
        if let (Some(chosen), Some(declared)) = (chosen, manifest.version) {
            if chosen != declared {
                return Err(format!(
                    "the package index lists {name} {chosen}, but its zam.toml at that commit says version {declared}; report it to the package author"
                ));
            }
        }
        for (dependency, source) in &manifest.dependencies {
            pending.push((
                dependency.clone(),
                source.clone(),
                dir.clone(),
                label.clone(),
            ));
        }
        result.packages.insert(name, Package { dir, manifest });
    }
    Ok(result)
}

/// 索引里某个包的最新版本（不含撤回的），`zam add <名字>` 用。
pub fn latest(store: &Path, name: &str) -> Result<Version> {
    let entry = Index::new(store).entry(name)?;
    entry
        .best(|_| true)
        .map(|release| release.version)
        .ok_or_else(|| format!("package {name} has no versions to install yet"))
}

/// 仓库的本地镜像：`store/repos/<url 的哈希>`，没有就克隆，否则拉取最新的分支与标签。
pub fn mirror(store: &Path, url: &str, refresh: bool) -> Result<PathBuf> {
    let repo = store.join("repos").join(&hash(&[url.as_bytes()])[..16]);
    if !repo.exists() {
        fs::create_dir_all(store.join("repos")).map_err(|e| e.to_string())?;
        let scratch = repo.with_extension(format!("tmp{}", std::process::id()));
        let _ = fs::remove_dir_all(&scratch);
        git(
            &[
                "clone",
                "--quiet",
                "--bare",
                url,
                scratch.to_str().ok_or("non-UTF-8 path")?,
            ],
            None,
        )?;
        if fs::rename(&scratch, &repo).is_err() {
            let _ = fs::remove_dir_all(&scratch);
        }
    } else if refresh {
        git(
            &[
                "fetch",
                "--quiet",
                "--tags",
                "--force",
                url,
                "+refs/heads/*:refs/heads/*",
            ],
            Some(&repo),
        )?;
    }
    Ok(repo)
}

/// 把 rev（分支、标签、提交，缺省是默认分支）解析成提交哈希。
fn fetch_commit(store: &Path, url: &str, rev: &str) -> Result<String> {
    let repo = mirror(store, url, true)?;
    git(
        &["rev-parse", "--verify", &format!("{rev}^{{commit}}")],
        Some(&repo),
    )
    .map_err(|_| format!("cannot find rev {rev} in {url}"))
}

/// 锁定提交的源码树：`store/src/<名字>-<提交>`，只检出一次，之后只读。
fn checkout(store: &Path, name: &str, url: &str, commit: &str) -> Result<PathBuf> {
    if commit.len() < 7 || !commit.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("zam.lock: invalid commit {commit} for {name}"));
    }
    let dir = store.join("src").join(format!("{name}-{commit}"));
    if dir.join("zam.toml").exists() {
        return Ok(dir);
    }
    let mut repo = mirror(store, url, false)?;
    if git(
        &["cat-file", "-e", &format!("{commit}^{{commit}}")],
        Some(&repo),
    )
    .is_err()
    {
        repo = mirror(store, url, true)?;
    }
    fs::create_dir_all(store.join("src")).map_err(|e| e.to_string())?;
    let scratch = dir.with_extension(format!("tmp{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);
    git(
        &[
            "clone",
            "--quiet",
            "--no-checkout",
            repo.to_str().ok_or("non-UTF-8 path")?,
            scratch.to_str().ok_or("non-UTF-8 path")?,
        ],
        None,
    )?;
    git(&["checkout", "--quiet", commit], Some(&scratch))?;
    if fs::rename(&scratch, &dir).is_err() {
        // 另一个进程先检出了同一个提交：用它的。
        let _ = fs::remove_dir_all(&scratch);
    }
    if !dir.join("zam.toml").exists() {
        return Err(format!(
            "dependency {name}: {url} has no zam.toml at {commit}"
        ));
    }
    Ok(dir)
}

/// `zam add`：在 `[dependencies]` 里加一行（没有这一节就加在文件末尾），保留其余内容。
pub fn add(manifest_path: &Path, name: &str, source: &Source) -> Result<()> {
    if !identifier(name) || name == "std" {
        return Err(format!("invalid dependency name {name}"));
    }
    let text = fs::read_to_string(manifest_path)
        .map_err(|e| format!("{}: {e}", manifest_path.display()))?;
    let current = manifest(manifest_path, true)?;
    if current.dependencies.contains_key(name) {
        return Err(format!("dependency {name} already exists in zam.toml"));
    }
    let quote = |s: &str| toml::Value::from(s).to_string();
    let line = match source {
        Source::Registry(requirement) => format!("{name} = {}", quote(&requirement.text)),
        Source::Git { url, rev } if rev == "HEAD" => format!("{name} = {{ git = {} }}", quote(url)),
        Source::Git { url, rev } => {
            format!("{name} = {{ git = {}, rev = {} }}", quote(url), quote(rev))
        }
        Source::Path(path) => format!(
            "{name} = {{ path = {} }}",
            quote(&path.to_string_lossy().replace('\\', "/"))
        ),
    };
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<&str> = text.lines().collect();
    let updated = match lines.iter().position(|l| l.trim() == "[dependencies]") {
        Some(index) => {
            lines.insert(index + 1, &line);
            lines.join(newline) + newline
        }
        None => {
            let mut text = text.trim_end().to_owned();
            text.push_str(&format!(
                "{newline}{newline}[dependencies]{newline}{line}{newline}"
            ));
            text
        }
    };
    fs::write(manifest_path, updated).map_err(|e| format!("{}: {e}", manifest_path.display()))
}

/// `zam remove`：删掉 `[dependencies]` 里这个依赖的那一行，保留其余内容。
pub fn remove(manifest_path: &Path, name: &str) -> Result<()> {
    let text = fs::read_to_string(manifest_path)
        .map_err(|e| format!("{}: {e}", manifest_path.display()))?;
    if !manifest(manifest_path, true)?
        .dependencies
        .contains_key(name)
    {
        return Err(format!("dependency {name} is not in zam.toml"));
    }
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<&str> = text.lines().collect();
    let mut section = false;
    let mut found = None;
    for (index, line) in lines.iter().enumerate() {
        let line = line.trim();
        if line.starts_with('[') {
            section = line == "[dependencies]";
        } else if section
            && line
                .strip_prefix(name)
                .is_some_and(|rest| rest.trim_start().starts_with('='))
        {
            found = Some(index);
            break;
        }
    }
    let index = found.ok_or_else(|| {
        format!(
            "cannot find the line for {name} under [dependencies]; remove it from zam.toml by hand"
        )
    })?;
    lines.remove(index);
    fs::write(manifest_path, lines.join(newline) + newline)
        .map_err(|e| format!("{}: {e}", manifest_path.display()))
}
