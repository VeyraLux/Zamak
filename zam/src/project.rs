use crate::compiler::{self, Block, Expr, Instruction, Module};
use crate::native;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub type Result<T> = std::result::Result<T, String>;
static TEMP_ID: AtomicU64 = AtomicU64::new(0);
const VERSION: &str = concat!("zam-", env!("CARGO_PKG_VERSION"), "-native-c-37");

pub fn hash(parts: &[&[u8]]) -> String {
    let mut digest = Sha256::new();
    for part in parts {
        digest.update((part.len() as u64).to_le_bytes());
        digest.update(part);
    }
    digest
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn cache_dir() -> PathBuf {
    env::var_os("ZAMAK_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(
                env::var_os("USERPROFILE")
                    .or_else(|| env::var_os("HOME"))
                    .unwrap_or_else(|| ".".into()),
            )
            .join(".zam/cache")
        })
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or("output has no parent directory")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temp = parent.join(format!(
        ".zam-{}-{}.tmp",
        std::process::id(),
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| e.to_string())?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        drop(file);
        match fs::hard_link(&temp, path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let existing = fs::read(path).map_err(|e| e.to_string())?;
                if existing == bytes {
                    Ok(())
                } else {
                    Err(format!("corrupt cache entry {}", path.display()))
                }
            }
            Err(e) => Err(e.to_string()),
        }
    })();
    let cleanup = fs::remove_file(&temp);
    if result.is_ok() && temp.exists() {
        cleanup.map_err(|e| e.to_string())?;
    }
    result
}

struct Input {
    source: String,
    module: Module,
    // `zam fmt` 要落盘；模块 id 与文件名不一定同名（单文件入口恒为 main）。
    path: PathBuf,
}

pub struct Project {
    pub name: String,
    pub root: PathBuf,
    entry: String,
    modules: BTreeMap<String, Input>,
}

fn relative(path: &str) -> Result<PathBuf> {
    let path = PathBuf::from(path);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("project paths must be nonempty relative paths without '..'".into());
    }
    Ok(path)
}

fn is_source(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "zm")
}

type Parse = fn(&str) -> std::result::Result<compiler::Module, String>;

fn scan(
    dir: &Path,
    base: &Path,
    modules: &mut BTreeMap<String, Input>,
    parse: Parse,
) -> Result<()> {
    for item in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let item = item.map_err(|e| e.to_string())?;
        let kind = item.file_type().map_err(|e| e.to_string())?;
        let path = item.path();
        if kind.is_symlink() {
            return Err(format!(
                "symlink source is not supported: {}",
                path.display()
            ));
        }
        if kind.is_dir() {
            scan(&path, base, modules, parse)?;
        } else if is_source(&path) {
            let id = path
                .strip_prefix(base)
                .unwrap()
                .with_extension("")
                .to_str()
                .ok_or("module path must be UTF-8")?
                .replace('\\', "/");
            if !id.split('/').all(identifier) {
                return Err(format!("invalid module path {id}"));
            }
            let source =
                fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let module = parse(&source).map_err(|e| format!("{}: {e}", path.display()))?;
            if modules
                .insert(
                    id.clone(),
                    Input {
                        source,
                        module,
                        path,
                    },
                )
                .is_some()
            {
                return Err(format!("duplicate module {id}"));
            }
        }
    }
    Ok(())
}

pub fn identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl Project {
    pub fn load(input: &Path) -> Result<Self> {
        Self::load_with(input, compiler::parse)
    }

    /// `zam fmt` 用：行首 Tab 先展开再解析，Tab 缩进的文件也能被格式化。
    pub fn load_for_format(input: &Path) -> Result<Self> {
        Self::load_with(input, |source| {
            compiler::parse(&compiler::expand_tabs(source))
        })
    }

    fn load_with(input: &Path, parse: Parse) -> Result<Self> {
        if is_source(input) {
            let source = fs::read_to_string(input).map_err(|e| e.to_string())?;
            let module = parse(&source).map_err(|e| format!("{}: {e}", input.display()))?;
            return Ok(Self {
                name: "main".into(),
                root: input.parent().unwrap_or(Path::new(".")).into(),
                entry: "main".into(),
                modules: BTreeMap::from([(
                    "main".into(),
                    Input {
                        source,
                        module,
                        path: input.to_path_buf(),
                    },
                )]),
            });
        }
        let manifest = if input.is_dir() {
            input.join("zam.toml")
        } else {
            input.into()
        };
        let text =
            fs::read_to_string(&manifest).map_err(|e| format!("{}: {e}", manifest.display()))?;
        let config: toml::Table = text.parse().map_err(|e| format!("invalid zam.toml: {e}"))?;
        if config.len() != 1 {
            return Err("zam.toml accepts only [project]".into());
        }
        let table = config
            .get("project")
            .and_then(toml::Value::as_table)
            .ok_or("missing [project]")?;
        if table
            .keys()
            .any(|key| !matches!(key.as_str(), "name" | "entry" | "modules"))
        {
            return Err("unknown project setting".into());
        }
        let setting = |key: &str| -> Result<&str> {
            table
                .get(key)
                .and_then(toml::Value::as_str)
                .ok_or_else(|| format!("missing string project.{key}"))
        };
        let name = setting("name")?.to_owned();
        if !identifier(&name) {
            return Err("project.name must be an identifier".into());
        }
        let entry_path = relative(setting("entry")?)?;
        let module_path = relative(setting("modules")?)?;
        let entry = entry_path
            .strip_prefix(&module_path)
            .map_err(|_| "entry must be inside modules")?;
        if !is_source(entry) {
            return Err("entry must be a .zm file".into());
        }
        let entry = entry
            .with_extension("")
            .to_str()
            .ok_or("entry must be UTF-8")?
            .replace('\\', "/");
        let root = manifest.parent().unwrap_or(Path::new(".")).to_owned();
        let base = root.join(module_path);
        let mut modules = BTreeMap::new();
        scan(&base, &base, &mut modules, parse)?;
        if !modules.contains_key(&entry) {
            return Err(format!("missing entry module {entry}"));
        }
        Ok(Self {
            name,
            root,
            entry,
            modules,
        })
    }

    fn resolve(&self, module: &str, call: &str) -> Result<String> {
        // `Point.origin()`：本模块结构体或枚举里声明的函数，函数表里就叫 `Point.origin`。
        let owner = call
            .split_once('.')
            .is_some_and(|(alias, _)| self.modules[module].module.structs.contains_key(alias));
        let (target, name) = if owner {
            (module, call)
        } else if let Some((alias, name)) = call.split_once('.') {
            let matches: Vec<_> = self.modules[module]
                .module
                .imports
                .iter()
                .filter(|path| path.rsplit('/').next() == Some(alias))
                .collect();
            if matches.len() != 1 {
                return Err(format!("{module}: ambiguous or missing import for {call}"));
            }
            (matches[0].as_str(), name)
        } else {
            (module, call)
        };
        if target.starts_with("std/") {
            let qualified = format!("{target}:{name}");
            return if native::builtin(&qualified).is_some() {
                Ok(qualified)
            } else {
                Err(format!("{module}: unknown function {call}"))
            };
        }
        let function = self
            .modules
            .get(target)
            .and_then(|input| input.module.functions.get(name))
            .ok_or_else(|| format!("{module}: unknown function {call}"))?;
        if target != module && !function.public {
            return Err(format!("{module}: {call} is private"));
        }
        Ok(format!("{target}:{name}"))
    }

    /// 模块在磁盘上的真实路径，供 `zam fmt` 就地重写。
    pub fn module_paths(&self) -> Vec<PathBuf> {
        self.modules
            .values()
            .map(|input| input.path.clone())
            .collect()
    }

    pub fn check(&self) -> Result<()> {
        self.modules[&self.entry]
            .module
            .functions
            .get("main")
            .ok_or("entry module requires fn main()")?;
        for (id, input) in &self.modules {
            for import in &input.module.imports {
                if !matches!(
                    import.as_str(),
                    "std/io" | "std/fmt" | "std/string" | "std/fs" | "std/env" | "std/math"
                ) && !self.modules.contains_key(import)
                {
                    return Err(format!("{id}: missing module {import}"));
                }
            }
        }
        let mut keys = BTreeMap::new();
        for id in self.modules.keys() {
            self.key(id, &mut keys, &mut BTreeSet::new())?;
        }
        compiler::check(&self.linked()?, &format!("{}:main", self.entry)).map_err(|errors| {
            errors
                .split(compiler::DIAGNOSTIC_SEPARATOR)
                .map(|error| self.located_function(error.to_owned()))
                .collect::<Vec<_>>()
                .join(&compiler::DIAGNOSTIC_SEPARATOR.to_string())
        })
    }

    fn linked(&self) -> Result<Module> {
        let mut linked = Module {
            structs: BTreeMap::new(),
            type_names: BTreeMap::new(),
            imports: Vec::new(),
            functions: BTreeMap::new(),
            constants: BTreeMap::new(),
        };
        for (id, input) in &self.modules {
            for (name, structure) in &input.module.structs {
                let mut structure = structure.clone();
                for (_, ty) in &mut structure.fields {
                    self.resolve_type(id, ty)?;
                }
                linked.structs.insert(format!("{id}:{name}"), structure);
            }
            for (name, function) in &input.module.functions {
                let mut function = function.clone();
                let position = function.position;
                for (_, ty) in &mut function.params {
                    self.resolve_type(id, ty)
                        .map_err(|error| self.located(id, position, error))?;
                }
                self.resolve_type(id, &mut function.result)
                    .map_err(|error| self.located(id, position, error))?;
                let mut locals: BTreeSet<String> = function
                    .params
                    .iter()
                    .map(|(name, _)| name.clone())
                    .collect();
                declared(&function.body, &mut locals);
                // 不支持遮蔽：局部名不能和本模块的常量同名。
                if let Some(name) = locals
                    .iter()
                    .find(|name| input.module.constants.contains_key(name.as_str()))
                {
                    return Err(self.located(
                        id,
                        position,
                        format!("{name} is already a constant; choose another name for the local"),
                    ));
                }
                self.resolve_body(id, &locals, &mut function.body)?;
                linked.functions.insert(format!("{id}:{name}"), function);
            }
        }
        linked.functions.extend(native::builtin_functions());
        Ok(linked)
    }

    /// 把类型里的结构体 id（结构体本身、它的借用、列表元素或字典值）换成按模块限定的 id。
    fn resolve_type(&self, module: &str, ty: &mut compiler::Type) -> Result<()> {
        use compiler::{Elem, Type};
        let (Type::Record(id)
        | Type::RecordRef(id, _)
        | Type::List(Elem::Record(id))
        | Type::ListRef(Elem::Record(id), _)
        | Type::Map(_, Elem::Record(id))
        | Type::MapRef(_, Elem::Record(id), _)) = ty
        else {
            return Ok(());
        };
        let name = self.modules[module]
            .module
            .type_names
            .get(id)
            .ok_or("unknown type name")?;
        let mut constructor = Expr::Record(name.clone(), Vec::new());
        self.resolve_expr(module, &BTreeSet::new(), &mut constructor)?;
        if let Expr::Record(name, _) = constructor {
            *id = compiler::record_id(&name);
        }
        Ok(())
    }

    /// 模块解析期错误此前不带位置，AI 只能从 message 里抠模块名。指令级位置已经在
    /// `Block` 里，这里直接复用；粒度与类型检查器一致（都指向语句起始），不假装更精确。
    fn located(&self, module: &str, position: (usize, usize), error: String) -> String {
        // catch 内部的语句已定位，外层表达式不能覆盖更精确的位置。
        if self
            .modules
            .get(module)
            .is_some_and(|input| error.starts_with(&format!("{}: line ", input.path.display())))
        {
            return error;
        }
        self.with_module_path(
            module,
            format!("line {}, column {}: {error}", position.0, position.1),
        )
    }

    /// `compiler::check` 的错误自带 `line L, column C`，但函数名是链接后的 `模块:函数`、
    /// 不含文件路径；项目模式下 `main.rs` 因此只能把 `file` 退化成项目目录。这里按前缀
    /// 里的模块补回真实路径，使诊断契约里的 `file` 与解析期错误保持一致。
    fn located_function(&self, error: String) -> String {
        let Some((position, rest)) = error.split_once(": ") else {
            return error;
        };
        if !position.starts_with("line ") {
            return error;
        }
        let Some((module, _)) = rest.split_once(':') else {
            return error;
        };
        let module = module.to_string();
        self.with_module_path(&module, error)
    }

    fn with_module_path(&self, module: &str, error: String) -> String {
        match self.modules.get(module) {
            Some(input) => format!("{}: {error}", input.path.display()),
            None => error,
        }
    }

    fn resolve_body(
        &self,
        module: &str,
        locals: &BTreeSet<String>,
        body: &mut Block,
    ) -> Result<()> {
        for (instruction, position) in body {
            let position = *position;
            if let Instruction::Let(_, _, Some(ty), _) = instruction {
                self.resolve_type(module, ty)
                    .map_err(|error| self.located(module, position, error))?;
            }
            if let Instruction::Assign(name, _)
            | Instruction::AssignField(name, _, _)
            | Instruction::AssignIndex(name, _, _)
            | Instruction::AssignIndexField(name, _, _, _) = instruction
            {
                if !locals.contains(name.as_str())
                    && self.modules[module]
                        .module
                        .constants
                        .contains_key(name.as_str())
                {
                    return Err(self.located(
                        module,
                        position,
                        format!("cannot assign to the constant {name}"),
                    ));
                }
            }
            let expr = match instruction {
                Instruction::Let(_, _, _, expr)
                | Instruction::Assign(_, expr)
                | Instruction::AssignField(_, _, expr)
                | Instruction::Print(expr)
                | Instruction::Call(expr)
                | Instruction::Fail(expr) => Some(expr),
                Instruction::Return(expr) => expr.as_mut(),
                Instruction::AssignIndex(_, index, expr)
                | Instruction::AssignIndexField(_, index, _, expr) => {
                    self.resolve_expr(module, locals, index)
                        .map_err(|error| self.located(module, position, error))?;
                    Some(expr)
                }
                Instruction::If(condition, yes, no) => {
                    self.resolve_body(module, locals, yes)?;
                    self.resolve_body(module, locals, no)?;
                    Some(condition)
                }
                Instruction::While(condition, nested) => {
                    self.resolve_body(module, locals, nested)?;
                    Some(condition)
                }
                Instruction::For(_, start, end, nested) => {
                    self.resolve_expr(module, locals, start)
                        .map_err(|error| self.located(module, position, error))?;
                    self.resolve_body(module, locals, nested)?;
                    Some(end)
                }
                Instruction::Each(_, _, _, _, nested) => {
                    self.resolve_body(module, locals, nested)?;
                    None
                }
                Instruction::Match(scrutinee, arms) => {
                    for (_, body) in arms {
                        self.resolve_body(module, locals, body)?;
                    }
                    Some(scrutinee)
                }
                Instruction::Break | Instruction::Continue => None,
            };
            if let Some(expr) = expr {
                self.resolve_expr(module, locals, expr)
                    .map_err(|error| self.located(module, position, error))?;
            }
        }
        Ok(())
    }

    /// 常量的用法换成它的字面量：本模块的 `LIMIT`、导入模块的 `config.LIMIT`，以及字符串常量的
    /// 借用 `&NAME`（`&string` 参数本来就接受字符串字面量）。不是常量时返回 None。
    fn constant_use(
        &self,
        module: &str,
        locals: &BTreeSet<String>,
        expr: &Expr,
    ) -> Result<Option<Expr>> {
        let (path, borrowed) = match expr {
            Expr::Variable(name) => (name.clone(), false),
            Expr::Borrow(name, false) => (name.clone(), true),
            Expr::Field(root, field) => (format!("{root}.{field}"), false),
            _ => return Ok(None),
        };
        let (target, name) = match path.split_once('.') {
            None => (module.to_owned(), path.as_str()),
            Some((alias, name)) if !name.contains('.') && !locals.contains(alias) => {
                let imports: Vec<_> = self.modules[module]
                    .module
                    .imports
                    .iter()
                    .filter(|p| p.rsplit('/').next() == Some(alias))
                    .collect();
                match imports.as_slice() {
                    [target] if self.modules.contains_key(target.as_str()) => {
                        ((*target).clone(), name)
                    }
                    _ => return Ok(None),
                }
            }
            _ => return Ok(None),
        };
        if target == module && locals.contains(name) {
            return Ok(None);
        }
        let Some(constant) = self.modules[&target].module.constants.get(name) else {
            return Ok(None);
        };
        if target != module && !constant.public {
            return Err(format!(
                "private constant {path}; declare it with pub const"
            ));
        }
        if borrowed && !matches!(constant.value, Expr::Text(_)) {
            return Err(format!(
                "cannot borrow the constant {path}; only string constants can be passed as &string"
            ));
        }
        Ok(Some(constant.value.clone()))
    }

    fn resolve_expr(&self, module: &str, locals: &BTreeSet<String>, expr: &mut Expr) -> Result<()> {
        if let Some(value) = self.constant_use(module, locals, expr)? {
            *expr = value;
            return Ok(());
        }
        match expr {
            Expr::Record(name, fields) => {
                let (target, short) = if let Some((alias, short)) = name.split_once('.') {
                    let imports: Vec<_> = self.modules[module]
                        .module
                        .imports
                        .iter()
                        .filter(|p| p.rsplit('/').next() == Some(alias))
                        .collect();
                    if imports.len() != 1 {
                        if self.modules[module]
                            .module
                            .structs
                            .get(alias)
                            .is_some_and(|s| !s.variants.is_empty())
                        {
                            return Err(format!(
                                "type mismatch: {alias}.{short} is an enum variant; pass its values by position, such as {alias}.{short}(1.0)"
                            ));
                        }
                        return Err("missing or ambiguous struct import".into());
                    }
                    (imports[0].as_str(), short)
                } else {
                    (module, name.as_str())
                };
                let declaration = self
                    .modules
                    .get(target)
                    .and_then(|m| m.module.structs.get(short))
                    .ok_or("unknown struct")?;
                if target != module && !declaration.public {
                    return Err("private struct".into());
                }
                *name = format!("{target}:{short}");
                for (_, value) in fields {
                    self.resolve_expr(module, locals, value)?;
                }
            }
            // `math.pi` 这类常量：别名指向 std/math 且不是局部变量时，直接换成字面量。
            Expr::Field(root, name)
                if !locals.contains(root.as_str())
                    && self.modules[module]
                        .module
                        .imports
                        .iter()
                        .any(|path| path == "std/math" && root == "math") =>
            {
                *expr = Expr::Float(match name.as_str() {
                    "pi" => std::f64::consts::PI,
                    "e" => std::f64::consts::E,
                    "inf" => f64::INFINITY,
                    _ => {
                        return Err(format!(
                            "unknown variable math.{name}; std/math has the constants pi, e and inf"
                        ))
                    }
                });
            }
            // `Shape.Empty`、`model.Shape.Empty`：没有载荷的枚举变体，写法和字段访问一样。
            Expr::Field(root, field)
                if !locals.contains(root.as_str())
                    && field
                        .rsplit('.')
                        .next()
                        .is_some_and(|v| v.starts_with(char::is_uppercase)) =>
            {
                let full = format!("{root}.{field}");
                let (path, variant) = full.rsplit_once('.').expect("field path");
                let mut constructor = Expr::Record(path.to_owned(), Vec::new());
                if self.resolve_expr(module, locals, &mut constructor).is_ok() {
                    if let Expr::Record(qualified, _) = constructor {
                        *expr = Expr::Variant(qualified, variant.to_owned(), Vec::new());
                    }
                }
            }
            // `f64(n)`、`i64(x)`：数值转换，不需要导入。
            Expr::Call(name, args) if matches!(name.as_str(), "f64" | "i64") => {
                *name = format!("std/core:{name}");
                for arg in args {
                    self.resolve_expr(module, locals, arg)?;
                }
            }
            // `abs`/`min`/`max`/`copy`：全局内建函数；本模块自己定义了同名函数时用自己的。
            Expr::Call(name, args)
                if matches!(name.as_str(), "abs" | "min" | "max" | "copy")
                    && self.resolve(module, name).is_err() =>
            {
                *name = format!("std/core:{name}");
                for arg in args {
                    self.resolve_expr(module, locals, arg)?;
                }
            }
            Expr::Call(name, args) => {
                if matches!(
                    name.as_str(),
                    "string.byte_len" | "string.char_len" | "string.equal"
                ) && self.modules[module]
                    .module
                    .imports
                    .iter()
                    .any(|p| p == "std/string")
                {
                    if self.modules[module]
                        .module
                        .imports
                        .iter()
                        .filter(|p| p.rsplit('/').next() == Some("string"))
                        .count()
                        != 1
                    {
                        return Err("ambiguous string import".into());
                    }
                    let equal = name == "string.equal";
                    let count = if equal { 2 } else { 1 };
                    if args.len() != count {
                        return Err(format!(
                            "{name} requires {count} borrowed string argument(s)"
                        ));
                    }
                    for arg in args.iter_mut() {
                        self.resolve_expr(module, locals, arg)?;
                    }
                    let left = Box::new(args.remove(0));
                    *expr = if equal {
                        Expr::StringEqual(left, Box::new(args.remove(0)))
                    } else if name == "string.byte_len" {
                        Expr::ByteLen(left)
                    } else {
                        Expr::CharLen(left)
                    };
                    return Ok(());
                }
                // `a.b(...)`：`a` 是本函数声明过的局部名（且不是导入别名）就是方法调用，交给检查器
                // 按接收者类型处理；否则是模块调用，拼错的模块名照旧报缺少导入。
                if let Some((receiver, method)) = name.rsplit_once('.') {
                    let root = receiver.split('.').next().unwrap_or(receiver);
                    let imported = self.modules[module]
                        .module
                        .imports
                        .iter()
                        .any(|path| path.rsplit('/').next() == Some(root));
                    if locals.contains(root)
                        && (receiver.contains('.')
                            || !(imported
                                || matches!(root, "io" | "fmt" | "string" | "fs" | "env" | "math")))
                    {
                        let mut args = std::mem::take(args);
                        for arg in &mut args {
                            self.resolve_expr(module, locals, arg)?;
                        }
                        *expr = Expr::Method(receiver.to_owned(), method.to_owned(), args);
                        return Ok(());
                    }
                }
                *name = self.resolve(module, name)?;
                for arg in args {
                    self.resolve_expr(module, locals, arg)?;
                }
            }
            Expr::Method(_, _, args) => {
                for arg in args {
                    self.resolve_expr(module, locals, arg)?;
                }
            }
            Expr::ElementMethod(_, index, _, _, args) | Expr::Chain(index, _, args) => {
                self.resolve_expr(module, locals, index)?;
                for arg in args {
                    self.resolve_expr(module, locals, arg)?;
                }
            }
            Expr::BorrowIndex(_, index, _, _) | Expr::IndexField(_, index, _) => {
                self.resolve_expr(module, locals, index)?
            }
            Expr::MapLit(entries) => {
                for (key, value) in entries {
                    self.resolve_expr(module, locals, key)?;
                    self.resolve_expr(module, locals, value)?;
                }
            }
            Expr::Format(segments) => {
                for segment in segments {
                    self.resolve_expr(module, locals, segment)?;
                }
            }
            Expr::If(condition, yes, no) => {
                self.resolve_expr(module, locals, condition)?;
                self.resolve_expr(module, locals, yes)?;
                self.resolve_expr(module, locals, no)?;
            }
            // 枚举名按结构体名解析成限定名。
            Expr::Variant(name, variant, args) => {
                let mut constructor = Expr::Record(name.clone(), Vec::new());
                match self.resolve_expr(module, locals, &mut constructor) {
                    Ok(()) => {
                        if let Expr::Record(qualified, _) = constructor {
                            *name = qualified;
                        }
                    }
                    // `model.Empty()`：没有字段的跨模块结构体，写法和无载荷的变体一样。
                    Err(error) => {
                        let mut record = Expr::Record(format!("{name}.{variant}"), Vec::new());
                        if !args.is_empty()
                            || self.resolve_expr(module, locals, &mut record).is_err()
                        {
                            return Err(error);
                        }
                        *expr = record;
                        return Ok(());
                    }
                }
                for arg in args {
                    self.resolve_expr(module, locals, arg)?;
                }
            }
            Expr::Match(scrutinee, arms) => {
                self.resolve_expr(module, locals, scrutinee)?;
                for (_, value) in arms {
                    self.resolve_expr(module, locals, value)?;
                }
            }
            Expr::Unary(_, value)
            | Expr::ByteLen(value)
            | Expr::CharLen(value)
            | Expr::Try(value) => self.resolve_expr(module, locals, value)?,
            Expr::Catch(inner, _, block) => {
                self.resolve_expr(module, locals, inner)?;
                self.resolve_body(module, locals, block)?;
            }
            Expr::Array(values) => {
                for value in values {
                    self.resolve_expr(module, locals, value)?;
                }
            }
            Expr::Index(_, index) => self.resolve_expr(module, locals, index)?,
            Expr::Binary(_, left, right) | Expr::StringEqual(left, right) => {
                self.resolve_expr(module, locals, left)?;
                self.resolve_expr(module, locals, right)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn key(
        &self,
        id: &str,
        keys: &mut BTreeMap<String, String>,
        visiting: &mut BTreeSet<String>,
    ) -> Result<String> {
        if let Some(key) = keys.get(id) {
            return Ok(key.clone());
        }
        if !visiting.insert(id.into()) {
            return Err(format!("cyclic module dependency at {id}"));
        }
        let input = &self.modules[id];
        let mut parts = vec![
            VERSION.as_bytes().to_vec(),
            env::consts::OS.as_bytes().to_vec(),
            env::consts::ARCH.as_bytes().to_vec(),
            input.source.as_bytes().to_vec(),
        ];
        for dependency in &input.module.imports {
            parts.push(dependency.as_bytes().to_vec());
            if !dependency.starts_with("std/") {
                parts.push(self.key(dependency, keys, visiting)?.into_bytes());
            }
        }
        let key = hash(&parts.iter().map(Vec::as_slice).collect::<Vec<_>>());
        visiting.remove(id);
        keys.insert(id.into(), key.clone());
        Ok(key)
    }

    pub fn build(&self, cache: &Path) -> Result<PathBuf> {
        let build_started = Instant::now();
        let check_started = Instant::now();
        self.check()?;
        let check_ms = check_started.elapsed().as_secs_f64() * 1000.0;
        let key_started = Instant::now();
        let mut keys = BTreeMap::new();
        for id in self.modules.keys() {
            self.key(id, &mut keys, &mut BTreeSet::new())?;
        }
        let key_ms = key_started.elapsed().as_secs_f64() * 1000.0;
        let linked = self.linked()?;
        for (id, input) in &self.modules {
            let bytes = compiler::encode(&input.module);
            let path = cache.join(format!("{}.zmo", keys[id]));
            let hit = match fs::read(&path) {
                Ok(existing) => {
                    if existing != bytes {
                        return Err(format!("corrupt cache entry {}", path.display()));
                    }
                    true
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    atomic_write(&path, &bytes)?;
                    false
                }
                Err(e) => return Err(e.to_string()),
            };
            // ponytail: 进度走 stderr，stdout 留给程序输出与 --json 的机器可读结果。
            eprintln!(
                "CACHE {} {:.12} {id}",
                if hit { "HIT" } else { "MISS" },
                keys[id]
            );
        }
        let object_prepare_started = Instant::now();
        let (tool, identity) = native::tool()?;
        let entry = format!("{}:main", self.entry);
        // ponytail: 两段式。先把所有模块并行「生成 C + 查缓存」（每模块 3–8ms，忽略不计），
        // 再把 MISS 交给有界工作队列编译。已命中的模块不占编译位，空闲线程立即
        // 领取下一个 MISS，不再等待整批中最慢的模块。
        // 每个 MISS 仍然在自己的 `cl` 里编 `program.c`，对象字节不变。
        // 并发度按内存封顶（每个编译器约 30–60 MB，DESIGN §8 要求构建峰值 < 200 MB），
        // 需要调就设 ZAMAK_JOBS。
        let jobs = jobs();
        let mut slots: Vec<Option<(String, Vec<u8>)>> = Vec::with_capacity(self.modules.len());
        let mut misses: Vec<(usize, String, String)> = Vec::new();
        let mut object_ms = 0.0;
        for chunk in self.modules.keys().collect::<Vec<_>>().chunks(jobs) {
            let object_started = Instant::now();
            let results = std::thread::scope(|scope| {
                let handles: Vec<_> = chunk
                    .iter()
                    .map(|id| {
                        let (linked, entry, identity) = (&linked, &entry, &identity);
                        scope.spawn(move || -> Result<(String, String, Option<Vec<u8>>)> {
                            let source = native::generate(linked, entry, id);
                            let key = hash(&[
                                VERSION.as_bytes(),
                                source.as_bytes(),
                                identity.as_bytes(),
                                native::OPTIONS.as_bytes(),
                                env::consts::OS.as_bytes(),
                                env::consts::ARCH.as_bytes(),
                            ]);
                            let bytes = cached(cache, &key, "obj")?;
                            eprintln!(
                                "OBJECT {} {:.12} {id}",
                                if bytes.is_some() { "HIT" } else { "MISS" },
                                key
                            );
                            Ok((key, source, bytes))
                        })
                    })
                    .collect();
                handles
                    .into_iter()
                    .map(|handle| {
                        handle
                            .join()
                            .unwrap_or_else(|_| Err("worker panicked".into()))
                    })
                    .collect::<Vec<Result<(String, String, Option<Vec<u8>>)>>>()
            });
            object_ms += object_started.elapsed().as_secs_f64() * 1000.0;
            for result in results {
                let (key, source, bytes) = result?;
                let slot = slots.len();
                match bytes {
                    Some(bytes) => slots.push(Some((key, bytes))),
                    None => {
                        slots.push(None);
                        misses.push((slot, key, source));
                    }
                }
            }
        }
        // 完成一个模块就领取下一个，避免每批等待最慢的编译进程。
        let next = std::sync::atomic::AtomicUsize::new(0);
        let results = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..jobs.min(misses.len()))
                .map(|_| {
                    let (misses, next, tool) = (&misses, &next, &tool);
                    scope.spawn(move || -> Result<Vec<(usize, String, Vec<u8>)>> {
                        let mut completed = Vec::new();
                        loop {
                            let index = next.fetch_add(1, Ordering::Relaxed);
                            let Some((slot, key, source)) = misses.get(index) else {
                                break;
                            };
                            let (bytes, _) = native_cached(cache, key, "obj", |scratch| {
                                native::compile(tool, scratch, source)
                            })?;
                            completed.push((*slot, key.clone(), bytes));
                        }
                        Ok(completed)
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| {
                    handle
                        .join()
                        .unwrap_or_else(|_| Err("worker panicked".into()))
                })
                .collect::<Vec<_>>()
        });
        for result in results {
            for (slot, key, bytes) in result? {
                slots[slot] = Some((key, bytes));
            }
        }
        let object_prepare_ms = object_prepare_started.elapsed().as_secs_f64() * 1000.0;
        let mut objects = Vec::with_capacity(slots.len());
        let mut object_keys = Vec::with_capacity(slots.len());
        for slot in slots {
            let (key, bytes) = slot.ok_or("native cache entry was not produced")?;
            object_keys.push(key);
            objects.push(bytes);
        }
        let app_key = hash(&[
            VERSION.as_bytes(),
            object_keys.join("\n").as_bytes(),
            identity.as_bytes(),
            native::OPTIONS.as_bytes(),
        ]);
        let link_started = Instant::now();
        let (bytes, hit) = native_cached(cache, &app_key, "exe", |scratch| {
            native::link(&tool, scratch, &objects)
        })?;
        let link_ms = link_started.elapsed().as_secs_f64() * 1000.0;
        eprintln!("LINK {} {:.12}", if hit { "HIT" } else { "MISS" }, app_key);
        eprintln!(
            "TIMING check_ms={check_ms:.3} key_ms={key_ms:.3} object_ms={object_ms:.3} object_prepare_ms={object_prepare_ms:.3} link_ms={link_ms:.3} total_ms={:.3}",
            build_started.elapsed().as_secs_f64() * 1000.0
        );
        let suffix = if cfg!(windows) { ".exe" } else { "" };
        let artifact = self
            .root
            .join("build")
            .join(format!("{}-{app_key}{suffix}", self.name));
        atomic_write(&artifact, &bytes)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&artifact, fs::Permissions::from_mode(0o755))
                .map_err(|e| e.to_string())?;
        }
        eprintln!("OUTPUT {}", artifact.display());
        Ok(artifact)
    }
}

/// 并行编译的并发度：默认取可用核数，内存封顶 8；`ZAMAK_JOBS=1` 可关掉并行。
fn jobs() -> usize {
    env::var("ZAMAK_JOBS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map_or(1, std::num::NonZeroUsize::get)
                .min(8)
        })
}

/// 只查缓存，不编译：命中要校验 sha256，字节对不上就是坏条目，缺失则返回 `None`。
fn cached(cache: &Path, key: &str, extension: &str) -> Result<Option<Vec<u8>>> {
    let path = cache.join(format!("{key}.{extension}"));
    match fs::read(&path) {
        Ok(bytes) => {
            let checksum = cache.join(format!("{key}.{extension}.sha256"));
            let expected = fs::read_to_string(&checksum)
                .map_err(|e| format!("missing native cache checksum: {e}"))?;
            if hash(&[&bytes]) != expected {
                return Err(format!("corrupt cache entry {}", path.display()));
            }
            Ok(Some(bytes))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

fn native_cached(
    cache: &Path,
    key: &str,
    extension: &str,
    build: impl FnOnce(&Path) -> Result<Vec<u8>>,
) -> Result<(Vec<u8>, bool)> {
    if let Some(bytes) = cached(cache, key, extension)? {
        return Ok((bytes, true));
    }
    let checksum = cache.join(format!("{key}.{extension}.sha256"));
    let path = cache.join(format!("{key}.{extension}"));
    let scratch = cache.join(format!(
        ".native-{}-{}",
        std::process::id(),
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&scratch).map_err(|e| e.to_string())?;
    let result = build(&scratch);
    let cleanup = fs::remove_dir_all(&scratch);
    let bytes = result?;
    cleanup.map_err(|e| e.to_string())?;
    atomic_write(&checksum, hash(&[&bytes]).as_bytes())?;
    atomic_write(&path, &bytes)?;
    Ok((bytes, false))
}

/// 函数里声明过的所有局部名（`let`、循环变量、`catch` 绑定），不论作用域；方法调用的
/// 接收者只能是其中之一，作用域与有效性由检查器负责。
fn declared(body: &Block, names: &mut BTreeSet<String>) {
    fn expr(value: &Expr, names: &mut BTreeSet<String>) {
        match value {
            Expr::Catch(inner, binding, block) => {
                names.insert(binding.clone());
                declared(block, names);
                expr(inner, names);
            }
            Expr::Call(_, values)
            | Expr::Method(_, _, values)
            | Expr::Array(values)
            | Expr::Format(values) => values.iter().for_each(|value| expr(value, names)),
            Expr::Record(_, fields) => fields.iter().for_each(|(_, value)| expr(value, names)),
            Expr::ElementMethod(_, index, _, _, values) | Expr::Chain(index, _, values) => {
                expr(index, names);
                values.iter().for_each(|value| expr(value, names));
            }
            Expr::MapLit(entries) => entries.iter().for_each(|(key, value)| {
                expr(key, names);
                expr(value, names);
            }),
            Expr::If(condition, yes, no) => {
                expr(condition, names);
                expr(yes, names);
                expr(no, names);
            }
            Expr::Variant(_, _, values) => values.iter().for_each(|value| expr(value, names)),
            Expr::Match(scrutinee, arms) => {
                expr(scrutinee, names);
                for ((_, bindings), value) in arms {
                    names.extend(bindings.iter().cloned());
                    expr(value, names);
                }
            }
            Expr::Unary(_, inner)
            | Expr::Try(inner)
            | Expr::ByteLen(inner)
            | Expr::CharLen(inner)
            | Expr::Index(_, inner)
            | Expr::IndexField(_, inner, _)
            | Expr::BorrowIndex(_, inner, _, _) => expr(inner, names),
            Expr::Binary(_, left, right) | Expr::StringEqual(left, right) => {
                expr(left, names);
                expr(right, names);
            }
            _ => {}
        }
    }
    for (instruction, _) in body {
        match instruction {
            Instruction::Let(name, _, _, value) => {
                names.insert(name.clone());
                expr(value, names);
            }
            Instruction::Assign(_, value)
            | Instruction::AssignField(_, _, value)
            | Instruction::Print(value)
            | Instruction::Call(value)
            | Instruction::Fail(value)
            | Instruction::Return(Some(value)) => expr(value, names),
            Instruction::AssignIndex(_, index, value)
            | Instruction::AssignIndexField(_, index, _, value) => {
                expr(index, names);
                expr(value, names);
            }
            Instruction::If(condition, yes, no) => {
                expr(condition, names);
                declared(yes, names);
                declared(no, names);
            }
            Instruction::While(condition, nested) => {
                expr(condition, names);
                declared(nested, names);
            }
            Instruction::For(name, start, end, nested) => {
                names.insert(name.clone());
                expr(start, names);
                expr(end, names);
                declared(nested, names);
            }
            Instruction::Each(name, second, _, _, nested) => {
                names.insert(name.clone());
                names.extend(second.iter().cloned());
                declared(nested, names);
            }
            Instruction::Match(scrutinee, arms) => {
                expr(scrutinee, names);
                for ((_, bindings), body) in arms {
                    names.extend(bindings.iter().cloned());
                    declared(body, names);
                }
            }
            Instruction::Return(None) | Instruction::Break | Instruction::Continue => {}
        }
    }
}

pub fn run_artifact(path: &Path, args: &[std::ffi::OsString]) -> Result<()> {
    let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
    let status = std::process::Command::new(path)
        .args(args)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        return Ok(());
    }
    // 与 `cargo run`、`python prog.py` 一样，原样带回程序自己的退出码，不再加一行提示。
    match status.code() {
        Some(code) => std::process::exit(code),
        None => Err(format!("program failed: {status}")),
    }
}
