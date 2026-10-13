use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Word(String),
    Text(String),
    Number(String),
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
    Bool,
    Record(u64),
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
            Type::Shared | Type::Mutable | Type::ListRef(_, _) | Type::MapRef(_, _, _)
        )
    }
    /// 可被借用的值：（被借用的 owned 类型，本身已是借用时是否可写）。标量与结构体不可借用。
    pub fn borrowable(self) -> Option<(Type, Option<bool>)> {
        match self {
            Type::Owned => Some((Type::Owned, None)),
            Type::Shared => Some((Type::Owned, Some(false))),
            Type::Mutable => Some((Type::Owned, Some(true))),
            Type::List(elem) => Some((Type::List(elem), None)),
            Type::ListRef(elem, mutable) => Some((Type::List(elem), Some(mutable))),
            Type::Map(key, value) => Some((Type::Map(key, value), None)),
            Type::MapRef(key, value, mutable) => Some((Type::Map(key, value), Some(mutable))),
            _ => None,
        }
    }
}

/// 容器方法的签名：(是否独占借用接收者, 参数类型, 结果类型)。检查器与代码生成共用。
pub fn method_signature(receiver: Type, method: &str) -> Option<(bool, Vec<Type>, Type)> {
    Some(match (receiver.container()?, method) {
        (_, "len") => (false, Vec::new(), Type::Int),
        (Type::List(elem), "push") => (true, vec![elem.ty()], Type::Unit),
        (Type::List(elem), "pop") => (true, Vec::new(), elem.ty()),
        (Type::List(_), "swap") => (true, vec![Type::Int, Type::Int], Type::Unit),
        (Type::Map(key, _), "contains") => (false, vec![key.key_arg()], Type::Bool),
        (Type::Map(key, value), "get") => (false, vec![key.key_arg(), value.ty()], value.ty()),
        (Type::Map(key, _), "remove") => (true, vec![key.key_arg()], Type::Bool),
        (Type::Map(key, _), "keys") => (false, Vec::new(), Type::List(key)),
        _ => return None,
    })
}

/// `[T]` 的元素类型；v1 只支持标量与字符串，结构体元素等结构体借用落地后再加。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Elem {
    Int,
    Bool,
    Str,
}
impl Elem {
    pub fn ty(self) -> Type {
        match self {
            Elem::Int => Type::Int,
            Elem::Bool => Type::Bool,
            Elem::Str => Type::Owned,
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
    Bool(bool),
    Unary(char, Box<Expr>),
    Binary(String, Box<Expr>, Box<Expr>),
    Array(Vec<Expr>),
    Index(String, Box<Expr>),
    // `&xs[i]` / `&mut xs[i]`：借用列表里的字符串元素，只能作为调用参数。
    BorrowIndex(String, Box<Expr>, bool),
    // `xs.push(v)`：接收者是局部变量或字段路径，由模块解析把非导入别名的 `a.b(...)` 改写而来。
    Method(String, String, Vec<Expr>),
    // `{k: v, ...}` 字典字面量；空字典需要类型标注。
    MapLit(Vec<(Expr, Expr)>),
}

#[derive(Clone, Debug)]
pub enum Instruction {
    Let(String, bool, Option<Type>, Expr),
    Assign(String, Expr),
    AssignField(String, String, Expr),
    AssignIndex(String, Expr, Expr),
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
                    tokens.push(Token::Number(number.replace('_', "")));
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
            let value = match self.word()?.as_str() {
                "i64" => Elem::Int,
                "bool" => Elem::Bool,
                "string" => Elem::Str,
                _ => return Err("unsupported map value type; expected i64, bool or string".into()),
            };
            self.expect('}')?;
            return Ok(match ty {
                Type::Owned => Type::Map(key, value),
                Type::Shared => Type::MapRef(key, value, false),
                _ => Type::MapRef(key, value, true),
            });
        }
        if self.peek() == Some(&Token::Symbol('[')) {
            self.take();
            let element = self.word()?;
            if !self.symbol(';') {
                let elem = match element.as_str() {
                    "i64" => Elem::Int,
                    "bool" => Elem::Bool,
                    "string" => Elem::Str,
                    _ => {
                        return Err(
                            "unsupported list element type; expected [i64], [bool] or [string]"
                                .into(),
                        )
                    }
                };
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
        let mut name = self.word()?;
        while self.symbol('.') {
            name.push('.');
            name.push_str(&self.word()?);
        }
        match name.as_str() {
            "string" => Ok(ty),
            "i64" if ty == Type::Owned => Ok(Type::Int),
            "bool" if ty == Type::Owned => Ok(Type::Bool),
            _ if ty == Type::Owned
                && name
                    .rsplit('.')
                    .next()
                    .is_some_and(|n| n.starts_with(char::is_uppercase)) =>
            {
                let id = record_id(&name);
                self.type_names.insert(id, name);
                Ok(Type::Record(id))
            }
            _ => Err("unsupported type; expected string, i64, bool or owned struct".into()),
        }
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
        if self.keyword("true") {
            return Ok(Expr::Bool(true));
        }
        if self.keyword("false") {
            return Ok(Expr::Bool(false));
        }
        if let Some(Token::Text(text)) = self.peek().cloned() {
            self.take();
            return literal(&text);
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
                return Ok(Expr::BorrowIndex(name, Box::new(index), mutable));
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
                return Ok(Expr::Index(name, Box::new(index)));
            }
            if let Some((local, field)) = name.split_once('.') {
                return Ok(Expr::Field(local.into(), field.into()));
            }
            return Ok(Expr::Variable(name));
        }
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
            let instruction = if self.keyword("if") {
                body.push((self.if_statement(imports, result)?, position));
                self.separator()?;
                continue;
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
                            "unsupported for loop over &mut; index with for i in 0..items.len()"
                                .into(),
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
                    } else if result != Type::Unit
                        && self.tokens[self.position..]
                            .iter()
                            .find(|token| **token != Token::Newline)
                            == Some(&Token::Symbol('}'))
                    {
                        self.implicit.insert(position);
                        Instruction::Return(Some(expr))
                    } else {
                        Instruction::Call(expr)
                    }
                } else if matches!(expr, Expr::Try(_) | Expr::Catch(_, _, _)) {
                    // `write(&p)?` 这类语句：不是函数的最后一个值表达式，就只求值、不返回。
                    if result != Type::Unit
                        && self.tokens[self.position..]
                            .iter()
                            .find(|token| **token != Token::Newline)
                            == Some(&Token::Symbol('}'))
                    {
                        self.implicit.insert(position);
                        Instruction::Return(Some(expr))
                    } else {
                        Instruction::Call(expr)
                    }
                } else {
                    Instruction::Return(Some(expr))
                }
            };
            body.push((instruction, position));
            self.separator()?;
        }
        self.take();
        Ok(body)
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
                let valid = !path.is_empty()
                    && path.split('.').all(|part| {
                        part.starts_with(|c: char| c.is_alphabetic() || c == '_')
                            && part.chars().all(|c| c.is_alphanumeric() || c == '_')
                    });
                if !valid {
                    return Err(format!(
                        "unsupported interpolation {{{inner}}}: expected a variable or field path"
                    ));
                }
                segments.push(match path.split_once('.') {
                    Some((root, field)) => Expr::Field(root.into(), field.into()),
                    None => Expr::Variable(path.into()),
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
                if matches!(name.as_str(), "string" | "i64" | "bool") {
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
                        .insert(name.clone(), Struct { public, fields })
                        .is_some()
                {
                    return Err(format!("duplicate declaration {name}"));
                }
                p.separator()?;
                continue;
            }
            if !p.keyword("fn") {
                return Err("expected use, struct or fn declaration".into());
            }
            let name = p.word()?;
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
        .map(|(name, structure)| (name, structure.public, &structure.fields))
        .collect();
    format!(
        "ZAM-IR-16\n{declarations:?}\n{:?}\n{:?}\n{:?}\n",
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
impl Checker<'_> {
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
            if !by_ref.unwrap_or(self.locals[&root].mutable) {
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
            Expr::Record(name, _) => {
                self.structs
                    .get(name)
                    .ok_or_else(|| format!("unknown struct {name}"))?;
                Ok(Type::Record(record_id(name)))
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
                    Type::Bool => Some(Elem::Bool),
                    Type::Owned | Type::Shared | Type::Mutable => Some(Elem::Str),
                    _ => None,
                };
                match (elem(self.ty(key)?), elem(self.ty(value)?)) {
                    (Some(key @ (Elem::Int | Elem::Str)), Some(value)) => Ok(Type::Map(key, value)),
                    _ => Err("unsupported map literal; keys are i64 or string, values are i64, bool or string".into()),
                }
            }
            Expr::BorrowIndex(_, _, mutable) => Ok(if *mutable {
                Type::Mutable
            } else {
                Type::Shared
            }),
            Expr::Method(path, method, _) => self.method(path, method).map(|(_, _, ty)| ty),
            Expr::Text(_) => Ok(Type::Owned),
            Expr::Format(_) => Ok(Type::Owned),
            Expr::Int(_) | Expr::ByteLen(_) | Expr::CharLen(_) => Ok(Type::Int),
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
            Expr::Unary(op, _) => Ok(if *op == '!' { Type::Bool } else { Type::Int }),
            Expr::Binary(op, _, _) => Ok(if matches!(op.as_str(), "+" | "-" | "*" | "/" | "%") {
                Type::Int
            } else {
                Type::Bool
            }),
        }
    }
    fn expr(
        &mut self,
        expr: &Expr,
        expected: Type,
        loans: &mut BTreeMap<String, Type>,
    ) -> Result<(), String> {
        match expr {
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
                if elem.ty() != expected {
                    return Err("type mismatch: element type differs".into());
                }
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
                if self.binding(name)?.ty != expected {
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
                    // 插值只读取变量，不移动；未支持的类型在检查阶段拒绝。
                    if !matches!(
                        self.ty(segment)?,
                        Type::Owned | Type::Shared | Type::Mutable | Type::Int | Type::Bool
                    ) {
                        return Err("cannot interpolate this value".into());
                    }
                }
                Ok(())
            }
            Expr::Text(_) if expected == Type::Owned => Ok(()),
            // 字面量与插值可以直接传给 `&string` 参数：借用一个只活到语句结束的临时字符串。
            Expr::Text(_) | Expr::Format(_) if expected == Type::Shared => {
                self.expr(expr, Type::Owned, loans)
            }
            Expr::Int(_) if expected == Type::Int => Ok(()),
            Expr::Bool(_) if expected == Type::Bool => Ok(()),
            Expr::Variable(name) if matches!(expected, Type::Int | Type::Bool) => {
                if self.binding(name)?.ty == expected {
                    Ok(())
                } else {
                    Err("type mismatch".into())
                }
            }
            Expr::Unary(op, value) => {
                let ty = if *op == '!' { Type::Bool } else { Type::Int };
                if expected != ty {
                    return Err("type mismatch".into());
                }
                self.expr(value, ty, loans)
            }
            Expr::Binary(op, left, right) => {
                let ty = self.ty(expr)?;
                if expected != ty {
                    return Err("type mismatch".into());
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
                let operand = if matches!(op.as_str(), "==" | "!=") {
                    self.ty(left)?
                } else {
                    Type::Int
                };
                if !matches!(operand, Type::Int | Type::Bool) {
                    return Err("comparison requires i64 or bool".into());
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
                if requested && !by_ref.unwrap_or(self.locals[&root].mutable) {
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
            Expr::BorrowIndex(path, index, mutable) => {
                if !matches!(expected, Type::Shared | Type::Mutable) {
                    return Err("borrow escape: references only belong in call arguments".into());
                }
                if (expected == Type::Mutable) != *mutable {
                    return Err(format!("borrow type mismatch for {path}[..]"));
                }
                let (key, elem) = Self::subscript(self.container(path, *mutable, loans)?);
                if elem != Elem::Str {
                    return Err("scalar references are not supported".into());
                }
                self.expr(index, key, loans)?;
                let root = path.split('.').next().unwrap_or(path).to_owned();
                loans.insert(root, expected);
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
                            ty: if elem == Elem::Str {
                                Type::Shared
                            } else {
                                elem.ty()
                            },
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
                    if !self.binding(local)?.mutable {
                        return Err(format!("cannot assign immutable value {local}"));
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
                    if matches!(ty, Type::Unit | Type::Record(_) | Type::ArrayInt(_))
                        || ty.container().is_some()
                    {
                        return Err("cannot print void, struct, array, list or map".into());
                    }
                    if let Expr::Variable(local) = expr {
                        self.binding(local)?;
                    } else if let Expr::Field(_, _) = expr {
                        self.ty(expr)?;
                    } else if let Expr::BorrowIndex(_, _, mutable) = expr {
                        self.expr(
                            expr,
                            if *mutable {
                                Type::Mutable
                            } else {
                                Type::Shared
                            },
                            &mut loans,
                        )?;
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

pub fn check(module: &Module, entry: &str) -> Result<(), String> {
    record_order(module, &module.structs.keys().cloned().collect())?;
    let main = module.functions.get(entry).ok_or("missing main function")?;
    if !main.params.is_empty() || main.result != Type::Unit {
        return Err("main must have no parameters or return value".into());
    }
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
        result.map_err(|error: String| {
            let (line, column) = checker.position;
            format!("line {line}, column {column}: {name}: {error}")
        })?;
    }
    Ok(())
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
