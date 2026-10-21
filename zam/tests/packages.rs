use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

struct Workspace(PathBuf);
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn zam(args: &[&str], dir: &Path, cache: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_zam"))
        .args(args)
        .current_dir(dir)
        .env("ZAMAK_CACHE_DIR", cache)
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap()
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args([
            "-c",
            "user.name=zam",
            "-c",
            "user.email=zam@example.com",
            "-c",
            "init.defaultBranch=main",
            "-c",
            "core.autocrlf=false",
        ])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&status.stderr)
    );
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// 本地目录当 git 仓库用：`file://` 地址。
fn url(dir: &Path) -> String {
    let path = fs::canonicalize(dir).unwrap();
    let text = path.to_string_lossy().replace('\\', "/");
    let text = text.trim_start_matches("//?/");
    if text.starts_with('/') {
        format!("file://{text}")
    } else {
        format!("file:///{text}")
    }
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn git_and_path_dependencies_lock_update_and_work_offline() {
    let root = std::env::temp_dir().join(format!("zam-packages-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let _workspace = Workspace(root.clone());
    let cache = root.join("cache");

    // shapes：git 包，被 mathx 依赖（传递依赖）。
    let shapes = root.join("shapes");
    write(
        &shapes.join("zam.toml"),
        "[project]\nname = \"shapes\"\nmodules = \"src\"\n",
    );
    write(
        &shapes.join("src/shapes.zm"),
        "pub struct Square:\n    side: i64\n\npub fn area(s: &Square) -> i64: s.side * s.side\n\nfn test_area():\n    assert false, \"dependency tests are not run\"\n",
    );
    git(&shapes, &["init", "-q"]);
    git(&shapes, &["add", "-A"]);
    git(&shapes, &["commit", "-qm", "shapes"]);

    // mathx：根模块 src/mathx.zm（`use mathx`）与 src/geo.zm（`use mathx/geo`），包内互相 `use geo`。
    let mathx = root.join("mathx");
    write(
        &mathx.join("zam.toml"),
        &format!(
            "[project]\nname = \"mathx\"\nmodules = \"src\"\n\n[dependencies]\nshapes = {{ git = \"{}\" }}\n",
            url(&shapes)
        ),
    );
    write(
        &mathx.join("src/mathx.zm"),
        "use geo\n\npub fn double(x: i64) -> i64: geo.twice(x)\n\nfn hidden() -> i64: 1\n",
    );
    write(
        &mathx.join("src/geo.zm"),
        "use shapes\n\npub fn twice(x: i64) -> i64: x * 2\n\npub fn square_area(n: i64) -> i64:\n    let square = shapes.Square(side: n)\n    shapes.area(&square)\n",
    );
    git(&mathx, &["init", "-q"]);
    git(&mathx, &["add", "-A"]);
    git(&mathx, &["commit", "-qm", "v1"]);
    git(&mathx, &["tag", "v1"]);

    // greet：本地路径依赖。
    let greet = root.join("greet");
    write(
        &greet.join("zam.toml"),
        "[project]\nname = \"greet\"\nmodules = \"src\"\n",
    );
    write(
        &greet.join("src/greet.zm"),
        "pub fn hello(name: &string) -> string: \"hello, \" + name\n",
    );

    let app = root.join("app");
    write(
        &app.join("zam.toml"),
        "[project]\nname = \"app\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    );
    write(
        &app.join("src/main.zm"),
        "use mathx\nuse mathx/geo\nuse greet\n\nfn main():\n    println(mathx.double(21))\n    println(geo.square_area(3))\n    println(greet.hello(\"zam\"))\n\nfn test_double():\n    assert mathx.double(2) == 4\n",
    );

    // zam add：git 依赖（带 rev）与路径依赖，写进 zam.toml 并生成 zam.lock。
    let output = zam(&["add", "mathx", &url(&mathx), "v1"], &app, &cache);
    assert!(output.status.success(), "{}", stderr(&output));
    let output = zam(&["add", "greet", "../greet"], &app, &cache);
    assert!(output.status.success(), "{}", stderr(&output));
    let manifest = fs::read_to_string(app.join("zam.toml")).unwrap();
    assert!(manifest.contains("[dependencies]"), "{manifest}");
    assert!(
        manifest.contains(&format!(
            "mathx = {{ git = \"{}\", rev = \"v1\" }}",
            url(&mathx)
        )),
        "{manifest}"
    );
    assert!(
        manifest.contains("greet = { path = \"../greet\" }"),
        "{manifest}"
    );
    let lock = fs::read_to_string(app.join("zam.lock")).unwrap();
    assert!(
        lock.contains("name = \"mathx\"") && lock.contains("name = \"shapes\""),
        "{lock}"
    );
    assert!(
        !lock.contains("greet"),
        "path dependencies are not locked: {lock}"
    );

    let output = zam(&["run", "."], &app, &cache);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(stdout(&output), "42\n9\nhello, zam\n");

    // 依赖包里的测试不跑；项目自己的测试照常。
    let output = zam(&["test", "."], &app, &cache);
    assert!(output.status.success(), "{}", stdout(&output));
    assert_eq!(
        stdout(&output),
        "test test_double ... ok\n\ntest result: ok. 1 passed; 0 failed\n"
    );

    // 标签移到新提交：锁文件不变时仍用旧提交，zam update 之后用新的。
    write(
        &mathx.join("src/mathx.zm"),
        "use geo\n\npub fn double(x: i64) -> i64: geo.twice(x) + 1000\n\nfn hidden() -> i64: 1\n",
    );
    git(&mathx, &["add", "-A"]);
    git(&mathx, &["commit", "-qm", "v2"]);
    git(&mathx, &["tag", "-f", "v1"]);
    assert_eq!(
        stdout(&zam(&["run", "."], &app, &cache)),
        "42\n9\nhello, zam\n"
    );
    let output = zam(&["update", "."], &app, &cache);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_ne!(fs::read_to_string(app.join("zam.lock")).unwrap(), lock);
    assert_eq!(
        stdout(&zam(&["run", "."], &app, &cache)),
        "1042\n9\nhello, zam\n"
    );

    // 源码树在缓存里后不再需要仓库（离线）。
    fs::rename(&mathx, root.join("mathx-gone")).unwrap();
    fs::rename(&shapes, root.join("shapes-gone")).unwrap();
    assert_eq!(
        stdout(&zam(&["run", "."], &app, &cache)),
        "1042\n9\nhello, zam\n"
    );
    fs::rename(root.join("mathx-gone"), &mathx).unwrap();
    fs::rename(root.join("shapes-gone"), &shapes).unwrap();

    // 依赖里的私有函数不能用。
    write(
        &app.join("src/main.zm"),
        "use mathx\n\nfn main(): println(mathx.hidden())\n",
    );
    let output = zam(&["check", "."], &app, &cache);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("private"), "{}", stderr(&output));

    // 项目自己的模块不能和依赖同名。
    write(&app.join("src/main.zm"), "fn main(): pass\n");
    write(&app.join("src/greet.zm"), "pub fn x() -> i64: 1\n");
    let output = zam(&["check", "."], &app, &cache);
    assert!(
        stderr(&output).contains("has the same name as the dependency greet"),
        "{}",
        stderr(&output)
    );
    fs::remove_file(app.join("src/greet.zm")).unwrap();

    // zam add 失败（找不到 rev）时 zam.toml 与 zam.lock 保持原样。
    let before = (
        fs::read_to_string(app.join("zam.toml")).unwrap(),
        fs::read_to_string(app.join("zam.lock")).unwrap(),
    );
    let output = zam(&["add", "other", &url(&mathx), "no-such-tag"], &app, &cache);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("cannot find rev no-such-tag"),
        "{}",
        stderr(&output)
    );
    assert_eq!(
        before,
        (
            fs::read_to_string(app.join("zam.toml")).unwrap(),
            fs::read_to_string(app.join("zam.lock")).unwrap(),
        )
    );

    // 依赖名与包名不一致、写法不对。
    write(
        &app.join("zam.toml"),
        &format!(
            "[project]\nname = \"app\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n\n[dependencies]\nwrong = {{ git = \"{}\" }}\n",
            url(&mathx)
        ),
    );
    let output = zam(&["check", "."], &app, &cache);
    assert!(
        stderr(&output).contains("points to a package named mathx"),
        "{}",
        stderr(&output)
    );
    write(
        &app.join("zam.toml"),
        "[project]\nname = \"app\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n\n[dependencies]\nmathx = 1\n",
    );
    let output = zam(&["check", "."], &app, &cache);
    assert!(
        stderr(&output).contains("must be written as"),
        "{}",
        stderr(&output)
    );
}

fn zam_index(args: &[&str], dir: &Path, cache: &Path, index: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_zam"))
        .args(args)
        .current_dir(dir)
        .env("ZAMAK_CACHE_DIR", cache)
        .env("ZAMAK_INDEX", index)
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap()
}

/// 发布一个版本：改 zam.toml 里的版本号与根模块，提交、打标签、推到 origin，再 `zam publish`。
fn release(
    package: &Path,
    cache: &Path,
    index: &Path,
    manifest: &str,
    source: &str,
    version: &str,
) {
    write(&package.join("zam.toml"), manifest);
    let name = package.file_name().unwrap().to_str().unwrap();
    write(&package.join(format!("src/{name}.zm")), source);
    git(package, &["add", "-A"]);
    git(package, &["commit", "-qm", version]);
    git(package, &["tag", &format!("v{version}")]);
    git(package, &["push", "-q", "origin", "HEAD", "--tags"]);
    let output = zam_index(&["publish", "."], package, cache, index);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stderr(&output).contains(&format!("PUBLISH {name} {version}")));
}

/// 带远程（本地裸仓库）的空 git 仓库。
fn repository(root: &Path, name: &str) -> (PathBuf, PathBuf) {
    let dir = root.join(name);
    let remote = root.join(format!("{name}-remote.git"));
    fs::create_dir_all(&remote).unwrap();
    git(&remote, &["init", "-q", "--bare"]);
    fs::create_dir_all(&dir).unwrap();
    git(&dir, &["init", "-q"]);
    git(&dir, &["remote", "add", "origin", &url(&remote)]);
    (dir, remote)
}

#[test]
fn registry_versions_resolve_lock_update_and_conflict() {
    let root = std::env::temp_dir().join(format!("zam-registry-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let _workspace = Workspace(root.clone());
    let cache = root.join("cache");
    let index = root.join("index");
    fs::create_dir_all(index.join("packages")).unwrap();

    // mathx：库（没有 entry）。
    let (mathx, remote) = repository(&root, "mathx");
    let manifest = |version: &str| {
        format!(
            "[project]\nname = \"mathx\"\nmodules = \"src\"\nversion = \"{version}\"\ndescription = \"Small math helpers\"\n"
        )
    };
    let source = |version: &str| format!("pub fn version() -> string: \"{version}\"\n");

    // 发布前的检查：没有版本号、没提交、没打标签、标签没推送都报错。
    write(
        &mathx.join("zam.toml"),
        "[project]\nname = \"mathx\"\nmodules = \"src\"\n",
    );
    write(&mathx.join("src/mathx.zm"), &source("1.0.0"));
    git(&mathx, &["add", "-A"]);
    git(&mathx, &["commit", "-qm", "init"]);
    let publish = || zam_index(&["publish", "."], &mathx, &cache, &index);
    assert!(
        stderr(&publish()).contains("add version = \"0.1.0\""),
        "{}",
        stderr(&publish())
    );
    write(&mathx.join("zam.toml"), &manifest("1.0.0"));
    assert!(stderr(&publish()).contains("commit your changes"));
    git(&mathx, &["commit", "-qam", "version"]);
    assert!(stderr(&publish()).contains("git tag v1.0.0"));
    git(&mathx, &["tag", "v1.0.0"]);
    assert!(stderr(&publish()).contains("git push origin v1.0.0"));
    git(&mathx, &["push", "-q", "origin", "HEAD", "--tags"]);
    let output = publish();
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stderr(&publish()).contains("already published"));
    // 库可以检查，不能构建。
    assert!(zam_index(&["check", "."], &mathx, &cache, &index)
        .status
        .success());
    assert!(stderr(&zam_index(&["build", "."], &mathx, &cache, &index)).contains("is a library"));

    release(
        &mathx,
        &cache,
        &index,
        &manifest("1.1.0"),
        &source("1.1.0"),
        "1.1.0",
    );
    release(
        &mathx,
        &cache,
        &index,
        &manifest("2.0.0"),
        &source("2.0.0"),
        "2.0.0",
    );

    // geo：依赖 mathx >=1.1, <2。
    let (geo, geo_remote) = repository(&root, "geo");
    release(
        &geo,
        &cache,
        &index,
        "[project]\nname = \"geo\"\nmodules = \"src\"\nversion = \"0.1.0\"\ndescription = \"Shapes\"\n\n[dependencies]\nmathx = \">=1.1, <2\"\n",
        "use mathx\n\npub fn uses() -> string: \"geo on mathx \" + mathx.version()\n",
        "0.1.0",
    );

    let output = zam_index(&["search", "math"], &root, &cache, &index);
    assert_eq!(stdout(&output), "mathx 2.0.0  Small math helpers\n");

    let app = root.join("app");
    write(
        &app.join("zam.toml"),
        "[project]\nname = \"app\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    );
    write(
        &app.join("src/main.zm"),
        "use mathx\n\nfn main(): println(mathx.version())\n",
    );
    let run = || stdout(&zam_index(&["run", "."], &app, &cache, &index));

    // 不写版本：取最新版本，写成 ^2.0.0。
    let output = zam_index(&["add", "mathx"], &app, &cache, &index);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(fs::read_to_string(app.join("zam.toml"))
        .unwrap()
        .contains("mathx = \"^2.0.0\""));
    assert_eq!(run(), "2.0.0\n");

    // geo 要 mathx <2，与 ^2.0.0 冲突：报出双方的要求，zam.toml 恢复原样。
    let before = fs::read_to_string(app.join("zam.toml")).unwrap();
    let output = zam_index(&["add", "geo"], &app, &cache, &index);
    assert!(!output.status.success());
    let message = stderr(&output);
    for part in [
        "no version of mathx satisfies every requirement",
        "the project requires mathx ^2.0.0",
        "geo 0.1.0 requires mathx >=1.1, <2",
        "available versions: 1.0.0, 1.1.0, 2.0.0",
    ] {
        assert!(message.contains(part), "{message}");
    }
    assert_eq!(fs::read_to_string(app.join("zam.toml")).unwrap(), before);

    // 换成 ^1.0：取满足双方的最高版本 1.1.0，记进 zam.lock。
    assert!(zam_index(&["remove", "mathx"], &app, &cache, &index)
        .status
        .success());
    assert!(!app.join("zam.lock").exists());
    assert!(zam_index(&["add", "mathx", "^1.0"], &app, &cache, &index)
        .status
        .success());
    let output = zam_index(&["add", "geo"], &app, &cache, &index);
    assert!(output.status.success(), "{}", stderr(&output));
    write(
        &app.join("src/main.zm"),
        "use mathx\nuse geo\n\nfn main():\n    println(mathx.version())\n    println(geo.uses())\n",
    );
    assert_eq!(run(), "1.1.0\ngeo on mathx 1.1.0\n");
    let lock = fs::read_to_string(app.join("zam.lock")).unwrap();
    assert!(
        lock.contains("name = \"mathx\"\nversion = \"1.1.0\""),
        "{lock}"
    );

    // 新版本 1.2.0：锁文件不变时照用 1.1.0；zam update 之后用 1.2.0。
    release(
        &mathx,
        &cache,
        &index,
        &manifest("1.2.0"),
        &source("1.2.0"),
        "1.2.0",
    );
    assert_eq!(run(), "1.1.0\ngeo on mathx 1.1.0\n");
    assert!(zam_index(&["update", "."], &app, &cache, &index)
        .status
        .success());
    assert_eq!(run(), "1.2.0\ngeo on mathx 1.2.0\n");

    // 撤回 1.2.0：已锁定的照常能用，重新解析时不再选它。
    let entry = index.join("packages/mathx.toml");
    let text = fs::read_to_string(&entry).unwrap();
    let position = text.find("version = \"1.2.0\"").unwrap();
    fs::write(
        &entry,
        format!("{}yanked = true\n{}", &text[..position], &text[position..]),
    )
    .unwrap();
    assert_eq!(run(), "1.2.0\ngeo on mathx 1.2.0\n");
    assert!(zam_index(&["update", "."], &app, &cache, &index)
        .status
        .success());
    assert_eq!(run(), "1.1.0\ngeo on mathx 1.1.0\n");

    // 锁定后不需要索引和源码仓库（离线）。
    fs::rename(&index, root.join("index-gone")).unwrap();
    fs::rename(&remote, root.join("mathx-remote-gone")).unwrap();
    fs::rename(&geo_remote, root.join("geo-remote-gone")).unwrap();
    assert_eq!(run(), "1.1.0\ngeo on mathx 1.1.0\n");
}
