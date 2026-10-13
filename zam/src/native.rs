use crate::compiler::{
    field_path, record_id, record_order, Block, Expr, Instruction, Module, Type,
};
use crate::project::Result;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

#[path = "native_ranges.rs"]
mod ranges;

const RUNTIME: &str = r#"#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <inttypes.h>
#include <limits.h>
static void arithmetic_error(void) { fputs("zam: integer arithmetic error\n", stderr); exit(1); }
#if defined(__GNUC__) || defined(__clang__)
static int64_t add(int64_t a, int64_t b) {
    int64_t result;
    if (__builtin_add_overflow(a, b, &result)) arithmetic_error();
    return result;
}
static int64_t sub(int64_t a, int64_t b) {
    int64_t result;
    if (__builtin_sub_overflow(a, b, &result)) arithmetic_error();
    return result;
}
static int64_t mul(int64_t a, int64_t b) {
    int64_t result;
    if (__builtin_mul_overflow(a, b, &result)) arithmetic_error();
    return result;
}
#else
static int64_t add(int64_t a, int64_t b) {
    if ((b > 0 && a > INT64_MAX-b) || (b < 0 && a < INT64_MIN-b)) arithmetic_error();
    return a+b;
}
static int64_t sub(int64_t a, int64_t b) {
    if ((b < 0 && a > INT64_MAX+b) || (b > 0 && a < INT64_MIN+b)) arithmetic_error();
    return a-b;
}
static int64_t mul(int64_t a, int64_t b) {
    if (a > 0 ? (b > 0 ? a > INT64_MAX/b : b < INT64_MIN/a) :
        (a < 0 && (b > 0 ? a < INT64_MIN/b : b < 0 && a < INT64_MAX/b))) arithmetic_error();
    return a*b;
}
#endif
static int64_t quotient(int64_t a, int64_t b) {
    if (!b || (a == INT64_MIN && b == -1)) arithmetic_error();
#if defined(__GNUC__) || defined(__clang__)
    if (((uint64_t)a | (uint64_t)b) <= UINT32_MAX) return (int64_t)((uint32_t)a / (uint32_t)b);
#endif
    return a/b;
}
static int64_t remainder(int64_t a, int64_t b) {
    if (!b || (a == INT64_MIN && b == -1)) arithmetic_error();
#if defined(__GNUC__) || defined(__clang__)
    if (((uint64_t)a | (uint64_t)b) <= UINT32_MAX) return (int64_t)((uint32_t)a % (uint32_t)b);
#endif
    return a%b;
}
static int64_t checked_index(int64_t index, size_t length) {
    if (index < 0 || (uintmax_t)index >= length) { fputs("zam: array index out of bounds\n", stderr); exit(1); }
    return index;
}
#ifdef _WIN32
#include <io.h>
#include <fcntl.h>
#endif
static void output_mode(void) {
#ifdef _WIN32
    if (_setmode(_fileno(stdout), _O_BINARY) == -1) exit(1);
#endif
}
typedef struct { unsigned char *data; size_t len; } S;
static S own(const unsigned char *p, size_t n) {
    S s; s.data = (unsigned char*)malloc(n ? n : 1); s.len = n;
    if (!s.data) { fputs("zam: allocation failed\n", stderr); exit(1); }
    if (n) memcpy(s.data, p, n); return s;
}
static void drop(S *s) { free(s->data); s->data = NULL; s->len = 0; }
static S take(S *s) { S value = *s; s->data = NULL; s->len = 0; return value; }
static int64_t byte_len(S *s) { if ((uintmax_t)s->len > INT64_MAX) arithmetic_error(); return (int64_t)s->len; }
// ponytail: 按 UTF-8 续字节计数，不校验编码合法性；非法字节按其起始字节计一个字符。
static int64_t char_len(S *s) { int64_t n = 0; for (size_t i = 0; i < s->len; i++) { if ((s->data[i] & 0xC0) != 0x80) n++; } return n; }
static int string_equal(S *a, S *b) { return a->len == b->len && (!a->len || memcmp(a->data,b->data,a->len) == 0); }
static void print(S *s) {
    if ((s->len && fwrite(s->data, 1, s->len, stdout) != s->len) || fputc('\n', stdout) == EOF) exit(1);
}
// ponytail: 未捕获的错误只写 stderr 并以退出码 1 结束；不做栈回溯，错误值本身就是全部信息。
static void uncaught_error(S *s) {
    if (s->len && fwrite(s->data, 1, s->len, stderr) != s->len) exit(1);
    if (fputc('\n', stderr) == EOF) exit(1);
}
static S copy(S *s) { return own(s->data, s->len); }
static S cat(S a, S b) {
    size_t n = a.len + b.len;
    unsigned char *p = (unsigned char*)malloc(n ? n : 1);
    if (!p) { fputs("zam: allocation failed\n", stderr); exit(1); }
    if (a.len) memcpy(p, a.data, a.len);
    if (b.len) memcpy(p + a.len, b.data, b.len);
    drop(&a); drop(&b);
    S s; s.data = p; s.len = n; return s;
}
static S from_int(int64_t value) {
    char buffer[32];
    int n = snprintf(buffer, sizeof buffer, "%" PRId64, value);
    return own((const unsigned char*)buffer, (size_t)n);
}
static S from_bool(int value) {
    return own((const unsigned char*)(value ? "true" : "false"), value ? 4 : 5);
}
"#;

// 标准库里需要平台能力的函数共用的 C 辅助代码；某个目标文件只要用到任一内建函数就整段发射一次。
// Windows 不引入 <windows.h>（太重且宏会撞名），只声明用到的三个 kernel32 函数。
const STD_PRELUDE: &str = r#"#ifdef _WIN32
__declspec(dllimport) wchar_t *__stdcall GetCommandLineW(void);
__declspec(dllimport) int __stdcall MultiByteToWideChar(unsigned int, unsigned long, const char *, int, wchar_t *, int);
__declspec(dllimport) int __stdcall WideCharToMultiByte(unsigned int, unsigned long, const wchar_t *, int, char *, int, const char *, int *);
#endif
extern int zam_argc;
extern char **zam_argv;
static void zam_alloc_failed(void) { fputs("zam: allocation failed\n", stderr); exit(1); }
static void zam_string_bounds(void) { fputs("zam: string index out of bounds\n", stderr); exit(1); }
static S zam_message(const char *text, S *detail) {
    S head = own((const unsigned char*)text, strlen(text));
    return detail ? cat(head, copy(detail)) : head;
}
static void zam_stdin_binary(void) {
#ifdef _WIN32
    static int done;
    if (!done) { done = 1; _setmode(_fileno(stdin), _O_BINARY); }
#endif
}
// 路径按 UTF-8 解释；Windows 转成 UTF-16 走 _wfopen，中文路径才不会按本地代码页错读。
static FILE *zam_open(S *path, const char *mode) {
    FILE *file;
    size_t i;
    for (i = 0; i < path->len; i++) if (!path->data[i]) return NULL;
#ifdef _WIN32
    {
        int n = 0;
        wchar_t *wide, wmode[4];
        if (path->len > INT_MAX / 2) return NULL;
        if (path->len) {
            n = MultiByteToWideChar(65001, 8, (const char*)path->data, (int)path->len, NULL, 0);
            if (n <= 0) return NULL;
        }
        wide = (wchar_t*)malloc(((size_t)n + 1) * sizeof(wchar_t));
        if (!wide) zam_alloc_failed();
        if (n) MultiByteToWideChar(65001, 8, (const char*)path->data, (int)path->len, wide, n);
        wide[n] = 0;
        for (i = 0; mode[i] && i < 3; i++) wmode[i] = (wchar_t)mode[i];
        wmode[i] = 0;
        file = _wfopen(wide, wmode);
        free(wide);
    }
#else
    {
        char *name = (char*)malloc(path->len + 1);
        if (!name) zam_alloc_failed();
        if (path->len) memcpy(name, path->data, path->len);
        name[path->len] = 0;
        file = fopen(name, mode);
        free(name);
    }
#endif
    return file;
}
static int zam_read_all(FILE *file, S *out) {
    size_t cap = 4096, n = 0, got;
    unsigned char *p = (unsigned char*)malloc(cap), *q;
    if (!p) zam_alloc_failed();
    for (;;) {
        if (n == cap) {
            if (cap > SIZE_MAX / 2) { free(p); return 0; }
            q = (unsigned char*)realloc(p, cap * 2);
            if (!q) zam_alloc_failed();
            p = q; cap *= 2;
        }
        got = fread(p + n, 1, cap - n, file);
        if (!got) break;
        n += got;
    }
    if (ferror(file)) { free(p); return 0; }
    out->data = p; out->len = n;
    return 1;
}
#ifdef _WIN32
// 按 MSVC 运行库的规则切分 GetCommandLineW()，再转 UTF-8；main 收到的 argv 是本地代码页。
static S *zam_wargs;
static int64_t zam_wcount = -1;
static void zam_wpush(wchar_t **text, size_t *len, size_t *cap, wchar_t c) {
    if (*len == *cap) {
        wchar_t *next;
        *cap = *cap ? *cap * 2 : 16;
        next = (wchar_t*)realloc(*text, *cap * sizeof(wchar_t));
        if (!next) zam_alloc_failed();
        *text = next;
    }
    (*text)[(*len)++] = c;
}
static void zam_parse_args(void) {
    const wchar_t *p = GetCommandLineW();
    size_t count = 0, slots = 0;
    if (*p == L'"') { p++; while (*p && *p != L'"') p++; if (*p) p++; }
    else { while (*p && *p != L' ' && *p != L'\t') p++; }
    for (;;) {
        wchar_t *arg = NULL;
        size_t len = 0, cap = 0, slashes, i;
        int quoted = 0, n = 0;
        unsigned char *bytes;
        while (*p == L' ' || *p == L'\t') p++;
        if (!*p) break;
        while (*p && (quoted || (*p != L' ' && *p != L'\t'))) {
            slashes = 0;
            while (*p == L'\\') { slashes++; p++; }
            if (*p == L'"') {
                for (i = 0; i < slashes / 2; i++) zam_wpush(&arg, &len, &cap, L'\\');
                if (slashes % 2) { zam_wpush(&arg, &len, &cap, L'"'); p++; }
                else if (quoted && p[1] == L'"') { zam_wpush(&arg, &len, &cap, L'"'); p += 2; }
                else { quoted = !quoted; p++; }
            } else {
                for (i = 0; i < slashes; i++) zam_wpush(&arg, &len, &cap, L'\\');
                if (*p && (quoted || (*p != L' ' && *p != L'\t'))) zam_wpush(&arg, &len, &cap, *p++);
            }
        }
        if (len) n = WideCharToMultiByte(65001, 0, arg, (int)len, NULL, 0, NULL, NULL);
        bytes = (unsigned char*)malloc(n ? (size_t)n : 1);
        if (!bytes) zam_alloc_failed();
        if (n) WideCharToMultiByte(65001, 0, arg, (int)len, (char*)bytes, n, NULL, NULL);
        free(arg);
        if (count == slots) {
            S *next;
            slots = slots ? slots * 2 : 8;
            next = (S*)realloc(zam_wargs, slots * sizeof(S));
            if (!next) zam_alloc_failed();
            zam_wargs = next;
        }
        zam_wargs[count].data = bytes;
        zam_wargs[count].len = (size_t)n;
        count++;
    }
    zam_wcount = (int64_t)count;
}
#endif
static int64_t zam_arg_count(void) {
#ifdef _WIN32
    if (zam_wcount < 0) zam_parse_args();
    return zam_wcount;
#else
    return zam_argc > 1 ? (int64_t)zam_argc - 1 : 0;
#endif
}
static S zam_arg(int64_t index) {
#ifdef _WIN32
    return copy(&zam_wargs[index]);
#else
    const char *text = zam_argv[index + 1];
    return own((const unsigned char*)text, strlen(text));
#endif
}
"#;

/// 标准库函数：签名进入类型检查，函数体是 C，按 `static` 发射到用到它的每个目标文件里。
/// 参数在 C 里叫 `p0`、`p1`…；可失败函数用 `F_<类型>_ok/err` 构造结果。
pub struct Builtin {
    pub name: &'static str,
    params: &'static [Type],
    result: Type,
    fallible: bool,
    body: &'static str,
}
const BUILTINS: &[Builtin] = &[
    Builtin {
        name: "std/io:read_line",
        params: &[Type::Mutable],
        result: Type::Bool,
        fallible: true,
        body: r#"size_t cap = 64, n = 0; int c, any = 0;
unsigned char *p = (unsigned char*)malloc(cap), *q;
if (!p) zam_alloc_failed();
zam_stdin_binary();
while ((c = getc(stdin)) != EOF) {
    any = 1;
    if (c == '\n') break;
    if (n == cap) { q = (unsigned char*)realloc(p, cap * 2); if (!q) zam_alloc_failed(); p = q; cap *= 2; }
    p[n++] = (unsigned char)c;
}
if (ferror(stdin)) { free(p); return F_int_err(zam_message("cannot read standard input", NULL)); }
if (n && p[n - 1] == '\r') n--;
drop(p0); p0->data = p; p0->len = n;
return F_int_ok(any);"#,
    },
    Builtin {
        name: "std/io:read_all",
        params: &[],
        result: Type::Owned,
        fallible: true,
        body: r#"S text;
zam_stdin_binary();
if (!zam_read_all(stdin, &text)) return F_S_err(zam_message("cannot read standard input", NULL));
return F_S_ok(text);"#,
    },
    Builtin {
        name: "std/fs:read",
        params: &[Type::Shared],
        result: Type::Owned,
        fallible: true,
        body: r#"S text; int ok;
FILE *file = zam_open(p0, "rb");
if (!file) return F_S_err(zam_message("cannot open ", p0));
ok = zam_read_all(file, &text);
fclose(file);
if (!ok) return F_S_err(zam_message("cannot read ", p0));
return F_S_ok(text);"#,
    },
    Builtin {
        name: "std/fs:write",
        params: &[Type::Shared, Type::Shared],
        result: Type::Unit,
        fallible: true,
        body: r#"int ok;
FILE *file = zam_open(p0, "wb");
if (!file) return F_void_err(zam_message("cannot open ", p0));
ok = !p1->len || fwrite(p1->data, 1, p1->len, file) == p1->len;
if (fclose(file) != 0) ok = 0;
if (!ok) return F_void_err(zam_message("cannot write ", p0));
return F_void_ok();"#,
    },
    Builtin {
        name: "std/env:arg_count",
        params: &[],
        result: Type::Int,
        fallible: false,
        body: "return zam_arg_count();",
    },
    Builtin {
        name: "std/env:arg",
        params: &[Type::Int],
        result: Type::Owned,
        fallible: true,
        body: r#"if (p0 < 0 || p0 >= zam_arg_count()) return F_S_err(zam_message("argument index out of range", NULL));
return F_S_ok(zam_arg(p0));"#,
    },
    Builtin {
        name: "std/string:parse_i64",
        params: &[Type::Shared],
        result: Type::Int,
        fallible: true,
        body: r#"size_t i = 0; int negative = 0; uint64_t limit, value = 0, digit;
if (p0->len && (p0->data[0] == '-' || p0->data[0] == '+')) { negative = p0->data[0] == '-'; i = 1; }
if (i == p0->len) return F_int64_t_err(zam_message("invalid integer: ", p0));
limit = negative ? (uint64_t)INT64_MAX + 1 : (uint64_t)INT64_MAX;
for (; i < p0->len; i++) {
    if (p0->data[i] < '0' || p0->data[i] > '9') return F_int64_t_err(zam_message("invalid integer: ", p0));
    digit = (uint64_t)(p0->data[i] - '0');
    if (value > (limit - digit) / 10) return F_int64_t_err(zam_message("integer out of range: ", p0));
    value = value * 10 + digit;
}
if (!negative) return F_int64_t_ok((int64_t)value);
return F_int64_t_ok(value == (uint64_t)INT64_MAX + 1 ? INT64_MIN : -(int64_t)value);"#,
    },
    Builtin {
        name: "std/string:trim",
        params: &[Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: r#"size_t a = 0, b = p0->len;
while (a < b && (p0->data[a] == ' ' || p0->data[a] == '\t' || p0->data[a] == '\n' || p0->data[a] == '\r')) a++;
while (b > a && (p0->data[b - 1] == ' ' || p0->data[b - 1] == '\t' || p0->data[b - 1] == '\n' || p0->data[b - 1] == '\r')) b--;
return own(p0->data + a, b - a);"#,
    },
    Builtin {
        name: "std/string:byte_at",
        params: &[Type::Shared, Type::Int],
        result: Type::Int,
        fallible: false,
        body: r#"if (p1 < 0 || (uint64_t)p1 >= p0->len) zam_string_bounds();
return p0->data[p1];"#,
    },
    Builtin {
        name: "std/string:slice",
        params: &[Type::Shared, Type::Int, Type::Int],
        result: Type::Owned,
        fallible: false,
        body: r#"if (p1 < 0 || p2 < p1 || (uint64_t)p2 > p0->len) zam_string_bounds();
// 不能从 UTF-8 多字节字符中间切开。
if (((uint64_t)p1 < p0->len && (p0->data[p1] & 0xC0) == 0x80) || ((uint64_t)p2 < p0->len && (p0->data[p2] & 0xC0) == 0x80)) zam_string_bounds();
return own(p0->data + p1, (size_t)(p2 - p1));"#,
    },
    Builtin {
        name: "std/string:find",
        params: &[Type::Shared, Type::Shared],
        result: Type::Int,
        fallible: false,
        body: r#"size_t i;
if (!p1->len) return 0;
if (p1->len > p0->len) return -1;
for (i = 0; i + p1->len <= p0->len; i++) if (memcmp(p0->data + i, p1->data, p1->len) == 0) return (int64_t)i;
return -1;"#,
    },
];
pub fn builtin(name: &str) -> Option<&'static Builtin> {
    BUILTINS.iter().find(|builtin| builtin.name == name)
}
pub fn builtin_functions() -> impl Iterator<Item = (String, crate::compiler::Function)> {
    BUILTINS.iter().map(|builtin| {
        (
            builtin.name.to_owned(),
            crate::compiler::Function {
                public: true,
                params: builtin
                    .params
                    .iter()
                    .enumerate()
                    .map(|(index, ty)| (format!("p{index}"), *ty))
                    .collect(),
                result: builtin.result,
                error: builtin.fallible.then_some(Type::Owned),
                body: Vec::new(),
                position: (1, 1),
                builtin: true,
            },
        )
    })
}

struct Generator<'a> {
    module: &'a Module,
    symbols: &'a BTreeMap<String, String>,
    locals: BTreeMap<String, (String, Type)>,
    owners: Vec<(String, Type)>,
    loops: Vec<usize>,
    proofs: ranges::Proofs,
    call_targets: BTreeMap<usize, String>,
    code: String,
    next: usize,
    // 当前函数的签名：`?` 提前返回时要构造同样形状的结果结构体。
    result: Type,
    error: Option<Type>,
}
// 失败函数的结果结构体：ok=1 时 value 有效，ok=0 时 error 有效。
fn fallible_name(ty: Type) -> String {
    format!("F_{}", ctype(ty))
}
fn fallible_ok(ty: Type) -> String {
    format!("{}_ok", fallible_name(ty))
}
fn fallible_err(ty: Type) -> String {
    format!("{}_err", fallible_name(ty))
}
fn drop_fn(ty: Type) -> String {
    if matches!(ty, Type::Record(_)) {
        format!("drop{}", ctype(ty))
    } else {
        "drop".into()
    }
}
fn ctype(ty: Type) -> String {
    match ty {
        Type::Record(id) => return format!("R{id}"),
        Type::Owned => "S",
        Type::Shared | Type::Mutable => "S*",
        Type::Int => "int64_t",
        Type::Bool => "int",
        Type::ArrayInt(length) => return format!("int64_t[{length}]"),
        Type::Unit => "void",
    }
    .into()
}
impl Generator<'_> {
    fn ty(&self, expr: &Expr) -> Type {
        match expr {
            Expr::Record(name, _) => Type::Record(record_id(name)),
            Expr::Field(local, field) => {
                field_path(&self.module.structs, self.locals[local].1, field)
                    .expect("checked field path")
                    .1
            }
            Expr::Int(_) | Expr::ByteLen(_) | Expr::CharLen(_) => Type::Int,
            Expr::Array(values) => Type::ArrayInt(values.len()),
            Expr::Index(local, _) => match self.locals[local].1 {
                Type::ArrayInt(_) => Type::Int,
                _ => unreachable!(),
            },
            Expr::Bool(_) | Expr::StringEqual(_, _) => Type::Bool,
            Expr::Text(_) | Expr::Format(_) => Type::Owned,
            Expr::Variable(name) => self.locals[name].1,
            Expr::Borrow(_, mutable) => {
                if *mutable {
                    Type::Mutable
                } else {
                    Type::Shared
                }
            }
            Expr::Call(name, _) => self.module.functions[name].result,
            Expr::Try(inner) => self.ty(inner),
            Expr::Catch(inner, _, _) => self.ty(inner),
            Expr::Unary(op, _) => {
                if *op == '!' {
                    Type::Bool
                } else {
                    Type::Int
                }
            }
            Expr::Binary(op, _, _) => {
                if matches!(op.as_str(), "+" | "-" | "*" | "/" | "%") {
                    Type::Int
                } else {
                    Type::Bool
                }
            }
        }
    }
    fn scalar(&mut self, value: &str, ty: Type) -> String {
        let name = format!("v{}", self.next);
        self.next += 1;
        self.code
            .push_str(&format!("{} {name} = {value};\n", ctype(ty)));
        name
    }
    fn temporary(&mut self, value: &str) -> String {
        let name = format!("v{}", self.next);
        self.next += 1;
        self.code.push_str(&format!("S {name} = {value};\n"));
        self.owners.push((name.clone(), Type::Owned));
        name
    }
    fn literal(&self, text: &str) -> String {
        let bytes = text
            .as_bytes()
            .iter()
            .map(|b| format!("\\x{b:02x}"))
            .collect::<String>();
        format!("own((const unsigned char*)\"{bytes}\", {})", text.len())
    }
    fn text_of(&self, pointer: String, ty: Type) -> String {
        match ty {
            Type::Owned | Type::Shared | Type::Mutable => format!("copy({pointer})"),
            Type::Int => format!("from_int({pointer})"),
            _ => format!("from_bool({pointer})"),
        }
    }
    fn pointer(&self, name: &str) -> String {
        if let Some((root, field)) = name.split_once('.') {
            return format!("&{}", self.field(root, field));
        }
        let (local, ty) = &self.locals[name];
        if matches!(ty, Type::Owned | Type::Record(_)) {
            format!("&{local}")
        } else {
            local.clone()
        }
    }
    fn field(&self, local: &str, field: &str) -> String {
        let (indices, _) = field_path(&self.module.structs, self.locals[local].1, field)
            .expect("checked field path");
        let mut target = self.locals[local].0.clone();
        for index in indices {
            target.push_str(&format!(".m{index}"));
        }
        target
    }
    fn expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Record(name, fields) => {
                let ty = Type::Record(record_id(name));
                let local = self.scalar("{0}", ty);
                self.owners.push((local.clone(), ty));
                let declaration = &self.module.structs[name];
                for (field, expr) in fields {
                    let index = declaration
                        .fields
                        .iter()
                        .position(|(n, _)| n == field)
                        .unwrap();
                    let ty = self.ty(expr);
                    let value = self.expr(expr);
                    self.code.push_str(&format!(
                        "{local}.m{index} = {};\n",
                        if let Type::Record(_) = ty {
                            format!("take{}(&{value})", ctype(ty))
                        } else if ty == Type::Owned {
                            format!("take(&{value})")
                        } else {
                            value
                        }
                    ));
                }
                local
            }
            Expr::Field(local, field) => self.scalar(&self.field(local, field), self.ty(expr)),
            Expr::Variable(name) if matches!(self.locals[name].1, Type::Record(_)) => {
                let ty = self.locals[name].1;
                let local = self.scalar(&format!("take{}({})", ctype(ty), self.pointer(name)), ty);
                self.owners.push((local.clone(), ty));
                local
            }
            Expr::StringEqual(left, right) => {
                let pointer = |expr: &Expr| match expr {
                    Expr::Variable(name) | Expr::Borrow(name, _) => self.pointer(name),
                    _ => unreachable!("checked shared string"),
                };
                self.scalar(
                    &format!("string_equal({}, {})", pointer(left), pointer(right)),
                    Type::Bool,
                )
            }
            Expr::ByteLen(value) => {
                let pointer = match value.as_ref() {
                    Expr::Variable(name) | Expr::Borrow(name, _) => self.pointer(name),
                    _ => unreachable!("checked shared string"),
                };
                self.scalar(&format!("byte_len({pointer})"), Type::Int)
            }
            Expr::CharLen(value) => {
                let pointer = match value.as_ref() {
                    Expr::Variable(name) | Expr::Borrow(name, _) => self.pointer(name),
                    _ => unreachable!("checked shared string"),
                };
                self.scalar(&format!("char_len({pointer})"), Type::Int)
            }
            Expr::Array(values) => {
                let length = values.len();
                let name = format!("v{}", self.next);
                self.next += 1;
                self.code
                    .push_str(&format!("int64_t {name}[{length}] = {{0}};\n"));
                for (index, value) in values.iter().enumerate() {
                    let rendered = self.expr(value);
                    self.code
                        .push_str(&format!("{name}[{index}] = {rendered};\n"));
                }
                name
            }
            Expr::Index(local, index) => {
                let index = self.expr(index);
                let length = match self.locals[local].1 {
                    Type::ArrayInt(length) => length,
                    _ => unreachable!(),
                };
                let index = if self.proofs.index(expr) {
                    index
                } else {
                    format!("checked_index({index}, {length})")
                };
                self.scalar(&format!("{}[{index}]", self.locals[local].0), Type::Int)
            }
            Expr::Text(text) => self.temporary(&self.literal(text)),
            Expr::Format(segments) => {
                let mut parts = Vec::with_capacity(segments.len());
                for segment in segments {
                    parts.push(match segment {
                        Expr::Text(text) => self.literal(text),
                        Expr::Variable(name) => {
                            let pointer = self.pointer(name);
                            self.text_of(pointer, self.ty(segment))
                        }
                        Expr::Field(local, field) => {
                            let ty = self.ty(segment);
                            let target = self.field(local, field);
                            let target = if matches!(ty, Type::Int | Type::Bool) {
                                target
                            } else {
                                format!("&{target}")
                            };
                            self.text_of(target, ty)
                        }
                        _ => unreachable!("interpolation only produces text and field paths"),
                    });
                }
                let mut parts = parts.into_iter();
                let name = self.temporary(&parts.next().expect("interpolation has segments"));
                for part in parts {
                    self.code
                        .push_str(&format!("{name} = cat({name}, {part});\n"));
                }
                name
            }
            Expr::Int(value) => self.scalar(
                &if *value == i64::MIN {
                    "INT64_MIN".into()
                } else {
                    format!("INT64_C({value})")
                },
                Type::Int,
            ),
            Expr::Bool(value) => self.scalar(if *value { "1" } else { "0" }, Type::Bool),
            Expr::Unary(op, value) => {
                let ty = self.ty(expr);
                let value = self.expr(value);
                self.scalar(
                    &if *op == '-' && self.proofs.arithmetic(expr) {
                        format!("(-{value})")
                    } else if *op == '-' {
                        format!("sub(0, {value})")
                    } else {
                        format!("!{value}")
                    },
                    ty,
                )
            }
            Expr::Binary(op, left, right) => {
                let ty = self.ty(expr);
                // Only zero and -1 can make signed division or remainder invalid.
                let safe_divisor =
                    matches!(right.as_ref(), Expr::Int(value) if !matches!(*value, 0 | -1));
                // Dynamic divisors retain the runtime's guarded 32-bit fast path.
                // Unsigned power-of-two reduction exposes a precise result range;
                // other constants retain signed reduction to avoid slower magic division.
                let unsigned_constant_divisor = matches!(right.as_ref(), Expr::Int(value) if *value > 0 && (*value as u64).is_power_of_two());
                let left = self.expr(left);
                if matches!(op.as_str(), "&&" | "||") {
                    let value = self.scalar(&left, Type::Bool);
                    self.code.push_str(&format!(
                        "if ({}{value}) {{\n",
                        if op == "||" { "!" } else { "" }
                    ));
                    let scope_start = self.owners.len();
                    let right = self.expr(right);
                    self.code.push_str(&format!("{value} = {right};\n"));
                    for (name, ty) in &self.owners[scope_start..] {
                        self.code.push_str(&format!("{}(&{name});\n", drop_fn(*ty)));
                    }
                    self.owners.truncate(scope_start);
                    self.code.push_str("}\n");
                    return value;
                }
                let right = self.expr(right);
                if matches!(op.as_str(), "/" | "%")
                    && unsigned_constant_divisor
                    && self.proofs.unsigned_division(expr)
                {
                    // Both operands are nonnegative and the divisor is positive.
                    // Their quotient/remainder fits i64, so the cast back is exact.
                    return self.scalar(
                        &format!("(int64_t)((uint64_t){left} {op} (uint64_t){right})"),
                        ty,
                    );
                }
                let function = match op.as_str() {
                    "+" if !self.proofs.arithmetic(expr) => Some("add"),
                    "-" if !self.proofs.arithmetic(expr) => Some("sub"),
                    "*" if !self.proofs.arithmetic(expr) => Some("mul"),
                    "/" if !safe_divisor => Some("quotient"),
                    "%" if !safe_divisor => Some("remainder"),
                    _ => None,
                };
                self.scalar(
                    &if let Some(function) = function {
                        format!("{function}({left}, {right})")
                    } else {
                        format!("({left} {op} {right})")
                    },
                    ty,
                )
            }
            Expr::Variable(name) if matches!(self.locals[name].1, Type::Int | Type::Bool) => {
                let (value, ty) = self.locals[name].clone();
                self.scalar(&value, ty)
            }
            Expr::Variable(name) => self.temporary(&format!("take({})", self.pointer(name))),
            Expr::Borrow(name, _) => self.pointer(name),
            Expr::Call(name, args) => {
                let result = self.module.functions[name].result;
                let symbol = self
                    .call_targets
                    .get(&(expr as *const Expr as usize))
                    .cloned();
                let call = self.raw_call(name, args, symbol.as_deref());
                self.call_value(&call, result)
            }
            Expr::Try(inner) => {
                let Expr::Call(name, args) = &**inner else {
                    unreachable!("checked ? operand")
                };
                let result = self.module.functions[name].result;
                let call = self.raw_call(name, args, None);
                let structure = fallible_name(result);
                let temp = format!("v{}", self.next);
                self.next += 1;
                self.code
                    .push_str(&format!("{structure} {temp} = {call};\n"));
                // 失败：先把当前作用域的值都释放掉，再构造同样形状的错误结果返回。
                self.code.push_str(&format!("if (!{temp}.ok) {{\n"));
                let outer = fallible_err(self.result);
                self.cleanup();
                self.code
                    .push_str(&format!("return {outer}(take(&{temp}.error));\n}}\n"));
                if result == Type::Unit {
                    return String::new();
                }
                self.result_value(&temp, result)
            }
            Expr::Catch(inner, binding, block) => {
                let Expr::Call(name, args) = &**inner else {
                    unreachable!("checked catch operand")
                };
                let result = self.module.functions[name].result;
                let call = self.raw_call(name, args, None);
                let structure = fallible_name(result);
                let temp = format!("v{}", self.next);
                self.next += 1;
                self.code
                    .push_str(&format!("{structure} {temp} = {call};\n"));
                self.code.push_str(&format!("if (!{temp}.ok) {{\n"));
                let local = format!("v{}", self.next);
                self.next += 1;
                let previous = self
                    .locals
                    .insert(binding.clone(), (local.clone(), Type::Owned));
                self.code
                    .push_str(&format!("S {local} = take(&{temp}.error);\n"));
                self.owners.push((local, Type::Owned));
                // 代码块在所有路径上都 return（检查器保证），所以出块只需要清账本。
                self.block(block, self.result);
                self.owners.pop();
                self.locals.remove(binding);
                if let Some(previous) = previous {
                    self.locals.insert(binding.clone(), previous);
                }
                self.code.push_str("}\n");
                self.result_value(&temp, result)
            }
        }
    }
    // 从结果结构体里取出成功值；Unit 没有值，返回空串。
    fn result_value(&mut self, temp: &str, result: Type) -> String {
        if result == Type::Unit {
            return String::new();
        }
        if matches!(result, Type::Record(_)) {
            let value = self.scalar(&format!("take{}(&{temp}.value)", ctype(result)), result);
            self.owners.push((value.clone(), result));
            value
        } else if result == Type::Owned {
            self.temporary(&format!("take(&{temp}.value)"))
        } else {
            self.scalar(&format!("{temp}.value"), result)
        }
    }
    fn raw_call(&mut self, name: &str, args: &[Expr], symbol: Option<&str>) -> String {
        let function = &self.module.functions[name];
        let types: Vec<_> = function.params.iter().map(|(_, ty)| *ty).collect();
        let mut values = Vec::new();
        for (arg, ty) in args.iter().zip(types) {
            values.push(if let Type::Record(_) = ty {
                let value = self.expr(arg);
                format!("take{}(&{value})", ctype(ty))
            } else if ty == Type::Owned {
                let value = self.expr(arg);
                format!("take(&{value})")
            } else if matches!(ty, Type::Int | Type::Bool) {
                self.expr(arg)
            } else {
                match arg {
                    Expr::Variable(local) | Expr::Borrow(local, _) => self.pointer(local),
                    _ => unreachable!(),
                }
            });
        }
        format!(
            "{}({})",
            symbol.unwrap_or(&self.symbols[name]),
            values.join(", ")
        )
    }
    fn call_value(&mut self, call: &str, result: Type) -> String {
        if result == Type::Owned {
            self.temporary(call)
        } else if result != Type::Unit {
            let value = self.scalar(call, result);
            if matches!(result, Type::Record(_)) {
                self.owners.push((value.clone(), result));
            }
            value
        } else {
            self.code.push_str(&format!("{call};\n"));
            String::new()
        }
    }
    fn block(&mut self, body: &Block, result_type: Type) -> bool {
        let outer = self.locals.clone();
        let scope_start = self.owners.len();
        let mut returned = false;
        for (instruction, _) in body {
            match instruction {
                Instruction::If(condition, yes, no) => {
                    let value = self.expr(condition);
                    self.code.push_str(&format!("if ({value}) {{\n"));
                    let yes_returns = self.block(yes, result_type);
                    self.code.push_str("} else {\n");
                    let no_returns = self.block(no, result_type);
                    self.code.push_str("}\n");
                    returned = yes_returns && no_returns;
                }
                Instruction::While(condition, nested) => {
                    let invariant = self.invariant_condition(condition, nested);
                    let hoisted = if invariant {
                        let Expr::Binary(_, _, right) = condition else {
                            unreachable!("invariant condition validated as comparison")
                        };
                        Some(self.expr(right))
                    } else {
                        None
                    };
                    self.code.push_str("while (1) {\n");
                    let owners = self.owners.len();
                    let value = if let Some(right) = hoisted {
                        let Expr::Binary(op, left, _) = condition else {
                            unreachable!("invariant condition validated as comparison")
                        };
                        let left = self.expr(left);
                        self.scalar(&format!("({left} {op} {right})"), Type::Bool)
                    } else {
                        self.expr(condition)
                    };
                    self.code.push_str(&format!("int condition = {value};\n"));
                    for (name, ty) in &self.owners[owners..] {
                        self.code.push_str(&format!("{}(&{name});\n", drop_fn(*ty)));
                    }
                    self.owners.truncate(owners);
                    self.code.push_str("if (!condition) break;\n");
                    self.loops.push(self.owners.len());
                    self.block(nested, result_type);
                    self.loops.pop();
                    self.code.push_str("}\n");
                }
                Instruction::For(name, start, end, nested) => {
                    // 起止按源码顺序各求值一次；counter < end <= INT64_MAX，自增不会溢出。
                    let start = self.expr(start);
                    let end = self.expr(end);
                    let counter = format!("v{}", self.next);
                    self.next += 1;
                    self.code.push_str(&format!(
                        "for (int64_t {counter} = {start}; {counter} < {end}; {counter}++) {{\n"
                    ));
                    self.locals.insert(name.clone(), (counter, Type::Int));
                    self.loops.push(self.owners.len());
                    self.block(nested, result_type);
                    self.loops.pop();
                    self.locals.remove(name);
                    self.code.push_str("}\n");
                }
                Instruction::Let(name, _, _, expr) => {
                    let ty = self.ty(expr);
                    let value = self.expr(expr);
                    self.locals.insert(name.clone(), (value, ty));
                }
                Instruction::AssignIndex(local, index, expr) => {
                    let index = self.expr(index);
                    let value = self.expr(expr);
                    let length = match self.locals[local].1 {
                        Type::ArrayInt(length) => length,
                        _ => unreachable!(),
                    };
                    let index = if self.proofs.assignment_index(instruction) {
                        index
                    } else {
                        format!("checked_index({index}, {length})")
                    };
                    self.code
                        .push_str(&format!("{}[{index}] = {value};\n", self.locals[local].0));
                }
                Instruction::AssignField(local, field, expr) => {
                    let value = self.expr(expr);
                    let target = self.field(local, field);
                    let ty = self.ty(expr);
                    if matches!(ty, Type::Record(_)) {
                        self.code.push_str(&format!(
                            "{}(&{target}); {target} = take{}(&{value});\n",
                            drop_fn(ty),
                            ctype(ty)
                        ));
                    } else if ty == Type::Owned {
                        self.code
                            .push_str(&format!("drop(&{target}); {target} = take(&{value});\n"));
                    } else {
                        self.code.push_str(&format!("{target} = {value};\n"));
                    }
                }
                Instruction::Assign(name, expr) => {
                    let value = self.expr(expr);
                    if matches!(self.locals[name].1, Type::Int | Type::Bool) {
                        self.code
                            .push_str(&format!("{} = {value};\n", self.locals[name].0));
                        continue;
                    }
                    let ty = self.locals[name].1;
                    let pointer = self.pointer(name);
                    if matches!(ty, Type::Record(_)) {
                        self.code.push_str(&format!(
                            "{}({pointer}); *({pointer}) = take{}(&{value});\n",
                            drop_fn(ty),
                            ctype(ty)
                        ));
                        continue;
                    }
                    self.code.push_str(&format!(
                        "drop({pointer});\n*({pointer}) = take(&{value});\n"
                    ));
                }
                Instruction::Print(expr) => {
                    let ty = self.ty(expr);
                    if matches!(ty, Type::Int | Type::Bool) {
                        let value = self.expr(expr);
                        self.code.push_str(&if ty == Type::Int {
                            format!("if (printf(\"%\" PRId64 \"\\n\", {value}) < 0) exit(1);\n")
                        } else {
                            format!("if (puts({value} ? \"true\" : \"false\") == EOF) exit(1);\n")
                        });
                        continue;
                    }
                    let pointer = match expr {
                        Expr::Variable(name) | Expr::Borrow(name, _) => self.pointer(name),
                        Expr::Field(local, field) => format!("&{}", self.field(local, field)),
                        _ => {
                            let value = self.expr(expr);
                            format!("&{value}")
                        }
                    };
                    self.code.push_str(&format!("print({pointer});\n"));
                }
                Instruction::Call(expr) => {
                    self.expr(expr);
                }
                Instruction::Break => {
                    self.cleanup_from(*self.loops.last().expect("checked loop jump"));
                    self.code.push_str("break;\n");
                    returned = true;
                }
                Instruction::Continue => {
                    self.cleanup_from(*self.loops.last().expect("checked loop jump"));
                    self.code.push_str("continue;\n");
                    returned = true;
                }
                Instruction::Return(expr) => {
                    if let Some(expr) = expr {
                        let value = self.expr(expr);
                        self.code
                            .push_str(&if matches!(result_type, Type::Record(_)) {
                                format!(
                                    "{} result = take{}(&{value});\n",
                                    ctype(result_type),
                                    ctype(result_type)
                                )
                            } else if result_type == Type::Owned {
                                format!("S result = take(&{value});\n")
                            } else {
                                format!("{} result = {value};\n", ctype(result_type))
                            });
                    }
                    self.cleanup();
                    if self.error.is_some() {
                        // 失败函数：成功值要包进结果结构体。
                        let ok = fallible_ok(result_type);
                        self.code.push_str(&if expr.is_some() {
                            format!("return {ok}(result);\n")
                        } else {
                            format!("return {ok}();\n")
                        });
                    } else {
                        self.code.push_str(if expr.is_some() {
                            "return result;\n"
                        } else {
                            "return;\n"
                        });
                    }
                    returned = true;
                }
                Instruction::Fail(expr) => {
                    let value = self.expr(expr);
                    self.code
                        .push_str(&format!("S failure = take(&{value});\n"));
                    self.cleanup();
                    self.code
                        .push_str(&format!("return {}(failure);\n", fallible_err(self.result)));
                    returned = true;
                }
            }
        }
        if !returned {
            for (name, ty) in &self.owners[scope_start..] {
                self.code.push_str(&format!("{}(&{name});\n", drop_fn(*ty)));
            }
        }
        self.owners.truncate(scope_start);
        self.locals = outer;
        returned
    }
    fn cleanup_from(&mut self, start: usize) {
        for (name, ty) in &self.owners[start..] {
            self.code.push_str(&format!("{}(&{name});\n", drop_fn(*ty)));
        }
    }
    fn cleanup(&mut self) {
        self.cleanup_from(0);
    }

    fn scalar_invariant_expr(
        &self,
        expr: &Expr,
        mutated: &std::collections::BTreeSet<String>,
    ) -> bool {
        match expr {
            Expr::Int(_) | Expr::Bool(_) => true,
            Expr::Variable(name) => {
                matches!(
                    self.locals.get(name).map(|(_, ty)| ty),
                    Some(Type::Int | Type::Bool)
                ) && !mutated.contains(name)
            }
            Expr::Unary(op, value) => *op == '-' && self.scalar_invariant_expr(value, mutated),
            Expr::Binary(op, left, right) => {
                matches!(op.as_str(), "+" | "-" | "*" | "/" | "%")
                    && self.scalar_invariant_expr(left, mutated)
                    && self.scalar_invariant_expr(right, mutated)
            }
            _ => false,
        }
    }

    fn loop_mutations(body: &Block, mutated: &mut std::collections::BTreeSet<String>) {
        for (instruction, _) in body {
            match instruction {
                Instruction::Assign(name, _) => {
                    mutated.insert(name.clone());
                }
                Instruction::AssignField(name, _, _) | Instruction::AssignIndex(name, _, _) => {
                    mutated.insert(name.clone());
                }
                Instruction::If(_, yes, no) => {
                    Self::loop_mutations(yes, mutated);
                    Self::loop_mutations(no, mutated);
                }
                Instruction::While(_, nested) | Instruction::For(_, _, _, nested) => {
                    Self::loop_mutations(nested, mutated)
                }
                _ => {}
            }
        }
    }

    fn invariant_condition(&self, condition: &Expr, body: &Block) -> bool {
        let Expr::Binary(op, left, right) = condition else {
            return false;
        };
        if !matches!(op.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=") {
            return false;
        }
        if !matches!(
            left.as_ref(),
            Expr::Int(_) | Expr::Bool(_) | Expr::Variable(_)
        ) {
            return false;
        }
        if !matches!(right.as_ref(), Expr::Unary(_, _) | Expr::Binary(_, _, _)) {
            return false;
        }
        let mut mutated = std::collections::BTreeSet::new();
        Self::loop_mutations(body, &mut mutated);
        self.scalar_invariant_expr(right, &mutated)
    }
}

// Visit every expression in AST source order; callers select the nodes they need.
fn visit_exprs<'a>(body: &'a Block, visit: &mut impl FnMut(&'a Expr)) {
    fn expr<'a>(value: &'a Expr, visit: &mut impl FnMut(&'a Expr)) {
        visit(value);
        match value {
            Expr::Call(_, args) | Expr::Array(args) | Expr::Format(args) => {
                for arg in args {
                    expr(arg, visit);
                }
            }
            Expr::Unary(_, inner)
            | Expr::Try(inner)
            | Expr::ByteLen(inner)
            | Expr::CharLen(inner)
            | Expr::Index(_, inner) => expr(inner, visit),
            Expr::Binary(_, a, b) | Expr::StringEqual(a, b) => {
                expr(a, visit);
                expr(b, visit);
            }
            Expr::Catch(inner, _, body) => {
                expr(inner, visit);
                visit_exprs(body, visit);
            }
            Expr::Record(_, fields) => {
                for (_, value) in fields {
                    expr(value, visit);
                }
            }
            _ => {}
        }
    }
    for (instruction, _) in body {
        match instruction {
            Instruction::Let(_, _, _, value)
            | Instruction::Assign(_, value)
            | Instruction::AssignField(_, _, value)
            | Instruction::Print(value)
            | Instruction::Call(value)
            | Instruction::Fail(value)
            | Instruction::Return(Some(value)) => expr(value, visit),
            Instruction::AssignIndex(_, index, value) => {
                expr(index, visit);
                expr(value, visit);
            }
            Instruction::If(condition, yes, no) => {
                expr(condition, visit);
                visit_exprs(yes, visit);
                visit_exprs(no, visit);
            }
            Instruction::While(condition, body) => {
                expr(condition, visit);
                visit_exprs(body, visit);
            }
            Instruction::For(_, start, end, body) => {
                expr(start, visit);
                expr(end, visit);
                visit_exprs(body, visit);
            }
            _ => {}
        }
    }
}
struct Specialization {
    name: String,
    symbol: String,
    inputs: Vec<ranges::Interval>,
}
struct FunctionPlan {
    proofs: ranges::Proofs,
    targets: BTreeMap<usize, String>,
}
fn specialization_plan(
    module: &Module,
    unit: &str,
    symbols: &BTreeMap<String, String>,
) -> (BTreeMap<String, FunctionPlan>, Vec<Specialization>) {
    let graph: BTreeMap<String, std::collections::BTreeSet<String>> = module
        .functions
        .iter()
        .map(|(name, function)| {
            let mut edges = std::collections::BTreeSet::new();
            visit_exprs(&function.body, &mut |expr| {
                if let Expr::Call(name, _) = expr {
                    edges.insert(name.to_owned());
                }
            });
            (name.clone(), edges)
        })
        .collect();
    fn reaches(
        graph: &BTreeMap<String, std::collections::BTreeSet<String>>,
        current: &str,
        goal: &str,
        seen: &mut std::collections::BTreeSet<String>,
    ) -> bool {
        if !seen.insert(current.to_owned()) {
            return false;
        }
        graph.get(current).is_some_and(|edges| {
            edges
                .iter()
                .any(|next| next == goal || reaches(graph, next, goal, seen))
        })
    }
    let cyclic: std::collections::BTreeSet<_> = graph
        .keys()
        .filter(|name| reaches(&graph, name, name, &mut Default::default()))
        .cloned()
        .collect();
    let mut plans = BTreeMap::new();
    let mut versions: Vec<Specialization> = Vec::new();
    for (name, function) in &module.functions {
        if name.split_once(':').unwrap().0 != unit {
            continue;
        }
        let proofs = ranges::analyze(function);
        let mut targets = BTreeMap::new();
        visit_exprs(&function.body, &mut |expr| {
            let Expr::Call(callee, _) = expr else {
                return;
            };
            let target = &module.functions[callee];
            if callee.split_once(':').unwrap().0 != unit
                || cyclic.contains(callee)
                || target.result != Type::Int
                || target.error.is_some()
                || !target.params.iter().all(|(_, ty)| *ty == Type::Int)
            {
                return;
            }
            let Some(inputs) = proofs.call_inputs(expr) else {
                return;
            };
            if inputs.len() != target.params.len()
                || inputs.iter().all(|input| *input == ranges::Interval::TOP)
            {
                return;
            }
            let symbol = if let Some(version) = versions
                .iter()
                .find(|version| version.name == *callee && version.inputs == inputs)
            {
                version.symbol.clone()
            } else {
                let count = versions
                    .iter()
                    .filter(|version| version.name == *callee)
                    .count();
                if count >= 8 || versions.len() >= 64 {
                    return;
                }
                let symbol = format!("{}_spec{count}", symbols[callee]);
                versions.push(Specialization {
                    name: callee.to_owned(),
                    symbol: symbol.clone(),
                    inputs: inputs.to_vec(),
                });
                symbol
            };
            targets.insert(expr as *const Expr as usize, symbol);
        });
        plans.insert(name.clone(), FunctionPlan { proofs, targets });
    }
    (plans, versions)
}
pub fn generate(module: &Module, entry: &str, unit: &str) -> String {
    let symbols: BTreeMap<_, _> = module
        .functions
        .keys()
        .map(|name| {
            (
                name.clone(),
                format!(
                    "f{}",
                    name.as_bytes()
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                ),
            )
        })
        .collect();
    let (plans, specializations) = specialization_plan(module, unit, &symbols);
    let signature = |name: &str| {
        let function = &module.functions[name];
        let params = function
            .params
            .iter()
            .enumerate()
            .map(|(i, (_, ty))| format!("{} p{i}", ctype(*ty)))
            .collect::<Vec<_>>();
        format!(
            "{} {}({})",
            if function.error.is_some() {
                fallible_name(function.result)
            } else {
                ctype(function.result)
            },
            symbols[name],
            if params.is_empty() {
                "void".into()
            } else {
                params.join(", ")
            }
        )
    };
    let mut source = RUNTIME.to_owned();
    let mut records = std::collections::BTreeSet::new();
    let mut needed = std::collections::BTreeSet::new();
    for (name, function) in &module.functions {
        if name.split_once(':').unwrap().0 == unit {
            needed.insert(name.clone());
            visit_exprs(&function.body, &mut |expr| match expr {
                Expr::Call(name, _) => {
                    needed.insert(name.clone());
                }
                Expr::Record(name, _) => {
                    records.insert(name.clone());
                }
                _ => {}
            });
        }
    }
    if entry.split_once(':').unwrap().0 == unit {
        needed.insert(entry.into());
    }
    for name in &needed {
        let function = &module.functions[name];
        for ty in function
            .params
            .iter()
            .map(|(_, ty)| ty)
            .chain(std::iter::once(&function.result))
        {
            if let Type::Record(id) = ty {
                records.insert(
                    module
                        .structs
                        .keys()
                        .find(|name| record_id(name) == *id)
                        .unwrap()
                        .clone(),
                );
            }
        }
    }
    for name in record_order(module, &records).expect("checked struct layouts") {
        let structure = &module.structs[&name];
        let id = record_id(&name);
        source.push_str(&format!(
            "typedef struct {{ {} }} R{id};\n",
            if structure.fields.is_empty() {
                "int empty;".into()
            } else {
                structure
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, (_, ty))| format!("{} m{i};", ctype(*ty)))
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        ));
        source.push_str(&format!("static void dropR{id}(R{id} *p) {{"));
        for (i, (_, ty)) in structure.fields.iter().enumerate() {
            if matches!(ty, Type::Owned | Type::Record(_)) {
                source.push_str(&format!("{}(&p->m{i});", drop_fn(*ty)));
            }
        }
        source.push_str(&format!("memset(p,0,sizeof(*p)); }}\nstatic R{id} takeR{id}(R{id} *p) {{ R{id} v=*p; memset(p,0,sizeof(*p)); return v; }}\n"));
    }
    // 失败函数的结果结构体与构造器；调用点只读 ok 相符的那一半。
    let mut fallible: Vec<Type> = Vec::new();
    for name in &needed {
        let result = module.functions[name].result;
        if module.functions[name].error.is_some() && !fallible.contains(&result) {
            fallible.push(result);
        }
    }
    for ty in &fallible {
        let structure = fallible_name(*ty);
        source.push_str(&format!(
            "typedef struct {{ int ok; {}S error; }} {structure};\n",
            if *ty == Type::Unit {
                String::new()
            } else {
                format!("{} value; ", ctype(*ty))
            }
        ));
        if *ty == Type::Unit {
            source.push_str(&format!(
                "static {structure} {}(void) {{ {structure} r = {{0}}; r.ok = 1; return r; }}\n",
                fallible_ok(*ty)
            ));
        } else {
            source.push_str(&format!(
                "static {structure} {}({} value) {{ {structure} r = {{0}}; r.ok = 1; r.value = value; return r; }}\n",
                fallible_ok(*ty),
                ctype(*ty)
            ));
        }
        source.push_str(&format!(
            "static {structure} {}(S error) {{ {structure} r = {{0}}; r.error = error; return r; }}\n",
            fallible_err(*ty)
        ));
    }
    let mut prelude = false;
    for name in &needed {
        if let Some(builtin) = builtin(name) {
            if !prelude {
                source.push_str(STD_PRELUDE);
                prelude = true;
            }
            source.push_str(&format!(
                "static {} {{\n{}\n}}\n",
                signature(name),
                builtin.body
            ));
        } else {
            source.push_str(&format!("{};\n", signature(name)));
        }
    }
    for version in &specializations {
        source.push_str(&format!(
            "static {};\n",
            signature(&version.name).replacen(&symbols[&version.name], &version.symbol, 1)
        ));
    }
    let generic = plans.into_iter().map(|(name, plan)| (name, None, plan));
    let specialized = specializations.into_iter().map(|version| {
        let proofs = ranges::analyze_with_ranges(&module.functions[&version.name], &version.inputs);
        (
            version.name,
            Some(version.symbol),
            FunctionPlan {
                proofs,
                targets: BTreeMap::new(),
            },
        )
    });
    for (name, symbol, plan) in generic.chain(specialized) {
        let function = &module.functions[&name];
        let declaration = if let Some(symbol) = symbol {
            format!(
                "static {}",
                signature(&name).replacen(&symbols[&name], &symbol, 1)
            )
        } else {
            signature(&name)
        };
        source.push_str(&format!("{declaration} {{\n"));
        let mut g = Generator {
            module,
            symbols: &symbols,
            locals: BTreeMap::new(),
            owners: Vec::new(),
            loops: Vec::new(),
            proofs: plan.proofs,
            call_targets: plan.targets,
            code: String::new(),
            next: 0,
            result: function.result,
            error: function.error,
        };
        for (i, (param, ty)) in function.params.iter().enumerate() {
            let local = format!("p{i}");
            if matches!(ty, Type::Owned | Type::Record(_)) {
                g.owners.push((local.clone(), *ty));
            }
            g.locals.insert(param.clone(), (local, *ty));
        }
        let returned = g.block(&function.body, function.result);
        g.cleanup();
        source.push_str(&g.code);
        if function.error.is_some() && !returned && function.result == Type::Unit {
            // 失败函数返回 Unit 时也要显式给出成功值。
            source.push_str(&format!("return {}();\n", fallible_ok(Type::Unit)));
        }
        source.push_str("}\n");
    }
    if entry.split_once(':').unwrap().0 == unit {
        // argv 留给 std/env（Unix）；Windows 下 std/env 改读 GetCommandLineW 以拿到 UTF-8。
        source.push_str("int zam_argc;\nchar **zam_argv;\n");
        if module.functions[entry].error.is_some() {
            let structure = fallible_name(module.functions[entry].result);
            source.push_str(&format!(
                "int main(int argc, char **argv) {{ zam_argc = argc; zam_argv = argv; output_mode(); {structure} r = {}(); if (!r.ok) {{ uncaught_error(&r.error); return 1; }} return fflush(stdout) == EOF ? 1 : 0; }}\n",
                symbols[entry]
            ));
        } else {
            source.push_str(&format!(
                "int main(int argc, char **argv) {{ zam_argc = argc; zam_argv = argv; output_mode(); {}(); return fflush(stdout) == EOF ? 1 : 0; }}\n",
                symbols[entry]
            ));
        }
    }
    source
}

fn uses_msvc(tool: &str) -> bool {
    cfg!(windows)
        && Path::new(tool)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("bat"))
}

pub fn tool() -> Result<(String, String)> {
    if cfg!(windows) && std::env::var_os("CC").is_none() {
        let root = std::env::var_os("ProgramFiles(x86)").ok_or("missing ProgramFiles(x86)")?;
        let output =
            Command::new(Path::new(&root).join("Microsoft Visual Studio/Installer/vswhere.exe"))
                .args([
                    "-latest",
                    "-products",
                    "*",
                    "-requires",
                    "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                    "-property",
                    "installationPath",
                ])
                .output()
                .map_err(|e| format!("MSVC discovery failed: {e}"))?;
        let installation = String::from_utf8(output.stdout)
            .map_err(|e| e.to_string())?
            .trim()
            .to_owned();
        let setup = Path::new(&installation).join("VC/Auxiliary/Build/vcvars64.bat");
        if !output.status.success() || !setup.is_file() {
            return Err("install MSVC C++ build tools for native builds".into());
        }
        let setup = setup.to_string_lossy().into_owned();
        if setup.contains(['"', '%', '\r', '\n']) {
            return Err("unsupported MSVC setup path".into());
        }
        let tools = std::fs::read_to_string(
            Path::new(&installation)
                .join("VC/Auxiliary/Build/Microsoft.VCToolsVersion.default.txt"),
        )
        .map_err(|e| format!("MSVC default version unavailable: {e}"))?;
        let tools = tools.trim();
        if tools.is_empty() || !tools.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return Err("invalid MSVC default version".into());
        }
        let compiler = Path::new(&installation)
            .join("VC/Tools/MSVC")
            .join(tools)
            .join("bin/Hostx64/x64/cl.exe");
        let version = Command::new(compiler)
            .arg("/Bv")
            .output()
            .map_err(|e| format!("MSVC version query failed: {e}"))?;
        let identity = format!(
            "{}{}",
            String::from_utf8_lossy(&version.stdout),
            String::from_utf8_lossy(&version.stderr)
        );
        // /Bv without an input reports versions then exits 2 (D8003).
        if !matches!(version.status.code(), Some(0 | 2))
            || !identity.contains("cl.exe:")
            || !identity.contains("c2.dll:")
        {
            return Err(format!("MSVC version query failed:\n{identity}"));
        }
        Ok((setup, identity))
    } else {
        let compiler = std::env::var("CC").unwrap_or_else(|_| "cc".into());
        let output = Command::new(&compiler)
            .arg("--version")
            .output()
            .map_err(|e| format!("C compiler unavailable: {e}"))?;
        if !output.status.success() {
            return Err("C compiler version query failed".into());
        }
        Ok((
            compiler,
            String::from_utf8_lossy(&output.stdout).into_owned(),
        ))
    }
}

/// MSVC 环境装载一次就够：`vcvars64.bat` 每次执行约 4 秒（它派发一批子脚本与注册表查询），
/// 而它算出来的 INCLUDE/LIB 在同一个 VS 安装里是稳定的。缓存文件放临时目录
/// （不能放 Zamak 自己的缓存目录，那里的条目数被并发测试断言）。
/// 进程内再记一份：模块级并行编译时多个线程会同时要环境，`OnceLock` 保证只装载一次。
fn msvc_environment(setup: &str) -> Result<BTreeMap<String, String>> {
    static LOADED: std::sync::OnceLock<Result<BTreeMap<String, String>>> =
        std::sync::OnceLock::new();
    LOADED.get_or_init(|| load_environment(setup)).clone()
}

fn load_environment(setup: &str) -> Result<BTreeMap<String, String>> {
    let stamp = std::fs::metadata(setup)
        .map(|meta| {
            let modified = meta
                .modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |elapsed| elapsed.as_nanos());
            format!("{}-{modified}", meta.len())
        })
        .map_err(|e| format!("MSVC setup unavailable: {e}"))?;
    let name = crate::project::hash(&[setup.as_bytes()]);
    let path = std::env::temp_dir().join(format!("zam-msvc-{name}.txt"));
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Some((first, rest)) = text.split_once('\n') {
            if first == stamp {
                return Ok(parse_environment(rest));
            }
        }
    }
    // 命令写进 .bat 再执行：`cmd /c` 只认自己的引号规则，而 Rust 把参数里的引号转义成
    // `\"`（cmd 不认），带空格的 vcvars 路径就永远找不到。
    let script = std::env::temp_dir().join(format!("zam-msvc-{name}.bat"));
    let setup = setup.replace('/', r"\");
    std::fs::write(
        &script,
        format!("@echo off\r\ncall \"{setup}\" >nul\r\nset\r\n"),
    )
    .map_err(|e| format!("MSVC setup failed: {e}"))?;
    let output = Command::new("cmd")
        .args(["/d", "/s", "/c"])
        .arg(&script)
        .output()
        .map_err(|e| format!("MSVC setup failed: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "MSVC setup failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    if !text.lines().any(|line| line.starts_with("INCLUDE=")) {
        return Err("MSVC setup produced no INCLUDE".into());
    }
    // 写不进去也无所谓，只是下次还要重新装载一遍。
    let _ = std::fs::write(&path, format!("{stamp}\n{text}"));
    Ok(parse_environment(&text))
}

/// `set` 的输出按本机代码页编码（中文机器上是 936），用户目录里的中文会解码成替换字符，
/// 所以只收值全为 ASCII 的键——MSVC 自己的目录（INCLUDE/LIB/VCToolsInstallDir/…）都是 ASCII。
/// ponytail: 需要带非 ASCII 环境的编译选项时再处理编码。
fn parse_environment(text: &str) -> BTreeMap<String, String> {
    let mut environment = BTreeMap::new();
    for line in text.lines() {
        if let Some((key, value)) = line.split_once('=') {
            if !key.is_empty() && !key.starts_with('=') && value.is_ascii() {
                environment.insert(key.to_ascii_uppercase(), value.to_owned());
            }
        }
    }
    environment
}

/// 直接用绝对路径调 cl.exe/link.exe，不再借道 `cmd /c call vcvars64.bat`。
fn msvc_tool(environment: &BTreeMap<String, String>, name: &str) -> Result<std::path::PathBuf> {
    if let Some(dir) = environment.get("VCTOOLSINSTALLDIR") {
        let path = Path::new(dir).join("bin/Hostx64/x64").join(name);
        if path.is_file() {
            return Ok(path);
        }
    }
    for entry in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let path = entry.join(name);
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(format!("MSVC {name} not found"))
}

// ponytail: 只有产物字节真变才升版本（会让缓存整体失效）；改了编译/链接命令行但字节不变时，
// 先用旧缓存跑一次构建、确认仍是 OBJECT HIT 再决定不升。
pub const OPTIONS: &str = "c11-O2-unroll-MT-Brepro-pathmap-objects-gnu-windows-v6";
pub fn compile(tool: &str, dir: &Path, source: &str) -> Result<Vec<u8>> {
    std::fs::write(dir.join("program.c"), source).map_err(|e| e.to_string())?;
    let output = if uses_msvc(tool) {
        let environment = msvc_environment(tool)?;
        let mapped = std::fs::canonicalize(dir)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .to_owned();
        if mapped.contains(['"', '%', '\r', '\n']) {
            return Err("unsupported native cache path".into());
        }
        // ponytail: 源码按 UTF-8 写出，cl 默认按本地代码页(如 936)读取会吞掉中文注释后的下一行；/utf-8 固定两端编码。
        Command::new(msvc_tool(&environment, "cl.exe")?)
            .current_dir(dir)
            .envs(&environment)
            .args([
                "/nologo",
                "/O2",
                "/MT",
                "/Brepro",
                "/utf-8",
                "/c",
                &format!("/d2pathmap:{mapped}=Z:"),
                "program.c",
                "/Fo:program.obj",
            ])
            .output()
    } else {
        // ponytail: `-g`（用户经 CC 传入）会把编译目录写进 DWARF，同一个缓存键在不同临时目录
        // 就会编出不同字节，破坏「同键 ⇒ 同字节」并让并发构建报 corrupt cache entry。
        // 把临时目录前缀映射掉即可，与 Windows 的 /d2pathmap 对应。
        let map = format!("-ffile-prefix-map={}=.", dir.display());
        Command::new(tool)
            .current_dir(dir)
            .args([
                "-std=c11",
                "-O2",
                "-funroll-loops",
                map.as_str(),
                "-c",
                "program.c",
                "-o",
                "program.obj",
            ])
            .output()
    }
    .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "native compilation failed:\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    std::fs::read(dir.join("program.obj")).map_err(|e| e.to_string())
}

pub fn link(tool: &str, dir: &Path, objects: &[Vec<u8>]) -> Result<Vec<u8>> {
    let mut names = Vec::new();
    // ponytail: Windows 之外统一用 .o —— clang/ld（macOS）对 .obj 的识别不如 .o 可靠，MSVC 侧不变。
    let suffix = if cfg!(windows) { "obj" } else { "o" };
    for (index, bytes) in objects.iter().enumerate() {
        let name = format!("module{index}.{suffix}");
        std::fs::write(dir.join(&name), bytes).map_err(|e| e.to_string())?;
        names.push(name);
    }
    let output = if uses_msvc(tool) {
        let environment = msvc_environment(tool)?;
        Command::new(msvc_tool(&environment, "link.exe")?)
            .current_dir(dir)
            .envs(&environment)
            .args(["/nologo", "/Brepro", "/INCREMENTAL:NO", "/OUT:program.exe"])
            .args(&names)
            .output()
    } else {
        Command::new(tool)
            .current_dir(dir)
            .args(&names)
            .args(["-o", "program.exe"])
            .output()
    }
    .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "native linking failed:\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    std::fs::read(dir.join("program.exe")).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_proofs_select_raw_arithmetic_and_array_accesses() {
        let mut module = crate::compiler::parse(
            "fn main() {\nlet mut a = [0, 0, 0, 0]\nlet mut i = 0\nwhile i < 4 {\na[i] = i * 2\nprintln(a[i])\ni = i + 1\n}\n}",
        )
        .unwrap();
        let function = module.functions.remove("main").unwrap();
        module.functions.insert("case:main".into(), function);
        crate::compiler::check(&module, "case:main").unwrap();
        let source = generate(&module, "case:main", "case");
        let body = source
            .split_once("void f636173653a6d61696e(void) {\n")
            .unwrap()
            .1;
        assert!(!body.contains("checked_index("), "{body}");
        assert!(!body.contains("add("), "{body}");
        assert!(!body.contains("mul("), "{body}");
        assert!(body.contains(" * "), "{body}");
        assert!(body.contains(" + "), "{body}");
    }

    #[test]
    fn for_ranges_emit_c_loops_without_redundant_checks() {
        let mut module = crate::compiler::parse(
            "fn main() {\nlet mut a = [0, 0, 0, 0]\nfor i in 0..4 {\na[i] = i * 2\nprintln(a[i])\nprintln(i + 1)\n}\n}",
        )
        .unwrap();
        let function = module.functions.remove("main").unwrap();
        module.functions.insert("case:main".into(), function);
        crate::compiler::check(&module, "case:main").unwrap();
        let source = generate(&module, "case:main", "case");
        let body = source
            .split_once("void f636173653a6d61696e(void) {\n")
            .unwrap()
            .1;
        assert!(body.contains("for (int64_t "), "{body}");
        assert!(!body.contains("checked_index("), "{body}");
        assert!(!body.contains("add("), "{body}");
        assert!(!body.contains("mul("), "{body}");
    }

    fn scalar_module(source: &str) -> Module {
        fn qualify(expr: &mut Expr) {
            match expr {
                Expr::Call(name, args) => {
                    *name = format!("case:{name}");
                    for arg in args {
                        qualify(arg);
                    }
                }
                Expr::Unary(_, inner) | Expr::Try(inner) | Expr::Index(_, inner) => qualify(inner),
                Expr::Binary(_, a, b) => {
                    qualify(a);
                    qualify(b);
                }
                _ => {}
            }
        }
        fn body(block: &mut Block) {
            for (instruction, _) in block {
                match instruction {
                    Instruction::Let(_, _, _, value)
                    | Instruction::Assign(_, value)
                    | Instruction::Print(value)
                    | Instruction::Call(value)
                    | Instruction::Return(Some(value)) => qualify(value),
                    Instruction::If(value, yes, no) => {
                        qualify(value);
                        body(yes);
                        body(no);
                    }
                    Instruction::While(value, nested) => {
                        qualify(value);
                        body(nested);
                    }
                    Instruction::For(_, start, end, nested) => {
                        qualify(start);
                        qualify(end);
                        body(nested);
                    }
                    _ => {}
                }
            }
        }
        let mut module = crate::compiler::parse(source).unwrap();
        module.functions = module
            .functions
            .into_iter()
            .map(|(name, mut function)| {
                body(&mut function.body);
                (format!("case:{name}"), function)
            })
            .collect();
        crate::compiler::check(&module, "case:main").unwrap();
        module
    }
    #[test]
    fn bounded_scalar_specialization_keeps_generic_checks_and_deterministic_output() {
        let module = scalar_module("fn accum(seed: i64) -> i64 {\nlet mut acc = seed\nlet mut i = 0\nwhile i < 512 {\nacc = (acc * 31 + i) % 1000003\ni = i + 1\n}\nacc\n}\nfn main() {\nlet mut seed = 0\nwhile seed < 8 {\nprintln(accum(seed))\nseed = seed + 1\n}\n}");
        let source = generate(&module, "case:main", "case");
        assert_eq!(source, generate(&module.clone(), "case:main", "case"));
        let symbol = "f636173653a616363756d";
        let generic = source
            .split_once(&format!("int64_t {symbol}(int64_t p0) {{\n"))
            .unwrap()
            .1
            .split_once("}\nvoid")
            .unwrap()
            .0;
        assert!(generic.contains("mul("), "{generic}");
        let specialized = source
            .split_once(&format!("static int64_t {symbol}_spec0(int64_t p0) {{\n"))
            .unwrap()
            .1;
        assert!(!specialized.contains("mul("), "{specialized}");
        assert!(!specialized.contains("add("), "{specialized}");
        assert!(specialized.contains(" * ") && specialized.contains(" + "));
    }
    #[test]
    fn bounded_arithmetic_sum_specialization_proves_entire_512_iteration_body() {
        let module = scalar_module("fn sum(seed: i64) -> i64 {\nlet mut total = 0\nlet mut i = 0\nwhile i < 512 {\ntotal = total + (i + seed + 3) * (i * 5 - seed + 7)\ni = i + 1\n}\nreturn total\n}\nfn main() {\nlet mut r = 0\nwhile r < 8 {\nprintln(sum(r % 8))\nr = r + 1\n}\n}");
        let source = generate(&module, "case:main", "case");
        let specialized = source
            .split_once("static int64_t f636173653a73756d_spec0(int64_t p0) {\n")
            .unwrap()
            .1;
        assert!(!specialized.contains("add("), "{specialized}");
        assert!(!specialized.contains("sub("), "{specialized}");
        assert!(!specialized.contains("mul("), "{specialized}");
        assert!(specialized.contains(" * "));
    }
    #[test]
    fn recursive_callees_and_unknown_inputs_remain_generic() {
        let module = scalar_module("fn a(n: i64) -> i64 {\nif n < 1 { return n }\nb(n - 1)\n}\nfn b(n: i64) -> i64 { a(n) }\nfn unknown() -> i64 { 2 }\nfn add(n: i64) -> i64 { n + 1 }\nfn main() {\nprintln(a(2))\nprintln(add(unknown()))\n}");
        assert!(!generate(&module, "case:main", "case").contains("_spec"));
    }
    #[test]
    fn specialization_caps_reuse_and_nested_calls_stay_generic() {
        let calls = (0..12)
            .map(|i| format!("println(add({i}))\n"))
            .collect::<String>();
        let module = scalar_module(&format!("fn inner(n: i64) -> i64 {{ n + 1 }}\nfn add(n: i64) -> i64 {{ inner(n) }}\nfn main() {{\n{calls}println(add(0))\n}}"));
        let symbols = module
            .functions
            .keys()
            .map(|name| (name.clone(), name.clone()))
            .collect();
        let (_, versions) = specialization_plan(&module, "case", &symbols);
        assert_eq!(versions.len(), 8);
        assert!(versions.iter().all(|version| version.name == "case:add"));
        let source = generate(&module, "case:main", "case");
        let special = source
            .split_once("static int64_t f636173653a616464_spec0(int64_t p0) {\n")
            .unwrap()
            .1;
        assert!(special.contains("f636173653a696e6e6572("));
        assert!(!special.contains("f636173653a696e6e6572_spec"));
    }
    #[test]
    fn environment_takes_ascii_values_only() {
        let text = "Include=C:\\MSVC\\include\r\nLiB=D:\\lib\r\n=TEMP=C:\\tmp\r\n\
                    用户目录=C:\\中文\\路径\r\nEMPTY=\r\n";
        let environment = parse_environment(text);
        assert_eq!(
            environment.get("INCLUDE").map(String::as_str),
            Some("C:\\MSVC\\include")
        );
        assert_eq!(environment.get("LIB").map(String::as_str), Some("D:\\lib"));
        assert_eq!(environment.get("EMPTY").map(String::as_str), Some(""));
        assert!(!environment.contains_key("=TEMP"));
        assert!(!environment.contains_key("用户目录"));
    }
}
