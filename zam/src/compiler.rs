use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Word(String),
    Text(String),
    Number(String),
    // 已去掉分隔符、确认有限的浮点字面量文本，如 `1.5`、`2e-3`。
    Float(String),
    Symbol(char),
    Newline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Owned,
    Shared,
    Mutable,
    Unit,
    Int,
    Float,
    Bool,
    Record(u64),
    // 结构体的调用帧借用 `&T` / `&mut T`。
    RecordRef(u64, bool),
    ArrayInt(usize),
    // 可增长数组 `[T]`（owned）与它的调用帧借用 `&[T]` / `&mut [T]`。
    List(Elem),
    ListRef(Elem, bool),
    // 字典 `{K: V}`（owned）与它的调用帧借用；键只能是 i64 或 string。
    Map(Elem, Elem),
    MapRef(Elem, Elem, bool),
}
impl Type {
    /// `[T]`、`{K: V}` 或它们的借用 → 背后的 owned 容器类型。
    pub fn container(self) -> Option<Type> {
        match self {
            Type::List(elem) | Type::ListRef(elem, _) => Some(Type::List(elem)),
            Type::Map(key, value) | Type::MapRef(key, value, _) => Some(Type::Map(key, value)),
            _ => None,
        }
    }
    pub fn is_reference(self) -> bool {
        matches!(
            self,
            Type::Shared
                | Type::Mutable
                | Type::ListRef(_, _)
                | Type::MapRef(_, _, _)
                | Type::RecordRef(_, _)
        )
    }
    /// 可被借用的值：（被借用的 owned 类型，本身已是借用时是否可写）。标量不可借用。
    pub fn borrowable(self) -> Option<(Type, Option<bool>)> {
        match self {
            Type::Owned => Some((Type::Owned, None)),
            Type::Shared => Some((Type::Owned, Some(false))),
            Type::Mutable => Some((Type::Owned, Some(true))),
            Type::Record(id) => Some((Type::Record(id), None)),
            Type::RecordRef(id, mutable) => Some((Type::Record(id), Some(mutable))),
            Type::List(elem) => Some((Type::List(elem), None)),
            Type::ListRef(elem, mutable) => Some((Type::List(elem), Some(mutable))),
            Type::Map(key, value) => Some((Type::Map(key, value), None)),
            Type::MapRef(key, value, mutable) => Some((Type::Map(key, value), Some(mutable))),
            _ => None,
        }
    }
}

/// 借用一个 owned 值得到的借用类型；标量没有借用。
pub fn borrow_of(ty: Type, mutable: bool) -> Option<Type> {
    Some(match ty {
        Type::Owned if mutable => Type::Mutable,
        Type::Owned => Type::Shared,
        Type::Record(id) => Type::RecordRef(id, mutable),
        Type::List(elem) => Type::ListRef(elem, mutable),
        Type::Map(key, value) => Type::MapRef(key, value, mutable),
        _ => return None,
    })
}

/// 容器方法的签名：(是否独占借用接收者, 参数类型, 结果类型)。检查器与代码生成共用。
pub fn method_signature(receiver: Type, method: &str) -> Option<(bool, Vec<Type>, Type)> {
    Some(match (receiver.container()?, method) {
        (_, "len") => (false, Vec::new(), Type::Int),
        (Type::List(elem), "push") => (true, vec![elem.ty()], Type::Unit),
        (Type::List(elem), "pop") => (true, Vec::new(), elem.ty()),
        (Type::List(_), "swap") => (true, vec![Type::Int, Type::Int], Type::Unit),
        (Type::Map(key, _), "contains") => (false, vec![key.key_arg()], Type::Bool),
        // 结构体不能复制，`get` 的默认值语义对它不成立。
        (Type::Map(key, value), "get") if !matches!(value, Elem::Record(_)) => {
            (false, vec![key.key_arg(), value.ty()], value.ty())
        }
        (Type::Map(key, _), "remove") => (true, vec![key.key_arg()], Type::Bool),
        (Type::Map(key, _), "keys") => (false, Vec::new(), Type::List(key)),
        _ => return None,
    })
}

/// `[T]` 的元素与字典的值：标量、字符串或命名结构体（还没有嵌套容器）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Elem {
    Int,
    Float,
    Bool,
    Str,
    Record(u64),
}
impl Elem {
    pub fn ty(self) -> Type {
        match self {
            Elem::Int => Type::Int,
            Elem::Float => Type::Float,
            Elem::Bool => Type::Bool,
            Elem::Str => Type::Owned,
            Elem::Record(id) => Type::Record(id),
        }
    }
    /// `for x in &items` 拿到的元素：标量按值，字符串与结构体是只读借用。
    pub fn item(self) -> Type {
        match self {
            Elem::Str => Type::Shared,
            Elem::Record(id) => Type::RecordRef(id, false),
            _ => self.ty(),
        }
    }
    /// 字典键作为参数时的类型：字符串键只读借用，整数键按值。
    pub fn key_arg(self) -> Type {
        if self == Elem::Str {
            Type::Shared
        } else {
            self.ty()
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Record(String, Vec<(String, Expr)>),
    Field(String, String),
    Text(String),
    Format(Vec<Expr>),
    Variable(String),
    Borrow(String, bool),
    Call(String, Vec<Expr>),
    Try(Box<Expr>),
    Catch(Box<Expr>, String, Block),
    ByteLen(Box<Expr>),
    CharLen(Box<Expr>),
    StringEqual(Box<Expr>, Box<Expr>),
    Int(i64),
    Float(f64),
    Bool(bool),
    Unary(char, Box<Expr>),
    Binary(String, Box<Expr>, Box<Expr>),
    Array(Vec<Expr>),
    Index(String, Box<Expr>),
    // `xs[i].a.b`：读取结构体元素的字段路径。
    IndexField(String, Box<Expr>, String),
    // `&xs[i]`、`&mut xs[i].name`：借用元素或元素的字段（字段路径可为空），只能作为调用参数。
    BorrowIndex(String, Box<Expr>, String, bool),
    // `xs.push(v)`：接收者是局部变量或字段路径，由模块解析把非导入别名的 `a.b(...)` 改写而来。
    Method(String, String, Vec<Expr>),
    // `xs[i].tags.push(v)`：接收者是结构体元素里的容器字段（容器路径, 下标, 字段路径, 方法, 参数）。
    ElementMethod(String, Box<Expr>, String, String, Vec<Expr>),
    // `{k: v, ...}` 字典字面量；空字典需要类型标注。
    MapLit(Vec<(Expr, Expr)>),
    // `if c { a } else { b }` 作为值：每个分支是单个表达式，必须有 else。
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    // `Shape.Circle(1.0)`、`Shape.Empty`：构造枚举变体（模块解析后是限定的枚举名）。
    Variant(String, String, Vec<Expr>),
    // `match s { Circle(r) => ... }` 作为值：每个分支是单个表达式。
    Match(Box<Expr>, Vec<(Pattern, Expr)>),
}

/// match 分支的模式：变体名（`None` 是 `_`）和按位置绑定的载荷名（`_` 表示不绑定）。
pub type Pattern = (Option<String>, Vec<String>);

#[derive(Clone, Debug)]
pub enum Instruction {
    Let(String, bool, Option<Type>, Expr),
    Assign(String, Expr),
    AssignField(String, String, Expr),
    AssignIndex(String, Expr, Expr),
    // `xs[i].a.b = v`：写结构体元素的字段。
    AssignIndexField(String, Expr, String, Expr),
    Print(Expr),
    Call(Expr),
    Fail(Expr),
    Return(Option<Expr>),
    If(Expr, Block, Block),
    While(Expr, Block),
    // `for name in start..end`：区间左闭右开，起止只求值一次，循环变量是不可变的 i64。
    For(String, Expr, Expr, Block),
    // `for x in &items`：循环期间列表被冻结（不能修改或移走），字符串元素以共享借用绑定。
    Each(String, String, Block),
    // `match s { ... }` 语句：分支是代码块或单条语句；匹配期间被匹配的位置冻结。
    Match(Expr, Vec<(Pattern, Block)>),
    Break,
    Continue,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub public: bool,
    pub params: Vec<(String, Type)>,
    pub result: Type,
    pub error: Option<Type>,
    pub body: Block,
    pub position: SourcePosition,
    // 标准库函数：只有签名，函数体由原生后端按名字提供；检查器不检查它的空函数体。
    pub builtin: bool,
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub public: bool,
    pub fields: Vec<(String, Type)>,
    // 枚举：每个变体的（名字, 第一个载荷字段的下标, 载荷字段数）。字段 0 是隐藏的标签 `$tag`，
    // 各变体的载荷平铺在后面，名字形如 `Circle$r`；没选中的变体字段保持全零，释放时一并释放也安全。
    pub variants: Vec<(String, usize, usize)>,
}

#[derive(Clone, Debug)]
pub struct Module {
    pub structs: BTreeMap<String, Struct>,
    pub type_names: BTreeMap<u64, String>,
    pub imports: Vec<String>,
    pub functions: BTreeMap<String, Function>,
}

type SourcePosition = (usize, usize);
// 语句序列：每条语句带上它在源码中的起始位置，供语义错误定位。
pub type Block = Vec<(Instruction, SourcePosition)>;

fn lex(source: &str) -> Result<(Vec<Token>, Vec<SourcePosition>), String> {
    let mut position = (1, 1);
    let mut chars = source
        .chars()
        .map(|c| {
            let start = position;
            if c == '\n' {
                position = (position.0 + 1, 1);
            } else {
                position.1 += 1;
            }
            (c, start)
        })
        .peekable();
    let mut tokens = Vec::new();
    let mut lines = Vec::new();
    while let Some((c, (line, column))) = chars.next() {
        let before = tokens.len();
        // 整数后面紧跟的 `.` 拆成单独的记号时，记下点自己的位置。
        let mut split = None;
        let result: Result<(), String> = (|| {
            match c {
                '\n' => tokens.push(Token::Newline),
                c if c.is_whitespace() => {}
                '#' => {
                    while chars.peek().is_some_and(|c| c.0 != '\n') {
                        chars.next();
                    }
                }
                '/' if chars.peek().is_some_and(|(c, _)| *c == '/') => {
                    chars.next();
                    while chars.peek().is_some_and(|c| c.0 != '\n') {
                        chars.next();
                    }
                }
                '"' => {
                    let mut text = String::new();
                    let mut closed = false;
                    while let Some((c, _)) = chars.next() {
                        match c {
                            '"' => {
                                closed = true;
                                break;
                            }
                            '\\' => text.push(match chars.next().map(|(c, _)| c) {
                                Some('n') => '\n',
                                Some('r') => '\r',
                                Some('t') => '\t',
                                Some('"') => '"',
                                Some('\\') => '\\',
                                _ => return Err("unsupported string escape".into()),
                            }),
                            '\n' | '\r' => return Err("newline in string literal".into()),
                            c => text.push(c),
                        }
                    }
                    if !closed {
                        return Err("unterminated string literal".into());
                    }
                    tokens.push(Token::Text(text));
                }
                // 字符字面量 `'a'`：值是该字符的 Unicode 码点（i64），ASCII 字符等于 byte_at 读到的字节。
                '\'' => {
                    let single = "character literal must contain exactly one character";
                    let value = match chars.next().map(|(c, _)| c) {
                        Some('\\') => match chars.next().map(|(c, _)| c) {
                            Some('n') => '\n',
                            Some('r') => '\r',
                            Some('t') => '\t',
                            Some('0') => '\0',
                            Some('\\') => '\\',
                            Some('\'') => '\'',
                            Some('"') => '"',
                            _ => return Err("unsupported character escape".into()),
                        },
                        Some('\'' | '\n' | '\r') | None => return Err(single.into()),
                        Some(c) => c,
                    };
                    if chars.next().map(|(c, _)| c) != Some('\'') {
                        return Err(single.into());
                    }
                    tokens.push(Token::Number(u32::from(value).to_string()));
                }
                c if c.is_ascii_alphabetic() || c == '_' => {
                    let mut word = String::from(c);
                    while chars
                        .peek()
                        .is_some_and(|(c, _)| c.is_ascii_alphanumeric() || *c == '_')
                    {
                        word.push(chars.next().unwrap().0);
                    }
                    tokens.push(Token::Word(word));
                }
                c if c.is_ascii_digit() => {
                    let mut number = String::from(c);
                    while chars
                        .peek()
                        .is_some_and(|(c, _)| c.is_ascii_digit() || *c == '_')
                    {
                        number.push(chars.next().unwrap().0);
                    }
                    if number.ends_with('_') || number.contains("__") {
                        return Err("integer separators must occur between digits".into());
                    }
                    let mut float = false;
                    // `1.5` 是浮点数；`0..n` 的点后面不是数字，整数照旧，点交给区间。
                    if chars.peek().is_some_and(|(c, _)| *c == '.') {
                        let (_, dot) = chars.next().unwrap();
                        if !chars.peek().is_some_and(|(c, _)| c.is_ascii_digit()) {
                            tokens.push(Token::Number(number.replace('_', "")));
                            tokens.push(Token::Symbol('.'));
                            split = Some(dot);
                            return Ok(());
                        }
                        float = true;
                        number.push('.');
                        while chars
                            .peek()
                            .is_some_and(|(c, _)| c.is_ascii_digit() || *c == '_')
                        {
                            number.push(chars.next().unwrap().0);
                        }
                    }
                    if chars.peek().is_some_and(|(c, _)| matches!(c, 'e' | 'E')) {
                        float = true;
                        number.push(chars.next().unwrap().0);
                        if chars.peek().is_some_and(|(c, _)| matches!(c, '+' | '-')) {
                            number.push(chars.next().unwrap().0);
                        }
                        if !chars.peek().is_some_and(|(c, _)| c.is_ascii_digit()) {
                            return Err(
                                "expected digits in the f64 exponent, such as 1e9 or 2.5e-3".into(),
                            );
                        }
                        while chars.peek().is_some_and(|(c, _)| c.is_ascii_digit()) {
                            number.push(chars.next().unwrap().0);
                        }
                    }
                    if !float {
                        tokens.push(Token::Number(number.replace('_', "")));
                        return Ok(());
                    }
                    let bytes = number.as_bytes();
                    if (0..bytes.len()).any(|i| {
                        bytes[i] == b'_'
                            && !(bytes[i - 1].is_ascii_digit()
                                && bytes.get(i + 1).is_some_and(u8::is_ascii_digit))
                    }) {
                        return Err("digit separators must occur between digits".into());
                    }
                    let text = number.replace('_', "");
                    if !text.parse::<f64>().is_ok_and(f64::is_finite) {
                        return Err("f64 literal is out of range".into());
                    }
                    tokens.push(Token::Float(text));
                }
                '(' | ')' | '{' | '}' | '[' | ']' | '/' | '.' | ':' | ',' | '=' | '&' | '-'
                | '>' | '<' | '+' | '*' | '%' | '!' | '|' | ';' | '?' => {
                    tokens.push(Token::Symbol(c))
                }
                _ => return Err(format!("unexpected character {c:?}")),
            }
            Ok(())
        })();
        result.map_err(|error| format!("line {line}, column {column}: {error}"))?;
        if tokens.len() > before {
            lines.push((line, column));
        }
        lines.extend(split);
    }
    lines.push(position);
    Ok((tokens, lines))
}

/// 只规范化空白：按大括号深度重排每行缩进、去掉行尾空白、统一结尾换行。
/// 不重建 token，所以注释、字符串内容与行内空格原样保留，永远不会丢注释。
/// ponytail: 未做行内空格重排与长行折行；需要时再从 token 流重建整行。
pub fn format(source: &str) -> Result<String, String> {
    if source.is_empty() {
        return Ok(String::new());
    }
    let (tokens, positions) = lex(source)?;
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut lines = Vec::new();
    let mut depth = 0usize;
    let mut cursor = 0usize;
    for (index, text) in source.lines().enumerate() {
        let line = index + 1;
        // 先消费严格在本行之前的 token，depth 即本行起点的深度。
        while cursor < tokens.len() && positions[cursor].0 < line {
            match tokens[cursor] {
                Token::Symbol('{') => depth += 1,
                Token::Symbol('}') => depth = depth.saturating_sub(1),
                _ => {}
            }
            cursor += 1;
        }
        let closes = cursor < tokens.len()
            && positions[cursor].0 == line
            && tokens[cursor] == Token::Symbol('}');
        let indent = depth.saturating_sub(usize::from(closes));
        let content = text.trim();
        lines.push(if content.is_empty() {
            String::new()
        } else {
            format!("{}{content}", " ".repeat(indent * 4))
        });
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    let mut formatted = lines.join(newline);
    formatted.push_str(newline);
    Ok(formatted)
}

struct Parser {
    tokens: Vec<Token>,
    lines: Vec<SourcePosition>,
    position: usize,
    type_names: BTreeMap<u64, String>,
    // 当前函数的上下文；`catch` 的代码块出现在表达式位置，需要它来解析语句。
    imports: Vec<String>,
    result: Type,
    // 块末尾的调用被暂当作隐式返回值的语句位置；函数体解析完后由 `untail` 按尾部位置修正。
    implicit: BTreeSet<SourcePosition>,
    // 正在解析 match 的被匹配值：`match Shape.Empty {` 里的 `{` 属于 match，不是结构体字面量。
    scrutinee: bool,
}

/// 没有调用的表达式：重复求值不会产生额外副作用（复合赋值的下标用它判断）。
fn pure(expr: &Expr) -> bool {
    match expr {
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Bool(_)
        | Expr::Text(_)
        | Expr::Variable(_)
        | Expr::Borrow(_, _)
        | Expr::Field(_, _) => true,
        Expr::Format(segments) => segments.iter().all(pure),
        Expr::Unary(_, inner) | Expr::Index(_, inner) | Expr::IndexField(_, inner, _) => {
            pure(inner)
        }
        Expr::Binary(_, left, right) => pure(left) && pure(right),
        _ => false,
    }
}

fn is_text(ty: Type) -> bool {
    matches!(ty, Type::Owned | Type::Shared | Type::Mutable)
}

/// 算术运算的结果类型：左操作数是 f64 就是 f64，否则按 i64 检查。
pub fn numeric(ty: Type) -> Type {
    if ty == Type::Float {
        Type::Float
    } else {
        Type::Int
    }
}

/// 只有函数尾部的调用才是隐式返回值：函数体的最后一条，以及尾部 if/else 分支的最后一条。
/// 循环体与非尾部分支里的末尾调用只求值、不返回（否则 `while c { f() }` 会在第一轮就返回）。
/// `catch` 代码块必须返回，保持原样。
fn untail(body: &mut Block, tail: bool, implicit: &BTreeSet<SourcePosition>) {
    let last = body.len().saturating_sub(1);
    for (index, (instruction, position)) in body.iter_mut().enumerate() {
        let here = tail && index == last;
        match instruction {
            Instruction::Return(Some(expr)) if !here && implicit.contains(position) => {
                *instruction = Instruction::Call(expr.clone());
            }
            Instruction::If(_, yes, no) => {
                untail(yes, here, implicit);
                untail(no, here, implicit);
            }
            Instruction::Match(_, arms) => {
                for (_, arm) in arms {
                    untail(arm, here, implicit);
                }
            }
            Instruction::While(_, nested)
            | Instruction::For(_, _, _, nested)
            | Instruction::Each(_, _, nested) => untail(nested, false, implicit),
            _ => {}
        }
    }
}
impl Parser {
    fn at(&self) -> SourcePosition {
        self.lines[self.position.min(self.lines.len() - 1)]
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }
    fn take(&mut self) -> Option<Token> {
        let token = self.peek().cloned();
        self.position += usize::from(token.is_some());
        token
    }
    fn symbol(&mut self, c: char) -> bool {
        if self.peek() == Some(&Token::Symbol(c)) {
            self.take();
            true
        } else {
            false
        }
    }
    fn keyword(&mut self, word: &str) -> bool {
        if self.peek() == Some(&Token::Word(word.into())) {
            self.take();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, c: char) -> Result<(), String> {
        if self.symbol(c) {
            Ok(())
        } else {
            Err(format!("expected {c:?}, found {:?}", self.peek()))
        }
    }
    fn word(&mut self) -> Result<String, String> {
        match self.take() {
            Some(Token::Word(word)) => Ok(word),
            token => Err(format!("expected identifier, found {token:?}")),
        }
    }
    fn newlines(&mut self) {
        while self.peek() == Some(&Token::Newline) {
            self.take();
        }
    }
    fn separator(&mut self) -> Result<(), String> {
        match self.peek() {
            None | Some(Token::Newline) | Some(Token::Symbol('}')) => {
                self.newlines();
                Ok(())
            }
            token => Err(format!(
                "expected newline between statements, found {token:?}"
            )),
        }
    }
    /// 类型名：`i64`、`Point` 或跨模块的 `model.Point`。
    fn type_name(&mut self) -> Result<String, String> {
        let mut name = self.word()?;
        while self.symbol('.') {
            name.push('.');
            name.push_str(&self.word()?);
        }
        Ok(name)
    }
    /// 首字母大写的类型名是结构体：记下名字，返回它的 id。
    fn record(&mut self, name: &str) -> Option<u64> {
        if !name
            .rsplit('.')
            .next()
            .is_some_and(|n| n.starts_with(char::is_uppercase))
        {
            return None;
        }
        let id = record_id(name);
        self.type_names.insert(id, name.to_owned());
        Some(id)
    }
    /// 列表元素与字典值：标量、字符串或结构体。
    fn elem(&mut self, name: &str) -> Option<Elem> {
        Some(match name {
            "i64" => Elem::Int,
            "f64" => Elem::Float,
            "bool" => Elem::Bool,
            "string" => Elem::Str,
            _ => Elem::Record(self.record(name)?),
        })
    }
    fn ty(&mut self) -> Result<Type, String> {
        let ty = if self.symbol('&') {
            if self.keyword("mut") {
                Type::Mutable
            } else {
                Type::Shared
            }
        } else {
            Type::Owned
        };
        if self.peek() == Some(&Token::Symbol('{')) {
            self.take();
            let key = match self.word()?.as_str() {
                "i64" => Elem::Int,
                "string" => Elem::Str,
                _ => return Err("unsupported map key type; expected i64 or string keys".into()),
            };
            self.expect(':')?;
            let name = self.type_name()?;
            let value = self
                .elem(&name)
                .ok_or("unsupported map value type; expected i64, f64, bool, string or a struct")?;
            self.expect('}')?;
            return Ok(match ty {
                Type::Owned => Type::Map(key, value),
                Type::Shared => Type::MapRef(key, value, false),
                _ => Type::MapRef(key, value, true),
            });
        }
        if self.peek() == Some(&Token::Symbol('[')) {
            self.take();
            let element = self.type_name()?;
            if !self.symbol(';') {
                let elem = self.elem(&element).ok_or(
                    "unsupported list element type; expected [i64], [f64], [bool], [string] or a struct such as [Point]",
                )?;
                self.expect(']')?;
                return Ok(match ty {
                    Type::Owned => Type::List(elem),
                    Type::Shared => Type::ListRef(elem, false),
                    _ => Type::ListRef(elem, true),
                });
            }
            if element != "i64" {
                return Err("only [i64; N] arrays are supported".into());
            }
            if ty != Type::Owned {
                return Err("arrays are only supported as local values".into());
            }
            let length = match self.take() {
                Some(Token::Number(number)) => {
                    number.parse().map_err(|_| "invalid array length")?
                }
                _ => return Err("array length must be an integer".into()),
            };
            if length == 0 || length > 1024 {
                return Err("array length must be between 1 and 1024".into());
            }
            self.expect(']')?;
            return Ok(Type::ArrayInt(length));
        }
        let name = self.type_name()?;
        match name.as_str() {
            "string" => Ok(ty),
            "i64" if ty == Type::Owned => Ok(Type::Int),
            "f64" if ty == Type::Owned => Ok(Type::Float),
            "bool" if ty == Type::Owned => Ok(Type::Bool),
            _ => match (self.record(&name), ty) {
                (Some(id), Type::Owned) => Ok(Type::Record(id)),
                (Some(id), _) => Ok(Type::RecordRef(id, ty == Type::Mutable)),
                (None, _) => {
                    Err("unsupported type; expected string, i64, f64, bool or a struct".into())
                }
            },
        }
    }
    /// 元素后面的字段路径 `.a.b`；没有字段时为空串。`..` 不是字段访问。
    fn field_path(&mut self) -> Result<String, String> {
        let mut path = String::new();
        while self.peek() == Some(&Token::Symbol('.'))
            && matches!(self.tokens.get(self.position + 1), Some(Token::Word(_)))
        {
            self.take();
            if !path.is_empty() {
                path.push('.');
            }
            path.push_str(&self.word()?);
        }
        Ok(path)
    }
    fn expr(&mut self) -> Result<Expr, String> {
        self.binary(0)
    }
    fn binary(&mut self, minimum: u8) -> Result<Expr, String> {
        let mut left = self.atom()?;
        loop {
            let (op, priority, width) = match self.peek() {
                Some(Token::Symbol(c @ ('&' | '|')))
                    if self.tokens.get(self.position + 1) == Some(&Token::Symbol(*c)) =>
                {
                    (format!("{c}{c}"), if *c == '|' { 1 } else { 2 }, 2)
                }
                Some(Token::Symbol(c @ ('=' | '!' | '<' | '>'))) => {
                    let equal = self.tokens.get(self.position + 1) == Some(&Token::Symbol('='));
                    if matches!(c, '=' | '!') && !equal {
                        break;
                    }
                    (
                        format!("{c}{}", if equal { "=" } else { "" }),
                        3,
                        if equal { 2 } else { 1 },
                    )
                }
                // `x += 1` 里的 `+=` 是复合赋值，不是二元运算。
                Some(Token::Symbol('+' | '-' | '*' | '/' | '%'))
                    if self.tokens.get(self.position + 1) == Some(&Token::Symbol('=')) =>
                {
                    break
                }
                Some(Token::Symbol(c @ ('+' | '-'))) => (c.to_string(), 4, 1),
                Some(Token::Symbol(c @ ('*' | '/' | '%'))) => (c.to_string(), 5, 1),
                _ => break,
            };
            if priority < minimum {
                break;
            }
            self.position += width;
            let right = self.binary(priority + 1)?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn atom(&mut self) -> Result<Expr, String> {
        if self.symbol('(') {
            let expr = self.expr()?;
            self.expect(')')?;
            return Ok(expr);
        }
        if self.symbol('-') {
            if let Some(Token::Number(number)) = self.peek().cloned() {
                self.take();
                return Ok(Expr::Int(
                    format!("-{number}")
                        .parse()
                        .map_err(|_| "i64 literal overflow")?,
                ));
            }
            if let Some(Token::Float(text)) = self.peek().cloned() {
                self.take();
                return Ok(Expr::Float(-text.parse::<f64>().expect("lexed float")));
            }
            return Ok(Expr::Unary('-', Box::new(self.atom()?)));
        }
        if self.symbol('!') {
            return Ok(Expr::Unary('!', Box::new(self.atom()?)));
        }
        if let Some(Token::Number(number)) = self.peek().cloned() {
            self.take();
            return Ok(Expr::Int(
                number.parse().map_err(|_| "i64 literal overflow")?,
            ));
        }
        if let Some(Token::Float(text)) = self.peek().cloned() {
            self.take();
            return Ok(Expr::Float(text.parse().expect("lexed float")));
        }
        if self.keyword("if") {
            return self.if_value();
        }
        if self.keyword("match") {
            return self.match_value();
        }
        if self.keyword("true") {
            return Ok(Expr::Bool(true));
        }
        if self.keyword("false") {
            return Ok(Expr::Bool(false));
        }
        if let Some(Token::Text(text)) = self.peek().cloned() {
            self.take();
            return self.literal(&text);
        }
        if self.symbol('{') {
            // 字典字面量：`{k: v, k2: v2}`，条目之间用逗号，可以分行写。
            let mut entries = Vec::new();
            self.newlines();
            while !self.symbol('}') {
                let key = self.expr()?;
                self.expect(':')?;
                let value = self.expr()?;
                entries.push((key, value));
                self.newlines();
                if self.symbol('}') {
                    break;
                }
                self.expect(',')?;
                self.newlines();
            }
            return Ok(Expr::MapLit(entries));
        }
        if self.symbol('[') {
            let mut values = Vec::new();
            if !self.symbol(']') {
                loop {
                    values.push(self.expr()?);
                    if self.symbol(']') {
                        break;
                    }
                    self.expect(',')?;
                }
            }
            return Ok(Expr::Array(values));
        }
        if self.symbol('&') {
            let mutable = self.keyword("mut");
            let mut name = self.word()?;
            while self.symbol('.') {
                name.push('.');
                name.push_str(&self.word()?);
            }
            if self.symbol('[') {
                let index = self.expr()?;
                self.expect(']')?;
                let field = self.field_path()?;
                if self.peek() == Some(&Token::Symbol('(')) {
                    return Err(
                        "expected a field after &xs[..]; references cannot point at call results"
                            .into(),
                    );
                }
                return Ok(Expr::BorrowIndex(name, Box::new(index), field, mutable));
            }
            return Ok(Expr::Borrow(name, mutable));
        }
        let mut name = self.word()?;
        // 只有点号后面跟标识符才是字段路径；`a..b` 里的 `..` 留给 for 区间。
        while self.peek() == Some(&Token::Symbol('.'))
            && matches!(self.tokens.get(self.position + 1), Some(Token::Word(_)))
        {
            self.take();
            name.push('.');
            name.push_str(&self.word()?);
        }
        if name
            .rsplit('.')
            .next()
            .is_some_and(|n| n.starts_with(char::is_uppercase))
            && !self.scrutinee
            && self.symbol('{')
        {
            self.newlines();
            let mut fields = Vec::new();
            while !self.symbol('}') {
                let field = self.word()?;
                self.expect(':')?;
                fields.push((field, self.expr()?));
                // 字段之间换行，或在同一行直接接下一个 `name:`（表达式后紧跟 `名字:` 不会有歧义）。
                let next_field = matches!(self.peek(), Some(Token::Word(_)))
                    && self.tokens.get(self.position + 1) == Some(&Token::Symbol(':'));
                if !next_field {
                    self.separator()?;
                }
            }
            return Ok(Expr::Record(name, fields));
        }
        if !self.symbol('(') {
            if self.symbol('[') {
                let index = self.expr()?;
                self.expect(']')?;
                let field = self.field_path()?;
                if !field.is_empty() && self.symbol('(') {
                    let (field, method) = field.rsplit_once('.').unwrap_or(("", &field));
                    let args = self.arguments()?;
                    return Ok(Expr::ElementMethod(
                        name,
                        Box::new(index),
                        field.to_owned(),
                        method.to_owned(),
                        args,
                    ));
                }
                if !field.is_empty() {
                    return Ok(Expr::IndexField(name, Box::new(index), field));
                }
                return Ok(Expr::Index(name, Box::new(index)));
            }
            if let Some((local, field)) = name.split_once('.') {
                return Ok(Expr::Field(local.into(), field.into()));
            }
            return Ok(Expr::Variable(name));
        }
        let args = self.arguments()?;
        // `Shape.Circle(1.0)`：点号后首字母大写的调用是枚举变体。
        if let Some((enumeration, variant)) = name.rsplit_once('.') {
            if variant.starts_with(char::is_uppercase) {
                return Ok(Expr::Variant(enumeration.into(), variant.into(), args));
            }
        }
        let call = Expr::Call(name, args);
        // `?` 只作用于调用；其他位置的 `?` 不是合法语法。
        if self.symbol('?') {
            return Ok(Expr::Try(Box::new(call)));
        }
        // `call catch e { ... }` 就地处理错误，不传播。
        if self.keyword("catch") {
            let binding = self.word()?;
            let imports = self.imports.clone();
            let block = self.block(&imports, self.result)?;
            return Ok(Expr::Catch(Box::new(call), binding, block));
        }
        Ok(call)
    }
    /// 左括号之后的实参列表，读到右括号为止。
    fn arguments(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if !self.symbol(')') {
            loop {
                args.push(self.expr()?);
                if self.symbol(')') {
                    break;
                }
                self.expect(',')?;
            }
        }
        Ok(args)
    }
    /// `if` 出现在表达式位置：`if c { a } else if d { b } else { e }`，分支各是一个值。
    fn if_value(&mut self) -> Result<Expr, String> {
        let condition = self.expr()?;
        let yes = self.branch_value()?;
        self.newlines();
        if !self.keyword("else") {
            return Err("expected else: an if expression needs an else branch".into());
        }
        let no = if self.keyword("if") {
            self.if_value()?
        } else {
            self.branch_value()?
        };
        Ok(Expr::If(Box::new(condition), Box::new(yes), Box::new(no)))
    }
    fn branch_value(&mut self) -> Result<Expr, String> {
        self.expect('{')?;
        self.newlines();
        let value = self.expr()?;
        self.newlines();
        if !self.symbol('}') {
            return Err(
                "expected }: an if expression branch holds a single value; compute it with let before the if"
                    .into(),
            );
        }
        Ok(value)
    }
    /// 插值里的 `{...}`：变量或字段路径照旧；其他内容按表达式解析（不能含字符串字面量）。
    fn literal(&mut self, text: &str) -> Result<Expr, String> {
        let Expr::Format(segments) = literal(text)? else {
            return literal(text);
        };
        let mut parsed = Vec::with_capacity(segments.len());
        for segment in segments {
            parsed.push(match segment {
                Expr::Call(source, _) => {
                    let (tokens, lines) = lex(&source)
                        .map_err(|error| format!("in interpolation {{{source}}}: {error}"))?;
                    let count = tokens.len();
                    let mut inner = Parser {
                        tokens,
                        lines,
                        position: 0,
                        type_names: BTreeMap::new(),
                        imports: self.imports.clone(),
                        result: self.result,
                        implicit: BTreeSet::new(),
                        scrutinee: false,
                    };
                    let expr = inner
                        .expr()
                        .map_err(|error| format!("in interpolation {{{source}}}: {error}"))?;
                    if inner.position != count {
                        return Err(format!(
                            "unsupported interpolation {{{source}}}: expected one expression"
                        ));
                    }
                    self.type_names.extend(inner.type_names);
                    expr
                }
                other => other,
            });
        }
        Ok(Expr::Format(parsed))
    }
    fn if_statement(&mut self, imports: &[String], result: Type) -> Result<Instruction, String> {
        let condition = self.expr()?;
        let yes = self.block(imports, result)?;
        let boundary = self.position;
        self.newlines();
        let no = if self.keyword("else") {
            let position = self.at();
            if self.keyword("if") {
                vec![(self.if_statement(imports, result)?, position)]
            } else {
                self.block(imports, result)?
            }
        } else {
            self.position = boundary;
            Vec::new()
        };
        Ok(Instruction::If(condition, yes, no))
    }

    fn block(&mut self, imports: &[String], result: Type) -> Result<Block, String> {
        self.expect('{')?;
        self.newlines();
        let mut body = Vec::new();
        while self.peek() != Some(&Token::Symbol('}')) {
            if self.peek().is_none() {
                return Err("unterminated function block".into());
            }
            let position = self.at();
            let instruction = self.statement(imports, result, None)?;
            body.push((instruction, position));
            self.separator()?;
        }
        self.take();
        Ok(body)
    }
    /// 后面（跳过换行）紧跟 `}`：当前语句是代码块的最后一条。
    fn at_end(&self) -> bool {
        self.tokens[self.position..]
            .iter()
            .find(|token| **token != Token::Newline)
            == Some(&Token::Symbol('}'))
    }
    /// 一条语句。`last` 指定它是不是所在代码块的最后一条（match 分支的单条语句总是）；
    /// 为 `None` 时看后面是不是 `}`。
    fn statement(
        &mut self,
        imports: &[String],
        result: Type,
        last: Option<bool>,
    ) -> Result<Instruction, String> {
        let position = self.at();
        Ok(if self.keyword("if") {
            self.if_statement(imports, result)?
        } else if self.keyword("match") {
            self.match_statement(imports, result)?
        } else if self.keyword("while") {
            let condition = self.expr()?;
            let nested = self.block(imports, result)?;
            Instruction::While(condition, nested)
        } else if self.keyword("for") {
            let local = self.word()?;
            if !self.keyword("in") {
                return Err("expected `in` after the for loop variable".into());
            }
            if self.symbol('&') {
                if self.keyword("mut") {
                    return Err(
                        "unsupported for loop over &mut; index with for i in 0..items.len()".into(),
                    );
                }
                let mut path = self.word()?;
                while self.symbol('.') {
                    path.push('.');
                    path.push_str(&self.word()?);
                }
                let nested = self.block(imports, result)?;
                Instruction::Each(local, path, nested)
            } else {
                let start = self.expr()?;
                if !(self.symbol('.') && self.symbol('.')) {
                    return Err("expected `..` in for range: for name in start..end, or iterate a list with for x in &items".into());
                }
                let end = self.expr()?;
                let nested = self.block(imports, result)?;
                Instruction::For(local, start, end, nested)
            }
        } else if self.keyword("break") {
            Instruction::Break
        } else if self.keyword("continue") {
            Instruction::Continue
        } else if self.keyword("let") {
            let mutable = self.keyword("mut");
            let local = self.word()?;
            let annotation = if self.symbol(':') {
                Some(self.ty()?)
            } else {
                None
            };
            if annotation.is_some_and(Type::is_reference) {
                return Err("borrow escape: references cannot be stored in locals".into());
            }
            self.expect('=')?;
            Instruction::Let(local, mutable, annotation, self.expr()?)
        } else if self.keyword("return") {
            let expr = if matches!(
                self.peek(),
                None | Some(Token::Newline) | Some(Token::Symbol('}'))
            ) {
                None
            } else {
                Some(self.expr()?)
            };
            Instruction::Return(expr)
        } else if self.keyword("fail") {
            // `fail e` 在声明了错误类型的函数里返回错误值。
            Instruction::Fail(self.expr()?)
        } else {
            let expr = self.expr()?;
            if let Some(Token::Symbol(op @ ('+' | '-' | '*' | '/' | '%'))) = self.peek().cloned() {
                if self.tokens.get(self.position + 1) == Some(&Token::Symbol('=')) {
                    // `x op= e` 等价于 `x = x op e`；下标里不能有调用，保证只求值一次的语义。
                    self.position += 2;
                    let value = self.expr()?;
                    let combined =
                        Expr::Binary(op.to_string(), Box::new(expr.clone()), Box::new(value));
                    return Ok(match expr {
                            Expr::Variable(name) => Instruction::Assign(name, combined),
                            Expr::Field(local, field) => {
                                Instruction::AssignField(local, field, combined)
                            }
                            Expr::Index(local, index) if pure(&index) => {
                                Instruction::AssignIndex(local, *index, combined)
                            }
                            Expr::IndexField(local, index, field) if pure(&index) => {
                                Instruction::AssignIndexField(local, *index, field, combined)
                            }
                            Expr::Index(_, _) | Expr::IndexField(_, _, _) => return Err(
                                "unsupported compound assignment index: it must not call functions; bind it with let first"
                                    .into(),
                            ),
                            _ => {
                                return Err(format!(
                                    "expected a variable, field or index before {op}="
                                ))
                            }
                    });
                }
            }
            if let Expr::Field(local, field) = &expr {
                if self.symbol('=') {
                    Instruction::AssignField(local.clone(), field.clone(), self.expr()?)
                } else {
                    Instruction::Return(Some(expr))
                }
            } else if let Expr::Index(local, index) = &expr {
                if self.symbol('=') {
                    Instruction::AssignIndex(local.clone(), (**index).clone(), self.expr()?)
                } else {
                    Instruction::Return(Some(expr))
                }
            } else if let Expr::IndexField(local, index, field) = &expr {
                if self.symbol('=') {
                    Instruction::AssignIndexField(
                        local.clone(),
                        (**index).clone(),
                        field.clone(),
                        self.expr()?,
                    )
                } else {
                    Instruction::Return(Some(expr))
                }
            } else if let Expr::Variable(name) = &expr {
                if self.symbol('=') {
                    Instruction::Assign(name.clone(), self.expr()?)
                } else {
                    Instruction::Return(Some(expr))
                }
            } else if let Expr::Call(callee, args) = &expr {
                if matches!(callee.as_str(), "println" | "io.println" | "fmt.println") {
                    if let Some(namespace) = callee.strip_suffix(".println") {
                        if !imports.contains(&format!("std/{namespace}")) {
                            return Err(format!("{callee} requires use std/{namespace}"));
                        }
                    }
                    if args.len() != 1 {
                        return Err("println requires one string argument".into());
                    }
                    Instruction::Print(args[0].clone())
                } else if result != Type::Unit && last.unwrap_or_else(|| self.at_end()) {
                    self.implicit.insert(position);
                    Instruction::Return(Some(expr))
                } else {
                    Instruction::Call(expr)
                }
            } else if matches!(
                expr,
                Expr::Try(_) | Expr::Catch(_, _, _) | Expr::ElementMethod(_, _, _, _, _)
            ) {
                // `write(&p)?` 这类语句：不是函数的最后一个值表达式，就只求值、不返回。
                if result != Type::Unit && last.unwrap_or_else(|| self.at_end()) {
                    self.implicit.insert(position);
                    Instruction::Return(Some(expr))
                } else {
                    Instruction::Call(expr)
                }
            } else {
                Instruction::Return(Some(expr))
            }
        })
    }
    /// `match` 的分支模式：`Circle(r)`、`Rect(w, _)`、`Shape.Empty` 或 `_`，后接 `=>`。
    fn pattern(&mut self) -> Result<Pattern, String> {
        let pattern = if self.keyword("_") {
            (None, Vec::new())
        } else {
            let name = self.type_name()?;
            let variant = name.rsplit('.').next().unwrap_or(&name).to_owned();
            if !variant.starts_with(char::is_uppercase) {
                return Err(format!(
                    "expected an enum variant such as Circle(r), or _, in a match arm; found {name}"
                ));
            }
            let mut bindings = Vec::new();
            if self.symbol('(') && !self.symbol(')') {
                loop {
                    bindings.push(self.word()?);
                    if self.symbol(')') {
                        break;
                    }
                    self.expect(',')?;
                }
            }
            (Some(variant), bindings)
        };
        if !(self.symbol('=') && self.symbol('>')) {
            return Err("expected => after the match pattern".into());
        }
        Ok(pattern)
    }
    /// 语句形式的 match：分支是 `{ ... }` 代码块或一条语句。
    /// match 的被匹配值：其中不解析结构体字面量。
    fn scrutinee(&mut self) -> Result<Expr, String> {
        let outer = std::mem::replace(&mut self.scrutinee, true);
        let scrutinee = self.expr();
        self.scrutinee = outer;
        scrutinee
    }
    fn match_statement(&mut self, imports: &[String], result: Type) -> Result<Instruction, String> {
        let scrutinee = self.scrutinee()?;
        self.expect('{')?;
        self.newlines();
        let mut arms = Vec::new();
        while !self.symbol('}') {
            let pattern = self.pattern()?;
            let body = if self.peek() == Some(&Token::Symbol('{')) {
                self.block(imports, result)?
            } else {
                let position = self.at();
                vec![(self.statement(imports, result, Some(true))?, position)]
            };
            arms.push((pattern, body));
            self.separator()?;
        }
        if arms.is_empty() {
            return Err("expected at least one arm in the match".into());
        }
        Ok(Instruction::Match(scrutinee, arms))
    }
    /// 表达式形式的 match：`let v = match s { A => 1  B(x) => x }`，每个分支一个值。
    fn match_value(&mut self) -> Result<Expr, String> {
        let scrutinee = self.scrutinee()?;
        self.expect('{')?;
        self.newlines();
        let mut arms = Vec::new();
        while !self.symbol('}') {
            let pattern = self.pattern()?;
            if self.peek() == Some(&Token::Symbol('{')) {
                return Err(
                    "expected a value: a match expression arm holds a single value; compute it with let before the match, or write the match as a statement"
                        .into(),
                );
            }
            arms.push((pattern, self.expr()?));
            self.separator()?;
        }
        if arms.is_empty() {
            return Err("expected at least one arm in the match".into());
        }
        Ok(Expr::Match(Box::new(scrutinee), arms))
    }
}

// 字符串插值：`{name}` 读取变量或字段，`{{`/`}}` 输出字面大括号。
// ponytail: 花括号内只接受变量与字段路径；需要任意表达式时把插值下沉到代码生成。
fn literal(text: &str) -> Result<Expr, String> {
    let mut segments = Vec::new();
    let mut literal = String::new();
    let mut changed = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                literal.push('{');
                changed = true;
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                literal.push('}');
                changed = true;
            }
            '{' => {
                changed = true;
                if !literal.is_empty() {
                    segments.push(Expr::Text(std::mem::take(&mut literal)));
                }
                let mut inner = String::new();
                loop {
                    match chars.next() {
                        Some('}') => break,
                        Some('{') => return Err("nested { in string interpolation".into()),
                        Some(c) => inner.push(c),
                        None => return Err("unterminated { in string literal".into()),
                    }
                }
                let path = inner.trim();
                if path.is_empty() {
                    return Err("empty {} in string literal; write {{}} for literal braces".into());
                }
                let simple = path.split('.').all(|part| {
                    part.starts_with(|c: char| c.is_alphabetic() || c == '_')
                        && part.chars().all(|c| c.is_alphanumeric() || c == '_')
                });
                segments.push(if !simple {
                    // 由 Parser::literal 按表达式解析。
                    Expr::Call(path.into(), Vec::new())
                } else {
                    match path.split_once('.') {
                        Some((root, field)) => Expr::Field(root.into(), field.into()),
                        None => Expr::Variable(path.into()),
                    }
                });
            }
            '}' => return Err("unmatched } in string literal".into()),
            c => literal.push(c),
        }
    }
    if !changed {
        return Ok(Expr::Text(text.into()));
    }
    if !literal.is_empty() {
        segments.push(Expr::Text(literal));
    }
    Ok(Expr::Format(segments))
}

pub fn parse(source: &str) -> Result<Module, String> {
    let (tokens, lines) = lex(source)?;
    let mut p = Parser {
        tokens,
        lines,
        position: 0,
        type_names: BTreeMap::new(),
        imports: Vec::new(),
        result: Type::Unit,
        implicit: BTreeSet::new(),
        scrutinee: false,
    };
    let result = (|| {
        let mut module = Module {
            structs: BTreeMap::new(),
            type_names: BTreeMap::new(),
            imports: Vec::new(),
            functions: BTreeMap::new(),
        };
        p.newlines();
        while p.peek().is_some() {
            if p.keyword("use") {
                let mut path = p.word()?;
                while p.symbol('/') {
                    path.push('/');
                    path.push_str(&p.word()?);
                }
                if module.imports.contains(&path) {
                    return Err(format!("duplicate import {path}"));
                }
                module.imports.push(path);
                p.separator()?;
                continue;
            }
            let public = p.keyword("pub");
            let start = p.at();
            if p.keyword("struct") {
                let name = p.word()?;
                if matches!(name.as_str(), "string" | "i64" | "f64" | "bool") {
                    return Err(format!("reserved type name {name}"));
                }
                if !name.starts_with(char::is_uppercase) {
                    return Err("struct names must start with an uppercase letter".into());
                }
                p.expect('{')?;
                p.newlines();
                let mut fields = Vec::new();
                let mut names = BTreeSet::new();
                while !p.symbol('}') {
                    let field = p.word()?;
                    if !names.insert(field.clone()) {
                        return Err(format!("duplicate field {name}.{field}"));
                    }
                    p.expect(':')?;
                    let ty = p.ty()?;
                    if ty.is_reference() {
                        return Err("borrow escape: structs cannot store references".into());
                    }
                    if matches!(ty, Type::ArrayInt(_)) {
                        return Err("arrays are only supported as local values".into());
                    }
                    fields.push((field, ty));
                    p.separator()?;
                }
                if module.functions.contains_key(&name)
                    || module
                        .structs
                        .insert(
                            name.clone(),
                            Struct {
                                public,
                                fields,
                                variants: Vec::new(),
                            },
                        )
                        .is_some()
                {
                    return Err(format!("duplicate declaration {name}"));
                }
                p.separator()?;
                continue;
            }
            if p.keyword("enum") {
                let name = p.word()?;
                if matches!(name.as_str(), "string" | "i64" | "f64" | "bool") {
                    return Err(format!("reserved type name {name}"));
                }
                if !name.starts_with(char::is_uppercase) {
                    return Err("enum names must start with an uppercase letter".into());
                }
                p.expect('{')?;
                p.newlines();
                let mut fields = vec![("$tag".to_owned(), Type::Int)];
                let mut variants: Vec<(String, usize, usize)> = Vec::new();
                while !p.symbol('}') {
                    let variant = p.word()?;
                    if !variant.starts_with(char::is_uppercase) {
                        return Err("enum variant names must start with an uppercase letter".into());
                    }
                    if variants.iter().any(|(other, _, _)| *other == variant) {
                        return Err(format!("duplicate variant {name}.{variant}"));
                    }
                    let first = fields.len();
                    if p.symbol('(') && !p.symbol(')') {
                        let mut names = BTreeSet::new();
                        loop {
                            let field = p.word()?;
                            if !names.insert(field.clone()) {
                                return Err(format!("duplicate field {variant}.{field}"));
                            }
                            p.expect(':')?;
                            let ty = p.ty()?;
                            if ty.is_reference() {
                                return Err("borrow escape: enums cannot store references".into());
                            }
                            if matches!(ty, Type::ArrayInt(_)) {
                                return Err("arrays are only supported as local values".into());
                            }
                            fields.push((format!("{variant}${field}"), ty));
                            if p.symbol(')') {
                                break;
                            }
                            p.expect(',')?;
                        }
                    }
                    variants.push((variant, first, fields.len() - first));
                    p.separator()?;
                }
                if variants.is_empty() {
                    return Err(format!("expected at least one variant in enum {name}"));
                }
                if module.functions.contains_key(&name)
                    || module
                        .structs
                        .insert(
                            name.clone(),
                            Struct {
                                public,
                                fields,
                                variants,
                            },
                        )
                        .is_some()
                {
                    return Err(format!("duplicate declaration {name}"));
                }
                p.separator()?;
                continue;
            }
            if !p.keyword("fn") {
                return Err("expected use, struct, enum or fn declaration".into());
            }
            let name = p.word()?;
            // `i64(x)`、`f64(n)` 是内建的数值转换。
            if matches!(name.as_str(), "i64" | "f64") {
                return Err(format!("reserved type name {name}"));
            }
            if module.structs.contains_key(&name) {
                return Err(format!("duplicate declaration {name}"));
            }
            p.expect('(')?;
            let mut params = Vec::new();
            if !p.symbol(')') {
                loop {
                    let param = p.word()?;
                    p.expect(':')?;
                    let ty = p.ty()?;
                    if matches!(ty, Type::ArrayInt(_)) {
                        return Err("arrays are only supported as local values".into());
                    }
                    params.push((param, ty));
                    if p.symbol(')') {
                        break;
                    }
                    p.expect(',')?;
                }
            }
            let (result, error) = if p.symbol('-') {
                p.expect('>')?;
                // `-> !E` 表示没有成功值、只有错误值。
                if p.symbol('!') {
                    let error = p.ty()?;
                    (Type::Unit, Some(error))
                } else {
                    let ty = p.ty()?;
                    if ty.is_reference() {
                        return Err("borrow escape: functions cannot return references".into());
                    }
                    if matches!(ty, Type::ArrayInt(_)) {
                        return Err("arrays are only supported as local values".into());
                    }
                    let error = if p.symbol('!') { Some(p.ty()?) } else { None };
                    (ty, error)
                }
            } else {
                (Type::Unit, None)
            };
            p.imports = module.imports.clone();
            p.result = result;
            let mut body = p.block(&module.imports, result)?;
            untail(&mut body, true, &p.implicit);
            if module
                .functions
                .insert(
                    name.clone(),
                    Function {
                        public,
                        params,
                        result,
                        error,
                        body,
                        position: start,
                        builtin: false,
                    },
                )
                .is_some()
            {
                return Err(format!("duplicate function {name}"));
            }
            p.separator()?;
        }
        module.type_names = std::mem::take(&mut p.type_names);
        Ok(module)
    })();
    result.map_err(|error: String| {
        let (line, column) = p.at();
        format!("line {line}, column {column}: {error}")
    })
}

// Stable, path-free checked IR for module cache validation.
pub fn encode(module: &Module) -> Vec<u8> {
    let declarations: Vec<_> = module
        .structs
        .iter()
        .map(|(name, structure)| {
            (
                name,
                structure.public,
                &structure.fields,
                &structure.variants,
            )
        })
        .collect();
    format!(
        "ZAM-IR-20\n{declarations:?}\n{:?}\n{:?}\n{:?}\n",
        module.type_names, module.imports, module.functions
    )
    .into_bytes()
}

#[derive(Clone)]
struct Binding {
    ty: Type,
    mutable: bool,
    live: bool,
}
pub fn record_id(name: &str) -> u64 {
    u64::from_str_radix(&crate::project::hash(&[name.as_bytes()])[..16], 16).unwrap()
}

pub fn field_path(
    structs: &BTreeMap<String, Struct>,
    mut ty: Type,
    path: &str,
) -> Result<(Vec<usize>, Type), String> {
    // 借用的结构体按它指向的结构体取字段。
    if let Type::RecordRef(id, _) = ty {
        ty = Type::Record(id);
    }
    let mut indices = Vec::new();
    for field in path.split('.') {
        let structure = structs
            .iter()
            .find(|(name, _)| ty == Type::Record(record_id(name)))
            .ok_or("field access requires a struct")?
            .1;
        let (index, (_, field_ty)) = structure
            .fields
            .iter()
            .enumerate()
            .find(|(_, (name, _))| name == field)
            .ok_or_else(|| format!("unknown field {field}"))?;
        indices.push(index);
        ty = *field_ty;
    }
    Ok((indices, ty))
}

struct Checker<'a> {
    structs: &'a BTreeMap<String, Struct>,
    functions: &'a BTreeMap<String, Function>,
    locals: BTreeMap<String, Binding>,
    loops: Vec<(BTreeMap<String, Binding>, Expr)>,
    position: SourcePosition,
    // 当前函数的错误类型（`-> T!E` 的 E）；`?`/`fail` 依赖它。
    error: Option<Type>,
    // 当前函数的成功类型；`catch` 的代码块按它检查 return。
    result: Type,
    // 正被 `for x in &items` 遍历的根绑定：循环体内不能修改、移走或独占借用它们。
    frozen: Vec<String>,
}
impl<'a> Checker<'a> {
    fn thawed(&self, root: &str) -> Result<(), String> {
        if self.frozen.iter().any(|name| name == root) {
            Err(format!(
                "borrow conflict: {root} is borrowed by an enclosing for loop"
            ))
        } else {
            Ok(())
        }
    }
    /// 局部变量或字段路径的根名与类型；根必须仍然有效。
    fn place(&self, path: &str) -> Result<(String, Type), String> {
        let (root, field) = path
            .split_once('.')
            .map_or((path, None), |(root, field)| (root, Some(field)));
        let binding = self.binding(root)?;
        let ty = match field {
            Some(field) => field_path(self.structs, binding.ty, field)?.1,
            None => binding.ty,
        };
        Ok((root.to_owned(), ty))
    }
    /// 把路径当容器借用一次：检查可变性、冻结与已有借用，返回背后的 owned 容器类型
    /// （`[T]` 或 `{K: V}`）。不记录借用，调用方按需要把根写进 `loans`。
    fn container(
        &self,
        path: &str,
        mutable: bool,
        loans: &BTreeMap<String, Type>,
    ) -> Result<Type, String> {
        let (root, ty) = self.place(path)?;
        let (Some(base), Some((_, by_ref))) = (ty.container(), ty.borrowable()) else {
            return Err(format!("type mismatch: {path} is not a list or map"));
        };
        if mutable {
            if !by_ref.unwrap_or(self.writable(&root)) {
                return Err(format!("cannot mutably borrow immutable value {path}"));
            }
            self.thawed(&root)?;
        }
        if loans
            .get(&root)
            .is_some_and(|old| *old == Type::Mutable || mutable)
        {
            return Err(format!("borrow conflict: {path}"));
        }
        Ok(base)
    }
    /// 容器下标的（键类型, 元素类型）：列表用 i64 下标，字典用键。
    fn subscript(container: Type) -> (Type, Elem) {
        match container {
            Type::List(elem) => (Type::Int, elem),
            Type::Map(key, value) => (key.key_arg(), value),
            _ => unreachable!("container type"),
        }
    }
    /// `ty` 是枚举或枚举的借用时，返回（限定名, 声明）。
    fn enumeration(&self, ty: Type) -> Option<(&'a String, &'a Struct)> {
        let (Type::Record(id) | Type::RecordRef(id, _)) = ty else {
            return None;
        };
        self.structs
            .iter()
            .find(|(name, structure)| record_id(name) == id && !structure.variants.is_empty())
    }
    /// match 被匹配值的类型，不求值（位置按位置取类型）。
    fn scrutinee_ty(&self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::Variable(name) => Ok(self.place(name)?.1),
            Expr::Field(local, field) => Ok(self.place(&format!("{local}.{field}"))?.1),
            Expr::Index(path, _) if self.place(path)?.1.container().is_some() => {
                Ok(self.element(path, "")?.1)
            }
            Expr::IndexField(path, _, field) => Ok(self.element(path, field)?.1),
            _ => self.ty(expr),
        }
    }
    /// 检查 match 的被匹配值：变量、字段与元素只借用（返回匹配期间要冻结的根绑定），
    /// 其他表达式求值成临时值。
    fn scrutinee(
        &mut self,
        expr: &Expr,
        loans: &mut BTreeMap<String, Type>,
    ) -> Result<(u64, Option<String>), String> {
        let ty = self.scrutinee_ty(expr)?;
        let Some((name, _)) = self.enumeration(ty) else {
            return Err("type mismatch: match requires an enum value".into());
        };
        let id = record_id(name);
        let root = match expr {
            Expr::Variable(name) | Expr::Field(name, _) => {
                Some(name.split('.').next().unwrap_or(name).to_owned())
            }
            Expr::Index(path, index) | Expr::IndexField(path, index, _)
                if self.place(path)?.1.container().is_some() =>
            {
                let (key, _) = self.element(path, "")?;
                self.container(path, false, loans)?;
                self.expr(index, key, loans)?;
                Some(path.split('.').next().unwrap_or(path).to_owned())
            }
            _ => {
                self.expr(expr, ty, loans)?;
                None
            }
        };
        if let Some(root) = &root {
            if loans.get(root) == Some(&Type::Mutable) {
                return Err(format!("borrow conflict: {root}"));
            }
        }
        Ok((id, root))
    }
    /// 一个分支绑定的变量：标量载荷按值，其他载荷是只读借用。
    fn arm_bindings(&self, id: u64, pattern: &Pattern) -> Result<Vec<(String, Type)>, String> {
        let (name, declaration) = self.enumeration(Type::Record(id)).expect("checked enum");
        let Some(variant) = &pattern.0 else {
            return Ok(Vec::new());
        };
        let short = name.rsplit(':').next().unwrap_or(name);
        let (_, first, count) = declaration
            .variants
            .iter()
            .find(|(other, _, _)| other == variant)
            .ok_or_else(|| format!("unknown variant {variant} of {short}"))?;
        if pattern.1.len() != *count {
            return Err(format!(
                "type mismatch: {short}.{variant} has {count} field(s), but the pattern binds {}",
                pattern.1.len()
            ));
        }
        let mut bindings: Vec<(String, Type)> = Vec::new();
        for (binding, (_, ty)) in pattern.1.iter().zip(&declaration.fields[*first..]) {
            if binding == "_" {
                continue;
            }
            if self.locals.contains_key(binding) || bindings.iter().any(|(b, _)| b == binding) {
                return Err(format!("duplicate binding {binding}"));
            }
            let ty = if matches!(ty, Type::Int | Type::Float | Type::Bool) {
                *ty
            } else {
                borrow_of(*ty, false).expect("enum payloads are owned values")
            };
            bindings.push((binding.clone(), ty));
        }
        Ok(bindings)
    }
    /// 分支要覆盖所有变体（或有 `_`），不能重复，`_` 之后不能再有分支。
    fn arms_cover<'p>(
        &self,
        id: u64,
        patterns: impl Iterator<Item = &'p Pattern>,
    ) -> Result<(), String> {
        let (name, declaration) = self.enumeration(Type::Record(id)).expect("checked enum");
        let short = name.rsplit(':').next().unwrap_or(name);
        let mut seen = BTreeSet::new();
        let mut wildcard = false;
        for (variant, _) in patterns {
            if wildcard {
                return Err("unreachable match arm after _".into());
            }
            match variant {
                None => wildcard = true,
                Some(variant) => {
                    if !declaration.variants.iter().any(|(v, _, _)| v == variant) {
                        return Err(format!("unknown variant {variant} of {short}"));
                    }
                    if !seen.insert(variant) {
                        return Err(format!("duplicate match arm {variant}"));
                    }
                }
            }
        }
        let missing: Vec<&str> = declaration
            .variants
            .iter()
            .filter(|(v, _, _)| !seen.contains(v))
            .map(|(v, _, _)| v.as_str())
            .collect();
        if !wildcard && !missing.is_empty() {
            return Err(format!(
                "match on {short} does not cover {}; add an arm for each or a _ arm",
                missing.join(", ")
            ));
        }
        Ok(())
    }
    /// 根绑定能否经由它写：`let mut` 局部，或 `&mut T` 结构体借用。
    fn writable(&self, root: &str) -> bool {
        self.locals
            .get(root)
            .is_some_and(|b| b.mutable || matches!(b.ty, Type::RecordRef(_, true)))
    }
    /// `path[..]` 的元素，或元素里 `field` 路径上的值：（键类型, 目标类型）。
    fn element(&self, path: &str, field: &str) -> Result<(Type, Type), String> {
        let Some(container) = self.place(path)?.1.container() else {
            return Err(format!("type mismatch: {path} is not a list or map"));
        };
        let (key, elem) = Self::subscript(container);
        let target = if field.is_empty() {
            elem.ty()
        } else {
            field_path(self.structs, elem.ty(), field)?.1
        };
        Ok((key, target))
    }
    /// `path[..].field.method(...)` 的（下标类型, 是否独占借用接收者, 参数类型, 结果类型）。
    fn element_method(
        &self,
        path: &str,
        field: &str,
        method: &str,
    ) -> Result<(Type, bool, Vec<Type>, Type), String> {
        let (key, target) = self.element(path, field)?;
        let receiver = format!("{path}[..].{field}");
        let Some(container) = target.container() else {
            return Err(format!(
                "type mismatch: {receiver} has no method {method}; methods are only available on lists and maps"
            ));
        };
        let (mutable, params, result) = method_signature(target, method).ok_or_else(|| {
            let supported = if matches!(container, Type::List(_)) {
                "lists support len, push, pop and swap"
            } else {
                "maps support len, contains, get, remove and keys"
            };
            format!("unknown function {receiver}.{method}: {supported}")
        })?;
        Ok((key, mutable, params, result))
    }
    fn method(&self, path: &str, method: &str) -> Result<(bool, Vec<Type>, Type), String> {
        let root = path.split('.').next().unwrap_or(path);
        if !self.locals.contains_key(root) {
            return Err(format!("ambiguous or missing import for {path}.{method}"));
        }
        let ty = self.place(path)?.1;
        let Some(container) = ty.container() else {
            let message = "methods are only available on lists and maps";
            return Err(format!(
                "type mismatch: {path} has no method {method}; {message}"
            ));
        };
        method_signature(ty, method).ok_or_else(|| {
            if method == "get" && matches!(container, Type::Map(_, Elem::Record(_))) {
                return format!(
                    "unknown function {path}.get: struct values cannot be copied; check {path}.contains(k), then read {path}[k].field or borrow &{path}[k]"
                );
            }
            let supported = if matches!(container, Type::List(_)) {
                "lists support len, push, pop and swap"
            } else {
                "maps support len, contains, get, remove and keys"
            };
            format!("unknown function {path}.{method}: {supported}")
        })
    }
    fn binding(&self, name: &str) -> Result<&Binding, String> {
        let binding = self
            .locals
            .get(name)
            .ok_or_else(|| format!("unknown variable {name}"))?;
        if !binding.live {
            return Err(format!("use after move: {name}"));
        }
        Ok(binding)
    }
    fn ty(&self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::Record(name, _) | Expr::Variant(name, _, _) => {
                self.structs
                    .get(name)
                    .ok_or_else(|| format!("unknown struct {name}"))?;
                Ok(Type::Record(record_id(name)))
            }
            // 分支的值可能用到绑定：在带上第一个分支绑定的副本里求类型。
            Expr::Match(scrutinee, arms) => {
                let ty = self.scrutinee_ty(scrutinee)?;
                let Some((name, _)) = self.enumeration(ty) else {
                    return Err("type mismatch: match requires an enum value".into());
                };
                let (pattern, value) = arms.first().ok_or("a match needs at least one arm")?;
                let mut locals = self.locals.clone();
                for (binding, ty) in self.arm_bindings(record_id(name), pattern)? {
                    locals.insert(
                        binding,
                        Binding {
                            ty,
                            mutable: false,
                            live: true,
                        },
                    );
                }
                Checker {
                    structs: self.structs,
                    functions: self.functions,
                    locals,
                    loops: Vec::new(),
                    position: self.position,
                    error: self.error,
                    result: self.result,
                    frozen: Vec::new(),
                }
                .ty(value)
            }
            Expr::Field(local, field) => {
                field_path(self.structs, self.binding(local)?.ty, field).map(|(_, ty)| ty)
            }
            Expr::Array(values) if values.is_empty() => Err(
                "type mismatch: an empty list literal needs a type annotation, such as let xs: [i64] = []"
                    .into(),
            ),
            Expr::Array(values) => Ok(Type::ArrayInt(values.len())),
            Expr::Index(local, _) => match self.place(local)?.1 {
                Type::ArrayInt(_) => Ok(Type::Int),
                Type::List(elem) | Type::ListRef(elem, _) => Ok(elem.ty()),
                Type::Map(_, value) | Type::MapRef(_, value, _) => Ok(value.ty()),
                _ => Err("index requires an array, list or map".into()),
            },
            Expr::MapLit(entries) => {
                let Some((key, value)) = entries.first() else {
                    return Err("type mismatch: an empty map literal needs a type annotation, such as let m: {string: i64} = {}".into());
                };
                let elem = |ty| match ty {
                    Type::Int => Some(Elem::Int),
                    Type::Float => Some(Elem::Float),
                    Type::Bool => Some(Elem::Bool),
                    Type::Owned | Type::Shared | Type::Mutable => Some(Elem::Str),
                    Type::Record(id) => Some(Elem::Record(id)),
                    _ => None,
                };
                match (elem(self.ty(key)?), elem(self.ty(value)?)) {
                    (Some(key @ (Elem::Int | Elem::Str)), Some(value)) => Ok(Type::Map(key, value)),
                    _ => Err("unsupported map literal; keys are i64 or string, values are i64, f64, bool or string".into()),
                }
            }
            Expr::IndexField(path, _, field) => Ok(self.element(path, field)?.1),
            Expr::BorrowIndex(path, _, field, mutable) => {
                borrow_of(self.element(path, field)?.1, *mutable)
                    .ok_or_else(|| "scalar references are not supported".into())
            }
            Expr::Method(path, method, _) => self.method(path, method).map(|(_, _, ty)| ty),
            Expr::ElementMethod(path, _, field, method, _) => self
                .element_method(path, field, method)
                .map(|(_, _, _, ty)| ty),
            Expr::Text(_) => Ok(Type::Owned),
            Expr::Format(_) => Ok(Type::Owned),
            Expr::Int(_) | Expr::ByteLen(_) | Expr::CharLen(_) => Ok(Type::Int),
            Expr::Float(_) => Ok(Type::Float),
            Expr::Bool(_) | Expr::StringEqual(_, _) => Ok(Type::Bool),
            Expr::Variable(name) => Ok(self.binding(name)?.ty),
            Expr::Borrow(_, mutable) => Ok(if *mutable {
                Type::Mutable
            } else {
                Type::Shared
            }),
            Expr::Call(name, _) => self
                .functions
                .get(name)
                .map(|f| f.result)
                .ok_or_else(|| format!("unknown function {name}")),
            // `?` 的值就是被调函数的成功值。
            Expr::Try(inner) => self.ty(inner),
            // `catch` 的成功路径同样拿到被调函数的成功值。
            Expr::Catch(inner, _, _) => self.ty(inner),
            Expr::Unary('!', _) => Ok(Type::Bool),
            Expr::Unary(_, value) => Ok(numeric(self.ty(value)?)),
            Expr::Binary(op, left, _) if op == "+" && is_text(self.ty(left)?) => Ok(Type::Owned),
            Expr::Binary(op, left, _) => Ok(if matches!(op.as_str(), "+" | "-" | "*" | "/" | "%") {
                numeric(self.ty(left)?)
            } else {
                Type::Bool
            }),
            Expr::If(_, yes, _) => self.ty(yes),
        }
    }
    /// 二元运算两边一个是 i64、一个是 f64 时的报错：没有隐式转换。
    fn mixed(&self, op: &str, left: &Expr, right: &Expr) -> Option<String> {
        let types = (self.ty(left).ok()?, self.ty(right).ok()?);
        if !matches!(types, (Type::Int, Type::Float) | (Type::Float, Type::Int)) {
            return None;
        }
        let literal = |expr: &Expr| match expr {
            Expr::Int(value) => Some(*value),
            _ => None,
        };
        Some(match literal(left).or(literal(right)) {
            Some(value) => format!(
                "type mismatch: {op} mixes i64 and f64; write the integer {value} as {value}.0"
            ),
            None => format!(
                "type mismatch: {op} mixes i64 and f64; convert one side with f64(n) or i64(x)"
            ),
        })
    }
    /// 字符串运算（`+`、比较）与插值读取的操作数：变量、字段与元素只读，临时值移入。
    fn text_operand(
        &mut self,
        expr: &Expr,
        loans: &mut BTreeMap<String, Type>,
    ) -> Result<(), String> {
        match expr {
            Expr::Variable(name) if is_text(self.binding(name)?.ty) => Ok(()),
            Expr::Field(_, _) if self.ty(expr)? == Type::Owned => Ok(()),
            Expr::Borrow(_, _) => self.expr(expr, Type::Shared, &mut loans.clone()),
            Expr::Index(local, index) if self.ty(expr)? == Type::Owned => self.expr(
                &Expr::BorrowIndex(local.clone(), index.clone(), String::new(), false),
                Type::Shared,
                &mut loans.clone(),
            ),
            Expr::IndexField(local, index, field) if self.ty(expr)? == Type::Owned => self.expr(
                &Expr::BorrowIndex(local.clone(), index.clone(), field.clone(), false),
                Type::Shared,
                &mut loans.clone(),
            ),
            _ if self.ty(expr)? == Type::Owned => self.expr(expr, Type::Owned, loans),
            _ => Err("type mismatch: expected a string".into()),
        }
    }
    fn expr(
        &mut self,
        expr: &Expr,
        expected: Type,
        loans: &mut BTreeMap<String, Type>,
    ) -> Result<(), String> {
        match expr {
            Expr::Record(name, _)
                if self
                    .structs
                    .get(name)
                    .is_some_and(|s| !s.variants.is_empty()) =>
            {
                let short = name.rsplit(':').next().unwrap_or(name);
                Err(format!(
                    "type mismatch: {short} is an enum; construct a variant with {short}.Variant(...)"
                ))
            }
            Expr::Variant(name, variant, args) => {
                let declaration = self
                    .structs
                    .get(name)
                    .ok_or_else(|| format!("unknown struct {name}"))?;
                let short = name.rsplit(':').next().unwrap_or(name);
                if declaration.variants.is_empty() {
                    return Err(format!("type mismatch: {short} is a struct, not an enum"));
                }
                if expected != Type::Record(record_id(name)) {
                    return Err(format!("type mismatch: {short}.{variant} is a {short}"));
                }
                let (_, first, count) = declaration
                    .variants
                    .iter()
                    .find(|(other, _, _)| other == variant)
                    .ok_or_else(|| format!("unknown variant {variant} of {short}"))?;
                if args.len() != *count {
                    return Err(format!(
                        "argument count mismatch for {short}.{variant}: it takes {count} value(s)"
                    ));
                }
                let fields = declaration.fields[*first..first + count].to_vec();
                for (arg, (_, ty)) in args.iter().zip(&fields) {
                    self.expr(arg, *ty, loans)?;
                }
                Ok(())
            }
            Expr::Match(scrutinee, arms) => {
                if expected.is_reference() || matches!(expected, Type::ArrayInt(_) | Type::Unit) {
                    return Err(
                        "type mismatch: a match expression produces an owned value, not a borrow or fixed array"
                            .into(),
                    );
                }
                let (id, root) = self.scrutinee(scrutinee, loans)?;
                self.arms_cover(id, arms.iter().map(|(pattern, _)| pattern))?;
                let before = self.locals.clone();
                let mut states = Vec::new();
                for (pattern, value) in arms {
                    self.locals = before.clone();
                    let bindings = self.arm_bindings(id, pattern)?;
                    for (name, ty) in &bindings {
                        self.locals.insert(
                            name.clone(),
                            Binding {
                                ty: *ty,
                                mutable: false,
                                live: true,
                            },
                        );
                    }
                    self.frozen.extend(root.clone());
                    self.expr(value, expected, &mut loans.clone())?;
                    if root.is_some() {
                        self.frozen.pop();
                    }
                    for (name, _) in &bindings {
                        self.locals.remove(name);
                    }
                    states.push(self.locals.clone());
                }
                self.locals = before;
                for (name, binding) in &mut self.locals {
                    binding.live &= states.iter().all(|state| state[name].live);
                }
                Ok(())
            }
            Expr::Record(name, fields) => {
                if expected != Type::Record(record_id(name)) {
                    return Err("struct type mismatch".into());
                }
                let declaration = self
                    .structs
                    .get(name)
                    .ok_or("unknown struct")?
                    .fields
                    .clone();
                let mut seen = BTreeSet::new();
                for (field, value) in fields {
                    if !seen.insert(field) {
                        return Err(format!("duplicate initializer {field}"));
                    }
                    let ty = declaration
                        .iter()
                        .find(|(n, _)| n == field)
                        .ok_or_else(|| format!("unknown field {field}"))?
                        .1;
                    self.expr(value, ty, loans)?;
                }
                if seen.len() != declaration.len() {
                    return Err("missing struct field".into());
                }
                Ok(())
            }
            Expr::MapLit(entries) => {
                let Type::Map(key, value) = expected else {
                    return Err("type mismatch: unexpected map literal".into());
                };
                // 字面量里的键和值都移入字典。
                for (k, v) in entries {
                    self.expr(k, key.ty(), loans)?;
                    self.expr(v, value.ty(), loans)?;
                }
                Ok(())
            }
            Expr::Array(values) if matches!(expected, Type::List(_)) => {
                let Type::List(elem) = expected else {
                    unreachable!()
                };
                for value in values {
                    self.expr(value, elem.ty(), loans)?;
                }
                Ok(())
            }
            Expr::Array(values) => {
                if !matches!(expected, Type::ArrayInt(n) if n == values.len()) {
                    return Err("array length or element type mismatch".into());
                }
                for value in values {
                    self.expr(value, Type::Int, loans)?;
                }
                Ok(())
            }
            Expr::Index(local, index) if self.place(local)?.1.container().is_some() => {
                let (key, elem) = Self::subscript(self.container(local, false, loans)?);
                if elem == Elem::Str {
                    return Err(format!(
                        "cannot move a string out of {local}[..]; borrow it with &{local}[..] or copy it with string.copy(&{local}[..])"
                    ));
                }
                if let Elem::Record(_) = elem {
                    return Err(format!(
                        "cannot move a struct out of {local}[..]; borrow it with &{local}[..] or read a field with {local}[..].field"
                    ));
                }
                if elem.ty() != expected {
                    return Err("type mismatch: element type differs".into());
                }
                self.expr(index, key, loans)
            }
            Expr::IndexField(path, index, field) => {
                let (key, target) = self.element(path, field)?;
                if target != expected {
                    return Err("field type mismatch".into());
                }
                if !matches!(target, Type::Int | Type::Float | Type::Bool) {
                    return Err(format!(
                        "owned field moves are not supported; borrow it with &{path}[..].{field}"
                    ));
                }
                self.container(path, false, loans)?;
                self.expr(index, key, loans)
            }
            Expr::Index(local, index) => {
                if expected != Type::Int {
                    return Err("type mismatch: array indexing returns i64".into());
                }
                let length = match self.binding(local)?.ty {
                    Type::ArrayInt(length) => length,
                    _ => return Err("index requires an array or list".into()),
                };
                if let Expr::Int(value) = index.as_ref() {
                    if *value < 0 || (*value as u128) >= length as u128 {
                        return Err("array index out of bounds".into());
                    }
                }
                self.expr(index, Type::Int, loans)
            }
            Expr::Field(_, _) => {
                if self.ty(expr)? != expected {
                    return Err("field type mismatch".into());
                }
                if matches!(
                    expected,
                    Type::Owned | Type::Record(_) | Type::List(_) | Type::Map(_, _)
                ) {
                    return Err("owned field moves are not supported; move the whole struct".into());
                }
                Ok(())
            }
            Expr::Variable(_name) if matches!(expected, Type::ArrayInt(_)) => {
                Err("array values cannot be moved or copied yet".into())
            }
            Expr::Variable(name) if matches!(expected, Type::Record(_)) => {
                let ty = self.binding(name)?.ty;
                if ty.is_reference() {
                    return Err(format!("cannot move borrowed value {name}"));
                }
                if ty != expected {
                    return Err("struct type mismatch".into());
                }
                if loans.contains_key(name) {
                    return Err("borrow conflict".into());
                }
                self.thawed(name)?;
                self.locals.get_mut(name).unwrap().live = false;
                Ok(())
            }
            Expr::Variable(name) if matches!(expected, Type::List(_) | Type::Map(_, _)) => {
                let ty = self.binding(name)?.ty;
                if ty.is_reference() {
                    return Err(format!("cannot move borrowed value {name}"));
                }
                if ty != expected {
                    return Err("type mismatch: container element types differ".into());
                }
                if loans.contains_key(name) {
                    return Err(format!(
                        "borrow conflict: cannot move {name} during a borrow"
                    ));
                }
                self.thawed(name)?;
                self.locals.get_mut(name).unwrap().live = false;
                Ok(())
            }
            Expr::StringEqual(left, right) => {
                if expected != Type::Bool {
                    return Err("equal returns bool".into());
                }
                let mut frame = loans.clone();
                self.expr(left, Type::Shared, &mut frame)?;
                self.expr(right, Type::Shared, &mut frame)
            }
            Expr::ByteLen(value) => {
                if expected != Type::Int {
                    return Err("byte_len returns i64".into());
                }
                self.expr(value, Type::Shared, &mut loans.clone())
            }
            Expr::CharLen(value) => {
                if expected != Type::Int {
                    return Err("char_len returns i64".into());
                }
                self.expr(value, Type::Shared, &mut loans.clone())
            }
            Expr::Format(segments) if expected == Type::Owned => {
                for segment in segments {
                    // 插值只读取变量、字段与元素，不移动；其他表达式求值后插入。
                    let ty = self.ty(segment)?;
                    if !matches!(
                        ty,
                        Type::Owned
                            | Type::Shared
                            | Type::Mutable
                            | Type::Int
                            | Type::Float
                            | Type::Bool
                    ) {
                        return Err("cannot interpolate this value".into());
                    }
                    match segment {
                        Expr::Text(_) | Expr::Variable(_) | Expr::Field(_, _) => {}
                        _ if is_text(ty) => self.text_operand(segment, loans)?,
                        _ => self.expr(segment, ty, &mut loans.clone())?,
                    }
                }
                Ok(())
            }
            Expr::If(condition, yes, no) => {
                if expected.is_reference() || matches!(expected, Type::ArrayInt(_) | Type::Unit) {
                    return Err(
                        "type mismatch: an if expression produces an owned value, not a borrow or fixed array"
                            .into(),
                    );
                }
                self.expr(condition, Type::Bool, loans)?;
                let before = self.locals.clone();
                self.expr(yes, expected, &mut loans.clone())?;
                let after_yes = std::mem::replace(&mut self.locals, before);
                self.expr(no, expected, &mut loans.clone())?;
                for (name, binding) in &mut self.locals {
                    binding.live &= after_yes.get(name).is_none_or(|other| other.live);
                }
                Ok(())
            }
            Expr::Text(_) if expected == Type::Owned => Ok(()),
            // 字面量与插值可以直接传给 `&string` 参数：借用一个只活到语句结束的临时字符串。
            Expr::Text(_) | Expr::Format(_) if expected == Type::Shared => {
                self.expr(expr, Type::Owned, loans)
            }
            Expr::Int(_) if expected == Type::Int => Ok(()),
            Expr::Float(_) if expected == Type::Float => Ok(()),
            Expr::Bool(_) if expected == Type::Bool => Ok(()),
            Expr::Int(value) if expected == Type::Float => Err(format!(
                "type mismatch: expected f64, found the integer {value}; write {value}.0"
            )),
            Expr::Float(_) if expected == Type::Int => {
                Err("type mismatch: expected i64, found an f64 literal; convert with i64(x)".into())
            }
            Expr::Variable(name) if matches!(expected, Type::Int | Type::Float | Type::Bool) => {
                match (self.binding(name)?.ty, expected) {
                    (found, expected) if found == expected => Ok(()),
                    (Type::Int, Type::Float) => Err(format!(
                        "type mismatch: {name} is i64 but f64 is expected; convert with f64({name})"
                    )),
                    (Type::Float, Type::Int) => Err(format!(
                        "type mismatch: {name} is f64 but i64 is expected; convert with i64({name})"
                    )),
                    _ => Err("type mismatch".into()),
                }
            }
            Expr::Unary(op, value) => {
                let ty = if *op == '!' {
                    Type::Bool
                } else {
                    numeric(self.ty(value)?)
                };
                if expected != ty {
                    return Err("type mismatch".into());
                }
                self.expr(value, ty, loans)
            }
            Expr::Binary(op, left, right) => {
                if let Some(error) = self.mixed(op, left, right) {
                    return Err(error);
                }
                let ty = self.ty(expr)?;
                if expected != ty {
                    return Err(match (ty, expected) {
                        (Type::Int, Type::Float) => "type mismatch: this i64 expression is used as f64; convert it with f64(...)".into(),
                        (Type::Float, Type::Int) => "type mismatch: this f64 expression is used as i64; convert it with i64(...)".into(),
                        _ => "type mismatch".into(),
                    });
                }
                if matches!(op.as_str(), "&&" | "||") {
                    self.expr(left, Type::Bool, loans)?;
                    let before = self.locals.clone();
                    self.expr(right, Type::Bool, &mut loans.clone())?;
                    for (name, binding) in &mut self.locals {
                        binding.live &= before[name].live;
                    }
                    return Ok(());
                }
                let comparison = matches!(op.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=");
                if (op == "+" || comparison) && is_text(self.ty(left)?) {
                    // 字符串拼接与比较：两边都只读，不移动变量。
                    self.text_operand(left, loans)?;
                    return self.text_operand(right, loans);
                }
                let operand = if matches!(op.as_str(), "==" | "!=") {
                    self.ty(left)?
                } else {
                    numeric(self.ty(left)?)
                };
                if !matches!(operand, Type::Int | Type::Float | Type::Bool) {
                    return Err("comparison requires i64, f64, bool or string".into());
                }
                self.expr(left, operand, loans)?;
                self.expr(right, operand, loans)
            }
            Expr::Variable(name) if expected == Type::Owned => {
                if self.binding(name)?.ty != Type::Owned {
                    return Err(format!("cannot move borrowed value {name}"));
                }
                if loans.contains_key(name) {
                    return Err(format!(
                        "borrow conflict: cannot move {name} during a borrow"
                    ));
                }
                self.thawed(name)?;
                self.locals.get_mut(name).unwrap().live = false;
                Ok(())
            }
            Expr::Variable(name) | Expr::Borrow(name, _) if expected.is_reference() => {
                // 借用的目标：`&string` 借字符串，`&[T]`、`&{K: V}` 借同类型的容器。
                let (want, Some(want_mutable)) = expected.borrowable().expect("reference type")
                else {
                    unreachable!("reference types are borrows")
                };
                let (root, ty) = self.place(name)?;
                // 位置本身已是借用（参数、for 循环变量）时，记下它是否允许修改。
                let Some((base, by_ref)) = ty.borrowable() else {
                    return Err("scalar references are not supported".into());
                };
                if base != want {
                    return Err(format!("borrow type mismatch for {name}"));
                }
                let requested = match expr {
                    Expr::Borrow(_, mutable) => *mutable,
                    _ if by_ref.is_some() => want_mutable,
                    _ => return Err(format!("expected explicit borrow of {name}")),
                };
                if requested != want_mutable {
                    return Err(format!("borrow type mismatch for {name}"));
                }
                if requested && !by_ref.unwrap_or(self.writable(&root)) {
                    return Err(format!("cannot mutably borrow immutable value {name}"));
                }
                if requested {
                    self.thawed(&root)?;
                }
                if loans
                    .get(&root)
                    .is_some_and(|old| *old == Type::Mutable || requested)
                {
                    return Err(format!("borrow conflict: {name}"));
                }
                // ponytail: loans cover the root; split field loans if disjoint mutation is needed.
                loans.insert(
                    root,
                    if requested {
                        Type::Mutable
                    } else {
                        Type::Shared
                    },
                );
                Ok(())
            }
            Expr::BorrowIndex(path, index, field, mutable) => {
                if !expected.is_reference() {
                    return Err("borrow escape: references only belong in call arguments".into());
                }
                let (key, target) = self.element(path, field)?;
                let Some(borrowed) = borrow_of(target, *mutable) else {
                    return Err("scalar references are not supported".into());
                };
                if borrowed != expected {
                    return Err(format!("borrow type mismatch for {path}[..]"));
                }
                self.container(path, *mutable, loans)?;
                self.expr(index, key, loans)?;
                let root = path.split('.').next().unwrap_or(path).to_owned();
                loans.insert(
                    root,
                    if *mutable {
                        Type::Mutable
                    } else {
                        Type::Shared
                    },
                );
                Ok(())
            }
            Expr::Method(path, method, args) => {
                let (mutable, params, result) = self.method(path, method)?;
                if result != expected {
                    return Err(format!("return type mismatch for {path}.{method}"));
                }
                if params.len() != args.len() {
                    return Err(format!("argument count mismatch for {path}.{method}"));
                }
                // 参数先求值（与 C 侧一致），再借用接收者。
                let mut frame = loans.clone();
                for (arg, ty) in args.iter().zip(&params) {
                    self.expr(arg, *ty, &mut frame)?;
                }
                self.container(path, mutable, loans)?;
                Ok(())
            }
            Expr::ElementMethod(path, index, field, method, args) => {
                let (key, mutable, params, result) = self.element_method(path, field, method)?;
                if result != expected {
                    return Err(format!(
                        "return type mismatch for {path}[..].{field}.{method}"
                    ));
                }
                if params.len() != args.len() {
                    return Err(format!(
                        "argument count mismatch for {path}[..].{field}.{method}"
                    ));
                }
                // 参数、下标依次求值，最后借用接收者所在的容器。
                let mut frame = loans.clone();
                for (arg, ty) in args.iter().zip(&params) {
                    self.expr(arg, *ty, &mut frame)?;
                }
                self.expr(index, key, loans)?;
                self.container(path, mutable, loans)?;
                Ok(())
            }
            Expr::Call(name, args) => {
                let callee = self
                    .functions
                    .get(name)
                    .ok_or_else(|| format!("unknown function {name}"))?
                    .clone();
                // 会失败的调用必须先被 `?` 或 `catch` 处理，错误值不会静默丢弃。
                if callee.error.is_some() {
                    return Err(format!("fallible call {name} must be handled with ?"));
                }
                self.call(name, &callee, args, expected, loans)
            }
            Expr::Try(inner) => {
                let Expr::Call(name, args) = &**inner else {
                    return Err("? requires a function call".into());
                };
                let callee = self
                    .functions
                    .get(name)
                    .ok_or_else(|| format!("unknown function {name}"))?
                    .clone();
                let error = callee
                    .error
                    .ok_or_else(|| format!("? requires a fallible call, but {name} cannot fail"))?;
                if Some(error) != self.error {
                    return Err(format!(
                        "? cannot propagate the error of {name} from this function"
                    ));
                }
                self.call(name, &callee, args, expected, loans)
            }
            Expr::Catch(inner, binding, block) => {
                let Expr::Call(name, args) = &**inner else {
                    return Err("catch requires a function call".into());
                };
                let callee = self
                    .functions
                    .get(name)
                    .ok_or_else(|| format!("unknown function {name}"))?
                    .clone();
                let error = callee.error.ok_or_else(|| {
                    format!("catch requires a fallible call, but {name} cannot fail")
                })?;
                self.call(name, &callee, args, expected, loans)?;
                // 错误值只在 catch 的代码块里可见。
                if self.locals.contains_key(binding) {
                    return Err(format!("duplicate binding {binding}"));
                }
                self.locals.insert(
                    binding.clone(),
                    Binding {
                        ty: error,
                        mutable: false,
                        live: true,
                    },
                );
                let returned = self.block(block, self.result)?;
                self.locals.remove(binding);
                // ponytail: 只认 return；需要 break/continue 收尾时再放宽。
                if !returned {
                    return Err(format!("catch block for {name} must return a value"));
                }
                Ok(())
            }
            Expr::Borrow(_, _) => {
                Err("borrow escape: references only belong in call arguments".into())
            }
            _ => Err("string type mismatch".into()),
        }
    }
    fn call(
        &mut self,
        name: &str,
        callee: &Function,
        args: &[Expr],
        expected: Type,
        loans: &mut BTreeMap<String, Type>,
    ) -> Result<(), String> {
        if callee.result != expected {
            return Err(format!("return type mismatch for {name}"));
        }
        if callee.params.len() != args.len() {
            return Err(format!("argument count mismatch for {name}"));
        }
        // Nested calls inherit outer loans; their own loans end when they return.
        let mut frame = loans.clone();
        for (arg, (_, ty)) in args.iter().zip(&callee.params) {
            self.expr(arg, *ty, &mut frame)?;
        }
        Ok(())
    }
    fn block(&mut self, body: &Block, result: Type) -> Result<bool, String> {
        let outer_names = self.locals.keys().cloned().collect();
        let mut returned = false;
        let mut terminated = false;
        for (instruction, position) in body {
            self.position = *position;
            if returned || terminated {
                return Err("unreachable statement after return".into());
            }
            let mut loans = BTreeMap::new();
            match instruction {
                Instruction::If(condition, yes, no) => {
                    self.expr(condition, Type::Bool, &mut loans)?;
                    let before = self.locals.clone();
                    let yes_returns = self.block(yes, result)?;
                    let yes_state = self.locals.clone();
                    self.locals = before;
                    let no_returns = self.block(no, result)?;
                    for (name, binding) in &mut self.locals {
                        binding.live = if yes_returns {
                            binding.live
                        } else if no_returns {
                            yes_state[name].live
                        } else {
                            binding.live && yes_state[name].live
                        };
                    }
                    returned = yes_returns && no_returns;
                }
                Instruction::While(condition, nested) => {
                    let before = self.locals.clone();
                    self.expr(condition, Type::Bool, &mut loans)?;
                    self.loops.push((before.clone(), condition.clone()));
                    let returns = self.block(nested, result)?;
                    self.loops.pop();
                    if !returns {
                        self.expr(condition, Type::Bool, &mut BTreeMap::new())?;
                        for (name, binding) in &before {
                            if binding.live && !self.locals[name].live {
                                return Err(format!(
                                    "loop moves value without reinitialization: {name}"
                                ));
                            }
                        }
                    }
                    self.locals = before;
                    self.expr(condition, Type::Bool, &mut BTreeMap::new())?;
                    returned = false;
                }
                Instruction::Each(local, path, nested) => {
                    // 列表遍历元素，字典遍历键（插入顺序）。
                    let elem = match self.container(path, false, &loans)? {
                        Type::List(elem) | Type::Map(elem, _) => elem,
                        _ => unreachable!("container type"),
                    };
                    if self.locals.contains_key(local) {
                        return Err(format!("duplicate binding {local}"));
                    }
                    self.locals.insert(
                        local.clone(),
                        Binding {
                            ty: elem.item(),
                            mutable: false,
                            live: true,
                        },
                    );
                    let before = self.locals.clone();
                    let root = path.split('.').next().unwrap_or(path).to_owned();
                    self.loops.push((before.clone(), Expr::Bool(true)));
                    self.frozen.push(root);
                    let returns = self.block(nested, result)?;
                    self.frozen.pop();
                    self.loops.pop();
                    if !returns {
                        for (name, binding) in &before {
                            if binding.live && !self.locals[name].live {
                                return Err(format!(
                                    "loop moves value without reinitialization: {name}"
                                ));
                            }
                        }
                    }
                    self.locals = before;
                    self.locals.remove(local);
                    returned = false;
                }
                Instruction::Match(scrutinee, arms) => {
                    let (id, root) = self.scrutinee(scrutinee, &mut loans)?;
                    self.arms_cover(id, arms.iter().map(|(pattern, _)| pattern))?;
                    let before = self.locals.clone();
                    let mut states = Vec::new();
                    for (pattern, body) in arms {
                        self.locals = before.clone();
                        let bindings = self.arm_bindings(id, pattern)?;
                        for (name, ty) in &bindings {
                            self.locals.insert(
                                name.clone(),
                                Binding {
                                    ty: *ty,
                                    mutable: false,
                                    live: true,
                                },
                            );
                        }
                        self.frozen.extend(root.clone());
                        let returns = self.block(body, result)?;
                        if root.is_some() {
                            self.frozen.pop();
                        }
                        for (name, _) in &bindings {
                            self.locals.remove(name);
                        }
                        states.push((returns, self.locals.clone()));
                    }
                    self.locals = before;
                    // 没有返回的分支各自的所有权状态取交集。
                    for (name, binding) in &mut self.locals {
                        binding.live = states
                            .iter()
                            .filter(|(returns, _)| !returns)
                            .all(|(_, state)| state[name].live);
                    }
                    returned = states.iter().all(|(returns, _)| *returns);
                }
                Instruction::For(local, start, end, nested) => {
                    self.expr(start, Type::Int, &mut loans)?;
                    self.expr(end, Type::Int, &mut loans)?;
                    if self.locals.contains_key(local) {
                        return Err(format!("duplicate binding {local}"));
                    }
                    self.locals.insert(
                        local.clone(),
                        Binding {
                            ty: Type::Int,
                            mutable: false,
                            live: true,
                        },
                    );
                    let before = self.locals.clone();
                    // 上界在进入循环前已求值，continue 不需要再检查任何条件。
                    self.loops.push((before.clone(), Expr::Bool(true)));
                    let returns = self.block(nested, result)?;
                    self.loops.pop();
                    if !returns {
                        for (name, binding) in &before {
                            if binding.live && !self.locals[name].live {
                                return Err(format!(
                                    "loop moves value without reinitialization: {name}"
                                ));
                            }
                        }
                    }
                    self.locals = before;
                    self.locals.remove(local);
                    returned = false;
                }
                Instruction::Break | Instruction::Continue => {
                    let (before, condition) = self
                        .loops
                        .last()
                        .cloned()
                        .ok_or("loop jump outside while")?;
                    // ponytail: preserve loop-entry owners even on break; merge exit states if move-and-break is needed.
                    for (name, binding) in &before {
                        if binding.live && !self.locals[name].live {
                            return Err(format!(
                                "loop jump moves value without reinitialization: {name}"
                            ));
                        }
                    }
                    if matches!(instruction, Instruction::Continue) {
                        self.expr(&condition, Type::Bool, &mut BTreeMap::new())?;
                    }
                    terminated = true;
                }
                Instruction::Let(local, mutable, annotation, expr) => {
                    if self.locals.contains_key(local) {
                        return Err(format!("duplicate binding {local}"));
                    }
                    let ty = match annotation {
                        Some(ty) => *ty,
                        None => self.ty(expr)?,
                    };
                    if ty.is_reference() || ty == Type::Unit {
                        return Err("borrow escape: locals require an owned value".into());
                    }
                    self.expr(expr, ty, &mut loans)?;
                    self.locals.insert(
                        local.clone(),
                        Binding {
                            ty,
                            mutable: *mutable,
                            live: true,
                        },
                    );
                }
                Instruction::AssignIndex(local, index, expr)
                    if self.place(local)?.1.container().is_some() =>
                {
                    let container = self.place(local)?.1.container().expect("guarded");
                    let (key, elem) = Self::subscript(container);
                    self.expr(index, key, &mut loans)?;
                    self.container(local, true, &loans)?;
                    self.expr(expr, elem.ty(), &mut loans)?;
                    self.place(local)?;
                }
                Instruction::AssignIndexField(local, index, field, expr) => {
                    let (key, target) = self.element(local, field)?;
                    self.expr(index, key, &mut loans)?;
                    self.container(local, true, &loans)?;
                    self.expr(expr, target, &mut loans)?;
                    self.place(local)?;
                }
                Instruction::AssignIndex(local, index, expr) => {
                    if !self.binding(local)?.mutable {
                        return Err(format!("cannot assign immutable value {local}"));
                    }
                    if !matches!(self.binding(local)?.ty, Type::ArrayInt(_)) {
                        return Err("index requires i64 array".into());
                    }
                    self.expr(
                        &Expr::Index(local.clone(), Box::new(index.clone())),
                        Type::Int,
                        &mut loans,
                    )?;
                    self.expr(expr, Type::Int, &mut loans)?;
                    self.binding(local)?;
                }
                Instruction::AssignField(local, field, expr) => {
                    self.binding(local)?;
                    if !self.writable(local) {
                        return Err(format!(
                            "cannot assign immutable value {local}{}",
                            if self.locals[local].ty.is_reference() {
                                "; it is a shared borrow, declare the parameter as &mut and pass &mut"
                            } else {
                                ""
                            }
                        ));
                    }
                    self.thawed(local)?;
                    let ty = self.ty(&Expr::Field(local.clone(), field.clone()))?;
                    self.expr(expr, ty, &mut loans)?;
                    // Replacing a field cannot reinitialize a moved whole struct.
                    self.binding(local)?;
                }
                Instruction::Assign(local, expr) => {
                    let binding = self
                        .locals
                        .get(local)
                        .ok_or_else(|| format!("unknown variable {local}"))?;
                    let by_ref = binding.ty.is_reference();
                    if !(binding.mutable
                        || binding.ty.borrowable().is_some_and(|b| b.1 == Some(true)))
                    {
                        return Err(format!("cannot assign immutable value {local}"));
                    }
                    // 通过 `&mut` 参数赋值时写的是被借用的值。
                    let ty = if by_ref {
                        binding.ty.borrowable().expect("reference type").0
                    } else {
                        binding.ty
                    };
                    self.thawed(local)?;
                    if matches!(ty, Type::ArrayInt(_)) {
                        return Err("array values cannot be moved or copied yet".into());
                    }
                    self.expr(expr, ty, &mut loans)?;
                    self.locals.get_mut(local).unwrap().live = true;
                }
                Instruction::Print(expr) => {
                    let ty = self.ty(expr)?;
                    if !matches!(
                        ty,
                        Type::Owned
                            | Type::Shared
                            | Type::Mutable
                            | Type::Int
                            | Type::Float
                            | Type::Bool
                    ) {
                        return Err("cannot print void, struct, array, list or map".into());
                    }
                    if let Expr::Variable(local) = expr {
                        self.binding(local)?;
                    } else if let Expr::Field(_, _) = expr {
                        self.ty(expr)?;
                    } else if matches!(expr, Expr::Index(_, _) | Expr::IndexField(_, _, _))
                        && ty == Type::Owned
                    {
                        self.text_operand(expr, &mut loans)?;
                    } else if let Expr::BorrowIndex(_, _, _, _) = expr {
                        self.expr(expr, ty, &mut loans)?;
                    } else if let Expr::Borrow(local, mutable) = expr {
                        self.expr(
                            expr,
                            if *mutable {
                                Type::Mutable
                            } else {
                                Type::Shared
                            },
                            &mut loans,
                        )?;
                        self.binding(local.split('.').next().unwrap())?;
                    } else {
                        let ty = self.ty(expr)?;
                        self.expr(expr, ty, &mut loans)?;
                    }
                }
                Instruction::Call(expr) => {
                    let ty = if let Expr::Call(callee, _) = expr {
                        self.functions
                            .get(callee)
                            .ok_or_else(|| format!("unknown function {callee}"))?
                            .result
                    } else if matches!(
                        expr,
                        Expr::ByteLen(_)
                            | Expr::CharLen(_)
                            | Expr::StringEqual(_, _)
                            | Expr::Try(_)
                            | Expr::Catch(_, _, _)
                            | Expr::Method(_, _, _)
                            | Expr::ElementMethod(_, _, _, _, _)
                    ) {
                        self.ty(expr)?
                    } else {
                        return Err("expected function call".into());
                    };
                    self.expr(expr, ty, &mut loans)?;
                }
                Instruction::Fail(expr) => {
                    let error = self
                        .error
                        .ok_or("fail requires a function that declares an error type")?;
                    self.expr(expr, error, &mut loans)?;
                    returned = true;
                }
                Instruction::Return(expr) => {
                    if let Some(expr) = expr {
                        if result == Type::Unit {
                            return Err("void function cannot return a value".into());
                        }
                        self.expr(expr, result, &mut loans)?;
                    } else if result != Type::Unit {
                        return Err("missing string return value".into());
                    }
                    returned = true;
                }
            }
        }
        let outer: BTreeSet<_> = outer_names;
        self.locals.retain(|name, _| outer.contains(name));
        Ok(returned)
    }
}

pub fn record_order(module: &Module, roots: &BTreeSet<String>) -> Result<Vec<String>, String> {
    fn visit(
        module: &Module,
        name: &str,
        active: &mut BTreeSet<String>,
        done: &mut BTreeSet<String>,
        order: &mut Vec<String>,
    ) -> Result<(), String> {
        if done.contains(name) {
            return Ok(());
        }
        if !active.insert(name.into()) {
            return Err(format!("recursive struct layout: {name}"));
        }
        for (_, ty) in &module.structs[name].fields {
            if let Type::Record(id) = ty {
                let child = module
                    .structs
                    .keys()
                    .find(|name| record_id(name) == *id)
                    .ok_or("unknown field type")?;
                visit(module, child, active, done, order)?;
            }
        }
        active.remove(name);
        done.insert(name.into());
        order.push(name.into());
        Ok(())
    }
    let mut order = Vec::new();
    let mut active = BTreeSet::new();
    let mut done = BTreeSet::new();
    for name in roots {
        visit(module, name, &mut active, &mut done, &mut order)?;
    }
    Ok(order)
}

/// 多条诊断放进同一个错误字符串时用的分隔符（ASCII 记录分隔符），由命令行拆开逐条输出。
pub const DIAGNOSTIC_SEPARATOR: char = '\u{1e}';

/// 逐个函数独立检查：每个出错的函数报它的第一条错误，互不影响；全部检查完再一起返回。
pub fn check(module: &Module, entry: &str) -> Result<(), String> {
    record_order(module, &module.structs.keys().cloned().collect())?;
    let main = module.functions.get(entry).ok_or("missing main function")?;
    if !main.params.is_empty() || main.result != Type::Unit {
        return Err("main must have no parameters or return value".into());
    }
    let mut errors = Vec::new();
    for (name, function) in &module.functions {
        if function.builtin {
            continue;
        }
        let mut checker = Checker {
            structs: &module.structs,
            functions: &module.functions,
            locals: BTreeMap::new(),
            loops: Vec::new(),
            position: function.position,
            error: function.error,
            result: function.result,
            frozen: Vec::new(),
        };
        let result = (|| {
            // ponytail: v1 只支持 string 错误类型；泛化 E 是纯增量（C 侧结果结构体按 E 再分一个维度）。
            if function.error.is_some_and(|ty| ty != Type::Owned) {
                return Err("only string error types are supported for now".into());
            }
            let mut names = BTreeSet::new();
            for (param, ty) in &function.params {
                if !names.insert(param.clone()) {
                    return Err(format!("duplicate binding {param}"));
                }
                checker.locals.insert(
                    param.clone(),
                    Binding {
                        ty: *ty,
                        mutable: false,
                        live: true,
                    },
                );
            }
            let returned = checker.block(&function.body, function.result)?;
            checker.position = function.position;
            if function.result != Type::Unit && !returned {
                return Err("missing string return value".into());
            }
            Ok(())
        })();
        if let Err(error) = result {
            let (line, column) = checker.position;
            errors.push(format!("line {line}, column {column}: {name}: {error}"));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join(&DIAGNOSTIC_SEPARATOR.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_reindents_but_keeps_comments_and_text() {
        let source = "fn main() {\n//顶层注释\n let x =1\n if true {\n println(x) //行尾注释\n#井号注释\n }\n}\n";
        assert_eq!(
            format(source).unwrap(),
            "fn main() {\n    //顶层注释\n    let x =1\n    if true {\n        println(x) //行尾注释\n        #井号注释\n    }\n}\n"
        );
    }

    #[test]
    fn format_is_idempotent_and_keeps_line_endings() {
        let source =
            "fn main() {\r\n\tlet text = \"你好\"\r\nif true {\r\nprintln(&text)\r\n}\r\n}\r\n";
        let once = format(source).unwrap();
        assert_eq!(format(&once).unwrap(), once);
        assert!(once.contains("\r\n") && !once.replace("\r\n", "").contains('\n'));
        assert!(once.contains("\"你好\""));

        let unix = source.replace("\r\n", "\n");
        let trimmed = format(&format(&unix).unwrap()).unwrap();
        assert_eq!(trimmed, format(&unix).unwrap());
        assert!(!trimmed.contains('\r'));
    }

    #[test]
    fn format_drops_trailing_blank_lines_and_reports_broken_source() {
        assert_eq!(format("fn main() {\n}\n\n\n").unwrap(), "fn main() {\n}\n");
        assert!(format("fn main() {\nprintln(\"open\n}\n").is_err());
    }
}
