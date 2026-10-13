use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

pub struct Case {
    root: PathBuf,
}

impl Case {
    pub fn new(suite: &str, name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("zam-{suite}-{name}-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    /// 用例目录下的路径；不是每个引入本模块的测试都用到它。
    #[allow(dead_code)]
    pub fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    pub fn run(&self, source: &str) -> Output {
        let file = self.root.join("main.zm");
        fs::write(&file, source).unwrap();
        Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("run")
            .arg(file)
            .env("ZAMAK_CACHE_DIR", self.root.join("cache"))
            .output()
            .unwrap()
    }
}

impl Drop for Case {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
