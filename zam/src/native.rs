use crate::compiler::{
    borrow_of, field_path, method_signature, numeric, record_id, record_order, Block, Elem, Expr,
    Instruction, Module, Pattern, Type,
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
#include <math.h>
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
static int64_t zam_remainder(int64_t a, int64_t b) {
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
__declspec(dllimport) int __stdcall GetConsoleMode(void *, unsigned long *);
__declspec(dllimport) int __stdcall WriteConsoleW(void *, const void *, unsigned long, unsigned long *, void *);
__declspec(dllimport) int __stdcall MultiByteToWideChar(unsigned int, unsigned long, const char *, int, wchar_t *, int);
#endif
static void output_mode(void) {
#ifdef _WIN32
    if (_setmode(_fileno(stdout), _O_BINARY) == -1) exit(1);
#endif
}
// Windows 控制台按本地代码页（中文系统是 936）解释字节，直接写 UTF-8 会乱码。输出目标是
// 控制台时先转成 UTF-16 再用 WriteConsoleW；重定向到文件或管道时照常写原始 UTF-8 字节。
// 控制台输入见 STD_PRELUDE 的 zam_stdin_getc：同样按 UTF-16 读，再转成 UTF-8。
static int zam_write(FILE *f, const unsigned char *p, size_t n) {
#ifdef _WIN32
    void *console = (void*)_get_osfhandle(_fileno(f));
    unsigned long mode, done;
    if (n && n <= INT_MAX && GetConsoleMode(console, &mode)) {
        int count = MultiByteToWideChar(65001, 0, (const char*)p, (int)n, NULL, 0), at = 0, chunk;
        wchar_t *wide;
        if (count <= 0 || fflush(f) == EOF) return 0;
        wide = (wchar_t*)malloc((size_t)count * sizeof(wchar_t));
        if (!wide) { fputs("zam: allocation failed\n", stderr); exit(1); }
        MultiByteToWideChar(65001, 0, (const char*)p, (int)n, wide, count);
        while (at < count) {
            chunk = count - at < 8192 ? count - at : 8192;
            // 不把代理对拆到两次写入里。
            if (chunk > 1 && at + chunk < count && wide[at + chunk - 1] >= 0xD800 && wide[at + chunk - 1] <= 0xDBFF) chunk--;
            if (!WriteConsoleW(console, wide + at, (unsigned long)chunk, &done, NULL) || !done) { free(wide); return 0; }
            at += (int)done;
        }
        free(wide);
        return 1;
    }
#endif
    return !n || fwrite(p, 1, n, f) == n;
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
static int string_compare(S *a, S *b) {
    size_t n = a->len < b->len ? a->len : b->len;
    int c = n ? memcmp(a->data, b->data, n) : 0;
    if (c) return c < 0 ? -1 : 1;
    return a->len < b->len ? -1 : a->len > b->len;
}
static void print(S *s) {
    if (!zam_write(stdout, s->data, s->len) || fputc('\n', stdout) == EOF) exit(1);
}
// ponytail: 未捕获的错误只写 stderr 并以退出码 1 结束；不做栈回溯，错误值本身就是全部信息。
static void uncaught_error(S *s) {
    if (!zam_write(stderr, s->data, s->len)) exit(1);
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
// f64 的文本：能原样读回的最短十进制；十进制指数在 [-5, 16) 内写成定点（整数也带 .0），
// 否则写成 1e16、1.5e-7。out 至少 48 字节，返回长度。
static size_t zam_f64_text(double x, char *out) {
    char text[40], digits[24];
    int precision, exponent, count = 0, length = 0, i;
    uint64_t bits;
    const char *p;
    if (x != x) { memcpy(out, "NaN", 3); return 3; }
    memcpy(&bits, &x, sizeof bits);
    if (bits >> 63) out[length++] = '-';
    if (x == 0) { memcpy(out + length, "0.0", 3); return (size_t)length + 3; }
    if (x == HUGE_VAL || x == -HUGE_VAL) { memcpy(out + length, "inf", 3); return (size_t)length + 3; }
    for (precision = 1; precision < 17; precision++) {
        snprintf(text, sizeof text, "%.*e", precision - 1, x);
        if (strtod(text, NULL) == x) break;
    }
    snprintf(text, sizeof text, "%.*e", precision - 1, x);
    for (p = text + (text[0] == '-'); *p != 'e'; p++) if (*p != '.') digits[count++] = *p;
    exponent = atoi(p + 1);
    if (exponent >= -5 && exponent < 16) {
        if (exponent < 0) {
            out[length++] = '0'; out[length++] = '.';
            for (i = -1; i > exponent; i--) out[length++] = '0';
            for (i = 0; i < count; i++) out[length++] = digits[i];
        } else {
            for (i = 0; i <= exponent; i++) out[length++] = i < count ? digits[i] : '0';
            out[length++] = '.';
            if (count > exponent + 1) { for (i = exponent + 1; i < count; i++) out[length++] = digits[i]; }
            else out[length++] = '0';
        }
    } else {
        out[length++] = digits[0];
        if (count > 1) { out[length++] = '.'; for (i = 1; i < count; i++) out[length++] = digits[i]; }
        length += snprintf(out + length, 8, "e%d", exponent);
    }
    return (size_t)length;
}
static S from_f64(double value) {
    char text[48];
    size_t n = zam_f64_text(value, text);
    return own((const unsigned char*)text, n);
}
static void print_f64(double value) {
    char text[48];
    size_t n = zam_f64_text(value, text);
    text[n++] = '\n';
    if (fwrite(text, 1, n, stdout) != n) exit(1);
}
"#;

// 标准库里需要平台能力的函数共用的 C 辅助代码；某个目标文件只要用到任一内建函数就整段发射一次。
// Windows 不引入 <windows.h>（太重且宏会撞名），只声明用到的三个 kernel32 函数。
const STD_PRELUDE: &str = r#"#ifdef _WIN32
__declspec(dllimport) wchar_t *__stdcall GetCommandLineW(void);
__declspec(dllimport) int __stdcall MultiByteToWideChar(unsigned int, unsigned long, const char *, int, wchar_t *, int);
__declspec(dllimport) int __stdcall WideCharToMultiByte(unsigned int, unsigned long, const wchar_t *, int, char *, int, const char *, int *);
__declspec(dllimport) int __stdcall ReadConsoleW(void *, void *, unsigned long, unsigned long *, void *);
#endif
extern int zam_argc;
extern char **zam_argv;
static void zam_alloc_failed(void) { fputs("zam: allocation failed\n", stderr); exit(1); }
static int zam_space(unsigned char c) { return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\v' || c == '\f'; }
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
// 标准输入是 Windows 控制台时按 UTF-16 读（ReadConsoleW），转成 UTF-8 再逐字节交出去，
// 中文系统的键盘输入才不会按本地代码页（936）变成 GBK 字节；重定向的标准输入照常按字节读。
#ifdef _WIN32
static int zam_console = -1, zam_console_error, zam_console_eof;
static wchar_t zam_console_wide[4097];
static unsigned char zam_console_bytes[3 * 4097 + 4];
static size_t zam_console_at, zam_console_len;
static unsigned long zam_console_carry;
static int zam_stdin_console(void) {
    unsigned long mode;
    if (zam_console < 0) zam_console = GetConsoleMode((void*)_get_osfhandle(_fileno(stdin)), &mode) ? 1 : 0;
    return zam_console;
}
static int zam_console_fill(void) {
    unsigned long got = 0, total, keep;
    int n;
    if (zam_console_eof || zam_console_error) return 0;
    if (!ReadConsoleW((void*)_get_osfhandle(_fileno(stdin)), zam_console_wide + zam_console_carry, 4096, &got, NULL)) {
        zam_console_error = 1;
        return 0;
    }
    total = zam_console_carry + got;
    // 行首的 Ctrl+Z 表示输入结束，与 C 运行库的文本模式一致。
    if (!got || (zam_console_carry == 0 && zam_console_wide[0] == 0x1A)) { zam_console_eof = 1; return 0; }
    // 代理对的前半个留到下一次，和后半个一起转换。
    keep = zam_console_wide[total - 1] >= 0xD800 && zam_console_wide[total - 1] <= 0xDBFF ? 1 : 0;
    n = total > keep ? WideCharToMultiByte(65001, 0, zam_console_wide, (int)(total - keep), (char*)zam_console_bytes, (int)sizeof zam_console_bytes, NULL, NULL) : 0;
    if (total > keep && n <= 0) { zam_console_error = 1; return 0; }
    if (keep) zam_console_wide[0] = zam_console_wide[total - 1];
    zam_console_carry = keep;
    zam_console_at = 0;
    zam_console_len = (size_t)n;
    return 1;
}
#endif
static int zam_stdin_getc(void) {
#ifdef _WIN32
    if (zam_stdin_console()) {
        while (zam_console_at == zam_console_len) {
            if (!zam_console_fill()) return EOF;
        }
        return zam_console_bytes[zam_console_at++];
    }
#endif
    return getc(stdin);
}
static int zam_stdin_error(void) {
#ifdef _WIN32
    if (zam_stdin_console()) return zam_console_error;
#endif
    return ferror(stdin);
}
static int zam_stdin_all(S *out) {
#ifdef _WIN32
    if (zam_stdin_console()) {
        size_t cap = 4096, n = 0;
        unsigned char *p = (unsigned char*)malloc(cap), *q;
        int c;
        if (!p) zam_alloc_failed();
        while ((c = zam_stdin_getc()) != EOF) {
            if (n == cap) { q = (unsigned char*)realloc(p, cap * 2); if (!q) zam_alloc_failed(); p = q; cap *= 2; }
            p[n++] = (unsigned char)c;
        }
        if (zam_console_error) { free(p); return 0; }
        out->data = p; out->len = n;
        return 1;
    }
#endif
    return zam_read_all(stdin, out);
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
/// `std/math` 里 f64 → f64 的单参数函数：(名字, C 表达式)。
macro_rules! unary_math {
    ($($name:literal => $body:literal),* $(,)?) => {
        [$(Builtin {
            name: concat!("std/math:", $name),
            params: &[Type::Float],
            result: Type::Float,
            fallible: false,
            body: concat!("return ", $body, ";"),
        }),*]
    };
}
static MATH: [Builtin; 13] = unary_math! {
    "sqrt" => "sqrt(p0)",
    "exp" => "exp(p0)",
    "log" => "log(p0)",
    "log10" => "log10(p0)",
    "sin" => "sin(p0)",
    "cos" => "cos(p0)",
    "tan" => "tan(p0)",
    "floor" => "floor(p0)",
    "ceil" => "ceil(p0)",
    "round" => "round(p0)",
    "trunc" => "trunc(p0)",
    "abs" => "fabs(p0)",
    "atan" => "atan(p0)",
};
const BUILTINS: &[Builtin] = &[
    Builtin {
        name: "std/core:f64",
        params: &[Type::Int],
        result: Type::Float,
        fallible: false,
        body: "return (double)p0;",
    },
    Builtin {
        name: "std/core:i64",
        params: &[Type::Float],
        result: Type::Int,
        fallible: false,
        // 向零截断；NaN 与超出 i64 的值没有对应整数，按运行时错误停下。
        body: r#"if (!(p0 >= -9223372036854775808.0 && p0 < 9223372036854775808.0)) { fputs("zam: f64 value out of i64 range\n", stderr); exit(1); }
return (int64_t)p0;"#,
    },
    Builtin {
        name: "std/math:pow",
        params: &[Type::Float, Type::Float],
        result: Type::Float,
        fallible: false,
        body: "return pow(p0, p1);",
    },
    Builtin {
        name: "std/math:atan2",
        params: &[Type::Float, Type::Float],
        result: Type::Float,
        fallible: false,
        body: "return atan2(p0, p1);",
    },
    Builtin {
        name: "std/math:min",
        params: &[Type::Float, Type::Float],
        result: Type::Float,
        fallible: false,
        body: "return fmin(p0, p1);",
    },
    Builtin {
        name: "std/math:max",
        params: &[Type::Float, Type::Float],
        result: Type::Float,
        fallible: false,
        body: "return fmax(p0, p1);",
    },
    Builtin {
        name: "std/math:is_nan",
        params: &[Type::Float],
        result: Type::Bool,
        fallible: false,
        body: "return p0 != p0;",
    },
    Builtin {
        name: "std/fmt:fixed",
        params: &[Type::Float, Type::Int],
        result: Type::Owned,
        fallible: false,
        body: r#"char text[48]; int n; S s;
if (p1 < 0 || p1 > 30) { fputs("zam: fmt.fixed digits must be between 0 and 30\n", stderr); exit(1); }
if (p0 != p0 || p0 == HUGE_VAL || p0 == -HUGE_VAL) return own((const unsigned char*)text, zam_f64_text(p0, text));
n = snprintf(NULL, 0, "%.*f", (int)p1, p0);
s.data = (unsigned char*)malloc((size_t)n + 1);
if (!s.data) zam_alloc_failed();
snprintf((char*)s.data, (size_t)n + 1, "%.*f", (int)p1, p0);
s.len = (size_t)n;
return s;"#,
    },
    Builtin {
        name: "std/string:parse_f64",
        params: &[Type::Shared],
        result: Type::Float,
        fallible: true,
        // 只认十进制：可选符号、数字（可带小数点）、可选指数；不接受空白、inf、nan 与十六进制。
        body: r#"size_t i = 0, digits = 0, exponent = 0; char *text; double value;
if (i < p0->len && (p0->data[i] == '-' || p0->data[i] == '+')) i++;
while (i < p0->len && p0->data[i] >= '0' && p0->data[i] <= '9') { i++; digits++; }
if (i < p0->len && p0->data[i] == '.') { i++; while (i < p0->len && p0->data[i] >= '0' && p0->data[i] <= '9') { i++; digits++; } }
if (digits && i < p0->len && (p0->data[i] == 'e' || p0->data[i] == 'E')) {
    i++;
    if (i < p0->len && (p0->data[i] == '-' || p0->data[i] == '+')) i++;
    while (i < p0->len && p0->data[i] >= '0' && p0->data[i] <= '9') { i++; exponent++; }
    if (!exponent) digits = 0;
}
if (!digits || i != p0->len) return F_double_err(zam_message("invalid float: ", p0));
text = (char*)malloc(p0->len + 1);
if (!text) zam_alloc_failed();
memcpy(text, p0->data, p0->len);
text[p0->len] = 0;
value = strtod(text, NULL);
free(text);
if (value == HUGE_VAL || value == -HUGE_VAL) return F_double_err(zam_message("float out of range: ", p0));
return F_double_ok(value);"#,
    },
    Builtin {
        name: "std/io:print",
        params: &[Type::Shared],
        result: Type::Unit,
        fallible: false,
        body: "if (!zam_write(stdout, p0->data, p0->len)) exit(1);",
    },
    Builtin {
        name: "std/io:eprintln",
        params: &[Type::Shared],
        result: Type::Unit,
        fallible: false,
        body: r#"fflush(stdout);
if (!zam_write(stderr, p0->data, p0->len) || fputc('\n', stderr) == EOF) exit(1);"#,
    },
    Builtin {
        name: "std/io:read_line",
        params: &[Type::Mutable],
        result: Type::Bool,
        fallible: true,
        body: r#"size_t cap = 64, n = 0; int c, any = 0;
unsigned char *p = (unsigned char*)malloc(cap), *q;
if (!p) zam_alloc_failed();
zam_stdin_binary();
while ((c = zam_stdin_getc()) != EOF) {
    any = 1;
    if (c == '\n') break;
    if (n == cap) { q = (unsigned char*)realloc(p, cap * 2); if (!q) zam_alloc_failed(); p = q; cap *= 2; }
    p[n++] = (unsigned char)c;
}
if (zam_stdin_error()) { free(p); return F_int_err(zam_message("cannot read standard input", NULL)); }
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
if (!zam_stdin_all(&text)) return F_S_err(zam_message("cannot read standard input", NULL));
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
        name: "std/env:args",
        params: &[],
        result: Type::List(Elem::Str),
        fallible: false,
        body: r#"Ls out = {0}; int64_t i, n = zam_arg_count();
for (i = 0; i < n; i++) pushLs(&out, zam_arg(i));
return out;"#,
    },
    Builtin {
        name: "std/string:find_from",
        params: &[Type::Shared, Type::Shared, Type::Int],
        result: Type::Int,
        fallible: false,
        body: r#"size_t i;
if (p2 < 0 || (uint64_t)p2 > p0->len) zam_string_bounds();
if (!p1->len) return p2;
if (p1->len > p0->len) return -1;
for (i = (size_t)p2; i + p1->len <= p0->len; i++) if (memcmp(p0->data + i, p1->data, p1->len) == 0) return (int64_t)i;
return -1;"#,
    },
    Builtin {
        name: "std/string:starts_with",
        params: &[Type::Shared, Type::Shared],
        result: Type::Bool,
        fallible: false,
        body: "return !p1->len || (p1->len <= p0->len && memcmp(p0->data, p1->data, p1->len) == 0);",
    },
    Builtin {
        name: "std/string:ends_with",
        params: &[Type::Shared, Type::Shared],
        result: Type::Bool,
        fallible: false,
        body: "return !p1->len || (p1->len <= p0->len && memcmp(p0->data + (p0->len - p1->len), p1->data, p1->len) == 0);",
    },
    Builtin {
        name: "std/string:contains",
        params: &[Type::Shared, Type::Shared],
        result: Type::Bool,
        fallible: false,
        body: r#"size_t i;
if (!p1->len) return 1;
for (i = 0; i + p1->len <= p0->len; i++) if (memcmp(p0->data + i, p1->data, p1->len) == 0) return 1;
return 0;"#,
    },
    Builtin {
        name: "std/string:replace",
        params: &[Type::Shared, Type::Shared, Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: r#"size_t i = 0, count = 0, n, at = 0; unsigned char *p; S s;
if (!p1->len) return copy(p0);
while (i + p1->len <= p0->len) { if (memcmp(p0->data + i, p1->data, p1->len) == 0) { count++; i += p1->len; } else i++; }
if (p2->len > p1->len && count > (SIZE_MAX - p0->len) / (p2->len - p1->len)) zam_alloc_failed();
n = p0->len - count * p1->len + count * p2->len;
p = (unsigned char*)malloc(n ? n : 1);
if (!p) zam_alloc_failed();
i = 0;
while (i < p0->len) {
    if (i + p1->len <= p0->len && memcmp(p0->data + i, p1->data, p1->len) == 0) { if (p2->len) memcpy(p + at, p2->data, p2->len); at += p2->len; i += p1->len; }
    else p[at++] = p0->data[i++];
}
s.data = p; s.len = n;
return s;"#,
    },
    Builtin {
        name: "std/string:lower",
        params: &[Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: "S s = copy(p0); size_t i; for (i = 0; i < s.len; i++) if (s.data[i] >= 'A' && s.data[i] <= 'Z') s.data[i] += 32; return s;",
    },
    Builtin {
        name: "std/string:upper",
        params: &[Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: "S s = copy(p0); size_t i; for (i = 0; i < s.len; i++) if (s.data[i] >= 'a' && s.data[i] <= 'z') s.data[i] -= 32; return s;",
    },
    Builtin {
        name: "std/string:copy",
        params: &[Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: "return copy(p0);",
    },
    Builtin {
        name: "std/string:split",
        params: &[Type::Shared, Type::Shared],
        result: Type::List(Elem::Str),
        fallible: false,
        body: r#"Ls out = {0}; size_t start = 0, i = 0;
// 空分隔符不切分，整串作为唯一一项。
if (!p1->len) { pushLs(&out, copy(p0)); return out; }
while (i + p1->len <= p0->len) {
    if (memcmp(p0->data + i, p1->data, p1->len) == 0) { pushLs(&out, own(p0->data + start, i - start)); i += p1->len; start = i; }
    else i++;
}
pushLs(&out, own(p0->data + start, p0->len - start));
return out;"#,
    },
    Builtin {
        name: "std/string:words",
        params: &[Type::Shared],
        result: Type::List(Elem::Str),
        fallible: false,
        body: r#"Ls out = {0}; size_t i = 0, start;
while (i < p0->len) {
    while (i < p0->len && zam_space(p0->data[i])) i++;
    if (i == p0->len) break;
    start = i;
    while (i < p0->len && !zam_space(p0->data[i])) i++;
    pushLs(&out, own(p0->data + start, i - start));
}
return out;"#,
    },
    Builtin {
        name: "std/string:lines",
        params: &[Type::Shared],
        result: Type::List(Elem::Str),
        fallible: false,
        body: r#"Ls out = {0}; size_t start = 0, i, end;
for (i = 0; i <= p0->len; i++) {
    if (i < p0->len && p0->data[i] != '\n') continue;
    // 末尾换行之后不再产生一个空行。
    if (i == p0->len && start == p0->len) break;
    end = i;
    if (end > start && p0->data[end - 1] == '\r') end--;
    pushLs(&out, own(p0->data + start, end - start));
    start = i + 1;
}
return out;"#,
    },
    Builtin {
        name: "std/string:join",
        params: &[Type::ListRef(Elem::Str, false), Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: r#"size_t n = 0, at = 0, i; unsigned char *p; S s;
for (i = 0; i < p0->len; i++) {
    if (n > SIZE_MAX - p0->data[i].len - p1->len) zam_alloc_failed();
    n += p0->data[i].len + (i ? p1->len : 0);
}
p = (unsigned char*)malloc(n ? n : 1);
if (!p) zam_alloc_failed();
for (i = 0; i < p0->len; i++) {
    if (i && p1->len) { memcpy(p + at, p1->data, p1->len); at += p1->len; }
    if (p0->data[i].len) { memcpy(p + at, p0->data[i].data, p0->data[i].len); at += p0->data[i].len; }
}
s.data = p; s.len = n;
return s;"#,
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
    BUILTINS
        .iter()
        .chain(&MATH)
        .find(|builtin| builtin.name == name)
}
pub fn builtin_functions() -> impl Iterator<Item = (String, crate::compiler::Function)> {
    BUILTINS.iter().chain(&MATH).map(|builtin| {
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
    // 函数体里出现过的列表与字典类型，用来决定发射哪些容器辅助函数。
    containers: Containers,
}
/// 用到的容器：`(元素, None)` 是列表，`(键, Some(值))` 是字典。
type Containers = std::collections::BTreeSet<(Elem, Option<Elem>)>;
fn note(set: &mut Containers, ty: Type) {
    match ty.container() {
        Some(Type::List(elem)) => {
            set.insert((elem, None));
        }
        // 字典的 keys()/values() 返回键与值的列表，所以这两种列表一并生成。
        Some(Type::Map(key, value)) => {
            set.insert((key, None));
            if !matches!(value, Elem::Record(_)) {
                set.insert((value, None));
            }
            set.insert((key, Some(value)));
        }
        _ => {}
    }
}
fn is_text(ty: Type) -> bool {
    matches!(ty, Type::Owned | Type::Shared | Type::Mutable)
}
fn elem_of(ty: Type) -> Elem {
    match ty {
        Type::Int => Elem::Int,
        Type::Float => Elem::Float,
        Type::Bool => Elem::Bool,
        Type::Record(id) => Elem::Record(id),
        _ => Elem::Str,
    }
}
/// 类型里引用到的结构体：结构体本身、它的借用、结构体元素的列表或结构体值的字典。
fn record_of(ty: Type) -> Option<u64> {
    match ty {
        Type::Record(id)
        | Type::RecordRef(id, _)
        | Type::List(Elem::Record(id))
        | Type::ListRef(Elem::Record(id), _)
        | Type::Map(_, Elem::Record(id))
        | Type::MapRef(_, Elem::Record(id), _) => Some(id),
        _ => None,
    }
}
/// 按值传递、不需要释放的标量。
fn by_value(ty: Type) -> bool {
    matches!(ty, Type::Int | Type::Float | Type::Bool)
}
/// C 里的 f64 字面量：Rust 的 Debug 输出是能精确读回的最短十进制，C 也认这种写法。
fn float_literal(value: f64) -> String {
    if value.is_infinite() {
        if value > 0.0 {
            "HUGE_VAL"
        } else {
            "(-HUGE_VAL)"
        }
        .into()
    } else {
        format!("({value:?})")
    }
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
/// 结构体、列表与字典：有按类型生成的 `drop`/`take`，移走时不能当字符串处理。
fn aggregate(ty: Type) -> bool {
    matches!(ty, Type::Record(_) | Type::List(_) | Type::Map(_, _))
}
fn drop_fn(ty: Type) -> String {
    if aggregate(ty) {
        format!("drop{}", ctype(ty))
    } else {
        "drop".into()
    }
}
/// 移走 owned 值的 C 函数：字符串用运行时的 `take`，结构体与列表各有按类型生成的版本。
fn take_fn(ty: Type) -> String {
    if aggregate(ty) {
        format!("take{}", ctype(ty))
    } else {
        "take".into()
    }
}
fn owned(ty: Type) -> bool {
    ty == Type::Owned || aggregate(ty)
}
fn elem_suffix(elem: Elem) -> String {
    match elem {
        Elem::Int => "i".into(),
        Elem::Float => "f".into(),
        Elem::Bool => "b".into(),
        Elem::Str => "s".into(),
        Elem::Record(id) => format!("R{id}"),
    }
}
fn map_name(key: Elem, value: Elem) -> String {
    format!("M{}{}", elem_suffix(key), elem_suffix(value))
}
const MAP_PRELUDE: &str = r#"static void map_missing(void) { fputs("zam: key not found\n", stderr); exit(1); }
static uint64_t zam_hash_int(int64_t key) { uint64_t x = (uint64_t)key; x ^= x >> 33; x *= 0xff51afd7ed558ccdULL; x ^= x >> 33; x *= 0xc4ceb9fe1a85ec53ULL; x ^= x >> 33; return x; }
static uint64_t zam_hash_str(S *key) { uint64_t h = 1469598103934665603ULL; size_t i; for (i = 0; i < key->len; i++) { h ^= key->data[i]; h *= 1099511628211ULL; } return h; }
"#;
// 插入顺序表 + 开放寻址下标表：遍历按插入顺序，删除留墓碑，下标表装载率不超过 1/2，
// 重建时先压掉已删除的项。
// 类型分三段发射：前置声明、字典本体（只含条目指针）、条目（按值含键和值，要等结构体定义完）。
const MAP_DECLARATION: &str = "typedef struct $M $M;\ntypedef struct E$M E$M;\n";
const MAP_DEFINITION: &str =
    "struct $M { E$M *entries; size_t len, cap, count; int64_t *slots; size_t nslots; };\n";
const MAP_ENTRY: &str = "struct E$M { $KT key; $VT value; uint64_t hash; int live; };\n";
const MAP_TEMPLATE: &str = r#"static void drop$M($M *m) { size_t i; for (i = 0; i < m->len; i++) { E$M *e = &m->entries[i]; if (e->live) { $DROPKEY $DROPVAL } } free(m->entries); free(m->slots); memset(m, 0, sizeof(*m)); }
static $M take$M($M *m) { $M v = *m; memset(m, 0, sizeof(*m)); return v; }
static int64_t len$M($M *m) { return (int64_t)m->count; }
static int64_t find$M($M *m, $KP, uint64_t h) {
    size_t i; int64_t at;
    if (!m->nslots) return -1;
    i = (size_t)h & (m->nslots - 1);
    while ((at = m->slots[i]) >= 0) {
        E$M *e = &m->entries[at];
        if (e->live && e->hash == h && $EQ) return at;
        i = (i + 1) & (m->nslots - 1);
    }
    return -1;
}
static void place$M($M *m, size_t at) {
    size_t i = (size_t)m->entries[at].hash & (m->nslots - 1);
    while (m->slots[i] >= 0) i = (i + 1) & (m->nslots - 1);
    m->slots[i] = (int64_t)at;
}
static void rehash$M($M *m) {
    size_t i, j = 0, n = 16;
    for (i = 0; i < m->len; i++) if (m->entries[i].live) m->entries[j++] = m->entries[i];
    m->len = j;
    while (n / 4 < j + 1) { if (n > SIZE_MAX / 2 / sizeof(int64_t)) list_alloc_failed(); n *= 2; }
    free(m->slots);
    m->slots = (int64_t*)malloc(n * sizeof(int64_t));
    if (!m->slots) list_alloc_failed();
    for (i = 0; i < n; i++) m->slots[i] = -1;
    m->nslots = n;
    for (i = 0; i < m->len; i++) place$M(m, i);
}
static void set$M($M *m, $KP, $VT v) {
    uint64_t h = $HASH;
    int64_t at = find$M(m, k, h);
    E$M *e;
    if (at >= 0) { e = &m->entries[at]; $DROPVAL e->value = v; return; }
    if ((m->len + 1) * 2 > m->nslots) rehash$M(m);
    if (m->len == m->cap) {
        size_t cap = m->cap ? m->cap * 2 : 8;
        E$M *p;
        if (cap > SIZE_MAX / sizeof(E$M)) list_alloc_failed();
        p = (E$M*)realloc(m->entries, cap * sizeof(E$M));
        if (!p) list_alloc_failed();
        m->entries = p; m->cap = cap;
    }
    e = &m->entries[m->len];
    e->key = $STORE; e->value = v; e->hash = h; e->live = 1;
    place$M(m, m->len);
    m->len++; m->count++;
}
static $VT *at$M($M *m, $KP) { int64_t at = find$M(m, k, $HASH); if (at < 0) map_missing(); return &m->entries[at].value; }
static int has$M($M *m, $KP) { return find$M(m, k, $HASH) >= 0; }
static $VT getor$M($M *m, $KP, $VT d) { int64_t at = find$M(m, k, $HASH); if (at < 0) return d; $DROPDEF return $VALOUT; }
static int del$M($M *m, $KP) { int64_t at = find$M(m, k, $HASH); E$M *e; if (at < 0) return 0; e = &m->entries[at]; $DROPKEY $DROPVAL e->live = 0; m->count--; return 1; }
static $L keys$M($M *m) { $L out = {0}; size_t i; for (i = 0; i < m->len; i++) { E$M *e = &m->entries[i]; if (e->live) push$L(&out, $KEYOUT); } return out; }
static void clear$M($M *m) { size_t i; for (i = 0; i < m->len; i++) { E$M *e = &m->entries[i]; if (e->live) { $DROPKEY $DROPVAL } } m->len = 0; m->count = 0; for (i = 0; i < m->nslots; i++) m->slots[i] = -1; }
"#;
/// 字典的 `values()`：值的列表，字符串值复制；结构体值不能复制，没有这个方法。
fn map_values(key: Elem, value: Elem) -> String {
    let m = map_name(key, value);
    let l = list_suffix(value);
    let out = match value {
        Elem::Str => "copy(&e->value)",
        Elem::Record(_) => return String::new(),
        _ => "e->value",
    };
    format!("static {l} values{m}({m} *m) {{ {l} out = {{0}}; size_t i; for (i = 0; i < m->len; i++) {{ E{m} *e = &m->entries[i]; if (e->live) push{l}(&out, {out}); }} return out; }}\n")
}
/// 把字典模板里的占位符换成具体的键值类型。
fn map_helpers(template: &str, key: Elem, value: Elem) -> String {
    let text = key == Elem::Str;
    let text_value = value == Elem::Str;
    let pick = |yes: &'static str, no: &'static str, flag: bool| if flag { yes } else { no };
    let value_type = ctype(value.ty());
    let name = map_name(key, value);
    let list = list_suffix(key);
    // 结构体值的 `getor` 不会被调用（检查器拒绝 `get`），但模板照常生成。
    let drop_value = drop_elem(value, "&e->value");
    let drop_default = drop_elem(value, "&d");
    [
        ("$DROPKEY", pick("drop(&e->key);", "", text)),
        ("$DROPVAL", drop_value.as_str()),
        ("$DROPDEF", drop_default.as_str()),
        ("$KEYOUT", pick("copy(&e->key)", "e->key", text)),
        (
            "$VALOUT",
            pick(
                "copy(&m->entries[at].value)",
                "m->entries[at].value",
                text_value,
            ),
        ),
        ("$STORE", pick("copy(k)", "k", text)),
        ("$HASH", pick("zam_hash_str(k)", "zam_hash_int(k)", text)),
        ("$EQ", pick("string_equal(&e->key, k)", "e->key == k", text)),
        ("$KP", pick("S *k", "int64_t k", text)),
        ("$KT", pick("S", "int64_t", text)),
        ("$VT", value_type.as_str()),
        ("$L", list.as_str()),
        ("$M", name.as_str()),
    ]
    .iter()
    .fold(template.to_owned(), |text, (from, to)| {
        text.replace(from, to)
    })
}
fn list_suffix(elem: Elem) -> String {
    format!("L{}", elem_suffix(elem))
}
/// 释放一个 owned 元素的 C 语句（`slot` 是指向它的表达式）；标量不用释放。
fn drop_elem(elem: Elem, slot: &str) -> String {
    match elem {
        Elem::Str => format!("drop({slot});"),
        Elem::Record(id) => format!("dropR{id}({slot});"),
        _ => String::new(),
    }
}
const LIST_PRELUDE: &str = r#"static void list_bounds(void) { fputs("zam: list index out of bounds\n", stderr); exit(1); }
static void list_empty(void) { fputs("zam: pop from empty list\n", stderr); exit(1); }
static void list_alloc_failed(void) { fputs("zam: allocation failed\n", stderr); exit(1); }
"#;
/// 列表本体只含元素指针，元素类型声明过即可定义。
fn list_definition(elem: Elem) -> String {
    let l = list_suffix(elem);
    let t = ctype(elem.ty());
    format!("struct {l} {{ {t} *data; size_t len; size_t cap; }};\n")
}
/// 某种元素的列表函数：释放、移走、长度、追加、按下标取地址、弹出、交换。
fn list_helpers(elem: Elem) -> String {
    let l = list_suffix(elem);
    let t = ctype(elem.ty());
    let drop_items = match drop_elem(elem, "&l->data[i]") {
        drop if drop.is_empty() => drop,
        drop => format!("for (size_t i = 0; i < l->len; i++) {drop} "),
    };
    let comparable = comparable_list_helpers(elem);
    format!(
        "static void drop{l}({l} *l) {{ {drop_items}free(l->data); l->data = NULL; l->len = 0; l->cap = 0; }}
static {l} take{l}({l} *l) {{ {l} v = *l; l->data = NULL; l->len = 0; l->cap = 0; return v; }}
static int64_t len{l}({l} *l) {{ return (int64_t)l->len; }}
static void push{l}({l} *l, {t} v) {{
    if (l->len == l->cap) {{
        size_t cap = l->cap ? l->cap * 2 : 4;
        {t} *p;
        if (cap > SIZE_MAX / sizeof({t})) list_alloc_failed();
        p = ({t}*)realloc(l->data, cap * sizeof({t}));
        if (!p) list_alloc_failed();
        l->data = p; l->cap = cap;
    }}
    l->data[l->len++] = v;
}}
static {t} *at{l}({l} *l, int64_t i) {{ if (i < 0 || (uint64_t)i >= l->len) list_bounds(); return &l->data[i]; }}
static {t} pop{l}({l} *l) {{ if (!l->len) list_empty(); return l->data[--l->len]; }}
static void swap{l}({l} *l, int64_t i, int64_t j) {{ {t} tmp; if (i < 0 || j < 0 || (uint64_t)i >= l->len || (uint64_t)j >= l->len) list_bounds(); tmp = l->data[i]; l->data[i] = l->data[j]; l->data[j] = tmp; }}
static void insert{l}({l} *l, int64_t i, {t} v) {{ if (i < 0 || (uint64_t)i > l->len) list_bounds(); push{l}(l, v); memmove(&l->data[i + 1], &l->data[i], (l->len - 1 - (size_t)i) * sizeof({t})); l->data[i] = v; }}
static {t} removeat{l}({l} *l, int64_t i) {{ {t} v; if (i < 0 || (uint64_t)i >= l->len) list_bounds(); v = l->data[i]; memmove(&l->data[i], &l->data[i + 1], (l->len - 1 - (size_t)i) * sizeof({t})); l->len--; return v; }}
static void reverse{l}({l} *l) {{ size_t i, j; for (i = 0, j = l->len; i + 1 < j; i++, j--) {{ {t} tmp = l->data[i]; l->data[i] = l->data[j - 1]; l->data[j - 1] = tmp; }} }}
static void clear{l}({l} *l) {{ {drop_items}l->len = 0; }}
static void extend{l}({l} *l, {l} o) {{ size_t i; for (i = 0; i < o.len; i++) push{l}(l, o.data[i]); free(o.data); }}
{comparable}"
    )
}
/// 能比较的元素（整数、浮点、布尔、字符串）才有排序、查找与切片：结构体还不能比较或复制。
fn comparable_list_helpers(elem: Elem) -> String {
    let l = list_suffix(elem);
    let t = ctype(elem.ty());
    let (compare, param, equal, copy) = match elem {
        Elem::Int | Elem::Bool => ("(*a > *b) - (*a < *b)", t.clone(), "x == v", "x"),
        // NaN 排在所有数之后；排序稳定，-0.0 与 0.0 保持原来的先后。
        Elem::Float => (
            "*a < *b ? -1 : *a > *b ? 1 : isnan(*a) - isnan(*b)",
            t.clone(),
            "x == v",
            "x",
        ),
        Elem::Str => (
            "string_compare(a, b)",
            "S *".into(),
            "string_equal(&x, v)",
            "copy(&x)",
        ),
        Elem::Record(_) => return String::new(),
    };
    format!(
        "static int cmp{l}({t} *a, {t} *b) {{ return {compare}; }}
static void sort{l}({l} *l) {{
    size_t n = l->len, width, i;
    {t} *a = l->data, *b, *tmp;
    if (n < 2) return;
    if (n > SIZE_MAX / 2 / sizeof({t})) list_alloc_failed();
    b = ({t}*)malloc(n * sizeof({t}));
    if (!b) list_alloc_failed();
    for (width = 1; width < n; width *= 2) {{
        for (i = 0; i < n; i += 2 * width) {{
            size_t mid = i + width < n ? i + width : n, hi = i + 2 * width < n ? i + 2 * width : n;
            size_t x = i, y = mid, k = i;
            while (x < mid && y < hi) b[k++] = cmp{l}(&a[y], &a[x]) < 0 ? a[y++] : a[x++];
            while (x < mid) b[k++] = a[x++];
            while (y < hi) b[k++] = a[y++];
        }}
        tmp = a; a = b; b = tmp;
    }}
    if (a != l->data) {{ memcpy(l->data, a, n * sizeof({t})); free(a); }} else free(b);
}}
static int64_t index{l}({l} *l, {param} v) {{ size_t i; for (i = 0; i < l->len; i++) {{ {t} x = l->data[i]; if ({equal}) return (int64_t)i; }} return -1; }}
static int has{l}({l} *l, {param} v) {{ return index{l}(l, v) >= 0; }}
static {l} slice{l}({l} *l, int64_t from, int64_t to) {{ {l} out = {{0}}; int64_t i; if (from < 0 || to < from || (uint64_t)to > l->len) list_bounds(); for (i = from; i < to; i++) {{ {t} x = l->data[i]; push{l}(&out, {copy}); }} return out; }}
"
    )
}
fn ctype(ty: Type) -> String {
    match ty {
        Type::Record(id) => return format!("R{id}"),
        Type::RecordRef(id, _) => return format!("R{id}*"),
        Type::List(elem) => return list_suffix(elem),
        Type::ListRef(elem, _) => return format!("{}*", list_suffix(elem)),
        Type::Map(key, value) => return map_name(key, value),
        Type::MapRef(key, value, _) => return format!("{}*", map_name(key, value)),
        Type::Owned => "S",
        Type::Shared | Type::Mutable => "S*",
        Type::Int => "int64_t",
        Type::Float => "double",
        Type::Bool => "int",
        Type::ArrayInt(length) => return format!("int64_t[{length}]"),
        Type::Unit => "void",
    }
    .into()
}
impl Generator<'_> {
    fn ty(&self, expr: &Expr) -> Type {
        match expr {
            Expr::Record(name, _) | Expr::Variant(name, _, _) => Type::Record(record_id(name)),
            // 分支的值可能用到绑定：在带上第一个分支绑定的副本里求类型。
            Expr::Match(scrutinee, arms) => {
                let id = self.scrutinee_id(scrutinee);
                let (pattern, value) = &arms[0];
                let mut locals = self.locals.clone();
                for (name, _, ty) in self.arm_bindings(id, pattern) {
                    locals.insert(name, (String::new(), ty));
                }
                Generator {
                    module: self.module,
                    symbols: self.symbols,
                    locals,
                    owners: Vec::new(),
                    loops: Vec::new(),
                    proofs: ranges::Proofs::default(),
                    call_targets: BTreeMap::new(),
                    code: String::new(),
                    next: 0,
                    result: self.result,
                    error: self.error,
                    containers: Containers::new(),
                }
                .ty(value)
            }
            Expr::Field(local, field) => {
                field_path(&self.module.structs, self.locals[local].1, field)
                    .expect("checked field path")
                    .1
            }
            Expr::Int(_) | Expr::ByteLen(_) | Expr::CharLen(_) => Type::Int,
            Expr::Float(_) => Type::Float,
            Expr::Array(values) => Type::ArrayInt(values.len()),
            Expr::Index(local, _) => match self.place_ty(local) {
                Type::ArrayInt(_) => Type::Int,
                Type::List(elem) | Type::ListRef(elem, _) => elem.ty(),
                Type::Map(_, value) | Type::MapRef(_, value, _) => value.ty(),
                _ => unreachable!(),
            },
            Expr::MapLit(entries) => {
                let (key, value) = entries.first().expect("checked non-empty map literal");
                Type::Map(elem_of(self.ty(key)), elem_of(self.ty(value)))
            }
            Expr::IndexField(path, _, field) => self.element_ty(path, field),
            Expr::BorrowIndex(path, _, field, mutable) => {
                borrow_of(self.element_ty(path, field), *mutable).expect("checked borrow")
            }
            Expr::Method(path, method, _) => {
                method_signature(self.place_ty(path), method)
                    .expect("checked method")
                    .2
            }
            Expr::ElementMethod(path, _, field, method, _) => {
                method_signature(self.element_ty(path, field), method)
                    .expect("checked method")
                    .2
            }
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
            Expr::Unary('!', _) => Type::Bool,
            Expr::Unary(_, value) => numeric(self.ty(value)),
            Expr::Binary(op, left, _) if op == "+" && is_text(self.ty(left)) => Type::Owned,
            Expr::Binary(op, left, _) => {
                if matches!(op.as_str(), "+" | "-" | "*" | "/" | "%") {
                    numeric(self.ty(left))
                } else {
                    Type::Bool
                }
            }
            Expr::If(_, yes, _) => self.ty(yes),
        }
    }
    /// 字符串操作数的只读指针：变量、字段与元素直接取地址，其他表达式先求值成临时串。
    fn text_value(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Variable(name) | Expr::Borrow(name, _) => self.pointer(name),
            Expr::Field(local, field) => format!("&{}", self.field(local, field)),
            Expr::Index(local, index) => self.expr(&Expr::BorrowIndex(
                local.clone(),
                index.clone(),
                String::new(),
                false,
            )),
            Expr::IndexField(local, index, field) => self.expr(&Expr::BorrowIndex(
                local.clone(),
                index.clone(),
                field.clone(),
                false,
            )),
            _ => {
                let value = self.expr(expr);
                format!("&{value}")
            }
        }
    }
    fn scalar(&mut self, value: &str, ty: Type) -> String {
        note(&mut self.containers, ty);
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
            Type::Float => format!("from_f64({pointer})"),
            _ => format!("from_bool({pointer})"),
        }
    }
    fn place_ty(&self, path: &str) -> Type {
        match path.split_once('.') {
            Some((root, field)) => {
                field_path(&self.module.structs, self.locals[root].1, field)
                    .expect("checked field path")
                    .1
            }
            None => self.locals[path].1,
        }
    }
    /// 借用参数位置上的值：变量、显式借用或列表元素借用，结果都是指针。
    fn borrowed(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Variable(name) | Expr::Borrow(name, _) => self.pointer(name),
            Expr::BorrowIndex(_, _, _, _) => self.expr(expr),
            // 字面量与插值借给 `&string` 参数：先生成临时字符串，作用域结束时释放。
            Expr::Text(_) | Expr::Format(_) => {
                let value = self.expr(expr);
                format!("&{value}")
            }
            _ => unreachable!("checked borrow"),
        }
    }
    /// 按形参类型传值：owned 移交，标量按值，借用传指针。
    fn argument(&mut self, arg: &Expr, ty: Type) -> String {
        if owned(ty) {
            let value = self.expr_as(arg, ty);
            self.moved(&value, ty)
        } else if by_value(ty) {
            self.expr(arg)
        } else {
            self.borrowed(arg)
        }
    }
    /// 容器下标：列表是 i64，字典是键（字符串键传指针）。
    fn key(&mut self, index: &Expr, container: Type) -> String {
        match container {
            Type::Map(Elem::Str, _) => self.borrowed(index),
            _ => self.expr(index),
        }
    }
    /// 按目标类型生成值：列表字面量只有知道目标类型才能和定长数组区分开。
    fn expr_as(&mut self, expr: &Expr, ty: Type) -> String {
        match (expr, ty) {
            (Expr::Array(values), Type::List(elem)) => {
                let local = self.scalar("{0}", ty);
                self.owners.push((local.clone(), ty));
                for value in values {
                    let value = self.expr(value);
                    let value = self.moved(&value, elem.ty());
                    self.code
                        .push_str(&format!("push{}(&{local}, {value});\n", list_suffix(elem)));
                }
                local
            }
            (Expr::MapLit(entries), Type::Map(key, value)) => {
                let local = self.scalar("{0}", ty);
                self.owners.push((local.clone(), ty));
                for (k, v) in entries {
                    let k = self.expr(k);
                    let k = if key == Elem::Str { format!("&{k}") } else { k };
                    let v = self.expr(v);
                    let v = self.moved(&v, value.ty());
                    self.code
                        .push_str(&format!("set{}(&{local}, {k}, {v});\n", ctype(ty)));
                }
                local
            }
            _ => self.expr(expr),
        }
    }
    /// 把已生成的 owned 值移交出去（实参、字段、返回值）；标量原样返回。
    fn moved(&self, value: &str, ty: Type) -> String {
        if owned(ty) {
            format!("{}(&{value})", take_fn(ty))
        } else {
            value.to_owned()
        }
    }
    fn pointer(&self, name: &str) -> String {
        if let Some((root, field)) = name.split_once('.') {
            return format!("&{}", self.field(root, field));
        }
        let (local, ty) = &self.locals[name];
        if owned(*ty) {
            format!("&{local}")
        } else {
            local.clone()
        }
    }
    fn field(&self, local: &str, field: &str) -> String {
        let (name, ty) = &self.locals[local];
        let target = if matches!(ty, Type::RecordRef(_, _)) {
            format!("(*{name})")
        } else {
            name.clone()
        };
        target + &self.members(*ty, field)
    }
    /// 结构体里字段路径对应的 C 成员访问后缀，如 `.m0.m2`；路径为空时是空串。
    fn members(&self, ty: Type, field: &str) -> String {
        if field.is_empty() {
            return String::new();
        }
        field_path(&self.module.structs, ty, field)
            .expect("checked field path")
            .0
            .iter()
            .map(|index| format!(".m{index}"))
            .collect()
    }
    /// 容器元素（或元素里的字段）的类型。
    fn element_ty(&self, path: &str, field: &str) -> Type {
        let elem = match self.place_ty(path).container() {
            Some(Type::List(elem) | Type::Map(_, elem)) => elem.ty(),
            _ => unreachable!("checked element"),
        };
        if field.is_empty() {
            elem
        } else {
            field_path(&self.module.structs, elem, field)
                .expect("checked field path")
                .1
        }
    }
    /// 容器元素（或元素里的字段）的 C 左值：先求下标，再取容器指针。
    fn element(&mut self, path: &str, index: &Expr, field: &str) -> String {
        let container = self.place_ty(path).container().expect("checked element");
        let elem = match container {
            Type::List(elem) | Type::Map(_, elem) => elem.ty(),
            _ => unreachable!("container type"),
        };
        let key = self.key(index, container);
        let pointer = self.pointer(path);
        format!(
            "(*at{}({pointer}, {key})){}",
            ctype(container),
            self.members(elem, field)
        )
    }
    fn expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Record(name, fields) => {
                let ty = Type::Record(record_id(name));
                let local = self.scalar("{0}", ty);
                self.owners.push((local.clone(), ty));
                let declaration = self.module.structs[name].clone();
                for (field, expr) in fields {
                    let index = declaration
                        .fields
                        .iter()
                        .position(|(n, _)| n == field)
                        .unwrap();
                    let ty = declaration.fields[index].1;
                    let value = self.expr_as(expr, ty);
                    self.code
                        .push_str(&format!("{local}.m{index} = {};\n", self.moved(&value, ty)));
                }
                local
            }
            Expr::Field(local, field) => self.scalar(&self.field(local, field), self.ty(expr)),
            Expr::Variable(name) if aggregate(self.locals[name].1) => {
                let ty = self.locals[name].1;
                let local = self.scalar(&format!("take{}({})", ctype(ty), self.pointer(name)), ty);
                self.owners.push((local.clone(), ty));
                local
            }
            Expr::StringEqual(left, right) => {
                let left = self.borrowed(left);
                let right = self.borrowed(right);
                self.scalar(&format!("string_equal({left}, {right})"), Type::Bool)
            }
            Expr::ByteLen(value) => {
                let pointer = self.borrowed(value);
                self.scalar(&format!("byte_len({pointer})"), Type::Int)
            }
            Expr::CharLen(value) => {
                let pointer = self.borrowed(value);
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
            Expr::Index(local, index) if self.place_ty(local).container().is_some() => {
                let container = self.place_ty(local).container().expect("guarded");
                let key = self.key(index, container);
                let pointer = self.pointer(local);
                self.scalar(
                    &format!("*at{}({pointer}, {key})", ctype(container)),
                    self.ty(expr),
                )
            }
            Expr::BorrowIndex(path, index, field, _) => {
                let ty = self.ty(expr);
                let target = self.element(path, index, field);
                self.scalar(&format!("&{target}"), ty)
            }
            Expr::IndexField(path, index, field) => {
                let ty = self.ty(expr);
                let target = self.element(path, index, field);
                if by_value(ty) {
                    self.scalar(&target, ty)
                } else {
                    self.temporary(&format!("copy(&{target})"))
                }
            }
            Expr::Method(path, method, args) => {
                let receiver = self.place_ty(path);
                let (_, params, result) =
                    method_signature(receiver, method).expect("checked method");
                // 参数先求值，再取接收者指针（与检查器的借用顺序一致）。
                let mut values = Vec::new();
                for (arg, ty) in args.iter().zip(&params) {
                    values.push(self.argument(arg, *ty));
                }
                values.insert(0, self.pointer(path));
                self.method_call(receiver, method, &values, result)
            }
            Expr::ElementMethod(path, index, field, method, args) => {
                let receiver = self.element_ty(path, field);
                let (_, params, result) =
                    method_signature(receiver, method).expect("checked method");
                let mut values = Vec::new();
                for (arg, ty) in args.iter().zip(&params) {
                    values.push(self.argument(arg, *ty));
                }
                let target = self.element(path, index, field);
                values.insert(0, format!("&{target}"));
                self.method_call(receiver, method, &values, result)
            }
            Expr::MapLit(_) => {
                let ty = self.ty(expr);
                self.expr_as(expr, ty)
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
                let mut name: Option<String> = None;
                for segment in segments {
                    let part = match segment {
                        Expr::Text(text) => self.literal(text),
                        Expr::Variable(name) => {
                            let pointer = self.pointer(name);
                            self.text_of(pointer, self.ty(segment))
                        }
                        Expr::Field(local, field) => {
                            let ty = self.ty(segment);
                            let target = self.field(local, field);
                            let target = if by_value(ty) {
                                target
                            } else {
                                format!("&{target}")
                            };
                            self.text_of(target, ty)
                        }
                        _ => {
                            let ty = self.ty(segment);
                            if is_text(ty) {
                                let pointer = self.text_value(segment);
                                format!("copy({pointer})")
                            } else {
                                let value = self.expr(segment);
                                self.text_of(value, ty)
                            }
                        }
                    };
                    // 每段求值后立刻拼接，保持从左到右的顺序。
                    match &name {
                        None => name = Some(self.temporary(&part)),
                        Some(name) => self
                            .code
                            .push_str(&format!("{name} = cat({name}, {part});\n")),
                    }
                }
                name.expect("interpolation has segments")
            }
            Expr::Variant(name, variant, args) => {
                let id = record_id(name);
                let ty = Type::Record(id);
                let local = self.scalar("{0}", ty);
                self.owners.push((local.clone(), ty));
                let (tag, first) = self.variant_of(id, variant);
                self.code.push_str(&format!("{local}.m0 = {tag};\n"));
                let fields = self.module.structs[name].fields.clone();
                for (j, arg) in args.iter().enumerate() {
                    let field = fields[first + j].1;
                    let value = self.expr_as(arg, field);
                    self.code.push_str(&format!(
                        "{local}.m{} = {};\n",
                        first + j,
                        self.moved(&value, field)
                    ));
                }
                local
            }
            Expr::Match(scrutinee, arms) => {
                let ty = self.ty(expr);
                let id = self.scrutinee_id(scrutinee);
                let handle = self.scrutinee(scrutinee, id);
                let result = self.scalar("{0}", ty);
                for (index, (pattern, value)) in arms.iter().enumerate() {
                    let opener =
                        self.arm_opener(&handle, id, pattern, index, index + 1 == arms.len());
                    self.code.push_str(&opener);
                    let names = self.bind(&handle, id, pattern);
                    self.branch_into(&result, value, ty);
                    for name in names {
                        self.locals.remove(&name);
                    }
                }
                self.code.push_str("}\n");
                if owned(ty) {
                    self.owners.push((result.clone(), ty));
                }
                result
            }
            Expr::If(condition, yes, no) => {
                let ty = self.ty(expr);
                let condition = self.expr(condition);
                let result = self.scalar("{0}", ty);
                self.code.push_str(&format!("if ({condition}) {{\n"));
                self.branch_into(&result, yes, ty);
                self.code.push_str("} else {\n");
                self.branch_into(&result, no, ty);
                self.code.push_str("}\n");
                if owned(ty) {
                    self.owners.push((result.clone(), ty));
                }
                result
            }
            Expr::Int(value) => self.scalar(
                &if *value == i64::MIN {
                    "INT64_MIN".into()
                } else {
                    format!("INT64_C({value})")
                },
                Type::Int,
            ),
            Expr::Float(value) => self.scalar(&float_literal(*value), Type::Float),
            Expr::Bool(value) => self.scalar(if *value { "1" } else { "0" }, Type::Bool),
            Expr::Unary(op, value) => {
                let ty = self.ty(expr);
                let value = self.expr(value);
                self.scalar(
                    &if *op == '-' && (ty == Type::Float || self.proofs.arithmetic(expr)) {
                        format!("(-{value})")
                    } else if *op == '-' {
                        format!("sub(0, {value})")
                    } else {
                        format!("!{value}")
                    },
                    ty,
                )
            }
            Expr::Binary(op, left, right)
                if is_text(self.ty(left))
                    && matches!(op.as_str(), "+" | "==" | "!=" | "<" | "<=" | ">" | ">=") =>
            {
                let a = self.text_value(left);
                let b = self.text_value(right);
                if op == "+" {
                    self.temporary(&format!("cat(copy({a}), copy({b}))"))
                } else {
                    self.scalar(&format!("(string_compare({a}, {b}) {op} 0)"), Type::Bool)
                }
            }
            // f64 按 IEEE 754 运算：不检查溢出，除以零得到 inf 或 NaN。
            Expr::Binary(op, left, right) if self.ty(left) == Type::Float => {
                let ty = self.ty(expr);
                let a = self.expr(left);
                let b = self.expr(right);
                self.scalar(
                    &if op == "%" {
                        format!("fmod({a}, {b})")
                    } else {
                        format!("({a} {op} {b})")
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
                    "%" if !safe_divisor => Some("zam_remainder"),
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
            Expr::Variable(name) if by_value(self.locals[name].1) => {
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
    /// 枚举里某个变体的（标签, 第一个载荷字段的下标）。
    fn variant_of(&self, id: u64, variant: &str) -> (usize, usize) {
        let structure = self
            .module
            .structs
            .iter()
            .find(|(name, _)| record_id(name) == id)
            .expect("checked enum")
            .1;
        structure
            .variants
            .iter()
            .enumerate()
            .find(|(_, (other, _, _))| other == variant)
            .map(|(tag, (_, first, _))| (tag, *first))
            .expect("checked variant")
    }
    /// match 被匹配值的枚举 id。
    fn scrutinee_id(&self, expr: &Expr) -> u64 {
        let ty = match expr {
            Expr::Variable(name) => self.locals[name].1,
            Expr::Field(local, field) => self.place_ty(&format!("{local}.{field}")),
            Expr::Index(path, _) if self.place_ty(path).container().is_some() => {
                self.element_ty(path, "")
            }
            Expr::IndexField(path, _, field) => self.element_ty(path, field),
            _ => self.ty(expr),
        };
        match ty {
            Type::Record(id) | Type::RecordRef(id, _) => id,
            _ => unreachable!("checked match"),
        }
    }
    /// 分支的绑定：（名字, 载荷成员下标, 类型）；标量按值，其他载荷是只读借用。
    fn arm_bindings(&self, id: u64, pattern: &Pattern) -> Vec<(String, usize, Type)> {
        let Some(variant) = &pattern.0 else {
            return Vec::new();
        };
        let (_, first) = self.variant_of(id, variant);
        let fields = &self
            .module
            .structs
            .iter()
            .find(|(name, _)| record_id(name) == id)
            .expect("checked enum")
            .1
            .fields;
        pattern
            .1
            .iter()
            .enumerate()
            .filter(|(_, binding)| *binding != "_")
            .map(|(j, binding)| {
                let ty = fields[first + j].1;
                let bound = if by_value(ty) {
                    ty
                } else {
                    borrow_of(ty, false).expect("owned payload")
                };
                (binding.clone(), first + j, bound)
            })
            .collect()
    }
    /// 求出被匹配值，返回指向它的 C 变量：位置直接取地址，其他表达式先求值成临时值。
    fn scrutinee(&mut self, expr: &Expr, id: u64) -> String {
        let pointer = match expr {
            Expr::Variable(name) => self.pointer(name),
            Expr::Field(local, field) => format!("&{}", self.field(local, field)),
            Expr::Index(path, index) if self.place_ty(path).container().is_some() => {
                format!("&{}", self.element(path, index, ""))
            }
            Expr::IndexField(path, index, field) => {
                format!("&{}", self.element(path, index, field))
            }
            _ => {
                let value = self.expr(expr);
                format!("&{value}")
            }
        };
        self.scalar(&pointer, Type::RecordRef(id, false))
    }
    /// 第 `index` 个分支的开头；最后一个分支不再判断标签（检查器保证分支覆盖了所有变体）。
    fn arm_opener(
        &self,
        handle: &str,
        id: u64,
        pattern: &Pattern,
        index: usize,
        last: bool,
    ) -> String {
        let condition = match &pattern.0 {
            Some(variant) if !last => Some(format!(
                "{handle}->m0 == {}",
                self.variant_of(id, variant).0
            )),
            _ => None,
        };
        match (index, condition) {
            (0, Some(condition)) => format!("if ({condition}) {{\n"),
            (0, None) => "{\n".into(),
            (_, Some(condition)) => format!("}} else if ({condition}) {{\n"),
            (_, None) => "} else {\n".into(),
        }
    }
    /// 在分支开头声明绑定，返回绑定名，分支结束后由调用方移除。
    fn bind(&mut self, handle: &str, id: u64, pattern: &Pattern) -> Vec<String> {
        let mut names = Vec::new();
        for (name, member, ty) in self.arm_bindings(id, pattern) {
            let access = format!("{handle}->m{member}");
            let value = if ty.is_reference() {
                self.scalar(&format!("&{access}"), ty)
            } else {
                self.scalar(&access, ty)
            };
            self.locals.insert(name.clone(), (value, ty));
            names.push(name);
        }
        names
    }
    /// 容器方法的 C 调用；`values` 的第一项是接收者指针。
    fn method_call(
        &mut self,
        receiver: Type,
        method: &str,
        values: &[String],
        result: Type,
    ) -> String {
        let container = receiver.container().expect("checked method");
        let list = matches!(container, Type::List(_));
        let name = match method {
            "contains" => "has",
            "get" => "getor",
            "remove" if list => "removeat",
            "remove" => "del",
            "index_of" => "index",
            other => other,
        };
        let call = format!("{name}{}({})", ctype(container), values.join(", "));
        self.call_value(&call, result)
    }
    /// if 表达式的一个分支：求值、把结果移进 `target`，再释放分支里的临时值。
    fn branch_into(&mut self, target: &str, expr: &Expr, ty: Type) {
        let start = self.owners.len();
        let value = self.expr_as(expr, ty);
        let value = self.moved(&value, ty);
        self.code.push_str(&format!("{target} = {value};\n"));
        for (name, ty) in &self.owners[start..] {
            self.code.push_str(&format!("{}(&{name});\n", drop_fn(*ty)));
        }
        self.owners.truncate(start);
    }
    // 从结果结构体里取出成功值；Unit 没有值，返回空串。
    fn result_value(&mut self, temp: &str, result: Type) -> String {
        if result == Type::Unit {
            return String::new();
        }
        if aggregate(result) {
            let value = self.scalar(&format!("{}(&{temp}.value)", take_fn(result)), result);
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
            values.push(self.argument(arg, ty));
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
            if owned(result) {
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
                Instruction::Match(scrutinee, arms) => {
                    let id = self.scrutinee_id(scrutinee);
                    let handle = self.scrutinee(scrutinee, id);
                    let mut all = true;
                    for (index, (pattern, body)) in arms.iter().enumerate() {
                        let opener =
                            self.arm_opener(&handle, id, pattern, index, index + 1 == arms.len());
                        self.code.push_str(&opener);
                        let names = self.bind(&handle, id, pattern);
                        all &= self.block(body, result_type);
                        for name in names {
                            self.locals.remove(&name);
                        }
                    }
                    self.code.push_str("}\n");
                    returned = all;
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
                Instruction::Let(name, _, annotation, expr) => {
                    let ty = annotation.unwrap_or_else(|| self.ty(expr));
                    let value = self.expr_as(expr, ty);
                    self.locals.insert(name.clone(), (value, ty));
                }
                Instruction::Each(name, path, nested) => {
                    let container = self.place_ty(path).container().expect("checked loop");
                    let borrowed = match container {
                        Type::List(elem) => Type::ListRef(elem, false),
                        Type::Map(key, value) => Type::MapRef(key, value, false),
                        _ => unreachable!(),
                    };
                    // 循环体不能修改被遍历的容器（检查器冻结了根绑定），长度与存储整轮不变。
                    let handle = self.scalar(&self.pointer(path), borrowed);
                    let index = format!("v{}", self.next);
                    self.next += 1;
                    self.code.push_str(&format!(
                        "for (size_t {index} = 0; {index} < {handle}->len; {index}++) {{\n"
                    ));
                    // 列表取元素；字典按插入顺序取键，跳过已删除的项。
                    let (elem, access) = match container {
                        Type::List(elem) => (elem, format!("{handle}->data[{index}]")),
                        Type::Map(key, _) => {
                            self.code.push_str(&format!(
                                "if (!{handle}->entries[{index}].live) continue;\n"
                            ));
                            (key, format!("{handle}->entries[{index}].key"))
                        }
                        _ => unreachable!(),
                    };
                    let ty = elem.item();
                    let item = if ty.is_reference() {
                        self.scalar(&format!("&{access}"), ty)
                    } else {
                        self.scalar(&access, ty)
                    };
                    self.locals.insert(name.clone(), (item, ty));
                    self.loops.push(self.owners.len());
                    self.block(nested, result_type);
                    self.loops.pop();
                    self.locals.remove(name);
                    self.code.push_str("}\n");
                }
                Instruction::AssignIndex(local, index, expr)
                    if self.place_ty(local).container().is_some() =>
                {
                    let container = self.place_ty(local).container().expect("guarded");
                    let key = self.key(index, container);
                    let value = self.expr(expr);
                    let pointer = self.pointer(local);
                    let c = ctype(container);
                    self.code.push_str(&match container {
                        Type::Map(_, elem) => format!(
                            "set{c}({pointer}, {key}, {});\n",
                            self.moved(&value, elem.ty())
                        ),
                        Type::List(elem) if owned(elem.ty()) => format!(
                            "{{ {} *slot = at{c}({pointer}, {key}); {}(slot); *slot = {}(&{value}); }}\n",
                            ctype(elem.ty()),
                            drop_fn(elem.ty()),
                            take_fn(elem.ty())
                        ),
                        Type::List(_) => format!("*at{c}({pointer}, {key}) = {value};\n"),
                        _ => unreachable!(),
                    });
                }
                Instruction::AssignIndexField(local, index, field, expr) => {
                    // 下标、新值、容器指针依次求值，与检查器的借用顺序一致。
                    let container = self.place_ty(local).container().expect("checked element");
                    let elem = match container {
                        Type::List(elem) | Type::Map(_, elem) => elem.ty(),
                        _ => unreachable!("container type"),
                    };
                    let ty = self.element_ty(local, field);
                    let key = self.key(index, container);
                    let value = self.expr_as(expr, ty);
                    let pointer = self.pointer(local);
                    let target = format!(
                        "(*at{}({pointer}, {key})){}",
                        ctype(container),
                        self.members(elem, field)
                    );
                    self.code.push_str(&if owned(ty) {
                        format!(
                            "{{ {} *slot = &{target}; {}(slot); *slot = {}(&{value}); }}\n",
                            ctype(ty),
                            drop_fn(ty),
                            take_fn(ty)
                        )
                    } else {
                        format!("{target} = {value};\n")
                    });
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
                    let ty = field_path(&self.module.structs, self.locals[local].1, field)
                        .expect("checked field path")
                        .1;
                    let value = self.expr_as(expr, ty);
                    let target = self.field(local, field);
                    if owned(ty) {
                        self.code.push_str(&format!(
                            "{}(&{target}); {target} = {}(&{value});\n",
                            drop_fn(ty),
                            take_fn(ty)
                        ));
                    } else {
                        self.code.push_str(&format!("{target} = {value};\n"));
                    }
                }
                Instruction::Assign(name, expr) => {
                    // 通过 `&mut` 参数赋值时，写的是被借用的那个值。
                    let local_ty = self.locals[name].1;
                    let ty = if local_ty.is_reference() {
                        local_ty.borrowable().expect("reference type").0
                    } else {
                        local_ty
                    };
                    let value = self.expr_as(expr, ty);
                    if by_value(ty) {
                        self.code
                            .push_str(&format!("{} = {value};\n", self.locals[name].0));
                        continue;
                    }
                    let pointer = self.pointer(name);
                    self.code.push_str(&format!(
                        "{}({pointer});\n*({pointer}) = {}(&{value});\n",
                        drop_fn(ty),
                        take_fn(ty)
                    ));
                }
                Instruction::Print(expr) => {
                    let ty = self.ty(expr);
                    if by_value(ty) {
                        let value = self.expr(expr);
                        self.code.push_str(&match ty {
                            Type::Int => {
                                format!("if (printf(\"%\" PRId64 \"\\n\", {value}) < 0) exit(1);\n")
                            }
                            Type::Float => format!("print_f64({value});\n"),
                            _ => format!(
                                "if (puts({value} ? \"true\" : \"false\") == EOF) exit(1);\n"
                            ),
                        });
                        continue;
                    }
                    let pointer = match expr {
                        Expr::Variable(name) | Expr::Borrow(name, _) => self.pointer(name),
                        Expr::BorrowIndex(_, _, _, _)
                        | Expr::Index(_, _)
                        | Expr::IndexField(_, _, _) => self.text_value(expr),
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
                        let value = self.expr_as(expr, result_type);
                        self.code.push_str(&format!(
                            "{} result = {};\n",
                            ctype(result_type),
                            self.moved(&value, result_type)
                        ));
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
                Instruction::AssignField(name, _, _)
                | Instruction::AssignIndex(name, _, _)
                | Instruction::AssignIndexField(name, _, _, _) => {
                    mutated.insert(name.clone());
                }
                Instruction::If(_, yes, no) => {
                    Self::loop_mutations(yes, mutated);
                    Self::loop_mutations(no, mutated);
                }
                Instruction::While(_, nested)
                | Instruction::For(_, _, _, nested)
                | Instruction::Each(_, _, nested) => Self::loop_mutations(nested, mutated),
                Instruction::Match(_, arms) => {
                    for (_, body) in arms {
                        Self::loop_mutations(body, mutated);
                    }
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
            Expr::Call(_, args)
            | Expr::Method(_, _, args)
            | Expr::Variant(_, _, args)
            | Expr::Array(args)
            | Expr::Format(args) => {
                for arg in args {
                    expr(arg, visit);
                }
            }
            Expr::Match(scrutinee, arms) => {
                expr(scrutinee, visit);
                for (_, value) in arms {
                    expr(value, visit);
                }
            }
            Expr::ElementMethod(_, index, _, _, args) => {
                expr(index, visit);
                for arg in args {
                    expr(arg, visit);
                }
            }
            Expr::Unary(_, inner)
            | Expr::Try(inner)
            | Expr::ByteLen(inner)
            | Expr::CharLen(inner)
            | Expr::Index(_, inner)
            | Expr::IndexField(_, inner, _)
            | Expr::BorrowIndex(_, inner, _, _) => expr(inner, visit),
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
            Expr::MapLit(entries) => {
                for (key, value) in entries {
                    expr(key, visit);
                    expr(value, visit);
                }
            }
            Expr::If(condition, yes, no) => {
                expr(condition, visit);
                expr(yes, visit);
                expr(no, visit);
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
            Instruction::AssignIndex(_, index, value)
            | Instruction::AssignIndexField(_, index, _, value) => {
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
            Instruction::Each(_, _, body) => visit_exprs(body, visit),
            Instruction::Match(scrutinee, arms) => {
                expr(scrutinee, visit);
                for (_, body) in arms {
                    visit_exprs(body, visit);
                }
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
    let record_name = |id: u64| {
        module
            .structs
            .keys()
            .find(|name| record_id(name) == id)
            .expect("checked struct")
            .clone()
    };
    let mut records = std::collections::BTreeSet::new();
    let mut needed = std::collections::BTreeSet::new();
    for (name, function) in &module.functions {
        if name.split_once(':').unwrap().0 == unit {
            needed.insert(name.clone());
            visit_exprs(&function.body, &mut |expr| match expr {
                Expr::Call(name, _) => {
                    needed.insert(name.clone());
                }
                Expr::Record(name, _) | Expr::Variant(name, _, _) => {
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
            if let Some(id) = record_of(*ty) {
                records.insert(record_name(id));
            }
        }
    }
    // 列表与字典的 C 辅助按用到的类型生成：签名、结构体字段，以及函数体里出现过的每个值。
    let mut containers = Containers::new();
    for name in &needed {
        let function = &module.functions[name];
        for (_, ty) in &function.params {
            note(&mut containers, *ty);
        }
        note(&mut containers, function.result);
    }
    let specialized_decls: String = specializations
        .iter()
        .map(|version| {
            format!(
                "static {};\n",
                signature(&version.name).replacen(&symbols[&version.name], &version.symbol, 1)
            )
        })
        .collect();
    // 先生成函数体并收集其中的容器类型，再按依赖顺序拼装：容器、结构体、结果结构体、声明、函数体。
    let mut bodies = String::new();
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
        bodies.push_str(&format!("{declaration} {{\n"));
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
            containers: Containers::new(),
        };
        for (i, (param, ty)) in function.params.iter().enumerate() {
            let local = format!("p{i}");
            if owned(*ty) {
                g.owners.push((local.clone(), *ty));
            }
            g.locals.insert(param.clone(), (local, *ty));
        }
        let returned = g.block(&function.body, function.result);
        g.cleanup();
        bodies.push_str(&g.code);
        if function.error.is_some() && !returned && function.result == Type::Unit {
            // 失败函数返回 Unit 时也要显式给出成功值。
            bodies.push_str(&format!("return {}();\n", fallible_ok(Type::Unit)));
        }
        bodies.push_str("}\n");
        containers.extend(g.containers);
    }
    // 结构体与容器互相引用（结构体字段是列表，列表元素是结构体），收集到不再增加为止。
    loop {
        let before = (records.len(), containers.len());
        for name in records.clone() {
            for (_, ty) in &module.structs[&name].fields {
                note(&mut containers, *ty);
                if let Some(id) = record_of(*ty) {
                    records.insert(record_name(id));
                }
            }
        }
        for (key, value) in containers.clone() {
            for elem in std::iter::once(key).chain(value) {
                if let Elem::Record(id) = elem {
                    records.insert(record_name(id));
                }
            }
        }
        if (records.len(), containers.len()) == before {
            break;
        }
    }
    let records = record_order(module, &records).expect("checked struct layouts");
    let lists: Vec<Elem> = containers
        .iter()
        .filter(|(_, value)| value.is_none())
        .map(|(elem, _)| *elem)
        .collect();
    let maps: Vec<(Elem, Elem)> = containers
        .iter()
        .filter_map(|(key, value)| value.map(|value| (*key, value)))
        .collect();
    // 类型先全部前置声明，再按依赖定义：列表与字典本体只含指针，结构体按字段的按值依赖排序，
    // 字典条目按值含结构体，放在最后；函数都在类型之后，释放函数先给出原型以支持互相调用。
    for name in &records {
        let id = record_id(name);
        source.push_str(&format!("typedef struct R{id} R{id};\n"));
    }
    for elem in &lists {
        let l = list_suffix(*elem);
        source.push_str(&format!("typedef struct {l} {l};\n"));
    }
    for (key, value) in &maps {
        source.push_str(&map_helpers(MAP_DECLARATION, *key, *value));
    }
    for elem in &lists {
        source.push_str(&list_definition(*elem));
    }
    for (key, value) in &maps {
        source.push_str(&map_helpers(MAP_DEFINITION, *key, *value));
    }
    for name in &records {
        let structure = &module.structs[name];
        let id = record_id(name);
        source.push_str(&format!(
            "struct R{id} {{ {} }};\n",
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
    }
    for (key, value) in &maps {
        source.push_str(&map_helpers(MAP_ENTRY, *key, *value));
    }
    for name in &records {
        let id = record_id(name);
        source.push_str(&format!("static void dropR{id}(R{id} *p);\n"));
    }
    if !containers.is_empty() {
        source.push_str(LIST_PRELUDE);
    }
    if !maps.is_empty() {
        source.push_str(MAP_PRELUDE);
    }
    for elem in &lists {
        source.push_str(&list_helpers(*elem));
    }
    for (key, value) in &maps {
        source.push_str(&map_helpers(MAP_TEMPLATE, *key, *value));
        source.push_str(&map_values(*key, *value));
    }
    for name in &records {
        let structure = &module.structs[name];
        let id = record_id(name);
        source.push_str(&format!("static void dropR{id}(R{id} *p) {{"));
        for (i, (_, ty)) in structure.fields.iter().enumerate() {
            if owned(*ty) {
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
    source.push_str(&specialized_decls);
    source.push_str(&bodies);
    if entry.split_once(':').unwrap().0 == unit {
        // argv 留给 std/env（Unix）；Windows 下 std/env 改读 GetCommandLineW 以拿到 UTF-8。
        source.push_str("int zam_argc;\nchar **zam_argv;\n");
        if module.functions[entry].error.is_some() {
            let structure = fallible_name(module.functions[entry].result);
            source.push_str(&format!(
                "int main(int argc, char **argv) {{ zam_argc = argc; zam_argv = argv; output_mode(); {structure} r = {}(); if (!r.ok) {{ uncaught_error(&r.error); drop(&r.error); return 1; }} return fflush(stdout) == EOF ? 1 : 0; }}\n",
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
    select(
        std::env::var("CC").ok().as_deref(),
        cfg!(windows),
        msvc,
        gnu,
    )
}

/// 选 C 工具链：显式 `CC` 优先；Windows 默认 MSVC，找不到时依次试 PATH 上的 gcc、clang；
/// 其他平台依次试 cc、clang、gcc。都找不到时给出能照做的安装提示。
fn select(
    cc: Option<&str>,
    windows: bool,
    msvc: impl FnOnce() -> Result<(String, String)>,
    gnu: impl Fn(&str) -> Result<(String, String)>,
) -> Result<(String, String)> {
    if let Some(compiler) = cc {
        return gnu(compiler);
    }
    let mut missing = None;
    if windows {
        match msvc() {
            Ok(found) => return Ok(found),
            Err(error) => missing = Some(error),
        }
    }
    let fallbacks: &[&str] = if windows {
        &["gcc", "clang"]
    } else {
        &["cc", "clang", "gcc"]
    };
    for compiler in fallbacks {
        if let Ok(found) = gnu(compiler) {
            return Ok(found);
        }
    }
    let tried = fallbacks.join(", ");
    Err(match missing {
        Some(error) => format!(
            "no C compiler found: MSVC is unavailable ({error}) and none of {tried} is on PATH; install Visual Studio Build Tools with the \"Desktop development with C++\" workload, or MinGW-w64 gcc, or set CC to a GNU-style compiler"
        ),
        None => format!(
            "no C compiler found: none of {tried} is on PATH; install one (build-essential on Debian/Ubuntu, xcode-select --install on macOS) or set CC"
        ),
    })
}

fn msvc() -> Result<(String, String)> {
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
        Path::new(&installation).join("VC/Auxiliary/Build/Microsoft.VCToolsVersion.default.txt"),
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
}

fn gnu(compiler: &str) -> Result<(String, String)> {
    let output = Command::new(compiler)
        .arg("--version")
        .output()
        .map_err(|e| format!("C compiler unavailable: {e}"))?;
    if !output.status.success() {
        return Err("C compiler version query failed".into());
    }
    Ok((
        compiler.to_owned(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    ))
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
pub const OPTIONS: &str = "c11-O2-unroll-MT-Brepro-pathmap-objects-gnu-windows-lm-v8";
pub fn compile(tool: &str, dir: &Path, source: &str) -> Result<Vec<u8>> {
    std::fs::write(dir.join("program.c"), source).map_err(|e| e.to_string())?;
    // 编译器记录的是工作目录的写法；临时目录经过 8.3 短名（CI 的 `RUNNER~1`）、联接或
    // 符号链接（macOS 的 /var → /private/var）时，它和规范路径不同，路径映射就会失配，
    // 随机的临时目录名漏进目标文件，破坏「同键 ⇒ 同字节」。统一用规范路径做工作目录与映射源。
    let mapped = std::fs::canonicalize(dir)
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_owned();
    let output = if uses_msvc(tool) {
        let environment = msvc_environment(tool)?;
        if mapped.contains(['"', '%', '\r', '\n']) {
            return Err("unsupported native cache path".into());
        }
        // ponytail: 源码按 UTF-8 写出，cl 默认按本地代码页(如 936)读取会吞掉中文注释后的下一行；/utf-8 固定两端编码。
        Command::new(msvc_tool(&environment, "cl.exe")?)
            .current_dir(&mapped)
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
        let map = format!("-ffile-prefix-map={mapped}=.");
        Command::new(tool)
            .current_dir(&mapped)
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
        // std/math 的函数在 Linux 上属于 libm，要显式链接；macOS 与 MinGW 也接受 -lm。
        // MinGW 的 ld 默认把链接时间写进 PE 头，同一个链接键两次链接字节不同，
        // 会被缓存当成 corrupt cache entry；关掉时间戳，与 MSVC 的 /Brepro 对应。
        let mut command = Command::new(tool);
        command.current_dir(dir).args(&names);
        if cfg!(windows) {
            command.arg("-Wl,--no-insert-timestamp");
        }
        command.args(["-lm", "-o", "program.exe"]).output()
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
    fn toolchain_selection_prefers_cc_then_msvc_then_path_fallbacks() {
        let found = |name: &str| Ok((name.to_owned(), format!("{name} 1.0")));
        let missing = |_: &str| Err::<(String, String), String>("not found".into());
        let only = |wanted: &'static str| {
            move |name: &str| {
                if name == wanted {
                    found(name)
                } else {
                    missing(name)
                }
            }
        };
        let no_msvc = || Err("install MSVC C++ build tools for native builds".to_owned());
        // 显式 CC 总是优先，即使 MSVC 可用。
        assert_eq!(
            select(Some("clang"), true, || found("vcvars64.bat"), found)
                .unwrap()
                .0,
            "clang"
        );
        assert_eq!(
            select(None, true, || found("vcvars64.bat"), found)
                .unwrap()
                .0,
            "vcvars64.bat"
        );
        // Windows 缺 MSVC：先 gcc，再 clang。
        assert_eq!(select(None, true, no_msvc, found).unwrap().0, "gcc");
        assert_eq!(
            select(None, true, no_msvc, only("clang")).unwrap().0,
            "clang"
        );
        // 其他平台：cc、clang、gcc。
        assert_eq!(select(None, false, no_msvc, found).unwrap().0, "cc");
        assert_eq!(select(None, false, no_msvc, only("gcc")).unwrap().0, "gcc");
        let error = select(None, true, no_msvc, missing).unwrap_err();
        assert!(error.contains("Visual Studio Build Tools") && error.contains("gcc, clang"));
        let error = select(None, false, no_msvc, missing).unwrap_err();
        assert!(error.contains("xcode-select --install") && error.contains("cc, clang, gcc"));
    }

    #[test]
    fn interval_proofs_select_raw_arithmetic_and_array_accesses() {
        let mut module = crate::compiler::parse(
            "fn main():\n    let mut a = [0, 0, 0, 0]\n    let mut i = 0\n    while i < 4:\n        a[i] = i * 2\n        println(a[i])\n        i = i + 1",
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
            "fn main():\n    let mut a = [0, 0, 0, 0]\n    for i in 0..4:\n        a[i] = i * 2\n        println(a[i])\n        println(i + 1)",
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
        let module = scalar_module("fn accum(seed: i64) -> i64:\n    let mut acc = seed\n    let mut i = 0\n    while i < 512:\n        acc = (acc * 31 + i) % 1000003\n        i = i + 1\n    acc\nfn main():\n    let mut seed = 0\n    while seed < 8:\n        println(accum(seed))\n        seed = seed + 1");
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
        let module = scalar_module("fn sum(seed: i64) -> i64:\n    let mut total = 0\n    let mut i = 0\n    while i < 512:\n        total = total + (i + seed + 3) * (i * 5 - seed + 7)\n        i = i + 1\n    return total\nfn main():\n    let mut r = 0\n    while r < 8:\n        println(sum(r % 8))\n        r = r + 1");
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
        let module = scalar_module("fn a(n: i64) -> i64:\n    if n < 1: return n\n    b(n - 1)\nfn b(n: i64) -> i64: a(n)\nfn unknown() -> i64: 2\nfn add(n: i64) -> i64: n + 1\nfn main():\n    println(a(2))\n    println(add(unknown()))");
        assert!(!generate(&module, "case:main", "case").contains("_spec"));
    }
    #[test]
    fn specialization_caps_reuse_and_nested_calls_stay_generic() {
        let calls = (0..12)
            .map(|i| format!("    println(add({i}))\n"))
            .collect::<String>();
        let module = scalar_module(&format!("fn inner(n: i64) -> i64: n + 1\nfn add(n: i64) -> i64: inner(n)\nfn main():\n{calls}    println(add(0))"));
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
