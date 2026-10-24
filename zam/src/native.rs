use crate::compiler::{
    auto_borrow, bitwise, block_roots, borrow_of, bytes_method, captured_view, char_at, compound,
    constructed, copied, each_bindings, elem_from, field_function, field_path, func_error,
    func_sig, func_type, func_type_full, generic_value, instance_of, instance_with, is_bytes,
    is_string, lambda_body, lambda_prints, linked_tuple, loop_list, membership, method_args,
    method_signature, nest, nested, numeric, overload, promote, record_id, record_method_with,
    record_order, sort_by_type, string_method, terminates, tuple_fields, Block, Elem, Expr,
    Instruction, Module, Pattern, Receiver, Type, COPY, SET,
};
use crate::project::Result;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

#[path = "native_values.rs"]
mod native_values;
#[path = "native_ranges.rs"]
mod ranges;

const RUNTIME: &str = r#"#if !defined(_WIN32) && !defined(__APPLE__)
#define _POSIX_C_SOURCE 200809L
#endif
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <inttypes.h>
#include <limits.h>
#include <math.h>
#include <time.h>
// 线程局部变量：每个线程各有一份（报错信息缓冲区、随机数状态）。
#if defined(_MSC_VER)
#define ZAM_THREAD __declspec(thread)
#else
#define ZAM_THREAD _Thread_local
#endif
// std/thread 启动第一个线程之前置 1，之后不再改。只在单线程时安全的优化（列表缓冲区的空闲链表、
// 正则缓存）看到它就不再使用。定义在入口模块的目标文件里。
extern int zam_threaded;
// 运行时的报错（下标越界、除以零等）按 ZAMAK_LANG 显示：与编译器一致，设了 zh 就用中文，
// 不设时跟随系统语言（Unix 看 LC_ALL、LC_MESSAGES、LANG，Windows 再看用户界面语言）。只读一次。
#ifdef _WIN32
__declspec(dllimport) unsigned short __stdcall GetUserDefaultUILanguage(void);
#endif
static int zam_chinese(void) {
    static ZAM_THREAD int state = -1;
    if (state < 0) {
        const char *lang = getenv("ZAMAK_LANG");
        if (!lang || !*lang) lang = getenv("LC_ALL");
        if (!lang || !*lang) lang = getenv("LC_MESSAGES");
        if (!lang || !*lang) lang = getenv("LANG");
        if (lang && *lang) {
            state = (lang[0] == 'z' || lang[0] == 'Z') && (lang[1] == 'h' || lang[1] == 'H');
        } else {
#ifdef _WIN32
            /* 与编译器一致：没设环境变量时看用户界面语言（主语言 0x04 是中文）。 */
            state = (GetUserDefaultUILanguage() & 0x3ff) == 0x04;
#else
            state = 0;
#endif
        }
    }
    return state;
}
// 报错原文 → 中文。原文同时是查表的键，两边各写一次，其余地方只写原文。
static const char *zam_zh(const char *en) {
    static const struct { const char *en, *zh; } table[] = {
        {"integer arithmetic error", "整数运算出错"},
        {"division by zero", "除以零"},
        {"array index out of bounds", "数组下标越界"},
        {"allocation failed", "内存分配失败"},
        {"time value out of range", "时间值超出范围"},
        {"string index out of bounds", "字符串下标越界"},
        {"negative exponent in an integer power; use floats such as 2.0 ** -1.0", "整数次幂的指数不能为负；要用小数写，例如 2.0 ** -1.0"},
        {"random.randint needs a <= b", "random.randint 要求 a <= b"},
        {"random.randrange needs start < stop", "random.randrange 要求 start < stop"},
        {"exit code must be between 0 and 255", "退出码必须在 0 到 255 之间"},
        {"f64 value out of i64 range", "f64 的值超出 i64 的范围"},
        {"fmt.fixed digits must be between 0 and 30", "fmt.fixed 的小数位数必须在 0 到 30 之间"},
        {"invalid code point", "码点不合法"},
        {"key not found", "找不到这个键"},
        {"list index out of bounds", "列表下标越界"},
        {"pop from empty list", "对空列表调用 pop"},
        {"min() of an empty list", "对空列表调用 min()"},
        {"max() of an empty list", "对空列表调用 max()"},
        {"cannot start a thread", "无法启动线程"},
        {NULL, NULL},
    };
    int i;
    if (!zam_chinese()) return en;
    for (i = 0; table[i].en; i++) if (strcmp(table[i].en, en) == 0) return table[i].zh;
    return en;
}
static void zam_panic_one(const char *en) { fflush(stdout); fprintf(stderr, "zam: %s\n", zam_zh(en)); }
static void arithmetic_error(void) { zam_panic_one("integer arithmetic error"); exit(1); }
// 移位量只能是 0–63；左移按无符号做（丢掉移出的位，不算溢出），右移是算术右移（向负无穷取整）。
static int64_t zam_shl(int64_t a, int64_t n) { if (n < 0 || n > 63) arithmetic_error(); return (int64_t)((uint64_t)a << n); }
static int64_t zam_shr(int64_t a, int64_t n) { if (n < 0 || n > 63) arithmetic_error(); return a >= 0 ? a >> n : ~(~a >> n); }
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
#if defined(_MSC_VER) && defined(_M_X64)
#include <intrin.h>
/* MSVC x64：_mul128 一条指令给出 128 位乘积；高 64 位不是低 64 位的符号扩展就是溢出。
   可移植写法每次乘法都要做一次 64 位除法，热循环里会慢一倍。 */
static int64_t mul(int64_t a, int64_t b) {
    int64_t high;
    int64_t low = _mul128(a, b, &high);
    if (high != (low >> 63)) arithmetic_error();
    return low;
}
#else
static int64_t mul(int64_t a, int64_t b) {
    if (a > 0 ? (b > 0 ? a > INT64_MAX/b : b < INT64_MIN/a) :
        (a < 0 && (b > 0 ? a < INT64_MIN/b : b < 0 && a < INT64_MAX/b))) arithmetic_error();
    return a*b;
}
#endif
#endif
// 除以零与 Python 一样报 division by zero。
static void zam_zero_division(void) { zam_panic_one("division by zero"); exit(1); }
static int64_t quotient(int64_t a, int64_t b) {
    if (!b) zam_zero_division();
    if (a == INT64_MIN && b == -1) arithmetic_error();
    /* 两边都落在 32 位无符号范围时用 32 位除法，比 64 位除法快得多；所有编译器都适用。 */
    if (((uint64_t)a | (uint64_t)b) <= UINT32_MAX) return (int64_t)((uint32_t)a / (uint32_t)b);
    return a/b;
}
static int64_t zam_remainder(int64_t a, int64_t b) {
    if (!b) zam_zero_division();
    if (a == INT64_MIN && b == -1) arithmetic_error();
    if (((uint64_t)a | (uint64_t)b) <= UINT32_MAX) return (int64_t)((uint32_t)a % (uint32_t)b);
    return a%b;
}
// Python 的 `//` 与 `%`：商向下取整，余数的符号跟除数。
static int64_t zam_floordiv(int64_t a, int64_t b) {
    int64_t q = quotient(a, b);
    // 用乘法回推余数，不再做第二次除法；|q * b| <= |a|，不会溢出。
    if (a - q * b != 0 && ((a < 0) != (b < 0))) q -= 1;
    return q;
}
static int64_t zam_mod(int64_t a, int64_t b) {
    int64_t r;
    if (b == -1) return 0;
    // zam_remainder 负责除以零与 32 位快路径；这里只按除数的符号修正。
    r = zam_remainder(a, b);
    if (r != 0 && ((r < 0) != (b < 0))) r += b;
    return r;
}
// 除数是正的常量：不会除以零也不会溢出，只按符号修正。
static int64_t zam_floordiv_k(int64_t a, int64_t k) { int64_t q = a / k; return (a - q * k < 0) ? q - 1 : q; }
static int64_t zam_mod_k(int64_t a, int64_t k) { int64_t r = a % k; return r < 0 ? r + k : r; }
static double zam_fdiv(double a, double b) { if (b == 0.0) zam_zero_division(); return a / b; }
static double zam_ffloordiv(double a, double b) { if (b == 0.0) zam_zero_division(); return floor(a / b); }
static double zam_fmod_ieee(double a, double b) {
    double r = fmod(a, b);
    if (r != 0.0 && ((r < 0.0) != (b < 0.0))) r += b;
    return r;
}
static double zam_fmod(double a, double b) {
    if (b == 0.0) zam_zero_division();
    double r = fmod(a, b);
    if (r != 0.0 && ((r < 0.0) != (b < 0.0))) r += b;
    return r;
}
// Python 的切片边界：负数从末尾数，再收拢到 0..len，结束不早于开始。
static void zam_clamp(int64_t *from, int64_t *to, size_t length) {
    int64_t n = (int64_t)length;
    if (*from < 0) *from += n;
    if (*to < 0) *to += n;
    if (*from < 0) *from = 0;
    if (*from > n) *from = n;
    if (*to > n) *to = n;
    if (*to < *from) *to = *from;
}
static int64_t checked_index(int64_t index, size_t length) {
    if (index < 0) index += (int64_t)length;
    if (index < 0 || (uintmax_t)index >= length) { zam_panic_one("array index out of bounds"); exit(1); }
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
        if (!wide) { zam_panic_one("allocation failed"); exit(1); }
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
// chars 缓存字符数，0 表示还没数过；chars == len 说明全是 ASCII，字符位置就是字节位置。
// 规则：任何改了 data 或 len 的地方都要把 chars 清零。
typedef struct { unsigned char *data; size_t len; size_t chars; } S;
static S own(const unsigned char *p, size_t n) {
    S s; s.data = (unsigned char*)malloc(n ? n : 1); s.len = n; s.chars = 0;
    if (!s.data) { zam_panic_one("allocation failed"); exit(1); }
    if (n) memcpy(s.data, p, n); return s;
}
static void drop(S *s) { free(s->data); s->data = NULL; s->len = 0; s->chars = 0; }
static S take(S *s) { S value = *s; s->data = NULL; s->len = 0; s->chars = 0; return value; }
static int64_t byte_len(S *s) { if ((uintmax_t)s->len > INT64_MAX) arithmetic_error(); return (int64_t)s->len; }
// ponytail: 按 UTF-8 续字节计数，不校验编码合法性；非法字节按其起始字节计一个字符。
static size_t zam_chars(S *s) {
    if (!s->chars && s->len) { size_t n = 0, i; for (i = 0; i < s->len; i++) if ((s->data[i] & 0xC0) != 0x80) n++; s->chars = n; }
    return s->chars;
}
static int64_t char_len(S *s) { return (int64_t)zam_chars(s); }
// 第 k 个字符从哪个字节开始（k 等于字符数时是末尾）。
static size_t zam_byte_of(S *s, size_t k) {
    size_t i, n = 0;
    if (zam_chars(s) == s->len) return k < s->len ? k : s->len;
    for (i = 0; i < s->len; i++) if ((s->data[i] & 0xC0) != 0x80) { if (n == k) return i; n++; }
    return s->len;
}
// 字节位置 b 之前有几个字符。
static size_t zam_char_of(S *s, size_t b) {
    size_t i, n = 0;
    if (zam_chars(s) == s->len) return b;
    for (i = 0; i < b && i < s->len; i++) if ((s->data[i] & 0xC0) != 0x80) n++;
    return n;
}
static int string_equal(S *a, S *b) { return a->len == b->len && (!a->len || memcmp(a->data,b->data,a->len) == 0); }
static int string_compare(S *a, S *b) {
    size_t n = a->len < b->len ? a->len : b->len;
    int c = n ? memcmp(a->data, b->data, n) : 0;
    if (c) return c < 0 ? -1 : 1;
    return a->len < b->len ? -1 : a->len > b->len;
}
#ifdef _WIN32
#define zam_lock_stdout() _lock_file(stdout)
#define zam_unlock_stdout() _unlock_file(stdout)
#else
#define zam_lock_stdout() flockfile(stdout)
#define zam_unlock_stdout() funlockfile(stdout)
#endif
static void print(S *s) {
    int locked = zam_threaded, ok;
    if (locked) zam_lock_stdout();
    ok = zam_write(stdout, s->data, s->len) && fputc('\n', stdout) != EOF;
    if (locked) zam_unlock_stdout();
    if (!ok) exit(1);
}
// ponytail: 未捕获的错误只写 stderr 并以退出码 1 结束；不做栈回溯，错误值本身就是全部信息。
static void uncaught_error(S *s) {
    fflush(stdout);
    if (!zam_write(stderr, s->data, s->len)) exit(1);
    if (fputc('\n', stderr) == EOF) exit(1);
}
static S copy(S *s) { return own(s->data, s->len); }
// extern fn 的 &string 参数：复制成以 \0 结尾的 C 字符串，调用后释放。
static char *zam_cstr(S *s) {
    char *p = (char*)malloc(s->len + 1);
    if (!p) { zam_panic_one("allocation failed"); exit(1); }
    if (s->len) memcpy(p, s->data, s->len);
    p[s->len] = 0; return p;
}
static S cat(S a, S b) {
    size_t n = a.len + b.len;
    unsigned char *p = (unsigned char*)malloc(n ? n : 1);
    if (!p) { zam_panic_one("allocation failed"); exit(1); }
    if (a.len) memcpy(p, a.data, a.len);
    if (b.len) memcpy(p + a.len, b.data, b.len);
    drop(&a); drop(&b);
    S s; s.data = p; s.len = n; s.chars = 0; return s;
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
/* f"{x:spec}"：把文本按说明补白/对齐；precision 非负时按它重写数字的小数位数。 */
static S zam_format(S text, char align, char fill, size_t width, int precision) {
    if (precision >= 0 && text.len > 0 && text.len < 63) {
        /* 文本不带结尾的 \0：先抄进栈上的缓冲区再解析。不是数字时原样保留。 */
        char buffer[64];
        memcpy(buffer, text.data, text.len);
        buffer[text.len] = '\0';
        char *end = NULL;
        double value = strtod(buffer, &end);
        if (end != NULL && *end == '\0') {
            /* 很大的数（1e300）整数部分有三百多位：先量长度，再按需要分配。 */
            int n = snprintf(NULL, 0, "%.*f", precision, value);
            if (n > 0) {
                char *digits = (char*)malloc((size_t)n + 1);
                if (digits != NULL) {
                    snprintf(digits, (size_t)n + 1, "%.*f", precision, value);
                    drop(&text);
                    text = own((const unsigned char*)digits, (size_t)n);
                    free(digits);
                }
            }
        }
    }
    /* 宽度按字符数（UTF-8 码点）算，与 Python 和 string.pad_left 一致。 */
    size_t chars = 0;
    for (size_t i = 0; i < text.len; i++) if ((text.data[i] & 0xC0) != 0x80) chars++;
    if (width <= chars) return text;
    size_t pad = width - chars;
    size_t left = 0, right = 0;
    if (align == '<') {
        right = pad;
    } else if (align == '^') {
        left = pad / 2;
        right = pad - left;
    } else {
        left = pad;
    }
    S out = own((const unsigned char*)"", 0);
    /* 补零靠右时，零放在负号后面：-0042，不是 00-42。 */
    size_t sign = (fill == '0' && align == '>' && text.len > 0 && (text.data[0] == '-' || text.data[0] == '+')) ? 1 : 0;
    if (sign) out = cat(out, own(text.data, 1));
    for (size_t i = 0; i < left; i++) out = cat(out, own((const unsigned char*)&fill, 1));
    out = cat(out, own(text.data + sign, text.len - sign));
    for (size_t i = 0; i < right; i++) out = cat(out, own((const unsigned char*)&fill, 1));
    drop(&text);
    return out;
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
const HASH_PRELUDE: &str = r#"// std/hashlib：SHA-256、SHA-1、MD5 与 HMAC-SHA256，按 FIPS 180-4、RFC 1321、RFC 2104 实现。
// 消息补位后逐块处理，`zam_hash_byte` 给出补位后的第 at 个字节（big 为真时长度按大端写）。
static uint8_t zam_hash_byte(const uint8_t *p, size_t n, size_t at, size_t total, int big) {
    uint64_t bits = (uint64_t)n * 8;
    if (at < n) return p[at];
    if (at == n) return 0x80;
    if (at >= total - 8) {
        size_t j = at - (total - 8);
        return (uint8_t)(big ? bits >> (8 * (7 - j)) : bits >> (8 * j));
    }
    return 0;
}
static size_t zam_hash_total(size_t n) { return ((n + 9 + 63) / 64) * 64; }
#define ZAM_ROTR(x, n) (((x) >> (n)) | ((x) << (32 - (n))))
#define ZAM_ROTL(x, n) (((x) << (n)) | ((x) >> (32 - (n))))
static void zam_sha256(const uint8_t *p, size_t n, uint8_t out[32]) {
    static const uint32_t k[64] = {
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2};
    uint32_t h[8] = {0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19};
    size_t total = zam_hash_total(n), off, i;
    for (off = 0; off < total; off += 64) {
        uint32_t w[64], a, b, c, d, e, f, g, hh;
        for (i = 0; i < 16; i++) {
            w[i] = (uint32_t)zam_hash_byte(p, n, off + 4 * i, total, 1) << 24 | (uint32_t)zam_hash_byte(p, n, off + 4 * i + 1, total, 1) << 16
                 | (uint32_t)zam_hash_byte(p, n, off + 4 * i + 2, total, 1) << 8 | zam_hash_byte(p, n, off + 4 * i + 3, total, 1);
        }
        for (i = 16; i < 64; i++) {
            uint32_t s0 = ZAM_ROTR(w[i - 15], 7) ^ ZAM_ROTR(w[i - 15], 18) ^ (w[i - 15] >> 3);
            uint32_t s1 = ZAM_ROTR(w[i - 2], 17) ^ ZAM_ROTR(w[i - 2], 19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16] + s0 + w[i - 7] + s1;
        }
        a = h[0]; b = h[1]; c = h[2]; d = h[3]; e = h[4]; f = h[5]; g = h[6]; hh = h[7];
        for (i = 0; i < 64; i++) {
            uint32_t t1 = hh + (ZAM_ROTR(e, 6) ^ ZAM_ROTR(e, 11) ^ ZAM_ROTR(e, 25)) + ((e & f) ^ (~e & g)) + k[i] + w[i];
            uint32_t t2 = (ZAM_ROTR(a, 2) ^ ZAM_ROTR(a, 13) ^ ZAM_ROTR(a, 22)) + ((a & b) ^ (a & c) ^ (b & c));
            hh = g; g = f; f = e; e = d + t1; d = c; c = b; b = a; a = t1 + t2;
        }
        h[0] += a; h[1] += b; h[2] += c; h[3] += d; h[4] += e; h[5] += f; h[6] += g; h[7] += hh;
    }
    for (i = 0; i < 32; i++) out[i] = (uint8_t)(h[i / 4] >> (24 - 8 * (i % 4)));
}
static void zam_sha1(const uint8_t *p, size_t n, uint8_t out[20]) {
    uint32_t h[5] = {0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0};
    size_t total = zam_hash_total(n), off, i;
    for (off = 0; off < total; off += 64) {
        uint32_t w[80], a, b, c, d, e;
        for (i = 0; i < 16; i++) {
            w[i] = (uint32_t)zam_hash_byte(p, n, off + 4 * i, total, 1) << 24 | (uint32_t)zam_hash_byte(p, n, off + 4 * i + 1, total, 1) << 16
                 | (uint32_t)zam_hash_byte(p, n, off + 4 * i + 2, total, 1) << 8 | zam_hash_byte(p, n, off + 4 * i + 3, total, 1);
        }
        for (i = 16; i < 80; i++) w[i] = ZAM_ROTL(w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16], 1);
        a = h[0]; b = h[1]; c = h[2]; d = h[3]; e = h[4];
        for (i = 0; i < 80; i++) {
            uint32_t f, k, t;
            if (i < 20) { f = (b & c) | (~b & d); k = 0x5a827999; }
            else if (i < 40) { f = b ^ c ^ d; k = 0x6ed9eba1; }
            else if (i < 60) { f = (b & c) | (b & d) | (c & d); k = 0x8f1bbcdc; }
            else { f = b ^ c ^ d; k = 0xca62c1d6; }
            t = ZAM_ROTL(a, 5) + f + e + k + w[i];
            e = d; d = c; c = ZAM_ROTL(b, 30); b = a; a = t;
        }
        h[0] += a; h[1] += b; h[2] += c; h[3] += d; h[4] += e;
    }
    for (i = 0; i < 20; i++) out[i] = (uint8_t)(h[i / 4] >> (24 - 8 * (i % 4)));
}
static void zam_md5(const uint8_t *p, size_t n, uint8_t out[16]) {
    static const uint32_t k[64] = {
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
        0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
        0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
        0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
        0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
        0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
        0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391};
    static const uint32_t s[64] = {
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
        5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
        4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
        6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21};
    uint32_t h[4] = {0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476};
    size_t total = zam_hash_total(n), off, i;
    for (off = 0; off < total; off += 64) {
        uint32_t m[16], a = h[0], b = h[1], c = h[2], d = h[3];
        for (i = 0; i < 16; i++) {
            m[i] = (uint32_t)zam_hash_byte(p, n, off + 4 * i, total, 0) | (uint32_t)zam_hash_byte(p, n, off + 4 * i + 1, total, 0) << 8
                 | (uint32_t)zam_hash_byte(p, n, off + 4 * i + 2, total, 0) << 16 | (uint32_t)zam_hash_byte(p, n, off + 4 * i + 3, total, 0) << 24;
        }
        for (i = 0; i < 64; i++) {
            uint32_t f, g, t;
            if (i < 16) { f = (b & c) | (~b & d); g = (uint32_t)i; }
            else if (i < 32) { f = (d & b) | (~d & c); g = (uint32_t)(5 * i + 1) % 16; }
            else if (i < 48) { f = b ^ c ^ d; g = (uint32_t)(3 * i + 5) % 16; }
            else { f = c ^ (b | ~d); g = (uint32_t)(7 * i) % 16; }
            t = d; d = c; c = b;
            f = f + a + k[i] + m[g];
            b = b + ZAM_ROTL(f, s[i]);
            a = t;
        }
        h[0] += a; h[1] += b; h[2] += c; h[3] += d;
    }
    for (i = 0; i < 16; i++) out[i] = (uint8_t)(h[i / 4] >> (8 * (i % 4)));
}
static void zam_hmac_sha256(const uint8_t *key, size_t kn, const uint8_t *msg, size_t mn, uint8_t out[32]) {
    uint8_t k[64], inner[32], *buf;
    size_t i;
    memset(k, 0, sizeof k);
    if (kn > 64) zam_sha256(key, kn, k); else if (kn) memcpy(k, key, kn);
    if (mn > SIZE_MAX - 64) zam_alloc_failed();
    buf = (uint8_t *)malloc(64 + (mn > 32 ? mn : 32));
    if (!buf) zam_alloc_failed();
    for (i = 0; i < 64; i++) buf[i] = k[i] ^ 0x36;
    if (mn) memcpy(buf + 64, msg, mn);
    zam_sha256(buf, 64 + mn, inner);
    for (i = 0; i < 64; i++) buf[i] = k[i] ^ 0x5c;
    memcpy(buf + 64, inner, 32);
    zam_sha256(buf, 96, out);
    free(buf);
}
static Ly zam_digest_bytes(const uint8_t *digest, size_t n) { Ly out = {0}; reserveLy(&out, n); memcpy(out.data, digest, n); out.len = n; return out; }
"#;
const BASE64_PRELUDE: &str = r#"// std/base64：RFC 4648。解码时跳过空白；其他字符或补位不对返回错误（同 Python 的 binascii.Error）。
static S zam_b64encode(Ly *p, int url) {
    const char *alphabet = url ? "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_" : "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    size_t n = p->len, i, o = 0, size;
    unsigned char *q;
    S s;
    if (n > (SIZE_MAX - 3) / 4 * 3) zam_alloc_failed();
    size = (n + 2) / 3 * 4;
    q = (unsigned char *)malloc(size ? size : 1);
    if (!q) zam_alloc_failed();
    for (i = 0; i + 2 < n; i += 3) {
        uint32_t v = (uint32_t)p->data[i] << 16 | (uint32_t)p->data[i + 1] << 8 | p->data[i + 2];
        q[o++] = (unsigned char)alphabet[v >> 18]; q[o++] = (unsigned char)alphabet[(v >> 12) & 63];
        q[o++] = (unsigned char)alphabet[(v >> 6) & 63]; q[o++] = (unsigned char)alphabet[v & 63];
    }
    if (i < n) {
        uint32_t v = (uint32_t)p->data[i] << 16 | (i + 1 < n ? (uint32_t)p->data[i + 1] << 8 : 0);
        q[o++] = (unsigned char)alphabet[v >> 18]; q[o++] = (unsigned char)alphabet[(v >> 12) & 63];
        q[o++] = i + 1 < n ? (unsigned char)alphabet[(v >> 6) & 63] : '=';
        q[o++] = '=';
    }
    s.data = q; s.len = o; s.chars = o;
    return s;
}
static F_Ly zam_b64decode(S *text, int url) {
    Ly out = {0};
    uint32_t v = 0;
    size_t i, count = 0, pad = 0;
    for (i = 0; i < text->len; i++) {
        unsigned char c = text->data[i];
        int d;
        if (c == ' ' || c == '\n' || c == '\r' || c == '\t') continue;
        if (c == '=') { pad++; continue; }
        if (pad) { dropLy(&out); return F_Ly_err(zam_message("invalid base64: data after padding", NULL)); }
        d = c >= 'A' && c <= 'Z' ? c - 'A' : c >= 'a' && c <= 'z' ? c - 'a' + 26 : c >= '0' && c <= '9' ? c - '0' + 52
          : c == (url ? '-' : '+') ? 62 : c == (url ? '_' : '/') ? 63 : -1;
        if (d < 0) { S at = from_int((int64_t)i); S message = zam_message("invalid base64 character at position ", &at); drop(&at); dropLy(&out); return F_Ly_err(message); }
        v = v << 6 | (uint32_t)d;
        if (++count % 4 == 0) { pushLy(&out, (v >> 16) & 255); pushLy(&out, (v >> 8) & 255); pushLy(&out, v & 255); v = 0; }
    }
    if (count % 4 == 1 || (count + pad) % 4 != 0 || pad > 2) { dropLy(&out); return F_Ly_err(zam_message("invalid base64: incorrect padding", NULL)); }
    if (count % 4 == 2) pushLy(&out, (v >> 4) & 255);
    if (count % 4 == 3) { pushLy(&out, (v >> 10) & 255); pushLy(&out, (v >> 2) & 255); }
    return F_Ly_ok(out);
}
"#;
const PROCESS_PRELUDE: &str = r#"// std/_process：启动程序、等它结束、收集输出。std/process（Zamak 写的）在它上面包出 run、call、
// check_output。输出先写进临时文件再读回，不用管道，不会因为两个管道都写满而互相等待。
// 返回 [退出码, stdout, stderr]；被信号结束时退出码是负的信号编号（同 Python）。
#ifdef _WIN32
typedef struct { unsigned long cb; wchar_t *reserved, *desktop, *title; unsigned long x, y, xsize, ysize, xchars, ychars, fill, flags; unsigned short show, reserved2_size; unsigned char *reserved2; void *in, *out, *err; } zam_startup;
typedef struct { void *process, *thread; unsigned long pid, tid; } zam_procinfo;
typedef struct { unsigned long length; void *descriptor; int inherit; } zam_security;
#ifndef _WINDOWS_
__declspec(dllimport) int __stdcall CreateProcessW(const wchar_t *, wchar_t *, void *, void *, int, unsigned long, void *, const wchar_t *, void *, void *);
__declspec(dllimport) unsigned long __stdcall WaitForSingleObject(void *, unsigned long);
__declspec(dllimport) int __stdcall GetExitCodeProcess(void *, unsigned long *);
__declspec(dllimport) int __stdcall CloseHandle(void *);
__declspec(dllimport) void *__stdcall GetStdHandle(unsigned long);
__declspec(dllimport) unsigned long __stdcall GetTempPathW(unsigned long, wchar_t *);
__declspec(dllimport) unsigned int __stdcall GetTempFileNameW(const wchar_t *, const wchar_t *, unsigned int, wchar_t *);
__declspec(dllimport) void *__stdcall CreateFileW(const wchar_t *, unsigned long, unsigned long, void *, unsigned long, unsigned long, void *);
__declspec(dllimport) int __stdcall ReadFile(void *, void *, unsigned long, unsigned long *, void *);
__declspec(dllimport) int __stdcall WriteFile(void *, const void *, unsigned long, unsigned long *, void *);
__declspec(dllimport) unsigned long __stdcall SetFilePointer(void *, long, long *, unsigned long);
__declspec(dllimport) unsigned long __stdcall GetLastError(void);
#endif
// 可被子进程继承、关闭时自动删除的临时文件。
static void *zam_temp_handle(void) {
    wchar_t dir[300], name[300];
    zam_security sa;
    void *h;
    unsigned long n = GetTempPathW(260, dir);
    sa.length = sizeof sa; sa.descriptor = NULL; sa.inherit = 1;
    if (!n || n > 260 || !GetTempFileNameW(dir, L"zam", 0, name)) return NULL;
    h = CreateFileW(name, 0xC0000000UL, 7, (void *)&sa, 2, 0x100 | 0x04000000, NULL);
    return h == (void *)(intptr_t)-1 ? NULL : h;
}
static S zam_handle_text(void *h) {
    unsigned char *buf = NULL;
    size_t len = 0, cap = 0;
    unsigned long got;
    S text;
    SetFilePointer(h, 0, NULL, 0);
    for (;;) {
        if (len == cap) {
            unsigned char *grown;
            cap = cap ? cap * 2 : 4096;
            grown = (unsigned char *)realloc(buf, cap);
            if (!grown) zam_alloc_failed();
            buf = grown;
        }
        if (!ReadFile(h, buf + len, (unsigned long)(cap - len), &got, NULL) || !got) break;
        len += got;
    }
    CloseHandle(h);
    text = own(buf ? buf : (const unsigned char *)"", len);
    free(buf);
    return text;
}
static void zam_line_push(S *line, const char *p, size_t n) {
    unsigned char *grown = (unsigned char *)realloc(line->data, line->len + n + 1);
    if (!grown) zam_alloc_failed();
    line->data = grown;
    memcpy(line->data + line->len, p, n);
    line->len += n;
}
// 按 CommandLineToArgvW 的规则给参数加引号：程序看到的参数和列表里的一模一样。
static void zam_line_arg(S *line, S *arg) {
    size_t i, j, slashes;
    int quote = !arg->len;
    for (i = 0; i < arg->len; i++) if (arg->data[i] == ' ' || arg->data[i] == '\t' || arg->data[i] == '\n' || arg->data[i] == '\v' || arg->data[i] == '"') quote = 1;
    if (!quote) { zam_line_push(line, (const char *)arg->data, arg->len); return; }
    zam_line_push(line, "\"", 1);
    for (i = 0; i < arg->len; i++) {
        slashes = 0;
        while (i < arg->len && arg->data[i] == '\\') { slashes++; i++; }
        if (i == arg->len) { for (j = 0; j < slashes * 2; j++) zam_line_push(line, "\\", 1); break; }
        if (arg->data[i] == '"') { for (j = 0; j < slashes * 2 + 1; j++) zam_line_push(line, "\\", 1); }
        else for (j = 0; j < slashes; j++) zam_line_push(line, "\\", 1);
        zam_line_push(line, (const char *)&arg->data[i], 1);
    }
    zam_line_push(line, "\"", 1);
}
static F_Ls zam_process_run(Ls *args, S *input, int capture) {
    S line = {0};
    wchar_t *wide;
    void *files[3] = {NULL, NULL, NULL};
    zam_startup si;
    zam_procinfo pi;
    unsigned long code = 0;
    size_t i;
    int started;
    Ls out = {0};
    if (!args->len) return F_Ls_err(zam_message("process: the argument list is empty; put the program first", NULL));
    for (i = 0; i < args->len; i++) {
        if (i) zam_line_push(&line, " ", 1);
        zam_line_arg(&line, &args->data[i]);
    }
    wide = zam_path(&line);
    drop(&line);
    if (!wide) return F_Ls_err(zam_message("process: an argument contains a NUL character", NULL));
    memset(&si, 0, sizeof si);
    si.cb = sizeof si;
    if (input->len || capture) {
        si.flags = 0x100;
        si.in = GetStdHandle((unsigned long)-10);
        si.out = GetStdHandle((unsigned long)-11);
        si.err = GetStdHandle((unsigned long)-12);
    }
    if (input->len) {
        unsigned long done;
        files[0] = zam_temp_handle();
        if (files[0] && WriteFile(files[0], input->data, (unsigned long)input->len, &done, NULL)) { SetFilePointer(files[0], 0, NULL, 0); si.in = files[0]; }
    }
    if (capture) {
        files[1] = zam_temp_handle(); files[2] = zam_temp_handle();
        if (files[1]) si.out = files[1];
        if (files[2]) si.err = files[2];
    }
    fflush(stdout); fflush(stderr);
    started = CreateProcessW(NULL, wide, NULL, NULL, 1, 0, NULL, NULL, (void *)&si, (void *)&pi);
    free(wide);
    if (!started) {
        unsigned long error = GetLastError();
        S program = copy(&args->data[0]), number = from_int((int64_t)error), message;
        for (i = 0; i < 3; i++) if (files[i]) CloseHandle(files[i]);
        message = cat(zam_message("cannot run ", &program), own((const unsigned char *)(error == 2 ? ": program not found" : ": Windows error "), error == 2 ? 19 : 16));
        if (error != 2) message = cat(message, copy(&number));
        drop(&program); drop(&number);
        return F_Ls_err(message);
    }
    WaitForSingleObject(pi.process, 0xFFFFFFFFUL);
    GetExitCodeProcess(pi.process, &code);
    CloseHandle(pi.process); CloseHandle(pi.thread);
    if (files[0]) CloseHandle(files[0]);
    pushLs(&out, from_int((int64_t)(int)code));
    pushLs(&out, files[1] ? zam_handle_text(files[1]) : own((const unsigned char *)"", 0));
    pushLs(&out, files[2] ? zam_handle_text(files[2]) : own((const unsigned char *)"", 0));
    return F_Ls_ok(out);
}
#else
#include <spawn.h>
#include <sys/wait.h>
#include <unistd.h>
#include <errno.h>
extern char **environ;
// 打开后立刻删除的临时文件：只剩文件描述符，进程结束时自动回收。
static int zam_temp_fd(void) {
    char name[] = "/tmp/zamak-XXXXXX";
    int fd = mkstemp(name);
    if (fd >= 0) unlink(name);
    return fd;
}
static S zam_fd_text(int fd) {
    S text;
    FILE *f;
    if (lseek(fd, 0, SEEK_SET) < 0 || !(f = fdopen(fd, "rb"))) { close(fd); return own((const unsigned char *)"", 0); }
    if (!zam_read_all(f, &text)) text = own((const unsigned char *)"", 0);
    fclose(f);
    return text;
}
static F_Ls zam_process_run(Ls *args, S *input, int capture) {
    char **argv;
    size_t i;
    pid_t pid;
    int status = 0, error, fds[3] = {-1, -1, -1};
    int64_t code;
    posix_spawn_file_actions_t actions;
    Ls out = {0};
    if (!args->len) return F_Ls_err(zam_message("process: the argument list is empty; put the program first", NULL));
    argv = (char **)malloc((args->len + 1) * sizeof *argv);
    if (!argv) zam_alloc_failed();
    for (i = 0; i < args->len; i++) argv[i] = zam_cstr(&args->data[i]);
    argv[args->len] = NULL;
    posix_spawn_file_actions_init(&actions);
    if (input->len && (fds[0] = zam_temp_fd()) >= 0) {
        size_t done = 0;
        while (done < input->len) {
            ssize_t n = write(fds[0], input->data + done, input->len - done);
            if (n <= 0) break;
            done += (size_t)n;
        }
        lseek(fds[0], 0, SEEK_SET);
        posix_spawn_file_actions_adddup2(&actions, fds[0], 0);
    }
    if (capture) {
        if ((fds[1] = zam_temp_fd()) >= 0) posix_spawn_file_actions_adddup2(&actions, fds[1], 1);
        if ((fds[2] = zam_temp_fd()) >= 0) posix_spawn_file_actions_adddup2(&actions, fds[2], 2);
    }
    fflush(stdout); fflush(stderr);
    error = posix_spawnp(&pid, argv[0], &actions, NULL, argv, environ);
    posix_spawn_file_actions_destroy(&actions);
    if (!error) while (waitpid(pid, &status, 0) < 0 && errno == EINTR) {}
    for (i = 0; i < args->len; i++) free(argv[i]);
    free(argv);
    if (fds[0] >= 0) close(fds[0]);
    if (error) {
        S program = copy(&args->data[0]);
        const char *reason = error == ENOENT ? ": program not found" : strerror(error);
        S message = cat(zam_message("cannot run ", &program), own((const unsigned char *)reason, strlen(reason)));
        drop(&program);
        if (fds[1] >= 0) close(fds[1]);
        if (fds[2] >= 0) close(fds[2]);
        return F_Ls_err(message);
    }
    code = WIFEXITED(status) ? WEXITSTATUS(status) : WIFSIGNALED(status) ? -(int64_t)WTERMSIG(status) : -1;
    pushLs(&out, from_int(code));
    pushLs(&out, fds[1] >= 0 ? zam_fd_text(fds[1]) : own((const unsigned char *)"", 0));
    pushLs(&out, fds[2] >= 0 ? zam_fd_text(fds[2]) : own((const unsigned char *)"", 0));
    return F_Ls_ok(out);
}
#endif
"#;
const REGEX_PRELUDE: &str = r#"// std/_re：正则表达式引擎（Pike VM，同 RE2/Go 的做法）。匹配时间与文本长度成线性关系，
// 不会因为回溯卡死；代价是没有反向引用和环视。语法照 Python 的 re，按码点匹配。
// 解析成语法树（节点存在数组里、用下标互相引用），再编译成指令；执行时多条线程同步前进，
// 优先级高的线程先匹配，得到与回溯引擎相同的“最左、按优先级”结果。
enum { ZRX_CHAR, ZRX_ANY, ZRX_ANYNL, ZRX_CLASS, ZRX_SPLIT, ZRX_JMP, ZRX_SAVE, ZRX_MATCH, ZRX_BOL, ZRX_EOL, ZRX_BEGIN, ZRX_END, ZRX_WORDB, ZRX_NWORDB };
enum { ZRN_CHAR, ZRN_ANY, ZRN_CLASS, ZRN_CAT, ZRN_ALT, ZRN_REPEAT, ZRN_GROUP, ZRN_EMPTY, ZRN_ASSERT };
// 类里的预定义集合用负数表示：\d \w \s 与它们的补集。
#define ZRX_DIGIT (-1)
#define ZRX_WORD (-2)
#define ZRX_SPACE (-3)
#define ZRX_NDIGIT (-4)
#define ZRX_NWORD (-5)
#define ZRX_NSPACE (-6)
#define ZRX_LIMIT 100000
typedef struct { int op; int64_t c; int x, y; } zam_rx_inst;
typedef struct { int64_t lo, hi; } zam_rx_range;
typedef struct { int start, count, negate; } zam_rx_class;
typedef struct { int kind; int64_t c; int a, b, min, max, greedy; } zam_rx_node;
typedef struct {
    zam_rx_inst *code; int n, cap;
    zam_rx_range *ranges; int nr, rcap;
    zam_rx_class *classes; int nc, ccap;
    zam_rx_node *nodes; int nn, ncap;
    S *names; int groups;
    int icase, multiline, dotall;
    const unsigned char *p; size_t len, at;
    const char *error;
} zam_rx;
static void *zam_rx_grow(void *p, int *cap, int need, size_t size) {
    if (need <= *cap) return p;
    *cap = *cap ? *cap * 2 : 16;
    if (*cap < need) *cap = need;
    p = realloc(p, (size_t)*cap * size);
    if (!p) zam_alloc_failed();
    return p;
}
static int zam_rx_node_new(zam_rx *r, int kind) {
    zam_rx_node *n;
    r->nodes = (zam_rx_node *)zam_rx_grow(r->nodes, &r->ncap, r->nn + 1, sizeof *r->nodes);
    n = &r->nodes[r->nn];
    memset(n, 0, sizeof *n);
    n->kind = kind; n->a = n->b = -1;
    return r->nn++;
}
static void zam_rx_range_add(zam_rx *r, int64_t lo, int64_t hi) {
    r->ranges = (zam_rx_range *)zam_rx_grow(r->ranges, &r->rcap, r->nr + 1, sizeof *r->ranges);
    r->ranges[r->nr].lo = lo; r->ranges[r->nr].hi = hi; r->nr++;
}
static int zam_rx_class_new(zam_rx *r, int negate) {
    r->classes = (zam_rx_class *)zam_rx_grow(r->classes, &r->ccap, r->nc + 1, sizeof *r->classes);
    r->classes[r->nc].start = r->nr; r->classes[r->nc].count = 0; r->classes[r->nc].negate = negate;
    return r->nc++;
}
static int zam_rx_space(int64_t c) { return c == ' ' || (c >= 9 && c <= 13) || c == 0x1c || c == 0x1d || c == 0x1e || c == 0x1f || c == 0x85 || c == 0xa0 || c == 0x1680 || (c >= 0x2000 && c <= 0x200a) || c == 0x2028 || c == 0x2029 || c == 0x202f || c == 0x205f || c == 0x3000; }
// \w：ASCII 字母数字与下划线；ASCII 以外除了空白与常见标点区都算（中文、带音标的字母等）。
static int zam_rx_word(int64_t c) {
    if (c < 128) return (c >= '0' && c <= '9') || (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
    if (zam_rx_space(c)) return 0;
    if ((c >= 0x2000 && c <= 0x206f) || (c >= 0x3000 && c <= 0x303f) || (c >= 0xff00 && c <= 0xff0f) || (c >= 0xff1a && c <= 0xff20) || (c >= 0xff3b && c <= 0xff40) || (c >= 0xff5b && c <= 0xff65) || c == 0xa1 || c == 0xa7 || c == 0xab || c == 0xb6 || c == 0xb7 || c == 0xbb || c == 0xbf || c == 0xd7 || c == 0xf7) return 0;
    return 1;
}
static int64_t zam_rx_fold(int64_t c) { return c >= 'A' && c <= 'Z' ? c + 32 : c; }
static int zam_rx_in_class(zam_rx *r, int k, int64_t c) {
    zam_rx_class *cls = &r->classes[k];
    int i, hit = 0;
    for (i = 0; i < cls->count && !hit; i++) {
        zam_rx_range *g = &r->ranges[cls->start + i];
        switch (g->lo) {
        case ZRX_DIGIT: hit = c >= '0' && c <= '9'; break;
        case ZRX_WORD: hit = zam_rx_word(c); break;
        case ZRX_SPACE: hit = zam_rx_space(c); break;
        case ZRX_NDIGIT: hit = !(c >= '0' && c <= '9'); break;
        case ZRX_NWORD: hit = !zam_rx_word(c); break;
        case ZRX_NSPACE: hit = !zam_rx_space(c); break;
        default:
            hit = c >= g->lo && c <= g->hi;
            if (!hit && r->icase) {
                int64_t f = zam_rx_fold(c), u = f >= 'a' && f <= 'z' ? f - 32 : f;
                hit = (f >= g->lo && f <= g->hi) || (u >= g->lo && u <= g->hi);
            }
        }
    }
    return hit != cls->negate;
}
// ---- 解析 ----
static int64_t zam_rx_next(zam_rx *r) {
    int64_t c;
    size_t n;
    if (r->at >= r->len) return -1;
    n = zam_utf8_next(r->p + r->at, r->len - r->at, &c);
    if (!n) { c = r->p[r->at]; n = 1; }
    r->at += n;
    return c;
}
static int64_t zam_rx_peek(zam_rx *r) { size_t at = r->at; int64_t c = zam_rx_next(r); r->at = at; return c; }
static int zam_rx_hex(zam_rx *r, int width, int64_t *out) {
    int i;
    *out = 0;
    for (i = 0; i < width; i++) {
        int64_t c = zam_rx_next(r);
        int d = c >= '0' && c <= '9' ? (int)(c - '0') : c >= 'a' && c <= 'f' ? (int)(c - 'a' + 10) : c >= 'A' && c <= 'F' ? (int)(c - 'A' + 10) : -1;
        if (d < 0) return 0;
        *out = *out * 16 + d;
    }
    return 1;
}
// 转义：返回字符（>= 0），或预定义集合（负数）；`*assert` 置成断言的指令号。
static int64_t zam_rx_escape(zam_rx *r, int *assert, int in_class) {
    int64_t c = zam_rx_next(r), v;
    *assert = -1;
    switch (c) {
    case -1: r->error = "the pattern ends with a backslash"; return 0;
    case 'd': return ZRX_DIGIT;
    case 'w': return ZRX_WORD;
    case 's': return ZRX_SPACE;
    case 'D': return ZRX_NDIGIT;
    case 'W': return ZRX_NWORD;
    case 'S': return ZRX_NSPACE;
    case 'n': return '\n';
    case 't': return '\t';
    case 'r': return '\r';
    case 'f': return '\f';
    case 'v': return '\v';
    case 'a': return 7;
    case '0': return 0;
    case 'b': if (in_class) return 8; *assert = ZRX_WORDB; return 0;
    case 'B': if (!in_class) { *assert = ZRX_NWORDB; return 0; } break;
    case 'A': if (!in_class) { *assert = ZRX_BEGIN; return 0; } break;
    case 'Z': if (!in_class) { *assert = ZRX_END; return 0; } break;
    case 'x': if (zam_rx_hex(r, 2, &v)) return v; r->error = "\\x takes two hex digits"; return 0;
    case 'u': if (zam_rx_hex(r, 4, &v)) return v; r->error = "\\u takes four hex digits"; return 0;
    case 'U': if (zam_rx_hex(r, 8, &v)) return v; r->error = "\\U takes eight hex digits"; return 0;
    }
    if (c >= '1' && c <= '9') { r->error = "backreferences such as \\1 are not supported; Zamak matches in linear time"; return 0; }
    if ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')) { r->error = "unknown escape in the pattern"; return 0; }
    return c;
}
static void zam_rx_class_item(zam_rx *r, int64_t lo, int64_t hi) {
    zam_rx_range_add(r, lo, hi);
    r->classes[r->nc - 1].count++;
}
static int zam_rx_parse_class(zam_rx *r) {
    int negate = 0, k, first = 1, assert;
    if (zam_rx_peek(r) == '^') { zam_rx_next(r); negate = 1; }
    k = zam_rx_class_new(r, negate);
    for (;;) {
        int64_t c = zam_rx_next(r), lo, hi;
        if (c == -1) { r->error = "missing ] at the end of a character class"; return k; }
        if (c == ']' && !first) break;
        first = 0;
        lo = c;
        if (c == '\\') {
            lo = zam_rx_escape(r, &assert, 1);
            if (r->error) return k;
            if (lo < 0) { zam_rx_class_item(r, lo, lo); continue; }
        }
        hi = lo;
        if (zam_rx_peek(r) == '-') {
            size_t save = r->at;
            int64_t d;
            zam_rx_next(r);
            d = zam_rx_next(r);
            if (d == ']' || d == -1) { r->at = save; }
            else {
                if (d == '\\') { d = zam_rx_escape(r, &assert, 1); if (r->error) return k; if (d < 0) { r->error = "a range cannot end with \\d, \\w or \\s"; return k; } }
                if (d < lo) { r->error = "bad character range in a class"; return k; }
                hi = d;
            }
        }
        zam_rx_class_item(r, lo, hi);
    }
    return k;
}
static int zam_rx_alt(zam_rx *r, int depth);
static int zam_rx_atom(zam_rx *r, int depth) {
    int64_t c = zam_rx_next(r);
    int node, assert;
    switch (c) {
    case '(': {
        int group = -1, inner;
        S name = {0};
        if (zam_rx_peek(r) == '?') {
            zam_rx_next(r);
            c = zam_rx_next(r);
            if (c == ':') group = -1;
            else if (c == 'P' || c == '<') {
                size_t start;
                if (c == 'P' && zam_rx_next(r) != '<') { r->error = "named groups are written (?P<name>...)"; return -1; }
                start = r->at;
                while (zam_rx_peek(r) != '>' && zam_rx_peek(r) != -1) zam_rx_next(r);
                if (zam_rx_next(r) != '>' || r->at - 1 == start) { r->error = "a group name must end with >"; return -1; }
                name = own(r->p + start, r->at - 1 - start);
                group = ++r->groups;
            } else if (c == 'i' || c == 'm' || c == 's') {
                // 行内标志 (?i) (?m) (?s)，作用于整个表达式（同 Python 的要求：写在开头）。
                for (;;) {
                    if (c == 'i') r->icase = 1; else if (c == 'm') r->multiline = 1; else if (c == 's') r->dotall = 1;
                    else if (c == ')') return zam_rx_node_new(r, ZRN_EMPTY);
                    else { r->error = "unknown flag in (?...)"; return -1; }
                    c = zam_rx_next(r);
                }
            } else { r->error = "lookahead, lookbehind and other (?...) groups are not supported"; return -1; }
        } else group = ++r->groups;
        if (group > 0) {
            r->names = (S *)realloc(r->names, (size_t)(group + 1) * sizeof(S));
            if (!r->names) zam_alloc_failed();
            r->names[group] = name;
        }
        inner = zam_rx_alt(r, depth + 1);
        if (r->error) return -1;
        if (zam_rx_next(r) != ')') { r->error = "missing ) to close a group"; return -1; }
        if (group < 0) return inner;
        node = zam_rx_node_new(r, ZRN_GROUP);
        r->nodes[node].a = inner; r->nodes[node].c = group;
        return node;
    }
    case '[':
        node = zam_rx_node_new(r, ZRN_CLASS);
        r->nodes[node].c = zam_rx_parse_class(r);
        return node;
    case '.': return zam_rx_node_new(r, ZRN_ANY);
    case '^': node = zam_rx_node_new(r, ZRN_ASSERT); r->nodes[node].c = ZRX_BOL; return node;
    case '$': node = zam_rx_node_new(r, ZRN_ASSERT); r->nodes[node].c = ZRX_EOL; return node;
    case '\\':
        c = zam_rx_escape(r, &assert, 0);
        if (r->error) return -1;
        if (assert >= 0) { node = zam_rx_node_new(r, ZRN_ASSERT); r->nodes[node].c = assert; return node; }
        if (c < 0) {
            node = zam_rx_node_new(r, ZRN_CLASS);
            r->nodes[node].c = zam_rx_class_new(r, 0);
            zam_rx_class_item(r, c, c);
            return node;
        }
        break;
    case '*': case '+': case '?': r->error = "nothing to repeat before *, + or ?"; return -1;
    }
    node = zam_rx_node_new(r, ZRN_CHAR);
    r->nodes[node].c = c;
    return node;
}
// `{m}`、`{m,}`、`{m,n}`；写法不对时 `{` 是普通字符（同 Python）。
static int zam_rx_braces(zam_rx *r, int *min, int *max) {
    size_t save = r->at;
    int64_t c;
    int value = -1, low;
    zam_rx_next(r);
    while ((c = zam_rx_peek(r)) >= '0' && c <= '9') { zam_rx_next(r); value = (value < 0 ? 0 : value) * 10 + (int)(c - '0'); if (value > 1000) { r->error = "a repeat count is limited to 1000"; return 0; } }
    low = value;
    if (zam_rx_peek(r) == '}') { zam_rx_next(r); if (low < 0) { r->at = save; return 0; } *min = *max = low; return 1; }
    if (zam_rx_peek(r) != ',') { r->at = save; return 0; }
    zam_rx_next(r);
    value = -1;
    while ((c = zam_rx_peek(r)) >= '0' && c <= '9') { zam_rx_next(r); value = (value < 0 ? 0 : value) * 10 + (int)(c - '0'); if (value > 1000) { r->error = "a repeat count is limited to 1000"; return 0; } }
    if (zam_rx_peek(r) != '}') { r->at = save; return 0; }
    zam_rx_next(r);
    *min = low < 0 ? 0 : low;
    *max = value;
    if (value >= 0 && value < *min) { r->error = "a repeat range must not go down, as in {3,1}"; return 0; }
    return 1;
}
static int zam_rx_repeat(zam_rx *r, int depth) {
    int node = zam_rx_atom(r, depth), repeated = 0;
    for (;;) {
        int64_t c = zam_rx_peek(r);
        int min, max, rep;
        if (r->error) return -1;
        if (c == '*') { zam_rx_next(r); min = 0; max = -1; }
        else if (c == '+') { zam_rx_next(r); min = 1; max = -1; }
        else if (c == '?') { zam_rx_next(r); min = 0; max = 1; }
        else if (c == '{') { if (!zam_rx_braces(r, &min, &max)) { if (r->error) return -1; return node; } }
        else return node;
        if (repeated) { r->error = "multiple repeat, such as a**"; return -1; }
        if (r->nodes[node].kind == ZRN_ASSERT || r->nodes[node].kind == ZRN_EMPTY) { r->error = "nothing to repeat"; return -1; }
        repeated = 1;
        rep = zam_rx_node_new(r, ZRN_REPEAT);
        r->nodes[rep].a = node; r->nodes[rep].min = min; r->nodes[rep].max = max; r->nodes[rep].greedy = 1;
        if (zam_rx_peek(r) == '?') { zam_rx_next(r); r->nodes[rep].greedy = 0; }
        node = rep;
    }
}
static int zam_rx_cat(zam_rx *r, int depth) {
    int node = -1;
    for (;;) {
        int64_t c = zam_rx_peek(r);
        int next, cat;
        if (c == -1 || c == '|' || (c == ')' && depth > 0)) break;
        if (c == ')') { r->error = "unbalanced ) in the pattern"; return -1; }
        next = zam_rx_repeat(r, depth);
        if (r->error) return -1;
        if (node < 0) { node = next; continue; }
        cat = zam_rx_node_new(r, ZRN_CAT);
        r->nodes[cat].a = node; r->nodes[cat].b = next;
        node = cat;
    }
    return node < 0 ? zam_rx_node_new(r, ZRN_EMPTY) : node;
}
static int zam_rx_alt(zam_rx *r, int depth) {
    int node;
    if (depth > 200) { r->error = "groups are nested too deeply"; return -1; }
    node = zam_rx_cat(r, depth);
    while (!r->error && zam_rx_peek(r) == '|') {
        int alt, right;
        zam_rx_next(r);
        right = zam_rx_cat(r, depth);
        alt = zam_rx_node_new(r, ZRN_ALT);
        r->nodes[alt].a = node; r->nodes[alt].b = right;
        node = alt;
    }
    return node;
}
// ---- 编译 ----
static int zam_rx_emit(zam_rx *r, int op, int64_t c, int x, int y) {
    if (r->n >= ZRX_LIMIT) { r->error = "the regular expression is too large"; return r->n; }
    r->code = (zam_rx_inst *)zam_rx_grow(r->code, &r->cap, r->n + 1, sizeof *r->code);
    r->code[r->n].op = op; r->code[r->n].c = c; r->code[r->n].x = x; r->code[r->n].y = y;
    return r->n++;
}
static void zam_rx_compile(zam_rx *r, int k) {
    zam_rx_node node = r->nodes[k];
    int split, i;
    if (r->error) return;
    switch (node.kind) {
    case ZRN_CHAR: zam_rx_emit(r, ZRX_CHAR, node.c, 0, 0); break;
    case ZRN_ANY: zam_rx_emit(r, r->dotall ? ZRX_ANYNL : ZRX_ANY, 0, 0, 0); break;
    case ZRN_CLASS: zam_rx_emit(r, ZRX_CLASS, 0, (int)node.c, 0); break;
    case ZRN_ASSERT: zam_rx_emit(r, (int)node.c, 0, 0, 0); break;
    case ZRN_EMPTY: break;
    case ZRN_CAT: zam_rx_compile(r, node.a); zam_rx_compile(r, node.b); break;
    case ZRN_GROUP:
        zam_rx_emit(r, ZRX_SAVE, 0, 2 * (int)node.c, 0);
        zam_rx_compile(r, node.a);
        zam_rx_emit(r, ZRX_SAVE, 0, 2 * (int)node.c + 1, 0);
        break;
    case ZRN_ALT: {
        int jump;
        split = zam_rx_emit(r, ZRX_SPLIT, 0, 0, 0);
        r->code[split].x = r->n;
        zam_rx_compile(r, node.a);
        jump = zam_rx_emit(r, ZRX_JMP, 0, 0, 0);
        if (r->error) return;
        r->code[split].y = r->n;
        zam_rx_compile(r, node.b);
        if (r->error) return;
        r->code[jump].x = r->n;
        break;
    }
    case ZRN_REPEAT:
        for (i = 0; i < node.min && !r->error; i++) zam_rx_compile(r, node.a);
        if (node.max < 0) {
            // 循环：SPLIT 优先走（贪婪）进入循环体，或者优先离开（非贪婪）。
            split = zam_rx_emit(r, ZRX_SPLIT, 0, 0, 0);
            zam_rx_compile(r, node.a);
            zam_rx_emit(r, ZRX_JMP, 0, split, 0);
            if (r->error) return;
            if (node.greedy) { r->code[split].x = split + 1; r->code[split].y = r->n; }
            else { r->code[split].x = r->n; r->code[split].y = split + 1; }
        } else {
            int *exits = (int *)malloc(sizeof(int) * (size_t)(node.max - node.min + 1)), count = 0, j;
            if (!exits) zam_alloc_failed();
            for (i = node.min; i < node.max && !r->error; i++) {
                split = zam_rx_emit(r, ZRX_SPLIT, 0, 0, 0);
                exits[count++] = split;
                zam_rx_compile(r, node.a);
            }
            for (j = 0; j < count && !r->error; j++) {
                if (node.greedy) { r->code[exits[j]].x = exits[j] + 1; r->code[exits[j]].y = r->n; }
                else { r->code[exits[j]].x = r->n; r->code[exits[j]].y = exits[j] + 1; }
            }
            free(exits);
        }
        break;
    }
}
static void zam_rx_free(zam_rx *r) {
    int i;
    if (!r) return;
    for (i = 1; i <= r->groups; i++) if (r->names) drop(&r->names[i]);
    free(r->names); free(r->code); free(r->ranges); free(r->classes); free(r->nodes);
    free(r);
}
// 1 = IGNORECASE，2 = MULTILINE，4 = DOTALL（与 std/re 的常量一致）。
static zam_rx *zam_rx_build(S *pattern, int64_t flags, S *error) {
    zam_rx *r = (zam_rx *)calloc(1, sizeof *r);
    int root;
    if (!r) zam_alloc_failed();
    r->p = pattern->data; r->len = pattern->len;
    r->icase = (flags & 1) != 0; r->multiline = (flags & 2) != 0; r->dotall = (flags & 4) != 0;
    r->names = (S *)calloc(1, sizeof(S));
    if (!r->names) zam_alloc_failed();
    root = zam_rx_alt(r, 0);
    if (!r->error && r->at < r->len) r->error = "unbalanced ) in the pattern";
    if (!r->error) {
        zam_rx_emit(r, ZRX_SAVE, 0, 0, 0);
        zam_rx_compile(r, root);
        zam_rx_emit(r, ZRX_SAVE, 0, 1, 0);
        zam_rx_emit(r, ZRX_MATCH, 0, 0, 0);
    }
    if (r->error) {
        S at = from_int((int64_t)r->at);
        *error = cat(cat(zam_message("invalid regular expression at position ", &at), own((const unsigned char *)": ", 2)), own((const unsigned char *)r->error, strlen(r->error)));
        drop(&at);
        zam_rx_free(r);
        return NULL;
    }
    free(r->nodes); r->nodes = NULL;
    return r;
}
// 编译结果缓存：同一个表达式反复使用时不重新编译（同 Python 的 re 缓存）。
#define ZRX_CACHE 16
static struct { S pattern; int64_t flags; zam_rx *rx; } zam_rx_cache[ZRX_CACHE];
static int zam_rx_cache_next;
// 启动过线程之后不用缓存（缓存没有锁）：调用方用完后交给 zam_rx_done，不在缓存里的就释放。
static zam_rx *zam_rx_get(S *pattern, int64_t flags, S *error) {
    int i;
    zam_rx *r;
    if (zam_threaded) return zam_rx_build(pattern, flags, error);
    for (i = 0; i < ZRX_CACHE; i++) if (zam_rx_cache[i].rx && zam_rx_cache[i].flags == flags && string_equal(&zam_rx_cache[i].pattern, pattern)) return zam_rx_cache[i].rx;
    r = zam_rx_build(pattern, flags, error);
    if (!r) return NULL;
    i = zam_rx_cache_next; zam_rx_cache_next = (zam_rx_cache_next + 1) % ZRX_CACHE;
    if (zam_rx_cache[i].rx) { zam_rx_free(zam_rx_cache[i].rx); drop(&zam_rx_cache[i].pattern); }
    zam_rx_cache[i].pattern = copy(pattern); zam_rx_cache[i].flags = flags; zam_rx_cache[i].rx = r;
    return r;
}
static void zam_rx_done(zam_rx *r) {
    int i;
    if (!r) return;
    for (i = 0; i < ZRX_CACHE; i++) if (zam_rx_cache[i].rx == r) return;
    zam_rx_free(r);
}
// ---- 执行 ----
typedef struct { int *pc; int64_t *caps; int count; int *mark; int stamp; } zam_rx_list;
typedef struct { int pc, slot; int64_t old; } zam_rx_step;
typedef struct { zam_rx *r; int ncap; zam_rx_step *stack; int top, scap; } zam_rx_vm;
static int zam_rx_assert(zam_rx *r, int op, const unsigned char *p, size_t len, size_t at, int64_t prev, int64_t cur) {
    switch (op) {
    case ZRX_BOL: return at == 0 || (r->multiline && prev == '\n');
    case ZRX_EOL: return at == len || (cur == '\n' && (r->multiline || at + 1 == len));
    case ZRX_BEGIN: return at == 0;
    case ZRX_END: return at == len;
    case ZRX_WORDB: return (prev >= 0 && zam_rx_word(prev)) != (cur >= 0 && zam_rx_word(cur));
    case ZRX_NWORDB: return (prev >= 0 && zam_rx_word(prev)) == (cur >= 0 && zam_rx_word(cur));
    }
    (void)p;
    return 0;
}
static void zam_rx_push(zam_rx_vm *vm, int pc, int slot, int64_t old) {
    vm->stack = (zam_rx_step *)zam_rx_grow(vm->stack, &vm->scap, vm->top + 1, sizeof *vm->stack);
    vm->stack[vm->top].pc = pc; vm->stack[vm->top].slot = slot; vm->stack[vm->top].old = old; vm->top++;
}
// 从 pc 出发沿着不消耗字符的指令展开，把停在消耗字符或 MATCH 处的线程按优先级加进列表。
static void zam_rx_add(zam_rx_vm *vm, zam_rx_list *list, int pc0, int64_t *caps, const unsigned char *p, size_t len, size_t at, int64_t prev, int64_t cur) {
    zam_rx *r = vm->r;
    zam_rx_push(vm, pc0, -1, 0);
    while (vm->top) {
        zam_rx_step step = vm->stack[--vm->top];
        zam_rx_inst *in;
        if (step.slot >= 0) { caps[step.slot] = step.old; continue; }
        if (list->mark[step.pc] == list->stamp) continue;
        list->mark[step.pc] = list->stamp;
        in = &r->code[step.pc];
        switch (in->op) {
        case ZRX_JMP: zam_rx_push(vm, in->x, -1, 0); break;
        case ZRX_SPLIT: zam_rx_push(vm, in->y, -1, 0); zam_rx_push(vm, in->x, -1, 0); break;
        case ZRX_SAVE:
            zam_rx_push(vm, 0, in->x, caps[in->x]);
            caps[in->x] = (int64_t)at;
            zam_rx_push(vm, step.pc + 1, -1, 0);
            break;
        case ZRX_BOL: case ZRX_EOL: case ZRX_BEGIN: case ZRX_END: case ZRX_WORDB: case ZRX_NWORDB:
            if (zam_rx_assert(r, in->op, p, len, at, prev, cur)) zam_rx_push(vm, step.pc + 1, -1, 0);
            break;
        default:
            list->pc[list->count] = step.pc;
            memcpy(list->caps + (size_t)list->count * vm->ncap, caps, sizeof(int64_t) * (size_t)vm->ncap);
            list->count++;
        }
    }
}
static int zam_rx_consumes(zam_rx *r, zam_rx_inst *in, int64_t c) {
    if (c < 0) return 0;
    switch (in->op) {
    case ZRX_CHAR: return in->c == c || (r->icase && zam_rx_fold(in->c) == zam_rx_fold(c));
    case ZRX_ANY: return c != '\n';
    case ZRX_ANYNL: return 1;
    case ZRX_CLASS: return zam_rx_in_class(r, in->x, c);
    }
    return 0;
}
// 从字节位置 start 起找一次匹配；mode 0 = 任意位置，1 = 只在 start，2 = 从 start 到结尾整段。
// 线程以“待展开”的形式进入下一轮：到那时才知道当前字符，断言（\b、$）才判断得了。
// 找到时把各组的字节位置写进 out（没参与匹配的组是 -1），返回 1。
static int zam_rx_run(zam_rx_vm *vm, const unsigned char *p, size_t len, size_t start, int mode, int64_t *out) {
    zam_rx *r = vm->r;
    int n = r->n, ncap = vm->ncap, matched = 0, i;
    int *pend_pc = (int *)malloc(sizeof(int) * (size_t)n), *next_pc = (int *)malloc(sizeof(int) * (size_t)n), *swap_pc;
    int64_t *pend_caps = (int64_t *)malloc(sizeof(int64_t) * (size_t)n * (size_t)ncap);
    int64_t *next_caps = (int64_t *)malloc(sizeof(int64_t) * (size_t)n * (size_t)ncap), *swap_caps;
    int64_t *caps = (int64_t *)malloc(sizeof(int64_t) * (size_t)ncap), prev = -1, cur;
    int pending = 0, next;
    zam_rx_list list;
    size_t at = start, step;
    list.pc = (int *)malloc(sizeof(int) * (size_t)n);
    list.caps = (int64_t *)malloc(sizeof(int64_t) * (size_t)n * (size_t)ncap);
    list.mark = (int *)calloc((size_t)n, sizeof(int));
    list.stamp = 0;
    if (!pend_pc || !next_pc || !pend_caps || !next_caps || !caps || !list.pc || !list.caps || !list.mark) zam_alloc_failed();
    if (start > 0) {
        size_t back = start - 1;
        while (back > 0 && (p[back] & 0xC0) == 0x80) back--;
        if (!zam_utf8_next(p + back, len - back, &prev)) prev = p[start - 1];
    }
    for (;;) {
        step = 0;
        cur = -1;
        if (at < len) {
            step = zam_utf8_next(p + at, len - at, &cur);
            if (!step) { cur = p[at]; step = 1; }
        }
        // 展开：先是上一轮留下的线程（按优先级），最后是从这里新起的一条（优先级最低）。
        list.count = 0;
        list.stamp++;
        for (i = 0; i < pending; i++) {
            memcpy(caps, pend_caps + (size_t)i * ncap, sizeof(int64_t) * (size_t)ncap);
            zam_rx_add(vm, &list, pend_pc[i], caps, p, len, at, prev, cur);
        }
        if (!matched && (mode == 0 || at == start)) {
            for (i = 0; i < ncap; i++) caps[i] = -1;
            zam_rx_add(vm, &list, 0, caps, p, len, at, prev, cur);
        }
        // 这里一条线程也没有（比如开头的 \b 在这个位置不成立）：还没找到时继续往后扫。
        if (!list.count) {
            if (matched || mode != 0 || at >= len) break;
            pending = 0;
            prev = cur;
            at += step;
            continue;
        }
        next = 0;
        for (i = 0; i < list.count; i++) {
            zam_rx_inst *in = &r->code[list.pc[i]];
            int64_t *tc = list.caps + (size_t)i * ncap;
            if (in->op == ZRX_MATCH) {
                if (mode == 2 && at != len) continue;
                memcpy(out, tc, sizeof(int64_t) * (size_t)ncap);
                matched = 1;
                break;
            }
            if (zam_rx_consumes(r, in, cur)) {
                next_pc[next] = list.pc[i] + 1;
                memcpy(next_caps + (size_t)next * ncap, tc, sizeof(int64_t) * (size_t)ncap);
                next++;
            }
        }
        if (at >= len || (!next && (matched || mode != 0))) break;
        swap_pc = pend_pc; pend_pc = next_pc; next_pc = swap_pc;
        swap_caps = pend_caps; pend_caps = next_caps; next_caps = swap_caps;
        pending = next;
        prev = cur;
        at += step;
    }
    free(pend_pc); free(next_pc); free(pend_caps); free(next_caps); free(caps);
    free(list.pc); free(list.caps); free(list.mark);
    return matched;
}
// 字节位置 [from, to) 里有几个字符。
static int64_t zam_rx_chars(const unsigned char *p, size_t from, size_t to) {
    int64_t count = 0;
    while (from < to) {
        int64_t c;
        size_t n = zam_utf8_next(p + from, to - from, &c);
        from += n ? n : 1;
        count++;
    }
    return count;
}
// 各组的名字：下标 0 是整个匹配（空名字），没有名字的组也是空名字。
static F_Ls zam_re_groups(S *pattern, int64_t flags) {
    S error;
    zam_rx *r = zam_rx_get(pattern, flags, &error);
    Ls out = {0};
    int i;
    if (!r) return F_Ls_err(error);
    pushLs(&out, own((const unsigned char *)"", 0));
    for (i = 1; i <= r->groups; i++) pushLs(&out, copy(&r->names[i]));
    zam_rx_done(r);
    return F_Ls_ok(out);
}
// 从第 start 个字符起找匹配，最多 limit 个（0 表示全部）。每个匹配给出 2 × (组数 + 1) 个字符位置：
// 各组的开始与结束，没参与匹配的组是 -1。空匹配的规则同 Python 3.7：紧跟在空匹配后面的位置上
// 不再接受空匹配。
static F_Li zam_re_find(S *pattern, int64_t flags, S *text, int64_t start, int64_t mode, int64_t limit) {
    S error;
    zam_rx *r = zam_rx_get(pattern, flags, &error);
    zam_rx_vm vm;
    Li out = {0};
    int64_t *caps, count = 0, char_at = 0, prev_end = -1;
    size_t at = 0, scan = 0;
    int prev_empty = 0, i;
    if (!r) return F_Li_err(error);
    vm.r = r; vm.ncap = 2 * (r->groups + 1); vm.stack = NULL; vm.top = 0; vm.scap = 0;
    caps = (int64_t *)malloc(sizeof(int64_t) * (size_t)vm.ncap);
    if (!caps) zam_alloc_failed();
    if (start < 0) start = 0;
    while (at < text->len && char_at < start) {
        int64_t c;
        size_t n = zam_utf8_next(text->data + at, text->len - at, &c);
        at += n ? n : 1;
        char_at++;
    }
    scan = at;
    while (at <= text->len && (limit <= 0 || count < limit)) {
        int64_t base;
        if (!zam_rx_run(&vm, text->data, text->len, at, (int)mode, caps)) break;
        if (caps[0] == caps[1] && prev_empty && caps[0] == prev_end) {
            int64_t c;
            size_t n;
            if (at >= text->len) break;
            n = zam_utf8_next(text->data + at, text->len - at, &c);
            at += n ? n : 1;
            continue;
        }
        char_at += zam_rx_chars(text->data, scan, (size_t)caps[0]);
        scan = (size_t)caps[0];
        base = char_at;
        for (i = 0; i < vm.ncap; i++) pushLi(&out, caps[i] < 0 ? -1 : base + zam_rx_chars(text->data, (size_t)caps[0], (size_t)caps[i]));
        count++;
        prev_empty = caps[0] == caps[1];
        prev_end = caps[1];
        at = (size_t)caps[1];
        if (mode != 0) break;
    }
    free(caps); free(vm.stack);
    zam_rx_done(r);
    return F_Li_ok(out);
}
"#;
const NET_PRELUDE: &str = r#"// std/net 的 C 部分：TCP 连接、监听、收发。std/net（Zamak 写的）用 extern type 把套接字包成句柄，
// 离开作用域时自动关闭。句柄里存的是“套接字 + 1”，这样 0 一定表示没有套接字。
// 失败时返回空句柄或 -1，原因放在 zam_net_error() 里。
static ZAM_THREAD char zam_net_message[256];
static void zam_net_fail(const char *what, int code) {
    snprintf(zam_net_message, sizeof zam_net_message, "%s (error %d)", what, code);
}
const char *zam_net_error(void) { return zam_net_message; }
#ifdef _WIN32
#include <intrin.h>
// Windows 的 addrinfo：canonname 在 addr 前面（与 POSIX 相反）。调用处都经 void * 转换，
// 用户的头文件已经引入 winsock2.h 时也不冲突。
struct zam_addrinfo { int flags, family, socktype, protocol; size_t addrlen; char *canonname; void *addr; struct zam_addrinfo *next; };
#ifndef _WINSOCK2API_
__declspec(dllimport) int __stdcall WSAStartup(unsigned short, void *);
__declspec(dllimport) uintptr_t __stdcall socket(int, int, int);
__declspec(dllimport) int __stdcall connect(uintptr_t, const void *, int);
__declspec(dllimport) int __stdcall bind(uintptr_t, const void *, int);
__declspec(dllimport) int __stdcall listen(uintptr_t, int);
__declspec(dllimport) uintptr_t __stdcall accept(uintptr_t, void *, int *);
__declspec(dllimport) int __stdcall send(uintptr_t, const char *, int, int);
__declspec(dllimport) int __stdcall recv(uintptr_t, char *, int, int);
__declspec(dllimport) int __stdcall closesocket(uintptr_t);
__declspec(dllimport) int __stdcall setsockopt(uintptr_t, int, int, const char *, int);
__declspec(dllimport) int __stdcall getsockname(uintptr_t, void *, int *);
__declspec(dllimport) int __stdcall getaddrinfo(const char *, const char *, const void *, struct zam_addrinfo **);
__declspec(dllimport) void __stdcall freeaddrinfo(struct zam_addrinfo *);
__declspec(dllimport) int __stdcall WSAGetLastError(void);
#endif
#define zam_ai struct zam_addrinfo
#define zam_ai_addr(a) ((a)->addr)
#define zam_ai_next(a) ((a)->next)
#define zam_ai_len(a) ((a)->addrlen)
#define zam_ai_family(a) ((a)->family)
#define zam_ai_socktype(a) ((a)->socktype)
#define zam_ai_protocol(a) ((a)->protocol)
typedef uintptr_t zam_sock;
#define ZAM_BAD_SOCKET (~(uintptr_t)0)
#define ZAM_SOL_SOCKET 0xffff
#define ZAM_SO_REUSEADDR 4
#define ZAM_SO_RCVTIMEO 0x1006
#define ZAM_SO_SNDTIMEO 0x1005
static int zam_net_errno(void) { return WSAGetLastError(); }
static void zam_close_socket(zam_sock s) { closesocket(s); }
// 标志按原子操作读写：几个线程同时第一次联网时可能都调用一次 WSAStartup，它可以重复调用。
static int zam_net_ready(void) {
    static volatile long done;
    char data[512];
    if (!_InterlockedCompareExchange(&done, 0, 0)) {
        if (WSAStartup(0x0202, data) != 0) { zam_net_fail("cannot start Winsock", 0); return 0; }
        _InterlockedExchange(&done, 1);
    }
    return 1;
}
static void zam_set_timeout(zam_sock s, double seconds) {
    unsigned long ms = seconds <= 0 ? 0 : (unsigned long)(seconds * 1000);
    setsockopt(s, ZAM_SOL_SOCKET, ZAM_SO_RCVTIMEO, (const char *)&ms, sizeof ms);
    setsockopt(s, ZAM_SOL_SOCKET, ZAM_SO_SNDTIMEO, (const char *)&ms, sizeof ms);
}
#else
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <netdb.h>
#include <netinet/in.h>
#include <unistd.h>
#include <errno.h>
#include <signal.h>
typedef int zam_sock;
#define ZAM_BAD_SOCKET (-1)
#define ZAM_SOL_SOCKET SOL_SOCKET
#define ZAM_SO_REUSEADDR SO_REUSEADDR
#define zam_ai struct addrinfo
#define zam_ai_addr(a) ((a)->ai_addr)
#define zam_ai_next(a) ((a)->ai_next)
#define zam_ai_len(a) ((a)->ai_addrlen)
#define zam_ai_family(a) ((a)->ai_family)
#define zam_ai_socktype(a) ((a)->ai_socktype)
#define zam_ai_protocol(a) ((a)->ai_protocol)
static int zam_net_errno(void) { return errno; }
static void zam_close_socket(zam_sock s) { close(s); }
static int zam_net_ready(void) {
    // 对方关闭连接后再写不让进程收到 SIGPIPE 而直接退出，改成返回错误。
    // 标志按原子操作读写，几个线程同时第一次联网时可能都设一次，重复设置没有影响。
    static volatile long done;
    if (!__atomic_load_n(&done, __ATOMIC_ACQUIRE)) { signal(SIGPIPE, SIG_IGN); __atomic_store_n(&done, 1, __ATOMIC_RELEASE); }
    return 1;
}
static void zam_set_timeout(zam_sock s, double seconds) {
    struct timeval tv;
    tv.tv_sec = seconds <= 0 ? 0 : (time_t)seconds;
    tv.tv_usec = seconds <= 0 ? 0 : (suseconds_t)((seconds - (double)tv.tv_sec) * 1e6);
    setsockopt(s, SOL_SOCKET, SO_RCVTIMEO, (const char *)&tv, sizeof tv);
    setsockopt(s, SOL_SOCKET, SO_SNDTIMEO, (const char *)&tv, sizeof tv);
}
#endif
static void *zam_sock_handle(zam_sock s) { return (void *)((uintptr_t)s + 1); }
static zam_sock zam_handle_sock(void *h) { return (zam_sock)((uintptr_t)h - 1); }
// getaddrinfo 的提示：IPv4 或 IPv6、TCP；passive 时用于监听。
static int zam_resolve(const char *host, int64_t port, int passive, zam_ai **out) {
    char service[32];
    zam_ai hint, *found = NULL;
    int rc;
    memset(&hint, 0, sizeof hint);
    // 前四个字段两边一样：flags（AI_PASSIVE = 1）、family（不限）、socktype（SOCK_STREAM = 1）、
    // protocol（IPPROTO_TCP = 6）。
    ((int *)&hint)[0] = passive ? 1 : 0;
    ((int *)&hint)[2] = 1;
    ((int *)&hint)[3] = 6;
    snprintf(service, sizeof service, "%lld", (long long)port);
    rc = getaddrinfo(host[0] ? host : NULL, service, (const void *)&hint, (void *)&found);
    if (rc != 0 || !found) { zam_net_fail("cannot resolve the host name", rc); return 0; }
    *out = found;
    return 1;
}
// 连接、监听、接受都把句柄写进输出参数（Zamak 侧是 &mut Option[Socket]），成功返回 0，失败返回 -1。
int64_t zam_net_connect(const char *host, int64_t port, double timeout, void **out) {
    zam_ai *list, *a;
    zam_sock s = ZAM_BAD_SOCKET;
    *out = NULL;
    if (!zam_net_ready()) return -1;
    if (port < 0 || port > 65535) { zam_net_fail("the port must be between 0 and 65535", 0); return -1; }
    if (!zam_resolve(host, port, 0, &list)) return -1;
    for (a = list; a; a = zam_ai_next(a)) {
        s = socket(zam_ai_family(a), zam_ai_socktype(a), zam_ai_protocol(a));
        if (s == ZAM_BAD_SOCKET) continue;
        zam_set_timeout(s, timeout);
        if (connect(s, (const void *)zam_ai_addr(a), (int)zam_ai_len(a)) == 0) break;
        zam_net_fail("cannot connect", zam_net_errno());
        zam_close_socket(s);
        s = ZAM_BAD_SOCKET;
    }
    freeaddrinfo((void *)list);
    if (s == ZAM_BAD_SOCKET) return -1;
    *out = zam_sock_handle(s);
    return 0;
}
int64_t zam_net_listen(const char *host, int64_t port, void **out) {
    zam_ai *list, *a;
    zam_sock s = ZAM_BAD_SOCKET;
    int yes = 1;
    *out = NULL;
    if (!zam_net_ready()) return -1;
    if (port < 0 || port > 65535) { zam_net_fail("the port must be between 0 and 65535", 0); return -1; }
    if (!zam_resolve(host, port, 1, &list)) return -1;
    for (a = list; a; a = zam_ai_next(a)) {
        s = socket(zam_ai_family(a), zam_ai_socktype(a), zam_ai_protocol(a));
        if (s == ZAM_BAD_SOCKET) continue;
        setsockopt(s, ZAM_SOL_SOCKET, ZAM_SO_REUSEADDR, (const char *)&yes, sizeof yes);
        if (bind(s, (const void *)zam_ai_addr(a), (int)zam_ai_len(a)) == 0 && listen(s, 64) == 0) break;
        zam_net_fail("cannot listen on the address", zam_net_errno());
        zam_close_socket(s);
        s = ZAM_BAD_SOCKET;
    }
    freeaddrinfo((void *)list);
    if (s == ZAM_BAD_SOCKET) return -1;
    *out = zam_sock_handle(s);
    return 0;
}
int64_t zam_net_accept(void *listener, void **out) {
    zam_sock s = accept(zam_handle_sock(listener), NULL, NULL);
    *out = NULL;
    if (s == ZAM_BAD_SOCKET) { zam_net_fail("cannot accept a connection", zam_net_errno()); return -1; }
    *out = zam_sock_handle(s);
    return 0;
}
// 端口：监听 0 号端口时系统挑一个空闲的，用它查实际的端口。
int64_t zam_net_port(void *listener) {
    unsigned char address[128];
    int size = (int)sizeof address;
    if (getsockname(zam_handle_sock(listener), (void *)address, (void *)&size) != 0) return -1;
    return (int64_t)address[2] << 8 | address[3];
}
int64_t zam_net_send(void *conn, const uint8_t *data, int64_t n) {
    int64_t done = 0;
    while (done < n) {
        int chunk = n - done > 1 << 30 ? 1 << 30 : (int)(n - done);
        int sent = (int)send(zam_handle_sock(conn), (const char *)data + done, chunk, 0);
        if (sent <= 0) { zam_net_fail("cannot send", zam_net_errno()); return -1; }
        done += sent;
    }
    return done;
}
int64_t zam_net_recv(void *conn, uint8_t *buf, int64_t n) {
    int got;
    if (n <= 0) return 0;
    got = (int)recv(zam_handle_sock(conn), (char *)buf, n > 1 << 30 ? 1 << 30 : (int)n, 0);
    if (got < 0) { zam_net_fail("cannot receive (or the timeout passed)", zam_net_errno()); return -1; }
    return got;
}
void zam_net_timeout(void *conn, double seconds) { zam_set_timeout(zam_handle_sock(conn), seconds); }
void zam_net_close(void *conn) { zam_close_socket(zam_handle_sock(conn)); }
"#;
/// 用 Zamak 写的标准库模块自带的 C 代码：编译这个模块的目标文件时放在开头（extern fn 调用它们）。
const TLS_PRELUDE: &str = r#"// TLS（Mbed TLS）：在已经连上的 TCP 套接字上握手，之后的收发都加密。客户端校验服务器证书和主机名，
// 根证书来自系统（Windows 的证书库；其他平台的常见证书文件），或者环境变量 SSL_CERT_FILE 指定的文件。
#include "mbedtls/ssl.h"
#include "mbedtls/entropy.h"
#include "mbedtls/ctr_drbg.h"
#include "mbedtls/x509_crt.h"
#include "mbedtls/pk.h"
#include "mbedtls/error.h"
#include "psa/crypto.h"
#include "mbedtls/threading.h"
#define ZAM_TLS_SEND_FAILED (-0x004E)
#define ZAM_TLS_RECV_FAILED (-0x004C)
typedef struct {
    mbedtls_ssl_context ssl;
    mbedtls_ssl_config conf;
    mbedtls_x509_crt ca, cert;
    mbedtls_pk_context key;
    mbedtls_entropy_context entropy;
    mbedtls_ctr_drbg_context drbg;
    void *socket;
} zam_tls;
static ZAM_THREAD char zam_tls_message[512];
const char *zam_tls_error(void) { return zam_tls_message; }
static void zam_tls_fail(const char *what, int rc) {
    char detail[256];
    mbedtls_strerror(rc, detail, sizeof detail);
    snprintf(zam_tls_message, sizeof zam_tls_message, "%s: %s", what, detail);
}
static int zam_tls_bio_send(void *ctx, const unsigned char *buf, size_t len) {
    int sent = (int)send(zam_handle_sock(ctx), (const char *)buf, len > 1 << 30 ? 1 << 30 : (int)len, 0);
    return sent < 0 ? ZAM_TLS_SEND_FAILED : sent;
}
static int zam_tls_bio_recv(void *ctx, unsigned char *buf, size_t len) {
    int got = (int)recv(zam_handle_sock(ctx), (char *)buf, len > 1 << 30 ? 1 << 30 : (int)len, 0);
    return got < 0 ? ZAM_TLS_RECV_FAILED : got;
}
#ifdef _WIN32
typedef struct { unsigned long encoding; unsigned char *data; unsigned long size; void *info; void *store; } zam_cert_context;
__declspec(dllimport) void *__stdcall CertOpenSystemStoreW(uintptr_t, const wchar_t *);
__declspec(dllimport) const void *__stdcall CertEnumCertificatesInStore(void *, const void *);
__declspec(dllimport) int __stdcall CertCloseStore(void *, unsigned long);
#endif
// 系统的根证书。返回装进来的证书数；0 表示一张都没找到。
static int zam_tls_roots(mbedtls_x509_crt *ca) {
    const char *file = getenv("SSL_CERT_FILE");
    int loaded = 0;
    if (file && *file) return mbedtls_x509_crt_parse_file(ca, file) >= 0 && ca->raw.len ? 1 : 0;
#ifdef _WIN32
    {
        const wchar_t *stores[] = {L"ROOT", L"CA"};
        size_t k;
        for (k = 0; k < 2; k++) {
            void *store = CertOpenSystemStoreW(0, stores[k]);
            const zam_cert_context *c = NULL;
            if (!store) continue;
            while ((c = (const zam_cert_context *)CertEnumCertificatesInStore(store, c)) != NULL) {
                if ((c->encoding & 1) && mbedtls_x509_crt_parse_der(ca, c->data, c->size) == 0) loaded++;
            }
            CertCloseStore(store, 0);
        }
    }
#else
    {
        static const char *files[] = {
            "/etc/ssl/certs/ca-certificates.crt", "/etc/pki/tls/certs/ca-bundle.crt", "/etc/ssl/ca-bundle.pem",
            "/etc/pki/tls/cacert.pem", "/etc/ssl/cert.pem", "/usr/local/etc/openssl/cert.pem", "/opt/homebrew/etc/openssl@3/cert.pem", NULL};
        size_t k;
        for (k = 0; files[k] && !loaded; k++) {
            FILE *probe = fopen(files[k], "rb");
            if (!probe) continue;
            fclose(probe);
            if (mbedtls_x509_crt_parse_file(ca, files[k]) >= 0 && ca->raw.len) loaded = 1;
        }
        if (!loaded && mbedtls_x509_crt_parse_path(ca, "/etc/ssl/certs") >= 0 && ca->raw.len) loaded = 1;
    }
#endif
    return loaded;
}
// 整个程序只做一次的准备：Windows 上先给 Mbed TLS 装上互斥锁（SRW 锁），再初始化 PSA 加密库。
// 几个线程同时第一次用 TLS 时，只有一个去做，其余等它做完。
// mbedtls_ssl_list_ciphersuites 第一次调用时才填好一张静态表（没有锁），也放在这里先调一次。
static int zam_tls_ready;
#ifdef _WIN32
#include <intrin.h>
#ifdef _WINDOWS_
typedef SRWLOCK zam_tls_srw;
#else
typedef struct { void *p; } zam_tls_srw;
__declspec(dllimport) void __stdcall AcquireSRWLockExclusive(zam_tls_srw *);
__declspec(dllimport) void __stdcall ReleaseSRWLockExclusive(zam_tls_srw *);
#endif
static void zam_tls_mutex_init(mbedtls_threading_mutex_t *m) { m->lock = NULL; }
static void zam_tls_mutex_free(mbedtls_threading_mutex_t *m) { (void)m; }
static int zam_tls_mutex_lock(mbedtls_threading_mutex_t *m) { AcquireSRWLockExclusive((zam_tls_srw *)&m->lock); return 0; }
static int zam_tls_mutex_unlock(mbedtls_threading_mutex_t *m) { ReleaseSRWLockExclusive((zam_tls_srw *)&m->lock); return 0; }
static volatile long zam_tls_state; /* 0 还没开始，1 正在准备，2 准备好了 */
static int zam_tls_setup(void) {
    if (_InterlockedCompareExchange(&zam_tls_state, 1, 0) == 0) {
        mbedtls_threading_set_alt(zam_tls_mutex_init, zam_tls_mutex_free, zam_tls_mutex_lock, zam_tls_mutex_unlock);
        zam_tls_ready = psa_crypto_init() == PSA_SUCCESS;
        (void)mbedtls_ssl_list_ciphersuites();
        _InterlockedExchange(&zam_tls_state, 2);
    } else {
        while (_InterlockedCompareExchange(&zam_tls_state, 2, 2) != 2) {}
    }
    return zam_tls_ready;
}
#else
#include <pthread.h>
static pthread_once_t zam_tls_once = PTHREAD_ONCE_INIT;
static void zam_tls_prepare(void) { zam_tls_ready = psa_crypto_init() == PSA_SUCCESS; (void)mbedtls_ssl_list_ciphersuites(); }
static int zam_tls_setup(void) { pthread_once(&zam_tls_once, zam_tls_prepare); return zam_tls_ready; }
#endif
static zam_tls *zam_tls_new(void *socket, int server) {
    zam_tls *t = (zam_tls *)calloc(1, sizeof *t);
    int rc;
    if (!t) { snprintf(zam_tls_message, sizeof zam_tls_message, "out of memory"); return NULL; }
    if (!zam_tls_setup()) { snprintf(zam_tls_message, sizeof zam_tls_message, "cannot start the crypto library"); free(t); return NULL; }
    t->socket = socket;
    mbedtls_ssl_init(&t->ssl);
    mbedtls_ssl_config_init(&t->conf);
    mbedtls_x509_crt_init(&t->ca);
    mbedtls_x509_crt_init(&t->cert);
    mbedtls_pk_init(&t->key);
    mbedtls_entropy_init(&t->entropy);
    mbedtls_ctr_drbg_init(&t->drbg);
    rc = mbedtls_ctr_drbg_seed(&t->drbg, mbedtls_entropy_func, &t->entropy, (const unsigned char *)"zamak", 5);
    if (rc == 0) rc = mbedtls_ssl_config_defaults(&t->conf, server ? MBEDTLS_SSL_IS_SERVER : MBEDTLS_SSL_IS_CLIENT, MBEDTLS_SSL_TRANSPORT_STREAM, MBEDTLS_SSL_PRESET_DEFAULT);
    if (rc != 0) { zam_tls_fail("cannot set up TLS", rc); return t; }
    mbedtls_ssl_conf_rng(&t->conf, mbedtls_ctr_drbg_random, &t->drbg);
    return t;
}
void zam_tls_free(void *handle) {
    zam_tls *t = (zam_tls *)handle;
    if (!t) return;
    mbedtls_ssl_close_notify(&t->ssl);
    mbedtls_ssl_free(&t->ssl);
    mbedtls_ssl_config_free(&t->conf);
    mbedtls_x509_crt_free(&t->ca);
    mbedtls_x509_crt_free(&t->cert);
    mbedtls_pk_free(&t->key);
    mbedtls_ctr_drbg_free(&t->drbg);
    mbedtls_entropy_free(&t->entropy);
    free(t);
}
static int64_t zam_tls_handshake(zam_tls *t, void **out) {
    int rc;
    mbedtls_ssl_set_bio(&t->ssl, t->socket, zam_tls_bio_send, zam_tls_bio_recv, NULL);
    while ((rc = mbedtls_ssl_handshake(&t->ssl)) != 0) {
        if (rc == MBEDTLS_ERR_SSL_WANT_READ || rc == MBEDTLS_ERR_SSL_WANT_WRITE) continue;
        if (rc == MBEDTLS_ERR_X509_CERT_VERIFY_FAILED) {
            char why[256];
            mbedtls_x509_crt_verify_info(why, sizeof why, "", mbedtls_ssl_get_verify_result(&t->ssl));
            snprintf(zam_tls_message, sizeof zam_tls_message, "the server's certificate is not trusted: %s", why);
            { size_t n = strlen(zam_tls_message); while (n && (zam_tls_message[n - 1] == '\n' || zam_tls_message[n - 1] == ' ')) zam_tls_message[--n] = 0; }
        } else {
            zam_tls_fail("TLS handshake failed", rc);
        }
        zam_tls_free(t);
        return -1;
    }
    *out = t;
    return 0;
}
// 客户端：校验证书链与主机名（同时用作 SNI）。
int64_t zam_tls_connect(void *socket, const char *host, void **out) {
    zam_tls *t = zam_tls_new(socket, 0);
    int rc;
    *out = NULL;
    if (!t) return -1;
    if (!zam_tls_roots(&t->ca)) {
        snprintf(zam_tls_message, sizeof zam_tls_message, "no trusted root certificates found; set SSL_CERT_FILE to a PEM file of CA certificates");
        zam_tls_free(t);
        return -1;
    }
    mbedtls_ssl_conf_authmode(&t->conf, MBEDTLS_SSL_VERIFY_REQUIRED);
    mbedtls_ssl_conf_ca_chain(&t->conf, &t->ca, NULL);
    rc = mbedtls_ssl_setup(&t->ssl, &t->conf);
    if (rc == 0) rc = mbedtls_ssl_set_hostname(&t->ssl, host);
    if (rc != 0) { zam_tls_fail("cannot set up TLS", rc); zam_tls_free(t); return -1; }
    return zam_tls_handshake(t, out);
}
// 服务端：用 PEM 文件里的证书链与私钥。
int64_t zam_tls_accept(void *socket, const char *cert_file, const char *key_file, void **out) {
    zam_tls *t = zam_tls_new(socket, 1);
    int rc;
    *out = NULL;
    if (!t) return -1;
    rc = mbedtls_x509_crt_parse_file(&t->cert, cert_file);
    if (rc != 0) { zam_tls_fail("cannot read the certificate file", rc); zam_tls_free(t); return -1; }
    rc = mbedtls_pk_parse_keyfile(&t->key, key_file, NULL, mbedtls_ctr_drbg_random, &t->drbg);
    if (rc != 0) { zam_tls_fail("cannot read the private key file", rc); zam_tls_free(t); return -1; }
    rc = mbedtls_ssl_conf_own_cert(&t->conf, &t->cert, &t->key);
    if (rc == 0) rc = mbedtls_ssl_setup(&t->ssl, &t->conf);
    if (rc != 0) { zam_tls_fail("cannot set up TLS", rc); zam_tls_free(t); return -1; }
    return zam_tls_handshake(t, out);
}
int64_t zam_tls_send(void *handle, const uint8_t *data, int64_t n) {
    zam_tls *t = (zam_tls *)handle;
    int64_t done = 0;
    while (done < n) {
        int rc = mbedtls_ssl_write(&t->ssl, data + done, (size_t)(n - done > 16384 ? 16384 : n - done));
        if (rc == MBEDTLS_ERR_SSL_WANT_READ || rc == MBEDTLS_ERR_SSL_WANT_WRITE) continue;
        if (rc < 0) { zam_tls_fail("cannot send", rc); return -1; }
        done += rc;
    }
    return done;
}
// 对方正常关闭（close_notify 或连接结束）时返回 0。
int64_t zam_tls_recv(void *handle, uint8_t *buf, int64_t n) {
    zam_tls *t = (zam_tls *)handle;
    for (;;) {
        int rc = mbedtls_ssl_read(&t->ssl, buf, (size_t)(n > 1 << 30 ? 1 << 30 : n));
        if (rc == MBEDTLS_ERR_SSL_WANT_READ || rc == MBEDTLS_ERR_SSL_WANT_WRITE) continue;
#ifdef MBEDTLS_ERR_SSL_RECEIVED_NEW_SESSION_TICKET
        if (rc == MBEDTLS_ERR_SSL_RECEIVED_NEW_SESSION_TICKET) continue;
#endif
        // 有的服务器不发 close_notify 就断开连接，也算正常结束。
        if (rc == MBEDTLS_ERR_SSL_PEER_CLOSE_NOTIFY || rc == MBEDTLS_ERR_SSL_CONN_EOF || rc == 0) return 0;
        if (rc < 0) { zam_tls_fail("cannot receive", rc); return -1; }
        return rc;
    }
}
"#;
const THREAD_PRELUDE: &str = r#"// std/thread 的 C 部分：线程的启动、等待与释放，以及线程之间的队列。Windows 用 _beginthreadex、
// SRW 锁与条件变量，其他平台用 pthread。线程入口与按类型生成的队列函数见 native.rs 的 intrinsic。
#ifdef _WIN32
#include <process.h>
#include <intrin.h>
#ifdef _WINDOWS_
typedef SRWLOCK zam_lock_t;
typedef CONDITION_VARIABLE zam_cond_t;
#else
typedef struct { void *p; } zam_lock_t;
typedef struct { void *p; } zam_cond_t;
__declspec(dllimport) void __stdcall AcquireSRWLockExclusive(zam_lock_t *);
__declspec(dllimport) void __stdcall ReleaseSRWLockExclusive(zam_lock_t *);
__declspec(dllimport) int __stdcall SleepConditionVariableSRW(zam_cond_t *, zam_lock_t *, unsigned long, unsigned long);
__declspec(dllimport) void __stdcall WakeConditionVariable(zam_cond_t *);
__declspec(dllimport) void __stdcall WakeAllConditionVariable(zam_cond_t *);
__declspec(dllimport) unsigned long __stdcall WaitForSingleObject(void *, unsigned long);
__declspec(dllimport) int __stdcall CloseHandle(void *);
__declspec(dllimport) unsigned long __stdcall GetActiveProcessorCount(unsigned short);
#endif
static void zam_lock_init(zam_lock_t *l) { memset(l, 0, sizeof *l); }
static void zam_lock_free(zam_lock_t *l) { (void)l; }
static void zam_lock(zam_lock_t *l) { AcquireSRWLockExclusive(l); }
static void zam_unlock(zam_lock_t *l) { ReleaseSRWLockExclusive(l); }
static void zam_cond_init(zam_cond_t *c) { memset(c, 0, sizeof *c); }
static void zam_cond_free(zam_cond_t *c) { (void)c; }
static void zam_cond_wait(zam_cond_t *c, zam_lock_t *l) { SleepConditionVariableSRW(c, l, 0xFFFFFFFFu, 0); }
static void zam_cond_signal(zam_cond_t *c) { WakeConditionVariable(c); }
static void zam_cond_broadcast(zam_cond_t *c) { WakeAllConditionVariable(c); }
static void zam_flag_set(volatile long *flag) { _InterlockedExchange(flag, 1); }
static int zam_flag_get(volatile long *flag) { return _InterlockedCompareExchange(flag, 0, 0) != 0; }
#else
#include <pthread.h>
#include <unistd.h>
typedef pthread_mutex_t zam_lock_t;
typedef pthread_cond_t zam_cond_t;
static void zam_lock_init(zam_lock_t *l) { pthread_mutex_init(l, NULL); }
static void zam_lock_free(zam_lock_t *l) { pthread_mutex_destroy(l); }
static void zam_lock(zam_lock_t *l) { pthread_mutex_lock(l); }
static void zam_unlock(zam_lock_t *l) { pthread_mutex_unlock(l); }
static void zam_cond_init(zam_cond_t *c) { pthread_cond_init(c, NULL); }
static void zam_cond_free(zam_cond_t *c) { pthread_cond_destroy(c); }
static void zam_cond_wait(zam_cond_t *c, zam_lock_t *l) { pthread_cond_wait(c, l); }
static void zam_cond_signal(zam_cond_t *c) { pthread_cond_signal(c); }
static void zam_cond_broadcast(zam_cond_t *c) { pthread_cond_broadcast(c); }
static void zam_flag_set(volatile long *flag) { __atomic_store_n(flag, 1, __ATOMIC_RELEASE); }
static int zam_flag_get(volatile long *flag) { return __atomic_load_n(flag, __ATOMIC_ACQUIRE) != 0; }
#endif
static void zam_thread_alloc_failed(void) { zam_panic_one("allocation failed"); exit(1); }
// 线程的返回值先复制到堆上，join 时再取走。
static void *zam_thread_box(const void *value, size_t size) {
    void *p = malloc(size ? size : 1);
    if (!p) zam_thread_alloc_failed();
    memcpy(p, value, size);
    return p;
}
// 一个线程：入口函数与它的参数（job），结束后的返回值或错误。只有句柄的主人（一个 Zamak 值）
// 会等待、取结果和释放它，所以除了 finished 之外都不需要锁：等待线程结束本身就是同步。
typedef struct {
    void (*run)(void *job, void **result, S *error, int *failed);
    void *job, *result;
    void (*drop_result)(void *);
    S error;
    char *message;
    int failed, joined;
    volatile long finished;
#ifdef _WIN32
    void *os;
#else
    pthread_t os;
#endif
} zam_thread;
#ifdef _WIN32
static unsigned __stdcall zam_thread_main(void *p) {
#else
static void *zam_thread_main(void *p) {
#endif
    zam_thread *t = (zam_thread *)p;
    t->run(t->job, &t->result, &t->error, &t->failed);
    zam_flag_set(&t->finished);
    return 0;
}
static void *zam_thread_start(void (*run)(void *, void **, S *, int *), void *job, void (*drop_result)(void *)) {
    zam_thread *t = (zam_thread *)calloc(1, sizeof *t);
    if (!t) zam_thread_alloc_failed();
    t->run = run; t->job = job; t->drop_result = drop_result;
    // 第一个线程启动之前置位：之后各线程都不再用只在单线程时安全的优化。只写这一次，
    // 已经有别的线程在读它时不能再写（即使写的是同样的值，也算数据竞争）。
    if (!zam_threaded) zam_threaded = 1;
#ifdef _WIN32
    t->os = (void *)_beginthreadex(NULL, 0, zam_thread_main, t, 0, NULL);
    if (!t->os) { zam_panic_one("cannot start a thread"); exit(1); }
#else
    if (pthread_create(&t->os, NULL, zam_thread_main, t) != 0) { zam_panic_one("cannot start a thread"); exit(1); }
#endif
    return t;
}
static void zam_thread_wait(zam_thread *t) {
    if (t->joined) return;
#ifdef _WIN32
    WaitForSingleObject(t->os, 0xFFFFFFFFu);
    CloseHandle(t->os);
#else
    pthread_join(t->os, NULL);
#endif
    t->joined = 1;
}
// 等线程结束：成功是 0；入口函数失败时是 -1，错误信息由 zam_thread_error 给出。
int64_t zam_thread_join(void *handle) {
    zam_thread *t = (zam_thread *)handle;
    zam_thread_wait(t);
    if (!t->failed) return 0;
    if (!t->message) t->message = zam_cstr(&t->error);
    return -1;
}
const char *zam_thread_error(void *handle) {
    zam_thread *t = (zam_thread *)handle;
    return t->message ? t->message : "";
}
int zam_thread_alive(void *handle) { return !zam_flag_get(&((zam_thread *)handle)->finished); }
// 取走返回值（堆上的副本），之后由调用方释放。
static void *zam_thread_take(void *handle) {
    zam_thread *t = (zam_thread *)handle;
    void *result = t->result;
    t->result = NULL;
    return result;
}
// 句柄离开作用域：没有 join 的线程先等它结束，再释放没取走的返回值与错误。
void zam_thread_release(void *handle) {
    zam_thread *t = (zam_thread *)handle;
    if (!t) return;
    zam_thread_wait(t);
    if (t->result) {
        if (t->drop_result) t->drop_result(t->result);
        free(t->result);
    }
    if (t->failed) drop(&t->error);
    free(t->message);
    free(t);
}
int64_t zam_thread_cpus(void) {
#ifdef _WIN32
    unsigned long n = GetActiveProcessorCount(0xFFFF);
#else
    long n = sysconf(_SC_NPROCESSORS_ONLN);
#endif
    return n > 0 ? (int64_t)n : 1;
}
// 队列：环形缓冲区，每个元素 size 字节（元素在 C 里的值，按值复制进来，所有权一起移交）。
// 句柄有引用计数，最后一个释放时用 drop 释放没取走的元素。
typedef struct {
    zam_lock_t lock;
    zam_cond_t ready;
    unsigned char *items;
    size_t size, head, count, cap;
    void (*drop)(void *);
    int closed;
    int64_t refs;
} zam_queue;
static void *zam_queue_new(int64_t size, void (*drop)(void *)) {
    zam_queue *q = (zam_queue *)calloc(1, sizeof *q);
    if (!q) zam_thread_alloc_failed();
    zam_lock_init(&q->lock);
    zam_cond_init(&q->ready);
    q->size = (size_t)size;
    q->drop = drop;
    q->refs = 1;
    return q;
}
// 放进一个元素；队列已经关闭时由队列释放这个元素，返回 0。
static int zam_queue_put(void *handle, void *item) {
    zam_queue *q = (zam_queue *)handle;
    zam_lock(&q->lock);
    if (q->closed) {
        zam_unlock(&q->lock);
        if (q->drop) q->drop(item);
        return 0;
    }
    if (q->count == q->cap) {
        size_t cap = q->cap ? q->cap * 2 : 8, i;
        unsigned char *items = (unsigned char *)malloc(cap * q->size);
        if (!items) zam_thread_alloc_failed();
        for (i = 0; i < q->count; i++) memcpy(items + i * q->size, q->items + ((q->head + i) % q->cap) * q->size, q->size);
        free(q->items);
        q->items = items; q->cap = cap; q->head = 0;
    }
    memcpy(q->items + ((q->head + q->count) % q->cap) * q->size, item, q->size);
    q->count++;
    zam_cond_signal(&q->ready);
    zam_unlock(&q->lock);
    return 1;
}
// 取出最早的元素；队列空时等待。关闭并且取空之后返回 0。
static int zam_queue_get(void *handle, void *out) {
    zam_queue *q = (zam_queue *)handle;
    zam_lock(&q->lock);
    while (!q->count && !q->closed) zam_cond_wait(&q->ready, &q->lock);
    if (!q->count) { zam_unlock(&q->lock); return 0; }
    memcpy(out, q->items + q->head * q->size, q->size);
    q->head = (q->head + 1) % q->cap;
    q->count--;
    zam_unlock(&q->lock);
    return 1;
}
void zam_queue_close(void *handle) {
    zam_queue *q = (zam_queue *)handle;
    zam_lock(&q->lock);
    q->closed = 1;
    zam_cond_broadcast(&q->ready);
    zam_unlock(&q->lock);
}
int64_t zam_queue_len(void *handle) {
    zam_queue *q = (zam_queue *)handle;
    int64_t n;
    zam_lock(&q->lock);
    n = (int64_t)q->count;
    zam_unlock(&q->lock);
    return n;
}
int64_t zam_queue_retain(void *handle, void **out) {
    zam_queue *q = (zam_queue *)handle;
    zam_lock(&q->lock);
    q->refs++;
    zam_unlock(&q->lock);
    *out = q;
    return 0;
}
void zam_queue_release(void *handle) {
    zam_queue *q = (zam_queue *)handle;
    int last;
    if (!q) return;
    zam_lock(&q->lock);
    last = --q->refs == 0;
    zam_unlock(&q->lock);
    if (!last) return;
    if (q->drop) {
        size_t i;
        for (i = 0; i < q->count; i++) q->drop(q->items + ((q->head + i) % q->cap) * q->size);
    }
    free(q->items);
    zam_cond_free(&q->ready);
    zam_lock_free(&q->lock);
    free(q);
}
"#;
const UNIT_PRELUDES: &[(&str, &[&str])] = &[
    ("std/net", &[NET_PRELUDE, TLS_PRELUDE]),
    ("std/thread", &[THREAD_PRELUDE]),
    ("std/ui", &[UI_PRELUDE]),
    ("std/sqlite", &[SQLITE_PRELUDE]),
];
const SQLITE_PRELUDE: &str = r#"// std/sqlite 的 C 部分：SQLite 接口的薄包装。连接与语句是 C 句柄，离开作用域时关闭。
// 打开或准备失败时句柄留空、返回 -1，原因放在 zam_sqlite_failure 里。
#include "sqlite3.h"
static ZAM_THREAD char zam_sqlite_failure[512];
static int64_t zam_sqlite_fail(const char *message) {
    snprintf(zam_sqlite_failure, sizeof zam_sqlite_failure, "%s", message ? message : "unknown SQLite error");
    return -1;
}
const char *zam_sqlite_last_error(void) { return zam_sqlite_failure; }
int64_t zam_sqlite_open(const char *path, void **out) {
    sqlite3 *db = NULL;
    int rc = sqlite3_open_v2(path, &db, SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_URI, NULL);
    if (rc != SQLITE_OK) {
        zam_sqlite_fail(db ? sqlite3_errmsg(db) : sqlite3_errstr(rc));
        sqlite3_close_v2(db);
        return -1;
    }
    // 另一个连接正在写时最多等 5 秒，而不是马上报 database is locked。
    sqlite3_busy_timeout(db, 5000);
    *out = db;
    return 0;
}
void zam_sqlite_close(void *db) { sqlite3_close_v2((sqlite3 *)db); }
int64_t zam_sqlite_exec(void *db, const char *sql) {
    char *message = NULL;
    if (sqlite3_exec((sqlite3 *)db, sql, NULL, NULL, &message) != SQLITE_OK) {
        zam_sqlite_fail(message ? message : sqlite3_errmsg((sqlite3 *)db));
        sqlite3_free(message);
        return -1;
    }
    return 0;
}
int64_t zam_sqlite_prepare(void *db, const char *sql, void **out) {
    sqlite3_stmt *st = NULL;
    const char *rest = NULL;
    if (sqlite3_prepare_v2((sqlite3 *)db, sql, -1, &st, &rest) != SQLITE_OK) {
        return zam_sqlite_fail(sqlite3_errmsg((sqlite3 *)db));
    }
    if (!st) return zam_sqlite_fail("the SQL text has no statement");
    while (rest && (*rest == ' ' || *rest == '\t' || *rest == '\n' || *rest == '\r' || *rest == ';')) rest++;
    if (rest && *rest) {
        sqlite3_finalize(st);
        return zam_sqlite_fail("the SQL text has more than one statement; use execute_script for several");
    }
    *out = st;
    return 0;
}
void zam_sqlite_finalize(void *st) { sqlite3_finalize((sqlite3_stmt *)st); }
static int64_t zam_sqlite_bound(void *st, int rc) {
    return rc == SQLITE_OK ? 0 : zam_sqlite_fail(sqlite3_errmsg(sqlite3_db_handle((sqlite3_stmt *)st)));
}
int64_t zam_sqlite_bind_int(void *st, int64_t i, int64_t v) { return zam_sqlite_bound(st, sqlite3_bind_int64((sqlite3_stmt *)st, (int)i, v)); }
int64_t zam_sqlite_bind_real(void *st, int64_t i, double v) { return zam_sqlite_bound(st, sqlite3_bind_double((sqlite3_stmt *)st, (int)i, v)); }
// 文字参数在调用后就释放，所以让 SQLite 复制一份。
int64_t zam_sqlite_bind_text(void *st, int64_t i, const char *v) { return zam_sqlite_bound(st, sqlite3_bind_text((sqlite3_stmt *)st, (int)i, v, -1, SQLITE_TRANSIENT)); }
int64_t zam_sqlite_bind_null(void *st, int64_t i) { return zam_sqlite_bound(st, sqlite3_bind_null((sqlite3_stmt *)st, (int)i)); }
int64_t zam_sqlite_params(void *st) { return sqlite3_bind_parameter_count((sqlite3_stmt *)st); }
const char *zam_sqlite_param_name(void *st, int64_t i) { return sqlite3_bind_parameter_name((sqlite3_stmt *)st, (int)i); }
// 1 表示有一行，0 表示执行完了，-1 表示出错。
int64_t zam_sqlite_step(void *st) {
    int rc = sqlite3_step((sqlite3_stmt *)st);
    if (rc == SQLITE_ROW) return 1;
    if (rc == SQLITE_DONE) return 0;
    return zam_sqlite_fail(sqlite3_errmsg(sqlite3_db_handle((sqlite3_stmt *)st)));
}
int64_t zam_sqlite_columns(void *st) { return sqlite3_column_count((sqlite3_stmt *)st); }
const char *zam_sqlite_column_name(void *st, int64_t i) { return sqlite3_column_name((sqlite3_stmt *)st, (int)i); }
// SQLITE_INTEGER 1、SQLITE_FLOAT 2、SQLITE_TEXT 3、SQLITE_BLOB 4、SQLITE_NULL 5。
int64_t zam_sqlite_column_type(void *st, int64_t i) { return sqlite3_column_type((sqlite3_stmt *)st, (int)i); }
int64_t zam_sqlite_column_int(void *st, int64_t i) { return sqlite3_column_int64((sqlite3_stmt *)st, (int)i); }
double zam_sqlite_column_real(void *st, int64_t i) { return sqlite3_column_double((sqlite3_stmt *)st, (int)i); }
const char *zam_sqlite_column_text(void *st, int64_t i) { return (const char *)sqlite3_column_text((sqlite3_stmt *)st, (int)i); }
// 建表时声明的列类型（`BOOLEAN`）；表达式算出来的列没有，得到空串。
const char *zam_sqlite_column_decl(void *st, int64_t i) { return sqlite3_column_decltype((sqlite3_stmt *)st, (int)i); }
int64_t zam_sqlite_changes(void *db) { return sqlite3_changes64((sqlite3 *)db); }
int64_t zam_sqlite_last_id(void *db) { return sqlite3_last_insert_rowid((sqlite3 *)db); }
"#;
const UI_PRELUDE: &str = r#"// std/ui 的 C 部分：libui-ng 的薄包装。控件在 Zamak 里是整数句柄（控件指针），归父容器所有，
// 窗口关闭时一起销毁，所以句柄本身不释放。App 持有程序状态（按值复制进来的 T）与回调：
// 回调是不捕获局部变量的 Zamak 函数值，经按 T 生成的调用器（见 native.rs 的 intrinsic）调用。
// 状态在 run 期间放在 run 的栈帧里（按 T 生成的 _run），回调拿到的是它的地址。
#define _UI_STATIC
#include "ui.h"
static char zam_ui_failure[512];
const char *zam_ui_error(void) { return zam_ui_failure; }
// 界面只能在一个线程（通常是主线程）里用：第一次用时初始化。
int64_t zam_ui_start(void) {
    static int ready;
    uiInitOptions options;
    const char *error;
    if (ready) return 0;
    memset(&options, 0, sizeof options);
    error = uiInit(&options);
    if (error) {
        snprintf(zam_ui_failure, sizeof zam_ui_failure, "cannot start the user interface: %s", error);
        uiFreeInitError(error);
        return -1;
    }
    ready = 1;
    return 0;
}
// 创建控件之前确保界面已初始化；没有图形界面可用（比如没有显示器）时说明原因并退出。
static void zam_ui_need(void) {
    if (zam_ui_start() < 0) { fflush(stdout); fprintf(stderr, "zam: %s\n", zam_ui_failure); exit(1); }
}
#define ZAM_UI(h) ((void *)(intptr_t)(h))
#define ZAM_UI_ID(p) ((int64_t)(intptr_t)(p))
// libui 返回的文字要用 uiFreeText 释放：先复制到这个缓冲区（Zamak 立即再复制一份），再释放。
static char *zam_ui_text;
static const char *zam_ui_keep(char *text) {
    size_t n = text ? strlen(text) : 0;
    free(zam_ui_text);
    zam_ui_text = (char *)malloc(n + 1);
    if (!zam_ui_text) { zam_panic_one("allocation failed"); exit(1); }
    if (n) memcpy(zam_ui_text, text, n);
    zam_ui_text[n] = 0;
    if (text) uiFreeText(text);
    return zam_ui_text;
}
// ---- 所有控件 ----
void zam_ui_show(int64_t c) { uiControlShow(uiControl(ZAM_UI(c))); }
void zam_ui_hide(int64_t c) { uiControlHide(uiControl(ZAM_UI(c))); }
void zam_ui_enable(int64_t c) { uiControlEnable(uiControl(ZAM_UI(c))); }
void zam_ui_disable(int64_t c) { uiControlDisable(uiControl(ZAM_UI(c))); }
int zam_ui_visible(int64_t c) { return uiControlVisible(uiControl(ZAM_UI(c))); }
int zam_ui_enabled(int64_t c) { return uiControlEnabled(uiControl(ZAM_UI(c))); }
// ---- 窗口 ----
int64_t zam_ui_window(const char *title, int64_t width, int64_t height, int menubar) { zam_ui_need(); return ZAM_UI_ID(uiNewWindow(title, (int)width, (int)height, menubar)); }
// 窗口里的控件铺满窗口。Windows 与 macOS 本来就这样；GTK 上 libui 把它放进一个横向的盒子，控件不声明
// 要扩展时只有自然宽度（画布是 0），所以这里替它声明。只用到两个 GTK 函数，手写声明，不引入 GTK 头文件。
#if !defined(_WIN32) && !defined(__APPLE__)
void gtk_widget_set_hexpand(void *widget, int expand);
void gtk_widget_set_vexpand(void *widget, int expand);
#endif
void zam_ui_window_set_child(int64_t w, int64_t child) {
#if !defined(_WIN32) && !defined(__APPLE__)
    void *widget = (void *)uiControlHandle(uiControl(ZAM_UI(child)));
    gtk_widget_set_hexpand(widget, 1);
    gtk_widget_set_vexpand(widget, 1);
#endif
    uiWindowSetChild(uiWindow(ZAM_UI(w)), uiControl(ZAM_UI(child)));
}
void zam_ui_window_set_title(int64_t w, const char *title) { uiWindowSetTitle(uiWindow(ZAM_UI(w)), title); }
const char *zam_ui_window_title(int64_t w) { return zam_ui_keep(uiWindowTitle(uiWindow(ZAM_UI(w)))); }
void zam_ui_window_set_margined(int64_t w, int on) { uiWindowSetMargined(uiWindow(ZAM_UI(w)), on); }
void zam_ui_window_set_size(int64_t w, int64_t width, int64_t height) { uiWindowSetContentSize(uiWindow(ZAM_UI(w)), (int)width, (int)height); }
// ---- 布局 ----
int64_t zam_ui_box(int vertical) { zam_ui_need(); return ZAM_UI_ID(vertical ? uiNewVerticalBox() : uiNewHorizontalBox()); }
void zam_ui_box_add(int64_t b, int64_t child, int stretchy) { uiBoxAppend(uiBox(ZAM_UI(b)), uiControl(ZAM_UI(child)), stretchy); }
void zam_ui_box_set_padded(int64_t b, int on) { uiBoxSetPadded(uiBox(ZAM_UI(b)), on); }
int64_t zam_ui_group(const char *title) { zam_ui_need(); return ZAM_UI_ID(uiNewGroup(title)); }
void zam_ui_group_set_child(int64_t g, int64_t child) { uiGroupSetChild(uiGroup(ZAM_UI(g)), uiControl(ZAM_UI(child))); }
void zam_ui_group_set_margined(int64_t g, int on) { uiGroupSetMargined(uiGroup(ZAM_UI(g)), on); }
int64_t zam_ui_tabs(void) { zam_ui_need(); return ZAM_UI_ID(uiNewTab()); }
void zam_ui_tabs_add(int64_t t, const char *name, int64_t child) {
    uiTab *tab = uiTab(ZAM_UI(t));
    uiTabAppend(tab, name, uiControl(ZAM_UI(child)));
    uiTabSetMargined(tab, uiTabNumPages(tab) - 1, 1);
}
int64_t zam_ui_form(void) { zam_ui_need(); uiForm *f = uiNewForm(); uiFormSetPadded(f, 1); return ZAM_UI_ID(f); }
void zam_ui_form_add(int64_t f, const char *label, int64_t child, int stretchy) { uiFormAppend(uiForm(ZAM_UI(f)), label, uiControl(ZAM_UI(child)), stretchy); }
int64_t zam_ui_grid(void) { zam_ui_need(); uiGrid *g = uiNewGrid(); uiGridSetPadded(g, 1); return ZAM_UI_ID(g); }
void zam_ui_grid_add(int64_t g, int64_t child, int64_t column, int64_t row, int64_t width, int64_t height, int expand) {
    uiGridAppend(uiGrid(ZAM_UI(g)), uiControl(ZAM_UI(child)), (int)column, (int)row, (int)width, (int)height,
        expand, uiAlignFill, expand, uiAlignFill);
}
int64_t zam_ui_separator(int vertical) { zam_ui_need(); return ZAM_UI_ID(vertical ? uiNewVerticalSeparator() : uiNewHorizontalSeparator()); }
// ---- 文字与按钮 ----
int64_t zam_ui_label(const char *text) { zam_ui_need(); return ZAM_UI_ID(uiNewLabel(text)); }
void zam_ui_label_set_text(int64_t l, const char *text) { uiLabelSetText(uiLabel(ZAM_UI(l)), text); }
const char *zam_ui_label_text(int64_t l) { return zam_ui_keep(uiLabelText(uiLabel(ZAM_UI(l)))); }
int64_t zam_ui_button(const char *text) { zam_ui_need(); return ZAM_UI_ID(uiNewButton(text)); }
void zam_ui_button_set_text(int64_t b, const char *text) { uiButtonSetText(uiButton(ZAM_UI(b)), text); }
const char *zam_ui_button_text(int64_t b) { return zam_ui_keep(uiButtonText(uiButton(ZAM_UI(b)))); }
// 单行输入框：kind 0 普通，1 密码，2 搜索。
int64_t zam_ui_entry(int64_t kind) {
    zam_ui_need();
    return ZAM_UI_ID(kind == 1 ? uiNewPasswordEntry() : kind == 2 ? uiNewSearchEntry() : uiNewEntry());
}
const char *zam_ui_entry_text(int64_t e) { return zam_ui_keep(uiEntryText(uiEntry(ZAM_UI(e)))); }
void zam_ui_entry_set_text(int64_t e, const char *text) { uiEntrySetText(uiEntry(ZAM_UI(e)), text); }
void zam_ui_entry_set_read_only(int64_t e, int on) { uiEntrySetReadOnly(uiEntry(ZAM_UI(e)), on); }
int64_t zam_ui_text_area(int wrap) { zam_ui_need(); return ZAM_UI_ID(wrap ? uiNewMultilineEntry() : uiNewNonWrappingMultilineEntry()); }
const char *zam_ui_text_area_text(int64_t e) { return zam_ui_keep(uiMultilineEntryText(uiMultilineEntry(ZAM_UI(e)))); }
void zam_ui_text_area_set_text(int64_t e, const char *text) { uiMultilineEntrySetText(uiMultilineEntry(ZAM_UI(e)), text); }
void zam_ui_text_area_append(int64_t e, const char *text) { uiMultilineEntryAppend(uiMultilineEntry(ZAM_UI(e)), text); }
void zam_ui_text_area_set_read_only(int64_t e, int on) { uiMultilineEntrySetReadOnly(uiMultilineEntry(ZAM_UI(e)), on); }
// ---- 选择 ----
int64_t zam_ui_checkbox(const char *text) { zam_ui_need(); return ZAM_UI_ID(uiNewCheckbox(text)); }
int zam_ui_checkbox_checked(int64_t c) { return uiCheckboxChecked(uiCheckbox(ZAM_UI(c))); }
void zam_ui_checkbox_set_checked(int64_t c, int on) { uiCheckboxSetChecked(uiCheckbox(ZAM_UI(c)), on); }
int64_t zam_ui_slider(int64_t low, int64_t high) { zam_ui_need(); return ZAM_UI_ID(uiNewSlider((int)low, (int)high)); }
int64_t zam_ui_slider_value(int64_t s) { return uiSliderValue(uiSlider(ZAM_UI(s))); }
void zam_ui_slider_set_value(int64_t s, int64_t v) { uiSliderSetValue(uiSlider(ZAM_UI(s)), (int)v); }
int64_t zam_ui_spinbox(int64_t low, int64_t high) { zam_ui_need(); return ZAM_UI_ID(uiNewSpinbox((int)low, (int)high)); }
int64_t zam_ui_spinbox_value(int64_t s) { return uiSpinboxValue(uiSpinbox(ZAM_UI(s))); }
void zam_ui_spinbox_set_value(int64_t s, int64_t v) { uiSpinboxSetValue(uiSpinbox(ZAM_UI(s)), (int)v); }
int64_t zam_ui_progress(void) { zam_ui_need(); return ZAM_UI_ID(uiNewProgressBar()); }
void zam_ui_progress_set_value(int64_t p, int64_t v) { uiProgressBarSetValue(uiProgressBar(ZAM_UI(p)), (int)v); }
int64_t zam_ui_progress_value(int64_t p) { return uiProgressBarValue(uiProgressBar(ZAM_UI(p))); }
int64_t zam_ui_choice(void) { zam_ui_need(); return ZAM_UI_ID(uiNewCombobox()); }
void zam_ui_choice_add(int64_t c, const char *text) { uiComboboxAppend(uiCombobox(ZAM_UI(c)), text); }
int64_t zam_ui_choice_selected(int64_t c) { return uiComboboxSelected(uiCombobox(ZAM_UI(c))); }
void zam_ui_choice_set_selected(int64_t c, int64_t i) { uiComboboxSetSelected(uiCombobox(ZAM_UI(c)), (int)i); }
// 表格：只读的文字表格。数据在 C 里存一份（行 × 列的字符串），libui 的模型回调从这里读。
// handler 是第一个字段，所以模型回调拿到的 handler 指针就是整个结构体。表格的数据跟程序一样长寿。
// 程序自己改表格（换数据、set_selected）时不通知：libui 在 GTK 上会发选中变化的信号，在 Windows 上不会，
// 这里统一成只有用户选中别的行才调用 on_change。界面只在一个线程里用，所以一个全局标记就够了。
static int zam_ui_table_quiet;
typedef struct zam_ui_table_data {
    uiTableModelHandler handler;
    uiTableModel *model;
    uiTable *table;
    int columns, rows;
    char **cells;
} zam_ui_table_data;
static int zam_ui_table_columns(uiTableModelHandler *mh, uiTableModel *m) { (void)m; return ((zam_ui_table_data *)mh)->columns; }
static uiTableValueType zam_ui_table_type(uiTableModelHandler *mh, uiTableModel *m, int column) { (void)mh; (void)m; (void)column; return uiTableValueTypeString; }
static int zam_ui_table_rows(uiTableModelHandler *mh, uiTableModel *m) { (void)m; return ((zam_ui_table_data *)mh)->rows; }
static uiTableValue *zam_ui_table_value(uiTableModelHandler *mh, uiTableModel *m, int row, int column) {
    zam_ui_table_data *t = (zam_ui_table_data *)mh;
    const char *text = "";
    (void)m;
    if (row >= 0 && row < t->rows && column >= 0 && column < t->columns && t->cells[row * t->columns + column]) text = t->cells[row * t->columns + column];
    return uiNewTableValueString(text);
}
static void zam_ui_table_edit(uiTableModelHandler *mh, uiTableModel *m, int row, int column, const uiTableValue *v) { (void)mh; (void)m; (void)row; (void)column; (void)v; }
// 新建表格，返回数据的句柄；控件本身用 zam_ui_table_control 取。
int64_t zam_ui_table(int64_t columns) {
    zam_ui_table_data *t;
    uiTableParams params;
    zam_ui_need();
    t = (zam_ui_table_data *)calloc(1, sizeof *t);
    if (!t) { zam_panic_one("allocation failed"); exit(1); }
    t->handler.NumColumns = zam_ui_table_columns;
    t->handler.ColumnType = zam_ui_table_type;
    t->handler.NumRows = zam_ui_table_rows;
    t->handler.CellValue = zam_ui_table_value;
    t->handler.SetCellValue = zam_ui_table_edit;
    t->columns = (int)columns;
    t->model = uiNewTableModel(&t->handler);
    params.Model = t->model;
    params.RowBackgroundColorModelColumn = -1;
    t->table = uiNewTable(&params);
    uiTableSetSelectionMode(t->table, uiTableSelectionModeZeroOrOne);
    return ZAM_UI_ID(t);
}
int64_t zam_ui_table_control(int64_t data) { return ZAM_UI_ID(((zam_ui_table_data *)ZAM_UI(data))->table); }
void zam_ui_table_add_column(int64_t data, const char *name, int64_t index) {
    zam_ui_table_data *t = (zam_ui_table_data *)ZAM_UI(data);
    uiTableAppendTextColumn(t->table, name, (int)index, uiTableModelColumnNeverEditable, NULL);
}
// 改成 n 行：多出来的行释放（从末尾往前通知删除），新加的行先是空的（通知插入）。
void zam_ui_table_set_count(int64_t data, int64_t n) {
    zam_ui_table_data *t = (zam_ui_table_data *)ZAM_UI(data);
    int i, old = t->rows;
    zam_ui_table_quiet = 1;
    if (n < old) {
        for (i = old - 1; i >= (int)n; i--) {
            int c;
            for (c = 0; c < t->columns; c++) { free(t->cells[i * t->columns + c]); t->cells[i * t->columns + c] = NULL; }
            t->rows = i;
            uiTableModelRowDeleted(t->model, i);
        }
        zam_ui_table_quiet = 0;
        return;
    }
    if (n > old) {
        char **cells = (char **)realloc(t->cells, (size_t)n * (size_t)(t->columns ? t->columns : 1) * sizeof *cells);
        if (!cells) { zam_panic_one("allocation failed"); exit(1); }
        memset(cells + (size_t)old * t->columns, 0, (size_t)(n - old) * t->columns * sizeof *cells);
        t->cells = cells;
        for (i = old; i < (int)n; i++) { t->rows = i + 1; uiTableModelRowInserted(t->model, i); }
    }
    zam_ui_table_quiet = 0;
}
void zam_ui_table_set_cell(int64_t data, int64_t row, int64_t column, const char *text) {
    zam_ui_table_data *t = (zam_ui_table_data *)ZAM_UI(data);
    size_t n;
    char *copy;
    if (row < 0 || row >= t->rows || column < 0 || column >= t->columns) return;
    n = strlen(text);
    copy = (char *)malloc(n + 1);
    if (!copy) { zam_panic_one("allocation failed"); exit(1); }
    memcpy(copy, text, n + 1);
    free(t->cells[row * t->columns + column]);
    t->cells[row * t->columns + column] = copy;
}
void zam_ui_table_changed(int64_t data, int64_t row) { zam_ui_table_quiet = 1; uiTableModelRowChanged(((zam_ui_table_data *)ZAM_UI(data))->model, (int)row); zam_ui_table_quiet = 0; }
// 选中第 row 行；row 不在表格范围内（如 -1）时取消选中。不会触发 on_change 的回调。
void zam_ui_table_set_selected(int64_t data, int64_t row) {
    zam_ui_table_data *t = (zam_ui_table_data *)ZAM_UI(data);
    uiTableSelection s;
    int r = (int)row;
    s.NumRows = row >= 0 && row < t->rows ? 1 : 0;
    s.Rows = s.NumRows ? &r : NULL;
    zam_ui_table_quiet = 1;
    uiTableSetSelection(t->table, &s);
    zam_ui_table_quiet = 0;
}
int64_t zam_ui_table_selected(int64_t data) {
    uiTableSelection *s = uiTableGetSelection(((zam_ui_table_data *)ZAM_UI(data))->table);
    int64_t row = s && s->NumRows > 0 ? s->Rows[0] : -1;
    if (s) uiFreeTableSelection(s);
    return row;
}
int64_t zam_ui_radio(void) { zam_ui_need(); return ZAM_UI_ID(uiNewRadioButtons()); }
void zam_ui_radio_add(int64_t r, const char *text) { uiRadioButtonsAppend(uiRadioButtons(ZAM_UI(r)), text); }
int64_t zam_ui_radio_selected(int64_t r) { return uiRadioButtonsSelected(uiRadioButtons(ZAM_UI(r))); }
void zam_ui_radio_set_selected(int64_t r, int64_t i) { uiRadioButtonsSetSelected(uiRadioButtons(ZAM_UI(r)), (int)i); }
// ---- 对话框 ----
void zam_ui_message(int64_t w, const char *title, const char *text, int error) {
    if (error) uiMsgBoxError(uiWindow(ZAM_UI(w)), title, text);
    else uiMsgBox(uiWindow(ZAM_UI(w)), title, text);
}
// 文件对话框：取消时返回空字符串。kind 0 打开文件，1 保存文件，2 打开文件夹。
const char *zam_ui_pick(int64_t w, int64_t kind) {
    uiWindow *window = uiWindow(ZAM_UI(w));
    return zam_ui_keep(kind == 1 ? uiSaveFile(window) : kind == 2 ? uiOpenFolder(window) : uiOpenFile(window));
}
// ---- 程序状态与回调 ----
// 一个回调：Zamak 的函数值（{call, env}，按字节保存）与按 T 生成的调用器。
typedef struct zam_ui_handler {
    struct zam_ui_app *app;
    void (*invoke)(void *function, void *state);
    // 画布的回调多一个参数：画笔或事件的地址。
    void (*event)(void *function, void *state, int64_t arg);
    unsigned char function[2 * sizeof(void *)];
    struct zam_ui_handler *next;
} zam_ui_handler;
typedef struct zam_ui_app {
    void *state;
    zam_ui_handler *handlers;
    int64_t window;
    zam_ui_handler *closing;
} zam_ui_app;
int64_t zam_ui_app_new(void **out) {
    zam_ui_app *app;
    zam_ui_need();
    app = (zam_ui_app *)calloc(1, sizeof *app);
    if (!app) { zam_panic_one("allocation failed"); exit(1); }
    *out = app;
    return 0;
}
// App 离开作用域：释放回调。回调只在 run 里被调用，run 返回后不会再被调用。
void zam_ui_app_release(void *handle) {
    zam_ui_app *app = (zam_ui_app *)handle;
    zam_ui_handler *h, *next;
    if (!app) return;
    for (h = app->handlers; h; h = next) { next = h->next; free(h); }
    free(app);
}
static void zam_ui_call(zam_ui_handler *h) { if (h->app->state) h->invoke(h->function, h->app->state); }
static void zam_ui_on_button(uiButton *b, void *data) { (void)b; zam_ui_call((zam_ui_handler *)data); }
static void zam_ui_on_entry(uiEntry *e, void *data) { (void)e; zam_ui_call((zam_ui_handler *)data); }
static void zam_ui_on_text_area(uiMultilineEntry *e, void *data) { (void)e; zam_ui_call((zam_ui_handler *)data); }
static void zam_ui_on_checkbox(uiCheckbox *c, void *data) { (void)c; zam_ui_call((zam_ui_handler *)data); }
static void zam_ui_on_slider(uiSlider *s, void *data) { (void)s; zam_ui_call((zam_ui_handler *)data); }
static void zam_ui_on_spinbox(uiSpinbox *s, void *data) { (void)s; zam_ui_call((zam_ui_handler *)data); }
static void zam_ui_on_choice(uiCombobox *c, void *data) { (void)c; zam_ui_call((zam_ui_handler *)data); }
static void zam_ui_on_radio(uiRadioButtons *r, void *data) { (void)r; zam_ui_call((zam_ui_handler *)data); }
static int zam_ui_on_timer(void *data) { zam_ui_call((zam_ui_handler *)data); return 1; }
static void zam_ui_on_table(uiTable *t, void *data) { (void)t; if (!zam_ui_table_quiet) zam_ui_call((zam_ui_handler *)data); }
// 回调的种类，与 std/ui.zm 里的常量一致。
enum { ZAM_UI_CLICK, ZAM_UI_TEXT, ZAM_UI_TEXT_AREA, ZAM_UI_TOGGLE, ZAM_UI_SLIDE, ZAM_UI_SPIN, ZAM_UI_CHOOSE, ZAM_UI_RADIO, ZAM_UI_TIMER, ZAM_UI_CLOSE, ZAM_UI_SELECT, ZAM_UI_DRAW, ZAM_UI_MOUSE, ZAM_UI_KEY };
static void zam_ui_connect(void *handle, int64_t kind, int64_t control, const void *function, size_t size, void (*invoke)(void *, void *)) {
    zam_ui_app *app = (zam_ui_app *)handle;
    zam_ui_handler *h = (zam_ui_handler *)calloc(1, sizeof *h);
    void *c = ZAM_UI(control);
    if (!h || size > sizeof h->function) { zam_panic_one("allocation failed"); exit(1); }
    h->app = app; h->invoke = invoke;
    memcpy(h->function, function, size);
    h->next = app->handlers; app->handlers = h;
    switch (kind) {
    case ZAM_UI_CLICK: uiButtonOnClicked(uiButton(c), zam_ui_on_button, h); break;
    case ZAM_UI_TEXT: uiEntryOnChanged(uiEntry(c), zam_ui_on_entry, h); break;
    case ZAM_UI_TEXT_AREA: uiMultilineEntryOnChanged(uiMultilineEntry(c), zam_ui_on_text_area, h); break;
    case ZAM_UI_TOGGLE: uiCheckboxOnToggled(uiCheckbox(c), zam_ui_on_checkbox, h); break;
    case ZAM_UI_SLIDE: uiSliderOnChanged(uiSlider(c), zam_ui_on_slider, h); break;
    case ZAM_UI_SPIN: uiSpinboxOnChanged(uiSpinbox(c), zam_ui_on_spinbox, h); break;
    case ZAM_UI_CHOOSE: uiComboboxOnSelected(uiCombobox(c), zam_ui_on_choice, h); break;
    case ZAM_UI_RADIO: uiRadioButtonsOnSelected(uiRadioButtons(c), zam_ui_on_radio, h); break;
    case ZAM_UI_TIMER: uiTimer((int)control, zam_ui_on_timer, h); break;
    case ZAM_UI_CLOSE: app->closing = h; break;
    case ZAM_UI_SELECT: uiTableOnSelectionChanged(uiTable(c), zam_ui_on_table, h); break;
    }
}
// ---- 画布：用户自己画（libui 的 uiArea）----
// 画的时候有一支“画笔”：当前的绘图上下文与正在拼的路径。回调拿到的 Painter 里存的就是它的地址，
// 只在 on_draw 的回调期间有效。路径画完（fill/stroke）后还留着，可以接着描边；begin 换一条新的。
typedef struct zam_ui_paint {
    uiDrawContext *ctx;
    double width, height;
    uiDrawPath *path;
    int figure;
} zam_ui_paint;
typedef struct zam_ui_canvas_data {
    uiAreaHandler handler;
    uiArea *area;
    zam_ui_handler *draw, *mouse, *key;
} zam_ui_canvas_data;
static void zam_ui_fire(zam_ui_handler *h, void *arg) {
    if (h && h->app->state) h->event(h->function, h->app->state, (int64_t)(intptr_t)arg);
}
static void zam_ui_area_draw(uiAreaHandler *ah, uiArea *a, uiAreaDrawParams *p) {
    zam_ui_canvas_data *c = (zam_ui_canvas_data *)ah;
    zam_ui_paint paint;
    (void)a;
    paint.ctx = p->Context; paint.width = p->AreaWidth; paint.height = p->AreaHeight; paint.path = NULL; paint.figure = 0;
    zam_ui_fire(c->draw, &paint);
    if (paint.path) uiDrawFreePath(paint.path);
}
static void zam_ui_area_mouse(uiAreaHandler *ah, uiArea *a, uiAreaMouseEvent *e) { (void)a; zam_ui_fire(((zam_ui_canvas_data *)ah)->mouse, e); }
static void zam_ui_area_crossed(uiAreaHandler *ah, uiArea *a, int left) { (void)ah; (void)a; (void)left; }
static void zam_ui_area_drag(uiAreaHandler *ah, uiArea *a) { (void)ah; (void)a; }
static int zam_ui_area_key(uiAreaHandler *ah, uiArea *a, uiAreaKeyEvent *e) {
    zam_ui_canvas_data *c = (zam_ui_canvas_data *)ah;
    (void)a;
    if (!c->key) return 0;
    zam_ui_fire(c->key, e);
    return 1;
}
int64_t zam_ui_canvas(void) {
    zam_ui_canvas_data *c;
    zam_ui_need();
    c = (zam_ui_canvas_data *)calloc(1, sizeof *c);
    if (!c) { zam_panic_one("allocation failed"); exit(1); }
    c->handler.Draw = zam_ui_area_draw;
    c->handler.MouseEvent = zam_ui_area_mouse;
    c->handler.MouseCrossed = zam_ui_area_crossed;
    c->handler.DragBroken = zam_ui_area_drag;
    c->handler.KeyEvent = zam_ui_area_key;
    c->area = uiNewArea(&c->handler);
    return ZAM_UI_ID(c);
}
int64_t zam_ui_canvas_control(int64_t c) { return ZAM_UI_ID(((zam_ui_canvas_data *)ZAM_UI(c))->area); }
void zam_ui_canvas_redraw(int64_t c) { uiAreaQueueRedrawAll(((zam_ui_canvas_data *)ZAM_UI(c))->area); }
#define ZAM_PAINT ((zam_ui_paint *)ZAM_UI(p))
double zam_ui_paint_width(int64_t p) { return ZAM_PAINT->width; }
double zam_ui_paint_height(int64_t p) { return ZAM_PAINT->height; }
void zam_ui_paint_begin(int64_t p) {
    if (ZAM_PAINT->path) uiDrawFreePath(ZAM_PAINT->path);
    ZAM_PAINT->path = uiDrawNewPath(uiDrawFillModeWinding);
    ZAM_PAINT->figure = 0;
}
static uiDrawPath *zam_ui_open_path(int64_t p) {
    if (!ZAM_PAINT->path || uiDrawPathEnded(ZAM_PAINT->path)) zam_ui_paint_begin(p);
    return ZAM_PAINT->path;
}
void zam_ui_paint_move_to(int64_t p, double x, double y) { uiDrawPathNewFigure(zam_ui_open_path(p), x, y); ZAM_PAINT->figure = 1; }
void zam_ui_paint_line_to(int64_t p, double x, double y) {
    uiDrawPath *path = zam_ui_open_path(p);
    if (!ZAM_PAINT->figure) { uiDrawPathNewFigure(path, x, y); ZAM_PAINT->figure = 1; } else uiDrawPathLineTo(path, x, y);
}
void zam_ui_paint_curve_to(int64_t p, double c1x, double c1y, double c2x, double c2y, double x, double y) {
    uiDrawPath *path = zam_ui_open_path(p);
    if (!ZAM_PAINT->figure) { uiDrawPathNewFigure(path, c1x, c1y); ZAM_PAINT->figure = 1; }
    uiDrawPathBezierTo(path, c1x, c1y, c2x, c2y, x, y);
}
// 圆弧：角度是弧度，从 x 轴正方向开始、顺时针（屏幕坐标 y 向下）。
void zam_ui_paint_arc(int64_t p, double cx, double cy, double r, double start, double sweep) {
    uiDrawPath *path = zam_ui_open_path(p);
    if (sweep < 0) { start += sweep; sweep = -sweep; }
    if (!ZAM_PAINT->figure) { uiDrawPathNewFigureWithArc(path, cx, cy, r, start, sweep, 0); ZAM_PAINT->figure = 1; }
    else uiDrawPathArcTo(path, cx, cy, r, start, sweep, 0);
}
void zam_ui_paint_rect(int64_t p, double x, double y, double w, double h) { uiDrawPathAddRectangle(zam_ui_open_path(p), x, y, w, h); ZAM_PAINT->figure = 0; }
void zam_ui_paint_close(int64_t p) { if (ZAM_PAINT->figure) { uiDrawPathCloseFigure(zam_ui_open_path(p)); ZAM_PAINT->figure = 0; } }
static uiDrawPath *zam_ui_ended(int64_t p) {
    uiDrawPath *path = zam_ui_open_path(p);
    if (!uiDrawPathEnded(path)) uiDrawPathEnd(path);
    ZAM_PAINT->figure = 0;
    return path;
}
void zam_ui_paint_fill(int64_t p, double r, double g, double b, double a) {
    uiDrawBrush brush;
    memset(&brush, 0, sizeof brush);
    brush.Type = uiDrawBrushTypeSolid; brush.R = r; brush.G = g; brush.B = b; brush.A = a;
    uiDrawFill(ZAM_PAINT->ctx, zam_ui_ended(p), &brush);
}
void zam_ui_paint_stroke(int64_t p, double width, double r, double g, double b, double a) {
    uiDrawBrush brush;
    uiDrawStrokeParams stroke;
    memset(&brush, 0, sizeof brush);
    memset(&stroke, 0, sizeof stroke);
    brush.Type = uiDrawBrushTypeSolid; brush.R = r; brush.G = g; brush.B = b; brush.A = a;
    stroke.Cap = uiDrawLineCapRound; stroke.Join = uiDrawLineJoinRound; stroke.Thickness = width; stroke.MiterLimit = uiDrawDefaultMiterLimit;
    uiDrawStroke(ZAM_PAINT->ctx, zam_ui_ended(p), &brush, &stroke);
}
// 渐变：线性从 (x0, y0) 到 (x1, y1)；径向从圆心 (x0, y0) 向外到半径 radius。两端各一个颜色。
void zam_ui_paint_gradient(int64_t p, int64_t radial, double x0, double y0, double x1, double y1, double radius,
                           double r0, double g0, double b0, double a0, double r1, double g1, double b1, double a1) {
    uiDrawBrush brush;
    uiDrawBrushGradientStop stops[2];
    memset(&brush, 0, sizeof brush);
    stops[0].Pos = 0; stops[0].R = r0; stops[0].G = g0; stops[0].B = b0; stops[0].A = a0;
    stops[1].Pos = 1; stops[1].R = r1; stops[1].G = g1; stops[1].B = b1; stops[1].A = a1;
    brush.Type = radial ? uiDrawBrushTypeRadialGradient : uiDrawBrushTypeLinearGradient;
    brush.X0 = x0; brush.Y0 = y0;
    brush.X1 = radial ? x0 : x1; brush.Y1 = radial ? y0 : y1;
    brush.OuterRadius = radius;
    brush.Stops = stops; brush.NumStops = 2;
    uiDrawFill(ZAM_PAINT->ctx, zam_ui_ended(p), &brush);
}
void zam_ui_paint_clip(int64_t p) { uiDrawClip(ZAM_PAINT->ctx, zam_ui_ended(p)); }
void zam_ui_paint_save(int64_t p) { uiDrawSave(ZAM_PAINT->ctx); }
void zam_ui_paint_restore(int64_t p) { uiDrawRestore(ZAM_PAINT->ctx); }
// kind：0 平移 (a, b)，1 绕原点旋转 a 弧度，2 缩放 (a, b)。
void zam_ui_paint_transform(int64_t p, int64_t kind, double a, double b) {
    uiDrawMatrix m;
    uiDrawMatrixSetIdentity(&m);
    if (kind == 0) uiDrawMatrixTranslate(&m, a, b);
    else if (kind == 1) uiDrawMatrixRotate(&m, 0, 0, a);
    else uiDrawMatrixScale(&m, 0, 0, a, b);
    uiDrawTransform(ZAM_PAINT->ctx, &m);
}
// 文字：系统界面字体，size 是磅值。width > 0 时在这个宽度里按 align（0 左、1 中、2 右）排版并换行。
// 系统字体只查一次：uiLoadControlFont 在 Windows 上每次都重新扫描系统字体集合，一次要几十毫秒。
static uiFontDescriptor zam_ui_font;
static int zam_ui_font_loaded;
static uiDrawTextLayout *zam_ui_layout(const char *text, double size, int bold, double width, int64_t align,
                                       double r, double g, double b, double a) {
    uiFontDescriptor font;
    uiDrawTextLayoutParams params;
    uiAttributedString *s = uiNewAttributedString(text);
    uiDrawTextLayout *layout;
    if (!zam_ui_font_loaded) { uiLoadControlFont(&zam_ui_font); zam_ui_font_loaded = 1; }
    font = zam_ui_font;
    font.Size = size;
    font.Weight = bold ? uiTextWeightBold : uiTextWeightNormal;
    if (strlen(text) > 0) uiAttributedStringSetAttribute(s, uiNewColorAttribute(r, g, b, a), 0, strlen(text));
    params.String = s; params.DefaultFont = &font;
    params.Width = width > 0 ? width : 1e6;
    params.Align = align == 1 ? uiDrawTextAlignCenter : align == 2 ? uiDrawTextAlignRight : uiDrawTextAlignLeft;
    layout = uiDrawNewTextLayout(&params);
    uiFreeAttributedString(s);
    return layout;
}
void zam_ui_paint_text(int64_t p, double x, double y, const char *text, double size, int bold, double width, int64_t align,
                       double r, double g, double b, double a) {
    uiDrawTextLayout *layout = zam_ui_layout(text, size, bold, width, align, r, g, b, a);
    uiDrawText(ZAM_PAINT->ctx, layout, x, y);
    uiDrawFreeTextLayout(layout);
}
// 文字排出来的宽（which 为 0）或高（which 为 1）。
double zam_ui_text_extent(const char *text, double size, int bold, double width, int64_t which) {
    double w = 0, h = 0;
    uiDrawTextLayout *layout = zam_ui_layout(text, size, bold, width, 0, 0, 0, 0, 1);
    uiDrawTextLayoutExtents(layout, &w, &h);
    uiDrawFreeTextLayout(layout);
    return which ? h : w;
}
#define ZAM_MOUSE ((uiAreaMouseEvent *)ZAM_UI(e))
double zam_ui_mouse_x(int64_t e) { return ZAM_MOUSE->X; }
double zam_ui_mouse_y(int64_t e) { return ZAM_MOUSE->Y; }
int64_t zam_ui_mouse_down(int64_t e) { return ZAM_MOUSE->Down; }
int64_t zam_ui_mouse_up(int64_t e) { return ZAM_MOUSE->Up; }
int64_t zam_ui_mouse_count(int64_t e) { return ZAM_MOUSE->Count; }
int zam_ui_mouse_held(int64_t e) { return ZAM_MOUSE->Held1To64 != 0; }
#define ZAM_KEY ((uiAreaKeyEvent *)ZAM_UI(e))
int zam_ui_key_up(int64_t e) { return ZAM_KEY->Up; }
// 按下的键：可打印字符就是这个字符，其他键是名字（"Escape"、"Left"、"F1"……），只按了修饰键时是空串。
const char *zam_ui_key_name(int64_t e) {
    static ZAM_THREAD char one[2];
    static const char *names[] = { "", "Escape", "Insert", "Delete", "Home", "End", "PageUp", "PageDown", "Up", "Down", "Left", "Right",
        "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12" };
    if (ZAM_KEY->Key) { one[0] = ZAM_KEY->Key; one[1] = 0; return one; }
    if (ZAM_KEY->ExtKey > 0 && ZAM_KEY->ExtKey < (int)(sizeof names / sizeof names[0])) return names[ZAM_KEY->ExtKey];
    return "";
}
// 按住的修饰键：1 Ctrl、2 Alt、4 Shift、8 Super（Windows 键 / Command），可以相加。
int64_t zam_ui_key_modifiers(int64_t e) { return ZAM_KEY->Modifiers; }
int64_t zam_ui_mouse_modifiers(int64_t e) { return ZAM_MOUSE->Modifiers; }
static void zam_ui_connect_event(void *handle, int64_t kind, int64_t canvas, const void *function, size_t size, void (*event)(void *, void *, int64_t)) {
    zam_ui_app *app = (zam_ui_app *)handle;
    zam_ui_canvas_data *c = (zam_ui_canvas_data *)ZAM_UI(canvas);
    zam_ui_handler *h = (zam_ui_handler *)calloc(1, sizeof *h);
    if (!h || size > sizeof h->function) { zam_panic_one("allocation failed"); exit(1); }
    h->app = app; h->event = event;
    memcpy(h->function, function, size);
    h->next = app->handlers; app->handlers = h;
    if (kind == ZAM_UI_DRAW) c->draw = h;
    else if (kind == ZAM_UI_MOUSE) c->mouse = h;
    else c->key = h;
}
// 关窗口就结束：先调用 on_close 的回调（如果有），再退出主循环。窗口由 libui 销毁。
static int zam_ui_on_closing(uiWindow *w, void *data) {
    zam_ui_app *app = (zam_ui_app *)data;
    (void)w;
    if (app->closing) zam_ui_call(app->closing);
    app->window = 0;
    uiQuit();
    return 1;
}
static int zam_ui_on_should_quit(void *data) {
    zam_ui_app *app = (zam_ui_app *)data;
    if (app->window) { uiControlDestroy(uiControl(ZAM_UI(app->window))); app->window = 0; }
    return 1;
}
// 显示窗口并运行主循环，直到窗口关闭或调用 ui.quit()。state 指向 run 栈帧里的状态。
static void zam_ui_run(void *handle, int64_t window, void *state) {
    zam_ui_app *app = (zam_ui_app *)handle;
    app->state = state;
    app->window = window;
    uiWindowOnClosing(uiWindow(ZAM_UI(window)), zam_ui_on_closing, app);
    uiOnShouldQuit(zam_ui_on_should_quit, app);
    uiControlShow(uiControl(ZAM_UI(window)));
    uiMain();
    // ui.quit() 结束的主循环：窗口还在，销毁它。
    if (app->window) { uiControlDestroy(uiControl(ZAM_UI(app->window))); app->window = 0; }
    app->state = NULL;
}
void zam_ui_quit(void) { uiQuit(); }
"#;
/// 只在用到某个标准库模块时才需要的 C 辅助代码：（模块前缀, 代码）。
const MODULE_PRELUDES: &[(&str, &str)] = &[
    ("std/hashlib:", HASH_PRELUDE),
    ("std/base64:", BASE64_PRELUDE),
    ("std/_process:", PROCESS_PRELUDE),
    ("std/_re:", REGEX_PRELUDE),
];
const STD_PRELUDE: &str = r#"#ifdef _WIN32
__declspec(dllimport) wchar_t *__stdcall GetCommandLineW(void);
__declspec(dllimport) int __stdcall MultiByteToWideChar(unsigned int, unsigned long, const char *, int, wchar_t *, int);
__declspec(dllimport) int __stdcall WideCharToMultiByte(unsigned int, unsigned long, const wchar_t *, int, char *, int, const char *, int *);
__declspec(dllimport) int __stdcall ReadConsoleW(void *, void *, unsigned long, unsigned long *, void *);
__declspec(dllimport) void __stdcall GetSystemTimePreciseAsFileTime(void *);
__declspec(dllimport) int __stdcall QueryPerformanceCounter(long long *);
__declspec(dllimport) int __stdcall QueryPerformanceFrequency(long long *);
__declspec(dllimport) void __stdcall Sleep(unsigned long);
__declspec(dllimport) int __stdcall MoveFileExW(const wchar_t *, const wchar_t *, unsigned long);
#endif
#ifdef _WIN32
#include <direct.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <wchar.h>
#else
#include <sys/types.h>
#include <sys/stat.h>
#include <dirent.h>
#include <unistd.h>
#endif
// std/random 的状态：每个线程一份（没调用 random.seed 的线程各自取种子），定义在入口模块的目标文件里。
extern ZAM_THREAD uint64_t zam_random_state[4];
extern ZAM_THREAD int zam_random_seeded;
// std/time：墙上时间（1970 年以来的秒）与单调时间（只用来量间隔）。
static double zam_time_now(void) {
#ifdef _WIN32
    uint64_t ticks; GetSystemTimePreciseAsFileTime(&ticks);
    return (double)(ticks - 116444736000000000ULL) / 1e7;
#else
    struct timespec ts; clock_gettime(CLOCK_REALTIME, &ts);
    return (double)ts.tv_sec + (double)ts.tv_nsec / 1e9;
#endif
}
static double zam_time_monotonic(void) {
#ifdef _WIN32
    long long count, frequency; QueryPerformanceCounter(&count); QueryPerformanceFrequency(&frequency);
    return (double)(count / frequency) + (double)(count % frequency) / (double)frequency;
#else
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec + (double)ts.tv_nsec / 1e9;
#endif
}
// 公历日期与 1970-01-01 起的天数互换（Howard Hinnant 的算法，对任何年份都成立）。
static void zam_civil(int64_t days, int64_t *y, int64_t *m, int64_t *d) {
    int64_t z = days + 719468, era = (z >= 0 ? z : z - 146096) / 146097;
    int64_t doe = z - era * 146097, yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    int64_t doy = doe - (365 * yoe + yoe / 4 - yoe / 100), mp = (5 * doy + 2) / 153;
    *d = doy - (153 * mp + 2) / 5 + 1; *m = mp < 10 ? mp + 3 : mp - 9; *y = yoe + era * 400 + (*m <= 2);
}
static int64_t zam_days(int64_t y, int64_t m, int64_t d) {
    int64_t era, yoe, doy, doe; y -= m <= 2;
    era = (y >= 0 ? y : y - 399) / 400; yoe = y - era * 400;
    doy = (153 * (m > 2 ? m - 3 : m + 9) + 2) / 5 + d - 1; doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    return era * 146097 + doe - 719468;
}
static void zam_time_check(double t) {
    if (t != t || t > 2.5e14 || t < -2.5e14) { zam_panic_one("time value out of range"); exit(1); }
}
// t 秒（向下取整）加上 offset 秒的时区偏移，写成 `2026-10-16T03:04:05`，再接上 zone。
static S zam_time_text(double t, int64_t offset, const char *zone) {
    char text[64]; int64_t secs, days, rest, y, m, d; int n;
    zam_time_check(t);
    secs = (int64_t)floor(t) + offset;
    days = (secs >= 0 ? secs : secs - 86399) / 86400; rest = secs - days * 86400;
    zam_civil(days, &y, &m, &d);
    n = snprintf(text, sizeof text, "%04" PRId64 "-%02" PRId64 "-%02" PRId64 "T%02" PRId64 ":%02" PRId64 ":%02" PRId64 "%s",
        y, m, d, rest / 3600, rest / 60 % 60, rest % 60, zone);
    return own((const unsigned char *)text, (size_t)n);
}
// 本地时区在 t 时刻相对 UTC 的偏移（秒），由 localtime 的日期反推，各平台都能用。
// localtime 的结果放在共用的静态缓冲区里，几个线程同时调用会互相覆盖：用各平台可重入的版本。
static int64_t zam_local_offset(double t) {
    time_t when = (time_t)floor(t); struct tm local;
#if defined(_WIN32)
    if (localtime_s(&local, &when) != 0) return 0;
#else
    if (!localtime_r(&when, &local)) return 0;
#endif
    return zam_days(local.tm_year + 1900, local.tm_mon + 1, local.tm_mday) * 86400
        + local.tm_hour * 3600 + local.tm_min * 60 + local.tm_sec - (int64_t)when;
}
// xoshiro256**：快、周期长、统计性质好，不适合密码学用途。种子用 splitmix64 展开。
static uint64_t zam_splitmix(uint64_t *x) {
    uint64_t z = (*x += 0x9E3779B97F4A7C15ULL);
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL; z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
    return z ^ (z >> 31);
}
static void zam_random_seed(uint64_t seed) {
    int i; for (i = 0; i < 4; i++) zam_random_state[i] = zam_splitmix(&seed);
    zam_random_seeded = 1;
}
static uint64_t zam_rotl(uint64_t x, int k) { return (x << k) | (x >> (64 - k)); }
static uint64_t zam_random_next(void) {
    uint64_t *s = zam_random_state, result, t;
    if (!zam_random_seeded) {
        // 没调用 random.seed 时用当前时间、单调时钟与栈地址混出种子，每次运行都不同。
        int marker; double now = zam_time_now(), mono = zam_time_monotonic(); uint64_t a, b;
        memcpy(&a, &now, sizeof a); memcpy(&b, &mono, sizeof b);
        zam_random_seed(a ^ (b * 0x9E3779B97F4A7C15ULL) ^ (uint64_t)(uintptr_t)&marker);
    }
    result = zam_rotl(s[1] * 5, 7) * 9; t = s[1] << 17;
    s[2] ^= s[0]; s[3] ^= s[1]; s[1] ^= s[2]; s[0] ^= s[3]; s[2] ^= t; s[3] = zam_rotl(s[3], 45);
    return result;
}
// [0, range) 里均匀的整数；range 为 0 表示整个 64 位范围。拒绝采样去掉取模偏差。
static uint64_t zam_random_below(uint64_t range) {
    uint64_t limit, r;
    if (!range) return zam_random_next();
    limit = (0 - range) % range;
    do r = zam_random_next(); while (r < limit);
    return r % range;
}
static double zam_random_unit(void) { return (double)(zam_random_next() >> 11) * (1.0 / 9007199254740992.0); }
extern int zam_argc;
extern char **zam_argv;
static void zam_alloc_failed(void) { zam_panic_one("allocation failed"); exit(1); }
// std/fs 的路径：UTF-8 里有 NUL 时返回 NULL（这样的路径不存在）。Windows 转成 UTF-16。
#ifdef _WIN32
static wchar_t *zam_path(S *path) {
    int n = 0; size_t i; wchar_t *wide;
    for (i = 0; i < path->len; i++) if (!path->data[i]) return NULL;
    if (path->len > INT_MAX / 2) return NULL;
    if (path->len) {
        n = MultiByteToWideChar(65001, 8, (const char*)path->data, (int)path->len, NULL, 0);
        if (n <= 0) return NULL;
    }
    wide = (wchar_t*)malloc(((size_t)n + 1) * sizeof(wchar_t));
    if (!wide) zam_alloc_failed();
    if (n) MultiByteToWideChar(65001, 8, (const char*)path->data, (int)path->len, wide, n);
    wide[n] = 0;
    return wide;
}
#else
static char *zam_path(S *path) {
    size_t i; char *name;
    for (i = 0; i < path->len; i++) if (!path->data[i]) return NULL;
    name = (char*)malloc(path->len + 1);
    if (!name) zam_alloc_failed();
    if (path->len) memcpy(name, path->data, path->len);
    name[path->len] = 0;
    return name;
}
#endif
// 0：不存在（或路径无效）；1：普通文件；2：目录；3：其他。size 可为 NULL。
static int zam_kind(S *path, int64_t *size) {
    int kind;
#ifdef _WIN32
    struct _stat64 st; wchar_t *name = zam_path(path);
    if (!name) return 0;
    kind = _wstat64(name, &st) != 0 ? 0 : (st.st_mode & _S_IFDIR) ? 2 : (st.st_mode & _S_IFREG) ? 1 : 3;
    free(name);
#else
    struct stat st; char *name = zam_path(path);
    if (!name) return 0;
    kind = stat(name, &st) != 0 ? 0 : S_ISDIR(st.st_mode) ? 2 : S_ISREG(st.st_mode) ? 1 : 3;
    free(name);
#endif
    if (kind && size) *size = (int64_t)st.st_size;
    return kind;
}
static int zam_mkdir_one(S *path) {
    int ok;
#ifdef _WIN32
    wchar_t *name = zam_path(path);
    if (!name) return 0;
    ok = _wmkdir(name) == 0;
#else
    char *name = zam_path(path);
    if (!name) return 0;
    ok = mkdir(name, 0777) == 0;
#endif
    free(name);
    return ok;
}
static int zam_separator(unsigned char c) {
#ifdef _WIN32
    return c == '/' || c == '\\';
#else
    return c == '/';
#endif
}
static int zam_name_order(const void *a, const void *b) { return string_compare((S*)a, (S*)b); }
// 路径最后一段的起点；末尾的分隔符不算（`a/b/` 的最后一段是 `b`）。
static void zam_last_part(S *p, size_t *start, size_t *end) {
    size_t e = p->len;
    while (e > 1 && zam_separator(p->data[e - 1])) e--;
    *end = e;
    while (e > 0 && !zam_separator(p->data[e - 1])) e--;
    *start = e;
}
static int zam_space(unsigned char c) { return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\v' || c == '\f'; }
static void zam_string_bounds(void) { zam_panic_one("string index out of bounds"); exit(1); }
// 从 p 开始的一个 UTF-8 字符：返回字节数并写出码点；不合法（截断、过长编码、代理区、超出范围）时返回 0。
static size_t zam_utf8_next(const unsigned char *p, size_t n, int64_t *code) {
    size_t len, i; uint32_t c;
    if (!n) return 0;
    if (p[0] < 0x80) { *code = p[0]; return 1; }
    if ((p[0] & 0xE0) == 0xC0) { len = 2; c = p[0] & 0x1F; }
    else if ((p[0] & 0xF0) == 0xE0) { len = 3; c = p[0] & 0x0F; }
    else if ((p[0] & 0xF8) == 0xF0) { len = 4; c = p[0] & 0x07; }
    else return 0;
    if (n < len) return 0;
    for (i = 1; i < len; i++) { if ((p[i] & 0xC0) != 0x80) return 0; c = (c << 6) | (p[i] & 0x3F); }
    if ((len == 2 && c < 0x80) || (len == 3 && c < 0x800) || (len == 4 && c < 0x10000) || c > 0x10FFFF || (c >= 0xD800 && c <= 0xDFFF)) return 0;
    *code = c;
    return len;
}
// 码点编码成 UTF-8，返回字节数；不是合法码点时返回 0。
static size_t zam_utf8_put(int64_t c, unsigned char *out) {
    if (c < 0 || c > 0x10FFFF || (c >= 0xD800 && c <= 0xDFFF)) return 0;
    if (c < 0x80) { out[0] = (unsigned char)c; return 1; }
    if (c < 0x800) { out[0] = (unsigned char)(0xC0 | (c >> 6)); out[1] = (unsigned char)(0x80 | (c & 0x3F)); return 2; }
    if (c < 0x10000) { out[0] = (unsigned char)(0xE0 | (c >> 12)); out[1] = (unsigned char)(0x80 | ((c >> 6) & 0x3F)); out[2] = (unsigned char)(0x80 | (c & 0x3F)); return 3; }
    out[0] = (unsigned char)(0xF0 | (c >> 18)); out[1] = (unsigned char)(0x80 | ((c >> 12) & 0x3F)); out[2] = (unsigned char)(0x80 | ((c >> 6) & 0x3F)); out[3] = (unsigned char)(0x80 | (c & 0x3F));
    return 4;
}
// 按字符数（不计 UTF-8 续字节）补空格到 width；left 为 1 时空格加在左边。
static S zam_pad(S *s, int64_t width, int left) {
    size_t chars = 0, i, extra; S out; out.chars = 0;
    for (i = 0; i < s->len; i++) if ((s->data[i] & 0xC0) != 0x80) chars++;
    if (width <= 0 || (uint64_t)width <= chars) return copy(s);
    extra = (size_t)width - chars;
    if (extra > SIZE_MAX - s->len) zam_alloc_failed();
    out.len = s->len + extra;
    out.data = (unsigned char*)malloc(out.len);
    if (!out.data) zam_alloc_failed();
    memset(out.data + (left ? 0 : s->len), ' ', extra);
    if (s->len) memcpy(out.data + (left ? extra : 0), s->data, s->len);
    return out;
}
// 用 fill（正好一个字符，可以是多字节的 UTF-8）补到 width 个字符：where 为 0 时补在右边（ljust），
// 1 时补在左边（rjust），2 时两边补（center，多出的一个按 Python 的规则放在左边或右边）。
static S zam_pad_with(S *s, int64_t width, int where, S *fill) {
    size_t chars = 0, i, extra, left, at = 0; S out; out.chars = 0;
    if (zam_chars(fill) != 1) { zam_panic_one("the fill character must be exactly one character"); exit(1); }
    for (i = 0; i < s->len; i++) if ((s->data[i] & 0xC0) != 0x80) chars++;
    if (width <= 0 || (uint64_t)width <= chars) return copy(s);
    extra = (size_t)width - chars;
    left = where == 1 ? extra : where == 2 ? extra / 2 + (extra & (size_t)width & 1) : 0;
    if (extra > (SIZE_MAX - s->len) / fill->len) zam_alloc_failed();
    out.len = s->len + extra * fill->len;
    out.data = (unsigned char*)malloc(out.len);
    if (!out.data) zam_alloc_failed();
    for (i = 0; i < left; i++, at += fill->len) memcpy(out.data + at, fill->data, fill->len);
    if (s->len) memcpy(out.data + at, s->data, s->len);
    at += s->len;
    for (i = left; i < extra; i++, at += fill->len) memcpy(out.data + at, fill->data, fill->len);
    return out;
}
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
// 环境变量按 UTF-8 读；Windows 用 _wgetenv，非 ASCII 的名字和值才不会按本地代码页错读。
static int zam_getenv(S *name, S *out) {
    size_t i;
    if (!name->len) return 0;
    for (i = 0; i < name->len; i++) if (!name->data[i]) return 0;
#ifdef _WIN32
    {
        int n; wchar_t *wide, *value; char *text;
        if (name->len > INT_MAX / 2) return 0;
        n = MultiByteToWideChar(65001, 8, (const char*)name->data, (int)name->len, NULL, 0);
        if (n <= 0) return 0;
        wide = (wchar_t*)malloc(((size_t)n + 1) * sizeof(wchar_t));
        if (!wide) zam_alloc_failed();
        MultiByteToWideChar(65001, 8, (const char*)name->data, (int)name->len, wide, n);
        wide[n] = 0;
        value = _wgetenv(wide);
        free(wide);
        if (!value) return 0;
        n = WideCharToMultiByte(65001, 0, value, -1, NULL, 0, NULL, NULL);
        if (n <= 0) return 0;
        text = (char*)malloc((size_t)n);
        if (!text) zam_alloc_failed();
        WideCharToMultiByte(65001, 0, value, -1, text, n, NULL, NULL);
        *out = own((const unsigned char*)text, (size_t)n - 1);
        free(text);
    }
#else
    {
        char *key = (char*)malloc(name->len + 1), *value;
        if (!key) zam_alloc_failed();
        memcpy(key, name->data, name->len);
        key[name->len] = 0;
        value = getenv(key);
        free(key);
        if (!value) return 0;
        *out = own((const unsigned char*)value, strlen(value));
    }
#endif
    return 1;
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
    out->data = p; out->len = n; out->chars = 0;
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
        out->data = p; out->len = n; out->chars = 0;
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
        zam_wargs[count].chars = 0;
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
        name: "std/core:abs",
        params: &[Type::Int],
        result: Type::Int,
        fallible: false,
        // 与 `-x` 一样，i64 最小值取绝对值会溢出。
        body: "if (p0 == INT64_MIN) arithmetic_error(); return p0 < 0 ? -p0 : p0;",
    },
    Builtin {
        name: "std/core:abs_f64",
        params: &[Type::Float],
        result: Type::Float,
        fallible: false,
        body: "return fabs(p0);",
    },
    Builtin {
        name: "std/core:min",
        params: &[Type::Int, Type::Int],
        result: Type::Int,
        fallible: false,
        body: "return p0 < p1 ? p0 : p1;",
    },
    Builtin {
        name: "std/core:min_f64",
        params: &[Type::Float, Type::Float],
        result: Type::Float,
        fallible: false,
        // 与 math.min 一样用 fmin：一边是 NaN 时返回另一边。
        body: "return fmin(p0, p1);",
    },
    Builtin {
        name: "std/core:max",
        params: &[Type::Int, Type::Int],
        result: Type::Int,
        fallible: false,
        body: "return p0 > p1 ? p0 : p1;",
    },
    Builtin {
        name: "std/core:max_f64",
        params: &[Type::Float, Type::Float],
        result: Type::Float,
        fallible: false,
        body: "return fmax(p0, p1);",
    },
    Builtin {
        name: "std/core:pow",
        params: &[Type::Int, Type::Int],
        result: Type::Int,
        fallible: false,
        // `a ** b`：按位快速幂，乘法溢出与 `*` 一样报错；整数不能有负指数。
        body: r#"if (p1 < 0) { zam_panic_one("negative exponent in an integer power; use floats such as 2.0 ** -1.0"); exit(1); }
int64_t r = 1;
while (p1 > 0) { if (p1 & 1) r = mul(r, p0); p1 >>= 1; if (p1 > 0) p0 = mul(p0, p0); }
return r;"#,
    },
    Builtin {
        name: "std/core:int",
        params: &[Type::Int],
        result: Type::Int,
        fallible: false,
        body: "return p0;",
    },
    Builtin {
        name: "std/core:float",
        params: &[Type::Float],
        result: Type::Float,
        fallible: false,
        body: "return p0;",
    },
    Builtin {
        name: "std/core:pow_f64",
        params: &[Type::Float, Type::Float],
        result: Type::Float,
        fallible: false,
        body: "return pow(p0, p1);",
    },
    Builtin {
        name: "std/time:time",
        params: &[],
        result: Type::Float,
        fallible: false,
        body: "return zam_time_now();",
    },
    Builtin {
        name: "std/time:monotonic",
        params: &[],
        result: Type::Float,
        fallible: false,
        body: "return zam_time_monotonic();",
    },
    Builtin {
        name: "std/time:sleep",
        params: &[Type::Float],
        result: Type::Unit,
        fallible: false,
        // 负数、NaN 当作 0；超过一天的等待按一天封顶，避免换算时溢出。
        body: r#"double seconds = p0 > 0 ? (p0 < 86400.0 ? p0 : 86400.0) : 0.0;
fflush(stdout);
#ifdef _WIN32
Sleep((unsigned long)(seconds * 1000.0 + 0.5));
#else
{ struct timespec ts; ts.tv_sec = (time_t)seconds; ts.tv_nsec = (long)((seconds - (double)ts.tv_sec) * 1e9);
  while (nanosleep(&ts, &ts) != 0) {} }
#endif"#,
    },
    Builtin {
        name: "std/time:iso_utc",
        params: &[Type::Float],
        result: Type::Owned,
        fallible: false,
        body: r#"return zam_time_text(p0, 0, "Z");"#,
    },
    Builtin {
        // 时间 t 所在时刻本地时区比 UTC 快多少秒（东八区是 28800），std/datetime 用它算本地时间。
        name: "std/time:local_offset",
        params: &[Type::Float],
        result: Type::Int,
        fallible: false,
        body: "zam_time_check(p0); return zam_local_offset(p0);",
    },
    Builtin {
        name: "std/time:iso_local",
        params: &[Type::Float],
        result: Type::Owned,
        fallible: false,
        body: r#"char zone[16]; int64_t offset, minutes;
zam_time_check(p0);
offset = zam_local_offset(p0); minutes = (offset < 0 ? -offset : offset) / 60;
snprintf(zone, sizeof zone, "%c%02d:%02d", offset < 0 ? '-' : '+', (int)(minutes / 60), (int)(minutes % 60));
return zam_time_text(p0, offset, zone);"#,
    },
    Builtin {
        name: "std/random:seed",
        params: &[Type::Int],
        result: Type::Unit,
        fallible: false,
        body: "zam_random_seed((uint64_t)p0);",
    },
    Builtin {
        name: "std/random:random",
        params: &[],
        result: Type::Float,
        fallible: false,
        body: "return zam_random_unit();",
    },
    Builtin {
        name: "std/random:uniform",
        params: &[Type::Float, Type::Float],
        result: Type::Float,
        fallible: false,
        body: "return p0 + (p1 - p0) * zam_random_unit();",
    },
    Builtin {
        name: "std/random:randint",
        params: &[Type::Int, Type::Int],
        result: Type::Int,
        fallible: false,
        // 两端都包含，与 Python 的 random.randint 相同。
        body: r#"if (p0 > p1) { zam_panic_one("random.randint needs a <= b"); exit(1); }
return (int64_t)((uint64_t)p0 + zam_random_below((uint64_t)p1 - (uint64_t)p0 + 1));"#,
    },
    Builtin {
        name: "std/random:randrange",
        params: &[Type::Int, Type::Int],
        result: Type::Int,
        fallible: false,
        // 不含终点，与 for i in a..b 相同。
        body: r#"if (p0 >= p1) { zam_panic_one("random.randrange needs start < stop"); exit(1); }
return (int64_t)((uint64_t)p0 + zam_random_below((uint64_t)p1 - (uint64_t)p0));"#,
    },
    Builtin {
        name: "std/fs:append",
        params: &[Type::Shared, Type::Shared],
        result: Type::Unit,
        fallible: true,
        body: r#"int ok;
FILE *file = zam_open(p0, "ab");
if (!file) return F_void_err(zam_message("cannot open ", p0));
ok = !p1->len || fwrite(p1->data, 1, p1->len, file) == p1->len;
if (fclose(file) != 0) ok = 0;
if (!ok) return F_void_err(zam_message("cannot write ", p0));
return F_void_ok();"#,
    },
    Builtin {
        name: "std/fs:exists",
        params: &[Type::Shared],
        result: Type::Bool,
        fallible: false,
        body: "return zam_kind(p0, NULL) != 0;",
    },
    Builtin {
        name: "std/fs:is_file",
        params: &[Type::Shared],
        result: Type::Bool,
        fallible: false,
        body: "return zam_kind(p0, NULL) == 1;",
    },
    Builtin {
        name: "std/fs:is_dir",
        params: &[Type::Shared],
        result: Type::Bool,
        fallible: false,
        body: "return zam_kind(p0, NULL) == 2;",
    },
    Builtin {
        name: "std/fs:size",
        params: &[Type::Shared],
        result: Type::Int,
        fallible: true,
        body: r#"int64_t size = 0;
if (zam_kind(p0, &size) != 1) return F_int64_t_err(zam_message("not a file: ", p0));
return F_int64_t_ok(size);"#,
    },
    Builtin {
        name: "std/fs:list",
        params: &[Type::Shared],
        result: Type::List(Elem::Str),
        fallible: true,
        // 目录里的名字（不含 . 与 ..），按字节排序，每个平台顺序相同。
        body: r#"Ls out = {0};
#ifdef _WIN32
wchar_t *dir = zam_path(p0), *pattern; size_t n; intptr_t handle; struct _wfinddata64_t data;
if (!dir || zam_kind(p0, NULL) != 2) { free(dir); return F_Ls_err(zam_message("cannot list ", p0)); }
n = wcslen(dir);
pattern = (wchar_t*)malloc((n + 3) * sizeof(wchar_t));
if (!pattern) zam_alloc_failed();
memcpy(pattern, dir, n * sizeof(wchar_t));
if (n && dir[n - 1] != L'/' && dir[n - 1] != L'\\') pattern[n++] = L'\\';
pattern[n++] = L'*'; pattern[n] = 0;
free(dir);
handle = _wfindfirst64(pattern, &data);
free(pattern);
if (handle == -1) return F_Ls_err(zam_message("cannot list ", p0));
do {
    int bytes; char *text;
    if (!wcscmp(data.name, L".") || !wcscmp(data.name, L"..")) continue;
    bytes = WideCharToMultiByte(65001, 0, data.name, -1, NULL, 0, NULL, NULL);
    if (bytes <= 0) continue;
    text = (char*)malloc((size_t)bytes);
    if (!text) zam_alloc_failed();
    WideCharToMultiByte(65001, 0, data.name, -1, text, bytes, NULL, NULL);
    pushLs(&out, own((const unsigned char*)text, (size_t)bytes - 1));
    free(text);
} while (_wfindnext64(handle, &data) == 0);
_findclose(handle);
#else
char *dir = zam_path(p0); DIR *stream; struct dirent *entry;
if (!dir) return F_Ls_err(zam_message("cannot list ", p0));
stream = opendir(dir);
free(dir);
if (!stream) return F_Ls_err(zam_message("cannot list ", p0));
while ((entry = readdir(stream)) != NULL) {
    if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
    pushLs(&out, own((const unsigned char*)entry->d_name, strlen(entry->d_name)));
}
closedir(stream);
#endif
if (out.len > 1) qsort(out.data, out.len, sizeof(S), zam_name_order);
return F_Ls_ok(out);"#,
    },
    Builtin {
        name: "std/fs:mkdir",
        params: &[Type::Shared],
        result: Type::Unit,
        fallible: true,
        // 连带创建上级目录；目录已经存在不算错误（Python 的 os.makedirs(exist_ok=True)）。
        body: r#"size_t i;
if (!p0->len) return F_void_err(zam_message("cannot create directory ", p0));
for (i = 1; i <= p0->len; i++) {
    if (i == p0->len || zam_separator(p0->data[i])) {
        S prefix; prefix.data = p0->data; prefix.len = i; prefix.chars = 0;
        if (zam_kind(&prefix, NULL) == 0) zam_mkdir_one(&prefix);
    }
}
if (zam_kind(p0, NULL) != 2) return F_void_err(zam_message("cannot create directory ", p0));
return F_void_ok();"#,
    },
    Builtin {
        name: "std/fs:remove",
        params: &[Type::Shared],
        result: Type::Unit,
        fallible: true,
        // 删除文件或空目录；目录里还有东西时报错，不会递归删除。
        body: r#"int kind = zam_kind(p0, NULL), ok;
#ifdef _WIN32
wchar_t *name = zam_path(p0);
ok = name && kind && (kind == 2 ? _wrmdir(name) : _wremove(name)) == 0;
#else
char *name = zam_path(p0);
ok = name && kind && remove(name) == 0;
#endif
free(name);
if (!ok) return F_void_err(zam_message(kind == 2 ? "cannot remove directory (is it empty?) " : "cannot remove ", p0));
return F_void_ok();"#,
    },
    Builtin {
        name: "std/fs:rename",
        params: &[Type::Shared, Type::Shared],
        result: Type::Unit,
        fallible: true,
        // 目标是已存在的文件时替换它，各平台一致。
        body: r#"int ok;
#ifdef _WIN32
wchar_t *from = zam_path(p0), *to = zam_path(p1);
ok = from && to && MoveFileExW(from, to, 1 | 2) != 0;
#else
char *from = zam_path(p0), *to = zam_path(p1);
ok = from && to && rename(from, to) == 0;
#endif
free(from); free(to);
if (!ok) return F_void_err(zam_message("cannot rename ", p0));
return F_void_ok();"#,
    },
    Builtin {
        name: "std/path:join",
        params: &[Type::Shared, Type::Shared],
        result: Type::Owned,
        fallible: false,
        // b 是绝对路径（/x、\x、C:x）时就是 b；否则用 / 连接，不重复分隔符。
        body: r#"S out; size_t n = p0->len; out.chars = 0;
int absolute = p1->len && (zam_separator(p1->data[0]) || (p1->len > 1 && p1->data[1] == ':'));
if (absolute || !p0->len) return copy(p1);
if (!p1->len) return copy(p0);
out.len = n + 1 + p1->len; out.data = (unsigned char*)malloc(out.len);
if (!out.data) zam_alloc_failed();
memcpy(out.data, p0->data, n);
if (!zam_separator(p0->data[n - 1])) out.data[n++] = '/';
memcpy(out.data + n, p1->data, p1->len);
out.len = n + p1->len;
return out;"#,
    },
    Builtin {
        name: "std/path:parent",
        params: &[Type::Shared],
        result: Type::Owned,
        fallible: false,
        // `a/b/c.txt` → `a/b`；`c.txt` → 空串；`/x` → `/`。
        body: r#"size_t start, end;
zam_last_part(p0, &start, &end);
while (start > 1 && zam_separator(p0->data[start - 1])) start--;
return own(p0->data, start);"#,
    },
    Builtin {
        name: "std/path:name",
        params: &[Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: r#"size_t start, end;
zam_last_part(p0, &start, &end);
if (end == 1 && zam_separator(p0->data[0])) return own(p0->data, 0);
return own(p0->data + start, end - start);"#,
    },
    Builtin {
        name: "std/path:stem",
        params: &[Type::Shared],
        result: Type::Owned,
        fallible: false,
        // 去掉最后一个扩展名：`archive.tar.gz` → `archive.tar`；以点开头的 `.bashrc` 没有扩展名。
        body: r#"size_t start, end, dot;
zam_last_part(p0, &start, &end);
if (end == 1 && zam_separator(p0->data[0])) return own(p0->data, 0);
for (dot = end; dot > start + 1 && p0->data[dot - 1] != '.'; dot--) {}
if (dot > start + 1) return own(p0->data + start, dot - 1 - start);
return own(p0->data + start, end - start);"#,
    },
    Builtin {
        name: "std/path:suffix",
        params: &[Type::Shared],
        result: Type::Owned,
        fallible: false,
        // 最后一个扩展名，带点：`report.txt` → `.txt`；没有扩展名时是空串。
        body: r#"size_t start, end, dot;
zam_last_part(p0, &start, &end);
if (end == 1 && zam_separator(p0->data[0])) return own(p0->data, 0);
for (dot = end; dot > start + 1 && p0->data[dot - 1] != '.'; dot--) {}
if (dot > start + 1) return own(p0->data + dot - 1, end - dot + 1);
return own(p0->data, 0);"#,
    },
    Builtin {
        name: "std/env:exit",
        params: &[Type::Int],
        result: Type::Unit,
        fallible: false,
        // 退出码限定在 0–255：Unix 只保留低 8 位，限定后三个平台行为一致。
        body: r#"if (p0 < 0 || p0 > 255) { zam_panic_one("exit code must be between 0 and 255"); exit(1); }
fflush(stdout);
exit((int)p0);"#,
    },
    Builtin {
        name: "std/env:var",
        params: &[Type::Shared],
        result: Type::Owned,
        fallible: true,
        body: "S value; if (!zam_getenv(p0, &value)) return F_S_err(zam_message(\"environment variable not set: \", p0)); return F_S_ok(value);",
    },
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
        body: r#"if (!(p0 >= -9223372036854775808.0 && p0 < 9223372036854775808.0)) { zam_panic_one("f64 value out of i64 range"); exit(1); }
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
        body: r#"char text[48]; int n; S s; s.chars = 0;
if (p1 < 0 || p1 > 30) { zam_panic_one("fmt.fixed digits must be between 0 and 30"); exit(1); }
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
    // `assert` 失败：报错写 stderr，退出码 1。
    Builtin {
        name: "std/core:assert_failed",
        params: &[Type::Shared],
        result: Type::Unit,
        fallible: false,
        body: r#"fflush(stdout);
zam_write(stderr, p0->data, p0->len);
fputc('\n', stderr);
exit(1);"#,
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
drop(p0); p0->data = p; p0->len = n; p0->chars = 0;
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
        // 位置按字符算；start 为负时从末尾数，超过末尾时找不到（与 Python 的 str.find 相同）。
        body: r#"size_t i, n = zam_chars(p0), start;
if (p2 < 0) p2 += (int64_t)n;
if (p2 < 0) p2 = 0;
if ((uint64_t)p2 > n) return -1;
start = zam_byte_of(p0, (size_t)p2);
if (!p1->len) return p2;
if (p1->len > p0->len) return -1;
for (i = start; i + p1->len <= p0->len; i++) if (memcmp(p0->data + i, p1->data, p1->len) == 0) return (int64_t)zam_char_of(p0, i);
return -1;"#,
    },
    Builtin {
        name: "std/string:byte_find_from",
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
s.data = p; s.len = n; s.chars = 0;
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
s.data = p; s.len = n; s.chars = 0;
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
    // Python 的 lstrip/rstrip 与 is* 判断（空白与 trim 相同；isalpha 把非 ASCII 字符算作字母）。
    Builtin {
        name: "std/string:lstrip",
        params: &[Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: r#"size_t a = 0;
while (a < p0->len && (p0->data[a] == ' ' || p0->data[a] == '\t' || p0->data[a] == '\n' || p0->data[a] == '\r')) a++;
return own(p0->data + a, p0->len - a);"#,
    },
    Builtin {
        name: "std/string:rstrip",
        params: &[Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: r#"size_t b = p0->len;
while (b > 0 && (p0->data[b - 1] == ' ' || p0->data[b - 1] == '\t' || p0->data[b - 1] == '\n' || p0->data[b - 1] == '\r')) b--;
return own(p0->data, b);"#,
    },
    Builtin {
        name: "std/string:isdigit",
        params: &[Type::Shared],
        result: Type::Bool,
        fallible: false,
        body: r#"size_t i;
if (!p0->len) return 0;
for (i = 0; i < p0->len; i++) if (p0->data[i] < '0' || p0->data[i] > '9') return 0;
return 1;"#,
    },
    Builtin {
        name: "std/string:isalpha",
        params: &[Type::Shared],
        result: Type::Bool,
        fallible: false,
        body: r#"size_t i; unsigned char c;
if (!p0->len) return 0;
for (i = 0; i < p0->len; i++) { c = p0->data[i]; if (c < 0x80 && !((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z'))) return 0; }
return 1;"#,
    },
    Builtin {
        name: "std/string:isalnum",
        params: &[Type::Shared],
        result: Type::Bool,
        fallible: false,
        body: r#"size_t i; unsigned char c;
if (!p0->len) return 0;
for (i = 0; i < p0->len; i++) { c = p0->data[i]; if (c < 0x80 && !((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9'))) return 0; }
return 1;"#,
    },
    Builtin {
        name: "std/string:isspace",
        params: &[Type::Shared],
        result: Type::Bool,
        fallible: false,
        body: r#"size_t i;
if (!p0->len) return 0;
for (i = 0; i < p0->len; i++) if (p0->data[i] != ' ' && p0->data[i] != '\t' && p0->data[i] != '\n' && p0->data[i] != '\r') return 0;
return 1;"#,
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
        // 位置按字符算，负数从末尾数，越界收拢（与 Python 的 s[a:b] 相同）。
        body: r#"size_t a, b;
zam_clamp(&p1, &p2, zam_chars(p0));
a = zam_byte_of(p0, (size_t)p1); b = zam_byte_of(p0, (size_t)p2);
return own(p0->data + a, b - a);"#,
    },
    Builtin {
        name: "std/string:char_at",
        params: &[Type::Shared, Type::Int],
        result: Type::Owned,
        fallible: false,
        // `s[i]`：第 i 个字符（负数从末尾数），越界时终止程序。
        body: r#"size_t n = zam_chars(p0), a, b;
if (p1 < 0) p1 += (int64_t)n;
if (p1 < 0 || (uint64_t)p1 >= n) zam_string_bounds();
a = zam_byte_of(p0, (size_t)p1); b = a + 1;
while (b < p0->len && (p0->data[b] & 0xC0) == 0x80) b++;
return own(p0->data + a, b - a);"#,
    },
    Builtin {
        name: "std/string:byte_slice",
        params: &[Type::Shared, Type::Int, Type::Int],
        result: Type::Owned,
        fallible: false,
        body: r#"zam_clamp(&p1, &p2, p0->len);
// 不能从 UTF-8 多字节字符中间切开。
if (((uint64_t)p1 < p0->len && (p0->data[p1] & 0xC0) == 0x80) || ((uint64_t)p2 < p0->len && (p0->data[p2] & 0xC0) == 0x80)) zam_string_bounds();
return own(p0->data + p1, (size_t)(p2 - p1));"#,
    },
    Builtin {
        name: "std/string:chars",
        params: &[Type::Shared],
        result: Type::List(Elem::Str),
        fallible: false,
        // 每个 UTF-8 字符一项；不合法的字节单独成一项，不丢内容。
        body: r#"Ls out = {0}; size_t i = 0, n; int64_t c;
while (i < p0->len) { n = zam_utf8_next(p0->data + i, p0->len - i, &c); if (!n) n = 1; pushLs(&out, own(p0->data + i, n)); i += n; }
return out;"#,
    },
    Builtin {
        name: "std/string:codepoints",
        params: &[Type::Shared],
        result: Type::List(Elem::Int),
        fallible: false,
        // 不合法的字节按 U+FFFD 计。
        body: r#"Li out = {0}; size_t i = 0, n; int64_t c;
while (i < p0->len) { n = zam_utf8_next(p0->data + i, p0->len - i, &c); if (!n) { n = 1; c = 0xFFFD; } pushLi(&out, c); i += n; }
return out;"#,
    },
    Builtin {
        name: "std/string:from_code",
        params: &[Type::Int],
        result: Type::Owned,
        fallible: false,
        body: r#"unsigned char text[4]; size_t n = zam_utf8_put(p0, text);
if (!n) { zam_panic_one("invalid code point"); exit(1); }
return own(text, n);"#,
    },
    Builtin {
        name: "std/string:bytes",
        params: &[Type::Shared],
        result: Type::List(Elem::Int),
        fallible: false,
        body: "Li out = {0}; size_t i; for (i = 0; i < p0->len; i++) pushLi(&out, p0->data[i]); return out;",
    },
    Builtin {
        name: "std/_re:groups",
        params: &[Type::Shared, Type::Int],
        result: Type::List(Elem::Str),
        fallible: true,
        body: "return zam_re_groups(p0, p1);",
    },
    Builtin {
        name: "std/_re:find",
        params: &[Type::Shared, Type::Int, Type::Shared, Type::Int, Type::Int, Type::Int],
        result: Type::List(Elem::Int),
        fallible: true,
        body: "return zam_re_find(p0, p1, p2, p3, p4, p5);",
    },
    Builtin {
        name: "std/_process:run",
        params: &[Type::ListRef(Elem::Str, false), Type::Shared, Type::Bool],
        result: Type::List(Elem::Str),
        fallible: true,
        body: "return zam_process_run(p0, p1, p2);",
    },
    Builtin {
        name: "std/base64:b64encode",
        params: &[Type::ListRef(Elem::Byte, false)],
        result: Type::Owned,
        fallible: false,
        body: "return zam_b64encode(p0, 0);",
    },
    Builtin {
        name: "std/base64:urlsafe_b64encode",
        params: &[Type::ListRef(Elem::Byte, false)],
        result: Type::Owned,
        fallible: false,
        body: "return zam_b64encode(p0, 1);",
    },
    Builtin {
        name: "std/base64:b64decode",
        params: &[Type::Shared],
        result: Type::List(Elem::Byte),
        fallible: true,
        body: "return zam_b64decode(p0, 0);",
    },
    Builtin {
        name: "std/base64:urlsafe_b64decode",
        params: &[Type::Shared],
        result: Type::List(Elem::Byte),
        fallible: true,
        body: "return zam_b64decode(p0, 1);",
    },
    Builtin {
        name: "std/hashlib:sha256",
        params: &[Type::ListRef(Elem::Byte, false)],
        result: Type::List(Elem::Byte),
        fallible: false,
        body: "uint8_t d[32]; zam_sha256(p0->data, p0->len, d); return zam_digest_bytes(d, 32);",
    },
    Builtin {
        name: "std/hashlib:sha1",
        params: &[Type::ListRef(Elem::Byte, false)],
        result: Type::List(Elem::Byte),
        fallible: false,
        body: "uint8_t d[20]; zam_sha1(p0->data, p0->len, d); return zam_digest_bytes(d, 20);",
    },
    Builtin {
        name: "std/hashlib:md5",
        params: &[Type::ListRef(Elem::Byte, false)],
        result: Type::List(Elem::Byte),
        fallible: false,
        body: "uint8_t d[16]; zam_md5(p0->data, p0->len, d); return zam_digest_bytes(d, 16);",
    },
    Builtin {
        name: "std/hashlib:hmac_sha256",
        params: &[Type::ListRef(Elem::Byte, false), Type::ListRef(Elem::Byte, false)],
        result: Type::List(Elem::Byte),
        fallible: false,
        body: "uint8_t d[32]; zam_hmac_sha256(p0->data, p0->len, p1->data, p1->len, d); return zam_digest_bytes(d, 32);",
    },
    Builtin {
        name: "std/core:bytes_literal",
        params: &[Type::Shared],
        result: Type::List(Elem::Byte),
        fallible: false,
        // 解析器已把 `b"..."` 的每个字节写成两位小写十六进制。
        body: r#"Ly out = {0}; size_t i;
reserveLy(&out, p0->len / 2);
for (i = 0; i + 1 < p0->len; i += 2) { unsigned char a = p0->data[i], b = p0->data[i + 1]; pushLy(&out, (a <= '9' ? a - '0' : a - 'a' + 10) * 16 + (b <= '9' ? b - '0' : b - 'a' + 10)); }
return out;"#,
    },
    Builtin {
        name: "std/string:encode",
        params: &[Type::Shared],
        result: Type::List(Elem::Byte),
        fallible: false,
        body: "Ly out = {0}; reserveLy(&out, p0->len); if (p0->len) memcpy(out.data, p0->data, p0->len); out.len = p0->len; return out;",
    },
    Builtin {
        name: "std/bytes:decode",
        params: &[Type::ListRef(Elem::Byte, false)],
        result: Type::Owned,
        fallible: true,
        body: r#"size_t i, n; int64_t c;
for (i = 0; i < p0->len; i += n) {
    n = zam_utf8_next(p0->data + i, p0->len - i, &c);
    if (!n) { S at = from_int((int64_t)i); S message = zam_message("invalid UTF-8 at byte ", &at); drop(&at); return F_S_err(message); }
}
return F_S_ok(own(p0->data, p0->len));"#,
    },
    Builtin {
        name: "std/bytes:hex",
        params: &[Type::ListRef(Elem::Byte, false)],
        result: Type::Owned,
        fallible: false,
        body: r#"static const char digits[] = "0123456789abcdef"; unsigned char *q; size_t i; S s;
if (p0->len > SIZE_MAX / 2) zam_alloc_failed();
q = (unsigned char*)malloc(p0->len ? p0->len * 2 : 1);
if (!q) zam_alloc_failed();
for (i = 0; i < p0->len; i++) { q[2 * i] = (unsigned char)digits[p0->data[i] >> 4]; q[2 * i + 1] = (unsigned char)digits[p0->data[i] & 15]; }
s.data = q; s.len = p0->len * 2; s.chars = s.len;
return s;"#,
    },
    Builtin {
        name: "std/bytes:from_hex",
        params: &[Type::Shared],
        result: Type::List(Elem::Byte),
        fallible: true,
        // 与 Python 的 bytes.fromhex 相同：两位一组，组与组之间可以有空白。
        body: r#"Ly out = {0}; size_t i = 0; int high = -1;
for (i = 0; i < p0->len; i++) {
    unsigned char c = p0->data[i]; int v;
    if (high < 0 && (c == ' ' || c == '\t' || c == '\n' || c == '\r')) continue;
    v = c >= '0' && c <= '9' ? c - '0' : c >= 'a' && c <= 'f' ? c - 'a' + 10 : c >= 'A' && c <= 'F' ? c - 'A' + 10 : -1;
    if (v < 0) { S at = from_int((int64_t)i); S message = zam_message("invalid hex digit at position ", &at); drop(&at); dropLy(&out); return F_Ly_err(message); }
    if (high < 0) high = v; else { pushLy(&out, high * 16 + v); high = -1; }
}
if (high >= 0) { dropLy(&out); return F_Ly_err(zam_message("odd number of hex digits", NULL)); }
return F_Ly_ok(out);"#,
    },
    Builtin {
        name: "std/bytes:zeros",
        params: &[Type::Int],
        result: Type::List(Elem::Byte),
        fallible: false,
        body: r#"Ly out = {0};
if (p0 < 0) { zam_panic_one("negative count"); exit(1); }
if ((uint64_t)p0 > SIZE_MAX) list_alloc_failed();
reserveLy(&out, (size_t)p0);
if (p0) memset(out.data, 0, (size_t)p0);
out.len = (size_t)p0;
return out;"#,
    },
    Builtin {
        name: "std/fs:read_bytes",
        params: &[Type::Shared],
        result: Type::List(Elem::Byte),
        fallible: true,
        body: r#"S text; int ok; Ly out = {0};
FILE *file = zam_open(p0, "rb");
if (!file) return F_Ly_err(zam_message("cannot open ", p0));
ok = zam_read_all(file, &text);
fclose(file);
if (!ok) return F_Ly_err(zam_message("cannot read ", p0));
reserveLy(&out, text.len);
if (text.len) memcpy(out.data, text.data, text.len);
out.len = text.len;
drop(&text);
return F_Ly_ok(out);"#,
    },
    Builtin {
        name: "std/fs:write_bytes",
        params: &[Type::Shared, Type::ListRef(Elem::Byte, false)],
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
        name: "std/fs:append_bytes",
        params: &[Type::Shared, Type::ListRef(Elem::Byte, false)],
        result: Type::Unit,
        fallible: true,
        body: r#"int ok;
FILE *file = zam_open(p0, "ab");
if (!file) return F_void_err(zam_message("cannot open ", p0));
ok = !p1->len || fwrite(p1->data, 1, p1->len, file) == p1->len;
if (fclose(file) != 0) ok = 0;
if (!ok) return F_void_err(zam_message("cannot write ", p0));
return F_void_ok();"#,
    },
    Builtin {
        name: "std/string:from_bytes",
        params: &[Type::ListRef(Elem::Int, false)],
        result: Type::Owned,
        fallible: true,
        body: r#"unsigned char *p = (unsigned char*)malloc(p0->len ? p0->len : 1); size_t i, n; int64_t c; S s;
if (!p) zam_alloc_failed();
for (i = 0; i < p0->len; i++) { if (p0->data[i] < 0 || p0->data[i] > 255) { free(p); return F_S_err(zam_message("byte out of range", NULL)); } p[i] = (unsigned char)p0->data[i]; }
for (i = 0; i < p0->len; i += n) { n = zam_utf8_next(p + i, p0->len - i, &c); if (!n) { free(p); return F_S_err(zam_message("invalid UTF-8", NULL)); } }
s.data = p; s.len = p0->len; s.chars = 0;
return F_S_ok(s);"#,
    },
    Builtin {
        name: "std/string:repeat",
        params: &[Type::Shared, Type::Int],
        result: Type::Owned,
        fallible: false,
        // 与 Python 一样，次数小于 1 得到空串。
        body: r#"unsigned char *p; size_t i, count = p1 > 0 ? (size_t)p1 : 0; S s;
if (p0->len && count > SIZE_MAX / p0->len) zam_alloc_failed();
p = (unsigned char*)malloc(p0->len * count ? p0->len * count : 1);
if (!p) zam_alloc_failed();
for (i = 0; i < count; i++) if (p0->len) memcpy(p + i * p0->len, p0->data, p0->len);
s.data = p; s.len = p0->len * count; s.chars = 0;
return s;"#,
    },
    Builtin {
        name: "std/string:count",
        params: &[Type::Shared, Type::Shared],
        result: Type::Int,
        fallible: false,
        // 不重叠地计数；空子串与 Python 一样算「字符数 + 1」。
        body: r#"size_t i = 0, n = 0, step; int64_t c;
if (!p1->len) { while (i < p0->len) { step = zam_utf8_next(p0->data + i, p0->len - i, &c); i += step ? step : 1; n++; } return (int64_t)n + 1; }
while (i + p1->len <= p0->len) { if (memcmp(p0->data + i, p1->data, p1->len) == 0) { n++; i += p1->len; } else i++; }
return (int64_t)n;"#,
    },
    Builtin {
        name: "std/string:pad_left",
        params: &[Type::Shared, Type::Int],
        result: Type::Owned,
        fallible: false,
        // 按字符数补空格到 width，已经够宽就原样复制；用于对齐输出。
        body: "return zam_pad(p0, p1, 1);",
    },
    Builtin {
        name: "std/string:pad_right",
        params: &[Type::Shared, Type::Int],
        result: Type::Owned,
        fallible: false,
        body: "return zam_pad(p0, p1, 0);",
    },
    // `s.ljust(5, "0")`、`s.rjust(5, "0")`、`s.center(9, "*")`：用给定的一个字符补，同 Python。
    Builtin {
        name: "std/string:pad_right_with",
        params: &[Type::Shared, Type::Int, Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: "return zam_pad_with(p0, p1, 0, p2);",
    },
    Builtin {
        name: "std/string:pad_left_with",
        params: &[Type::Shared, Type::Int, Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: "return zam_pad_with(p0, p1, 1, p2);",
    },
    Builtin {
        name: "std/string:center_with",
        params: &[Type::Shared, Type::Int, Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: "return zam_pad_with(p0, p1, 2, p2);",
    },
    Builtin {
        name: "std/string:find",
        params: &[Type::Shared, Type::Shared],
        result: Type::Int,
        fallible: false,
        body: r#"size_t i;
if (!p1->len) return 0;
if (p1->len > p0->len) return -1;
for (i = 0; i + p1->len <= p0->len; i++) if (memcmp(p0->data + i, p1->data, p1->len) == 0) return (int64_t)zam_char_of(p0, i);
return -1;"#,
    },
    // 最后一次出现的位置（按字符算），找不到是 -1。与 Python 的 str.rfind 相同。
    Builtin {
        name: "std/string:rfind",
        params: &[Type::Shared, Type::Shared],
        result: Type::Int,
        fallible: false,
        body: r#"size_t i;
if (!p1->len) return (int64_t)zam_chars(p0);
if (p1->len > p0->len) return -1;
for (i = p0->len - p1->len + 1; i-- > 0;) if (memcmp(p0->data + i, p1->data, p1->len) == 0) return (int64_t)zam_char_of(p0, i);
return -1;"#,
    },
    // 去掉开头的 prefix（没有就原样复制）。
    Builtin {
        name: "std/string:removeprefix",
        params: &[Type::Shared, Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: "if (p1->len <= p0->len && (!p1->len || memcmp(p0->data, p1->data, p1->len) == 0)) return own(p0->data + p1->len, p0->len - p1->len); return copy(p0);",
    },
    // 去掉结尾的 suffix（没有就原样复制）。
    Builtin {
        name: "std/string:removesuffix",
        params: &[Type::Shared, Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: "if (p1->len <= p0->len && (!p1->len || memcmp(p0->data + p0->len - p1->len, p1->data, p1->len) == 0)) return own(p0->data, p0->len - p1->len); return copy(p0);",
    },
    // 首字母大写，其余小写（ASCII 字母）。
    Builtin {
        name: "std/string:capitalize",
        params: &[Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: r#"S s = copy(p0); size_t i;
for (i = 0; i < s.len; i++) {
    unsigned char c = s.data[i];
    if (i == 0 && c >= 'a' && c <= 'z') s.data[i] = (unsigned char)(c - 32);
    else if (i > 0 && c >= 'A' && c <= 'Z') s.data[i] = (unsigned char)(c + 32);
}
return s;"#,
    },
    // 每个词的首字母大写，其余小写（ASCII 字母；字母以外的字符分开词）。
    Builtin {
        name: "std/string:title",
        params: &[Type::Shared],
        result: Type::Owned,
        fallible: false,
        body: r#"S s = copy(p0); size_t i; int inside = 0;
for (i = 0; i < s.len; i++) {
    unsigned char c = s.data[i];
    int letter = (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
    if (letter && !inside && c >= 'a' && c <= 'z') s.data[i] = (unsigned char)(c - 32);
    else if (letter && inside && c >= 'A' && c <= 'Z') s.data[i] = (unsigned char)(c + 32);
    inside = letter;
}
return s;"#,
    },
    // 居中：两边补空格到 width 个字符；补不匀时与 Python 的 str.center 一样分配。
    Builtin {
        name: "std/string:center",
        params: &[Type::Shared, Type::Int],
        result: Type::Owned,
        fallible: false,
        body: r#"int64_t n = (int64_t)zam_chars(p0), margin, left, i;
S s;
if (p1 <= n) return copy(p0);
margin = p1 - n;
left = margin / 2 + (margin & p1 & 1);
s.len = p0->len + (size_t)margin; s.chars = 0;
s.data = (unsigned char *)malloc(s.len);
if (!s.data) { zam_panic_one("allocation failed"); exit(1); }
for (i = 0; i < left; i++) s.data[i] = ' ';
if (p0->len) memcpy(s.data + left, p0->data, p0->len);
for (i = left + (int64_t)p0->len; i < (int64_t)s.len; i++) s.data[i] = ' ';
return s;"#,
    },
    // 左边补 0 到 width 个字符，正负号留在最前面（"-42".zfill(5) 是 "-0042"）。
    Builtin {
        name: "std/string:zfill",
        params: &[Type::Shared, Type::Int],
        result: Type::Owned,
        fallible: false,
        body: r#"int64_t n = (int64_t)zam_chars(p0), fill, i;
size_t sign;
S s;
if (p1 <= n) return copy(p0);
fill = p1 - n;
sign = p0->len && (p0->data[0] == '-' || p0->data[0] == '+') ? 1 : 0;
s.len = p0->len + (size_t)fill; s.chars = 0;
s.data = (unsigned char *)malloc(s.len);
if (!s.data) { zam_panic_one("allocation failed"); exit(1); }
if (sign) s.data[0] = p0->data[0];
for (i = 0; i < fill; i++) s.data[sign + (size_t)i] = '0';
memcpy(s.data + sign + (size_t)fill, p0->data + sign, p0->len - sign);
return s;"#,
    },
    // 至少有一个字母，并且字母都是大写（ASCII）。
    Builtin {
        name: "std/string:isupper",
        params: &[Type::Shared],
        result: Type::Bool,
        fallible: false,
        body: r#"size_t i; int cased = 0;
for (i = 0; i < p0->len; i++) {
    if (p0->data[i] >= 'a' && p0->data[i] <= 'z') return 0;
    if (p0->data[i] >= 'A' && p0->data[i] <= 'Z') cased = 1;
}
return cased;"#,
    },
    // 至少有一个字母，并且字母都是小写（ASCII）。
    Builtin {
        name: "std/string:islower",
        params: &[Type::Shared],
        result: Type::Bool,
        fallible: false,
        body: r#"size_t i; int cased = 0;
for (i = 0; i < p0->len; i++) {
    if (p0->data[i] >= 'A' && p0->data[i] <= 'Z') return 0;
    if (p0->data[i] >= 'a' && p0->data[i] <= 'z') cased = 1;
}
return cased;"#,
    },
    Builtin {
        name: "std/string:byte_find",
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
                generics: Vec::new(),
                external: false,
                wrapping: false,
                defaults: Vec::new(),
                dispatch: false,
                bounds: Vec::new(),
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
    // 当前函数带 `@wrapping`：整数算术回绕，浮点除法按 IEEE 754。
    wrapping: bool,
    // 可能被改写的名字（`let mut`、可变解构、非借用参数）：流式遍历只借用其余的名字。
    mutable_names: std::collections::BTreeSet<String>,
    call_targets: BTreeMap<usize, String>,
    code: String,
    next: usize,
    // 当前函数的签名：`?` 提前返回时要构造同样形状的结果结构体。
    result: Type,
    error: Option<Type>,
    // 函数体里出现过的列表与字典类型，用来决定发射哪些容器辅助函数。
    containers: Containers,
    // 用到了复合值的复制、相等或打印：这个目标文件要带上 native_values 生成的函数。
    values: bool,
    // 函数体里声明过的结构体类型（含推断出来的元组），都要发射定义。
    records: std::collections::BTreeSet<u64>,
    // 正按目标类型生成的表达式（地址）与目标类型：`Option.None` 这类构造只能从期望类型得知
    // 类型实参，检查器用的是同一个期望类型。只对这一个表达式生效，不传给子表达式。
    hint: Option<(usize, Type)>,
    // 用到的函数类型：每个生成一个 `F<id>` 结构体（函数指针 + 环境指针）。
    funcs: std::collections::BTreeSet<u64>,
    // 提到函数体外面的定义：匿名函数与具名函数当值用时的包装函数，（名字, 定义）。
    lifted: Vec<(String, String)>,
    // 匿名函数的 C 名字前缀：所在函数的符号。
    prefix: String,
    // 函数里定义的 fn：自己的（源码名字, 生成的 C 函数名, 函数类型编号）。函数体里调用自己
    // （递归）时直接调这个静态函数，不经过局部变量。
    own: Option<(String, String, u64)>,
    // 用到的装箱：（接口, 具体类型）。模块里为每一对生成虚表与装箱函数。
    boxes: std::collections::BTreeSet<(u64, u64)>,
}
/// 用到的容器：`(元素, None)` 是列表，`(键, Some(值))` 是字典。
type Containers = std::collections::BTreeSet<(Elem, Option<Elem>)>;
fn note(set: &mut Containers, ty: Type) {
    match ty.container() {
        Some(Type::List(elem)) => {
            if set.insert((elem, None)) {
                note_nested(set, elem);
            }
        }
        // 字典的 keys()/values() 返回键与值的列表，所以这两种列表一并生成。
        Some(Type::Map(key, value)) => {
            set.insert((key, None));
            if value.copyable() {
                set.insert((value, None));
            }
            if set.insert((key, Some(value))) {
                note_nested(set, value);
            }
        }
        _ => {}
    }
}
/// 内层容器也要生成自己的辅助函数。
fn note_nested(set: &mut Containers, elem: Elem) {
    if let Elem::Nested(id) = elem {
        note(set, nested(id));
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
        Type::List(_) | Type::Map(_, _) => nest(ty),
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
/// 按值传递、不需要释放的值：数字、布尔与函数值。
fn by_value(ty: Type) -> bool {
    copied(ty)
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
// 失败函数的结果结构体：ok=1 时 value 有效，ok=0 时 error 有效。错误是字符串时名字只看成功类型
// （`F_int64_t`），是结构体或枚举时再带上错误类型（`F_int64_t_R123`）。
fn fallible_name(ty: Type, error: Option<Type>) -> String {
    match error {
        None | Some(Type::Owned) => format!("F_{}", ctype(ty)),
        Some(error) => format!("F_{}_{}", ctype(ty), ctype(error)),
    }
}
/// 函数值的 C 返回类型：会失败时是结果结构体。
fn value_result(id: u64) -> String {
    let result = func_sig(id).1;
    match func_error(id) {
        Some(error) => fallible_name(result, Some(error)),
        None => ctype(result),
    }
}
fn fallible_ok(ty: Type, error: Option<Type>) -> String {
    format!("{}_ok", fallible_name(ty, error))
}
fn fallible_err(ty: Type, error: Option<Type>) -> String {
    format!("{}_err", fallible_name(ty, error))
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
        // 内层容器的名字直接拼进来：`[[i64]]` 是 `LLi`，`{string: [string]}` 是 `MsLs`。
        Elem::Nested(id) => match nested(id) {
            Type::List(elem) => format!("L{}", elem_suffix(elem)),
            Type::Map(key, value) => format!("M{}{}", elem_suffix(key), elem_suffix(value)),
            _ => unreachable!("nested elements are containers"),
        },
        Elem::Func(id) => format!("F{id}"),
        Elem::Byte => "y".into(),
    }
}
/// 列表里元素的 C 存储类型：`bytes` 按 uint8_t 存，其余与元素的值类型相同。
fn elem_ctype(elem: Elem) -> String {
    match elem {
        Elem::Byte => "uint8_t".into(),
        _ => ctype(elem.ty()),
    }
}
/// 写进列表元素的值：`bytes` 先检查 0–255。
fn elem_store(elem: Elem, value: &str) -> String {
    match elem {
        Elem::Byte => format!("zam_byte({value})"),
        _ => value.to_owned(),
    }
}
fn map_name(key: Elem, value: Elem) -> String {
    format!("M{}{}", elem_suffix(key), elem_suffix(value))
}
const MAP_PRELUDE: &str = r#"static void map_missing(void) { zam_panic_one("key not found"); exit(1); }
static uint64_t zam_hash_int(int64_t key) { uint64_t x = (uint64_t)key; x ^= x >> 33; x *= 0xff51afd7ed558ccdULL; x ^= x >> 33; x *= 0xc4ceb9fe1a85ec53ULL; x ^= x >> 33; return x; }
static uint64_t zam_hash_str(S *key) { uint64_t h = 1469598103934665603ULL; size_t i; for (i = 0; i < key->len; i++) { h ^= key->data[i]; h *= 1099511628211ULL; } return h; }
"#;
// 插入顺序表 + 开放寻址下标表：遍历按插入顺序，删除留墓碑，下标表装载率不超过 1/2，
// 重建时先压掉已删除的项。
// 类型分三段发射：前置声明、字典本体（只含条目指针）、条目（按值含键和值，要等结构体定义完）。
// 集合（值为占位的字典）的方法：加入、并交差、子集、排好序的列表。
const SET_TEMPLATE: &str = r#"static void add$M($M *m, $KP) { set$M(m, k, 1); }
static $M union$M($M *a, $M *b) { $M out = {0}; size_t i; for (i = 0; i < a->len; i++) if (a->entries[i].live) set$M(&out, $AKEY, 1); for (i = 0; i < b->len; i++) if (b->entries[i].live) set$M(&out, $BKEY, 1); return out; }
static $M intersection$M($M *a, $M *b) { $M out = {0}; size_t i; for (i = 0; i < a->len; i++) if (a->entries[i].live && has$M(b, $AKEY)) set$M(&out, $AKEY, 1); return out; }
static $M difference$M($M *a, $M *b) { $M out = {0}; size_t i; for (i = 0; i < a->len; i++) if (a->entries[i].live && !has$M(b, $AKEY)) set$M(&out, $AKEY, 1); return out; }
static int issubset$M($M *a, $M *b) { size_t i; for (i = 0; i < a->len; i++) if (a->entries[i].live && !has$M(b, $AKEY)) return 0; return 1; }
static $L sorted$M($M *m) { $L out = keys$M(m); sort$L(&out); return out; }
"#;
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
static int64_t insert$M($M *m, $KP, $VT v, uint64_t h) {
    E$M *e;
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
    return (int64_t)m->len - 1;
}
static void set$M($M *m, $KP, $VT v) {
    uint64_t h = $HASH;
    int64_t at = find$M(m, k, h);
    E$M *e;
    if (at >= 0) { e = &m->entries[at]; $DROPVAL e->value = v; return; }
    insert$M(m, k, v, h);
}
// `m[k] = m.get(k, d) + x`：只查一次，没有这个键时先放进 d，返回值的地址。
static $VT *slot$M($M *m, $KP, $VT d) {
    uint64_t h = $HASH;
    int64_t at = find$M(m, k, h);
    if (at < 0) at = insert$M(m, k, d, h);
    return &m->entries[at].value;
}
static $VT *at$M($M *m, $KP) { int64_t at = find$M(m, k, $HASH); if (at < 0) map_missing(); return &m->entries[at].value; }
static int has$M($M *m, $KP) { return find$M(m, k, $HASH) >= 0; }
static $VT getor$M($M *m, $KP, $VT d) { int64_t at = find$M(m, k, $HASH); if (at < 0) return d; $DROPDEF return $VALOUT; }
static int del$M($M *m, $KP) { int64_t at = find$M(m, k, $HASH); E$M *e; if (at < 0) return 0; e = &m->entries[at]; $DROPKEY $DROPVAL e->live = 0; m->count--; return 1; }
static $L keys$M($M *m) { $L out = {0}; size_t i; for (i = 0; i < m->len; i++) { E$M *e = &m->entries[i]; if (e->live) push$L(&out, $KEYOUT); } return out; }
static void clear$M($M *m) { size_t i; for (i = 0; i < m->len; i++) { E$M *e = &m->entries[i]; if (e->live) { $DROPKEY $DROPVAL } } m->len = 0; m->count = 0; for (i = 0; i < m->nslots; i++) m->slots[i] = -1; }
"#;
/// `grid[k][j]` 的读写落到内层字典上：`valat` 读（字符串得到副本），`put` 写入或替换。
fn map_slots(key: Elem, value: Elem) -> String {
    let m = map_name(key, value);
    let t = ctype(value.ty());
    let param = if key == Elem::Str {
        "S *k"
    } else {
        "int64_t k"
    };
    let read = match value {
        Elem::Str => "copy(at{m}(m, k))".replace("{m}", &m),
        _ if value.comparable() => format!("*at{m}(m, k)"),
        _ => return format!("static void put{m}({m} *m, {param}, {t} v) {{ set{m}(m, k, v); }}\n"),
    };
    format!("static void put{m}({m} *m, {param}, {t} v) {{ set{m}(m, k, v); }}\nstatic {t} valat{m}({m} *m, {param}) {{ return {read}; }}\n")
}
/// 字典的 `values()`：值的列表，字符串值复制；结构体值不能复制，没有这个方法。
fn map_values(key: Elem, value: Elem) -> String {
    let m = map_name(key, value);
    let l = list_suffix(value);
    let out = match value {
        Elem::Str => "copy(&e->value)",
        Elem::Record(_) | Elem::Nested(_) => return String::new(),
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
        (
            "$AKEY",
            pick("&a->entries[i].key", "a->entries[i].key", text),
        ),
        (
            "$BKEY",
            pick("&b->entries[i].key", "b->entries[i].key", text),
        ),
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
        Elem::Nested(id) => format!("drop{}({slot});", ctype(nested(id))),
        _ => String::new(),
    }
}
const LIST_PRELUDE: &str = r#"static void list_bounds(void) { zam_panic_one("list index out of bounds"); exit(1); }
static void list_empty(void) { zam_panic_one("pop from empty list"); exit(1); }
static void list_alloc_failed(void) { zam_panic_one("allocation failed"); exit(1); }
static uint8_t zam_byte(int64_t v) { if (v < 0 || v > 255) { zam_panic_one("byte must be in range(0, 256)"); exit(1); } return (uint8_t)v; }
// 小列表缓冲区的空闲链表：按 16 字节分级，释放时留给下一次同级分配，最多缓存 32 MB。
// 启动过线程之后不再存取（链表没有锁），已经缓存的块留在那里。
// 每块都是 malloc 来的真实内存块，别处直接 free 它也没问题，只是不回到这里。
// 放进第 k 级的块至少有 16k 字节（按声明大小向下取级），取出时按请求向上取级。
// AddressSanitizer 构建不缓存，释放后再用之类的错误照常能查出来。
#if defined(__SANITIZE_ADDRESS__)
#define ZAM_POOL_CLASSES 0
#elif defined(__has_feature)
#if __has_feature(address_sanitizer)
#define ZAM_POOL_CLASSES 0
#endif
#endif
#ifndef ZAM_POOL_CLASSES
#define ZAM_POOL_CLASSES 16
#endif
static void *zam_pool[ZAM_POOL_CLASSES + 1];
static size_t zam_pool_bytes;
static void *zam_list_alloc(size_t size) {
    size_t k = (size + 15) / 16;
    void *p;
    if (k <= ZAM_POOL_CLASSES && !zam_threaded && zam_pool[k]) {
        p = zam_pool[k];
        zam_pool[k] = *(void **)p;
        zam_pool_bytes -= k * 16;
        return p;
    }
    p = malloc(k <= ZAM_POOL_CLASSES ? k * 16 : size);
    if (!p) list_alloc_failed();
    return p;
}
static void zam_list_free(void *p, size_t size) {
    size_t k = size / 16;
    if (!p) return;
    if (k >= 1 && k <= ZAM_POOL_CLASSES && !zam_threaded && zam_pool_bytes + k * 16 <= ((size_t)32 << 20)) {
        *(void **)p = zam_pool[k];
        zam_pool[k] = p;
        zam_pool_bytes += k * 16;
        return;
    }
    free(p);
}
static void *zam_list_grow(void *old, size_t old_size, size_t size) {
    void *p;
    if (!old) return zam_list_alloc(size);
    if (size > ZAM_POOL_CLASSES * 16) {
        p = realloc(old, size);
        if (!p) list_alloc_failed();
        return p;
    }
    p = zam_list_alloc(size);
    memcpy(p, old, old_size);
    zam_list_free(old, old_size);
    return p;
}
"#;
/// 列表本体只含元素指针，元素类型声明过即可定义。
fn list_definition(elem: Elem) -> String {
    let l = list_suffix(elem);
    let t = elem_ctype(elem);
    format!("struct {l} {{ {t} *data; size_t len; size_t cap; }};\n")
}
/// 某种元素的列表函数：释放、移走、长度、追加、按下标取地址、弹出、交换。
fn list_helpers(elem: Elem) -> String {
    let l = list_suffix(elem);
    let t = elem_ctype(elem);
    // 追加、插入与替换收的是值类型（bytes 收 int64_t），存进去之前按 `elem_store` 检查。
    let a = ctype(elem.ty());
    let store = elem_store(elem, "v");
    let drop_items = match drop_elem(elem, "&l->data[i]") {
        drop if drop.is_empty() => drop,
        drop => format!("for (size_t i = 0; i < l->len; i++) {drop} "),
    };
    let comparable = comparable_list_helpers(elem);
    let drop_slot = drop_elem(elem, "slot");
    // `grid[i][j]` 读内层元素：标量按值，字符串得到副本。
    let valat = match elem {
        Elem::Str => {
            format!("static S valat{l}({l} *l, int64_t i) {{ return copy(at{l}(l, i)); }}\n")
        }
        _ if elem.comparable() => {
            format!("static {t} valat{l}({l} *l, int64_t i) {{ return *at{l}(l, i); }}\n")
        }
        _ => String::new(),
    };
    format!(
        "static void drop{l}({l} *l) {{ {drop_items}zam_list_free(l->data, l->cap * sizeof({t})); l->data = NULL; l->len = 0; l->cap = 0; }}
static {l} take{l}({l} *l) {{ {l} v = *l; l->data = NULL; l->len = 0; l->cap = 0; return v; }}
static int64_t len{l}({l} *l) {{ return (int64_t)l->len; }}
static void push{l}({l} *l, {a} v) {{
    if (l->len == l->cap) {{
        size_t cap = l->cap ? l->cap * 2 : 4;
        {t} *p;
        if (cap > SIZE_MAX / sizeof({t})) list_alloc_failed();
        p = ({t}*)zam_list_grow(l->data, l->cap * sizeof({t}), cap * sizeof({t}));
        l->data = p; l->cap = cap;
    }}
    l->data[l->len++] = {store};
}}
// 列表字面量：元素个数已知，一次分配到位。
static void reserve{l}({l} *l, size_t cap) {{
    {t} *p;
    if (cap <= l->cap) return;
    if (cap > SIZE_MAX / sizeof({t})) list_alloc_failed();
    p = ({t}*)zam_list_grow(l->data, l->cap * sizeof({t}), cap * sizeof({t}));
    l->data = p; l->cap = cap;
}}
static {t} *at{l}({l} *l, int64_t i) {{ if (i < 0) i += (int64_t)l->len; if (i < 0 || (uint64_t)i >= l->len) list_bounds(); return &l->data[i]; }}
static {t} pop{l}({l} *l) {{ if (!l->len) list_empty(); return l->data[--l->len]; }}
static void swap{l}({l} *l, int64_t i, int64_t j) {{ {t} tmp; if (i < 0) i += (int64_t)l->len; if (j < 0) j += (int64_t)l->len; if (i < 0 || j < 0 || (uint64_t)i >= l->len || (uint64_t)j >= l->len) list_bounds(); tmp = l->data[i]; l->data[i] = l->data[j]; l->data[j] = tmp; }}
static void insert{l}({l} *l, int64_t i, {a} v) {{ if (i < 0) i += (int64_t)l->len; if (i < 0) i = 0; if ((uint64_t)i > l->len) i = (int64_t)l->len; push{l}(l, v); memmove(&l->data[i + 1], &l->data[i], (l->len - 1 - (size_t)i) * sizeof({t})); l->data[i] = {store}; }}
static {t} removeat{l}({l} *l, int64_t i) {{ {t} v; if (i < 0) i += (int64_t)l->len; if (i < 0 || (uint64_t)i >= l->len) list_bounds(); v = l->data[i]; memmove(&l->data[i], &l->data[i + 1], (l->len - 1 - (size_t)i) * sizeof({t})); l->len--; return v; }}
static void reverse{l}({l} *l) {{ size_t i, j; for (i = 0, j = l->len; i + 1 < j; i++, j--) {{ {t} tmp = l->data[i]; l->data[i] = l->data[j - 1]; l->data[j - 1] = tmp; }} }}
static void clear{l}({l} *l) {{ {drop_items}l->len = 0; }}
static void extend{l}({l} *l, {l} o) {{ size_t i; for (i = 0; i < o.len; i++) push{l}(l, o.data[i]); free(o.data); }}
static void put{l}({l} *l, int64_t i, {a} v) {{ {t} *slot = at{l}(l, i); {drop_slot} *slot = {store}; }}
{valat}{comparable}"
    )
}
/// `xs.sort_by(before)`：稳定的归并排序，`before(b, a)` 为真时 b 才排到 a 前面。
fn sort_by_helper(elem: Elem) -> String {
    let l = list_suffix(elem);
    let t = elem_ctype(elem);
    let id = sort_by_type(elem);
    let (x, y) = if by_value(elem.ty()) {
        ("a[x]", "a[y]")
    } else {
        ("&a[x]", "&a[y]")
    };
    format!(
        "static void sortby{l}({l} *l, F{id} before) {{
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
            while (x < mid && y < hi) b[k++] = before.call(before.env, {y}, {x}) ? a[y++] : a[x++];
            while (x < mid) b[k++] = a[x++];
            while (y < hi) b[k++] = a[y++];
        }}
        tmp = a; a = b; b = tmp;
    }}
    if (a != l->data) {{ memcpy(l->data, a, n * sizeof({t})); free(a); }} else free(b);
}}
"
    )
}
/// 能比较的元素（整数、浮点、布尔、字符串）才有排序、查找与切片：结构体还不能比较或复制。
fn comparable_list_helpers(elem: Elem) -> String {
    let l = list_suffix(elem);
    let t = elem_ctype(elem);
    let (compare, param, equal, copy) = match elem {
        Elem::Int | Elem::Bool => ("(*a > *b) - (*a < *b)", t.clone(), "x == v", "x"),
        // 查找时按 i64 比较：`b.index_of(300)` 找不到，而不是截断成 44 再去找。
        Elem::Byte => ("(*a > *b) - (*a < *b)", "int64_t".into(), "x == v", "x"),
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
        Elem::Record(_) | Elem::Nested(_) | Elem::Func(_) => return String::new(),
    };
    // `xs.remove(x)` 取下来的元素：字符串要释放，数字不用。
    let drop = if elem == Elem::Str {
        "drop(&x);"
    } else {
        "(void)x;"
    };
    // `xs.sum()`：整数求和与 `+` 一样检查溢出。
    let sum = match elem {
        Elem::Int | Elem::Byte => format!("static int64_t sum{l}({l} *l) {{ int64_t s = 0; size_t i; for (i = 0; i < l->len; i++) s = add(s, l->data[i]); return s; }}\n"),
        Elem::Float => format!("static double sum{l}({l} *l) {{ double s = 0.0; size_t i; for (i = 0; i < l->len; i++) s += l->data[i]; return s; }}\n"),
        _ => String::new(),
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
static void removeval{l}({l} *l, {param} v) {{ int64_t i = index{l}(l, v); {t} x; if (i < 0) {{ zam_panic_one(\"list.remove(x): x is not in the list\"); exit(1); }} x = removeat{l}(l, i); {drop} }}
static {l} slice{l}({l} *l, int64_t from, int64_t to) {{ {l} out = {{0}}; int64_t i; zam_clamp(&from, &to, l->len); for (i = from; i < to; i++) {{ {t} x = l->data[i]; push{l}(&out, {copy}); }} return out; }}
static {l} sorted{l}({l} *l) {{ {l} out = slice{l}(l, 0, (int64_t)l->len); sort{l}(&out); return out; }}
static {t} min{l}({l} *l) {{ size_t i, best = 0; if (!l->len) {{ zam_panic_one(\"min() of an empty list\"); exit(1); }} for (i = 1; i < l->len; i++) if (cmp{l}(&l->data[i], &l->data[best]) < 0) best = i; {{ {t} x = l->data[best]; return {copy}; }} }}
static {t} max{l}({l} *l) {{ size_t i, best = 0; if (!l->len) {{ zam_panic_one(\"max() of an empty list\"); exit(1); }} for (i = 1; i < l->len; i++) if (cmp{l}(&l->data[i], &l->data[best]) > 0) best = i; {{ {t} x = l->data[best]; return {copy}; }} }}
{sum}"
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
        Type::Func(id) | Type::FuncRef(id) => return format!("F{id}"),
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
impl<'a> Generator<'a> {
    /// 同一个模块与符号表上的空白生成器：求匿名函数体的类型、生成提出去的函数。
    fn sub(&self, locals: BTreeMap<String, (String, Type)>, result: Type) -> Generator<'a> {
        Generator {
            module: self.module,
            symbols: self.symbols,
            locals,
            owners: Vec::new(),
            loops: Vec::new(),
            proofs: ranges::Proofs::default(),
            wrapping: false,
            mutable_names: Default::default(),
            call_targets: BTreeMap::new(),
            code: String::new(),
            next: 0,
            result,
            error: None,
            containers: Containers::new(),
            values: false,
            records: Default::default(),
            hint: None,
            funcs: Default::default(),
            lifted: Vec::new(),
            prefix: String::new(),
            own: None,
            boxes: Default::default(),
        }
    }
    /// 匿名函数捕获的外层变量：函数体里用到、又是当前局部变量的名字。
    fn captures(&self, params: &[(String, Option<Type>)], body: &Block) -> Vec<String> {
        let mut roots = std::collections::BTreeSet::new();
        block_roots(body, &mut roots);
        roots
            .into_iter()
            .filter(|name| self.locals.contains_key(name) && !params.iter().any(|(p, _)| p == name))
            .collect()
    }
    /// 没有期望类型时匿名函数的类型：参数用标注（或 `known`），返回类型用标注或函数体的类型。
    fn lambda_type(
        &self,
        params: &[(String, Option<Type>)],
        result: Option<Type>,
        body: &Block,
        known: Option<Vec<Type>>,
    ) -> Option<u64> {
        let types: Vec<Type> = match known {
            Some(types) => types,
            None => params.iter().map(|(_, ty)| *ty).collect::<Option<_>>()?,
        };
        let result = match result {
            Some(result) => result,
            None if lambda_prints(body) => Type::Unit,
            None => {
                let mut locals = BTreeMap::new();
                for name in self.captures(params, body) {
                    let ty = captured_view(self.locals[&name].1, true).ok()?;
                    locals.insert(name, (String::new(), ty));
                }
                for ((name, _), ty) in params.iter().zip(&types) {
                    locals.insert(name.clone(), (String::new(), *ty));
                }
                let sub = self.sub(locals, Type::Unit);
                let ty = sub.ty(lambda_body(body)?);
                ty.borrowable().map_or(ty, |(owned, _)| owned)
            }
        };
        Some(func_type(types, result))
    }
    /// 具名函数当值用：生成（一次）转调它的包装函数，第一个参数是用不到的环境。
    fn function_value(&mut self, name: &str) -> String {
        let function = &self.module.functions[name];
        let id = func_type_full(
            function.params.iter().map(|(_, ty)| *ty).collect(),
            function.result,
            function.error,
        );
        let symbol = &self.symbols[name];
        let wrapper = format!("{symbol}_value");
        if !self.lifted.iter().any(|(key, _)| *key == wrapper) {
            let params: Vec<String> = function
                .params
                .iter()
                .enumerate()
                .map(|(i, (_, ty))| format!(", {} a{i}", ctype(*ty)))
                .collect();
            let args: Vec<String> = (0..function.params.len())
                .map(|i| format!("a{i}"))
                .collect();
            let call = format!("{symbol}({})", args.join(", "));
            let body = if function.result == Type::Unit && function.error.is_none() {
                format!("{call};")
            } else {
                format!("return {call};")
            };
            self.lifted.push((
                wrapper.clone(),
                format!(
                    "static {} {wrapper}(void *env{}) {{ (void)env; {body} }}\n",
                    value_result(id),
                    params.concat()
                ),
            ));
        }
        self.scalar(&format!("{{ {wrapper}, NULL }}"), Type::Func(id))
    }
    /// 匿名函数：函数体提成一个静态函数，捕获的变量放进栈上的环境结构体（存它们的地址）。
    fn lambda(&mut self, params: &[(String, Option<Type>)], body: &Block, ty: Type) -> String {
        self.lambda_named(params, body, ty, None)
    }

    /// `self_name`：函数里定义的 fn 自己的名字。函数体里调用它自己（递归）时，生成的代码
    /// 直接调这个静态函数（环境指针原样传下去），不经过局部变量。
    fn lambda_named(
        &mut self,
        params: &[(String, Option<Type>)],
        body: &Block,
        ty: Type,
        self_name: Option<&str>,
    ) -> String {
        let (Type::Func(id) | Type::FuncRef(id)) = ty else {
            unreachable!("function type")
        };
        let (types, result) = func_sig(id);
        let name = format!("{}_fn{}", self.prefix, self.next);
        self.next += 1;
        let captures = self.captures(params, body);
        let mut locals = BTreeMap::new();
        let mut fields = Vec::new();
        let mut inits = Vec::new();
        for (index, capture) in captures.iter().enumerate() {
            let (local, ty) = self.locals[capture].clone();
            let view = captured_view(ty, true).expect("checked capture");
            if by_value(ty) {
                // 数字、布尔与函数：存地址，读的时候解引用，看到的是调用时的当前值。
                fields.push(format!("{} *c{index};", ctype(ty)));
                inits.push(format!("&{local}"));
                locals.insert(capture.clone(), (format!("(*e->c{index})"), ty));
            } else if ty.is_reference() {
                fields.push(format!("{} c{index};", ctype(view)));
                inits.push(local);
                locals.insert(capture.clone(), (format!("e->c{index}"), view));
            } else {
                fields.push(format!("{} c{index};", ctype(view)));
                inits.push(format!("&{local}"));
                locals.insert(capture.clone(), (format!("e->c{index}"), view));
            }
        }
        let failure = func_error(id);
        let mut g = self.sub(BTreeMap::new(), result);
        g.error = failure;
        g.prefix = name.clone();
        // 函数里定义的 fn 调用自己：直接调这个静态函数，环境指针原样传下去。
        if let Some(own) = self_name {
            g.own = Some((own.to_owned(), name.clone(), id));
        }
        let mut header = Vec::new();
        for (i, ((param, _), ty)) in params.iter().zip(&types).enumerate() {
            let local = format!("a{i}");
            if owned(*ty) {
                g.owners.push((local.clone(), *ty));
            }
            header.push(format!(", {} {local}", ctype(*ty)));
            g.locals.insert(param.clone(), (local, *ty));
        }
        g.locals.extend(locals);
        match lambda_body(body) {
            Some(value) if result != Type::Unit => {
                let value = g.expr_as(value, result);
                let value = g.moved(&value, result);
                g.code
                    .push_str(&format!("{} result = {value};\n", ctype(result)));
                g.cleanup();
                g.code.push_str(&match failure {
                    Some(_) => format!("return {}(result);\n", fallible_ok(result, failure)),
                    None => "return result;\n".to_owned(),
                });
            }
            // 没有返回值的函数体，或函数里定义的函数（完整代码块，return 语句自己返回）。
            _ => {
                let returned = g.block(body, result);
                g.cleanup();
                if failure.is_some() && !returned && result == Type::Unit {
                    g.code
                        .push_str(&format!("return {}();\n", fallible_ok(result, failure)));
                }
            }
        }
        let environment = if captures.is_empty() {
            "(void)envp;\n".to_owned()
        } else {
            format!("struct {name}_env *e = envp;\n")
        };
        let mut definition = String::new();
        if !captures.is_empty() {
            definition.push_str(&format!("struct {name}_env {{ {} }};\n", fields.join(" ")));
        }
        definition.push_str(&format!(
            "static {} {name}(void *envp{}) {{\n{environment}{}}}\n",
            value_result(id),
            header.concat(),
            g.code
        ));
        self.containers.extend(g.containers);
        self.values |= g.values;
        self.records.extend(g.records);
        self.boxes.extend(g.boxes);
        self.funcs.extend(g.funcs);
        self.lifted.extend(g.lifted);
        self.lifted.push((name.clone(), definition));
        if captures.is_empty() {
            return self.scalar(&format!("{{ {name}, NULL }}"), ty);
        }
        let environment = format!("v{}", self.next);
        self.next += 1;
        self.code.push_str(&format!(
            "struct {name}_env {environment} = {{ {} }};\n",
            inits.join(", ")
        ));
        self.scalar(&format!("{{ {name}, &{environment} }}"), ty)
    }
    /// `?`、`catch` 包着的函数值调用：（调用的 C 表达式, 结果类型, 错误类型）。不是时返回 None。
    fn value_call(&mut self, inner: &Expr) -> Option<(String, Type, Type)> {
        let (function, ty, args) = match inner {
            Expr::Apply(path, args) => {
                let ty = self.place_ty(path);
                let function = match path.split_once('.') {
                    Some((root, field)) => self.field(root, field),
                    None => self.locals.get(path)?.0.clone(),
                };
                (function, ty, args.clone())
            }
            Expr::Method(..) if self.field_call(inner).is_some() => {
                let (path, args) = self.field_call(inner).expect("field call");
                let (root, field) = path.split_once('.').expect("field path");
                (self.field(root, field), self.place_ty(&path), args)
            }
            Expr::Invoke(callee, args) => {
                let ty = self.value_type(callee);
                (self.expr_as(callee, ty), ty, args.clone())
            }
            _ => return None,
        };
        let (Type::Func(id) | Type::FuncRef(id)) = ty else {
            return None;
        };
        let (params, result) = func_sig(id);
        let mut values = vec![format!("{function}.env")];
        for (arg, ty) in args.iter().zip(params) {
            values.push(self.argument(arg, ty));
        }
        let error = func_error(id).expect("checked fallible function value");
        Some((
            format!("{function}.call({})", values.join(", ")),
            result,
            error,
        ))
    }
    /// 调用函数值：`f.call(f.env, 实参...)`。`path` 是局部变量或结构体里的函数字段。
    fn apply(&mut self, path: &str, args: &[Expr]) -> String {
        // 函数里定义的 fn 调用自己：直接调它自己的静态函数，环境指针原样传下去。
        // 这时局部变量里放的还是占位，值要等整条 let 语句生成完才有。
        if let Some((own, symbol, own_id)) = self.own.clone() {
            if path == own {
                // 递归调用：类型直接从函数签名取，函数体里没有这个名字的局部变量。
                let (params, result) = func_sig(own_id);
                let mut values = vec!["envp".to_owned()];
                for (arg, ty) in args.iter().zip(params) {
                    values.push(self.argument(arg, ty));
                }
                return self.call_value(&format!("{symbol}({})", values.join(", ")), result);
            }
        }
        let (Type::Func(id) | Type::FuncRef(id)) = self.place_ty(path) else {
            unreachable!("checked function value")
        };
        let function = match path.split_once('.') {
            Some((root, field)) => self.field(root, field),
            None => self.locals[path].0.clone(),
        };
        let (params, result) = func_sig(id);
        let mut values = vec![format!("{function}.env")];
        for (arg, ty) in args.iter().zip(params) {
            values.push(self.argument(arg, ty));
        }
        self.call_value(&format!("{function}.call({})", values.join(", ")), result)
    }
    /// `p.on_click(x)`：调用结构体里的函数字段。
    fn field_call(&self, expr: &Expr) -> Option<(String, Vec<Expr>)> {
        let Expr::Method(path, method, args) = expr else {
            return None;
        };
        let root = path.split('.').next().unwrap_or(path);
        if !self.locals.contains_key(root) {
            return None;
        }
        field_function(&self.module.structs, self.place_ty(path), method)?;
        Some((format!("{path}.{method}"), args.clone()))
    }
    /// 字符串上的方法改写成 std/string 的调用（检查器已确认存在）。
    /// `for w in text.words():` / `text.lines()`：解析器把它拆成隐藏变量 `$forN` 的 let
    /// 加上遍历。原文在循环期间不可能改变时（不可变绑定或只读借用，循环体里也没提到它），
    /// 不必先建出整个列表：边扫描边把指向原文的视图交给循环变量，一个字符串都不复制。
    fn streamed<'b>(
        &self,
        body: &'b Block,
        at: usize,
    ) -> Option<(&'b String, String, &'static str, &'b Block)> {
        let (Instruction::Let(hidden, false, _, value), _) = &body[at] else {
            return None;
        };
        let Some((Instruction::Each(name, None, path, false, nested), _)) = body.get(at + 1) else {
            return None;
        };
        if path != hidden || !hidden.starts_with("$for") {
            return None;
        }
        let call = self.string_call(value).unwrap_or_else(|| value.clone());
        let Expr::Call(function, args) = &call else {
            return None;
        };
        let kind = match function.as_str() {
            "std/string:words" => "words",
            "std/string:lines" => "lines",
            _ => return None,
        };
        let [Expr::Borrow(source, false) | Expr::Variable(source)] = args.as_slice() else {
            return None;
        };
        if source.contains('.') || self.mutable_names.contains(source) {
            return None;
        }
        if !matches!(
            self.locals.get(source),
            Some((_, Type::Owned | Type::Shared))
        ) {
            return None;
        }
        // 循环体里只要出现这个名字（读也算）就不做：保守，免得漏掉移动或借用。
        let text = format!("{nested:?}");
        if text.contains(&format!("\"{source}\"")) || text.contains(&format!("\"{source}.")) {
            return None;
        }
        Some((name, self.pointer(source), kind, nested))
    }
    fn stream(
        &mut self,
        name: &String,
        source: &str,
        kind: &str,
        nested: &Block,
        result_type: Type,
    ) {
        let src = format!("v{}", self.next);
        let at = format!("v{}", self.next + 1);
        let view = format!("v{}", self.next + 2);
        self.next += 3;
        self.code.push_str(&format!(
            "{{\nS *{src} = {source};\nsize_t {at} = 0;\n// zam: streamed {kind}\nfor (;;) {{\n"
        ));
        if kind == "words" {
            // 与 std/string:words 相同：按 ASCII 空白切分，跳过空段。
            let space = |c: &str| {
                format!("({c} == ' ' || {c} == '\\t' || {c} == '\\n' || {c} == '\\r' || {c} == '\\v' || {c} == '\\f')")
            };
            let c = format!("{src}->data[{at}]");
            self.code.push_str(&format!(
                "size_t start;\nwhile ({at} < {src}->len && {}) {at}++;\nif ({at} == {src}->len) break;\nstart = {at};\nwhile ({at} < {src}->len && !{}) {at}++;\nS {view} = {{0}};\n{view}.data = {src}->data + start;\n{view}.len = {at} - start;\n",
                space(&c),
                space(&c)
            ));
        } else {
            // 与 std/string:lines 相同：按 \n 切分，去掉行尾的 \r，末尾换行之后不再多一行。
            self.code.push_str(&format!(
                "size_t start = {at}, end;\nif (start > {src}->len) break;\nwhile ({at} < {src}->len && {src}->data[{at}] != '\\n') {at}++;\nif ({at} == {src}->len && start == {src}->len) break;\nend = {at};\nif (end > start && {src}->data[end - 1] == '\\r') end--;\nS {view} = {{0}};\n{view}.data = {src}->data + start;\n{view}.len = end - start;\n{at}++;\n"
            ));
        }
        let item = self.scalar(&format!("&{view}"), Type::Shared);
        self.locals.insert(name.clone(), (item, Type::Shared));
        self.loops.push(self.owners.len());
        self.block(nested, result_type);
        self.loops.pop();
        self.locals.remove(name);
        self.code.push_str("}\n}\n");
    }
    /// `counts[k] = counts.get(k, d) + x`（也可以是 `-`、`*`）：值是数字，键、默认值与 `x`
    /// 都是变量或字面量（没有副作用，也碰不到这个字典）时，合成一次查找。
    fn map_update<'b>(
        &self,
        local: &str,
        index: &'b Expr,
        expr: &'b Expr,
    ) -> Option<(&'b str, &'b Expr, &'b Expr)> {
        let Some(Type::Map(_, elem)) = self.place_ty(local).container() else {
            return None;
        };
        let ty = elem.ty();
        let Expr::Binary(op, left, right) = expr else {
            return None;
        };
        let Expr::Method(path, method, args) = left.as_ref() else {
            return None;
        };
        let [key, default] = args.as_slice() else {
            return None;
        };
        let simple = |e: &Expr| matches!(e, Expr::Variable(_) | Expr::Int(_) | Expr::Float(_));
        (matches!(op.as_str(), "+" | "-" | "*")
            && matches!(ty, Type::Int | Type::Float)
            && path == local
            && method == "get"
            && simple(index)
            && simple(default)
            && simple(right)
            && format!("{key:?}") == format!("{index:?}")
            && self.ty(default) == ty
            && self.ty(right) == ty)
            .then_some((op.as_str(), default, right.as_ref()))
    }
    fn bytes_call(&self, expr: &Expr) -> Option<Expr> {
        let (receiver, method, args) = match expr {
            Expr::Method(path, method, args) => {
                let root = path.split('.').next().unwrap_or(path);
                if !self.locals.contains_key(root) || !is_bytes(self.place_ty(path)) {
                    return None;
                }
                (Expr::Borrow(path.clone(), false), method, args)
            }
            Expr::Chain(inner, method, args) if is_bytes(self.ty(inner)) => {
                ((**inner).clone(), method, args)
            }
            Expr::ElementMethod(path, index, field, method, args)
                if is_bytes(self.element_ty(path, field)) =>
            {
                (
                    Expr::BorrowIndex(path.clone(), index.clone(), field.clone(), false),
                    method,
                    args,
                )
            }
            _ => return None,
        };
        bytes_method(&self.module.functions, receiver, method, args)
    }
    fn string_call(&self, expr: &Expr) -> Option<Expr> {
        if let Expr::Index(path, index) = expr {
            let root = path.split('.').next().unwrap_or(path);
            if self.locals.contains_key(root) && is_string(self.place_ty(path)) {
                return Some(char_at(path, index));
            }
            return None;
        }
        if let Expr::Binary(op, item, other) = expr {
            if op != "in" && op != "not in" {
                // 构造（`Option.None`）要按另一边定类型，不是数字运算。
                let constructor = |e: &Expr| matches!(e, Expr::Record(..) | Expr::Variant(..));
                if op == "&&"
                    || op == "||"
                    || bitwise(op)
                    || constructor(item)
                    || constructor(other)
                {
                    return None;
                }
                return promote(expr, self.ty(item), self.ty(other));
            }
            return membership(expr, is_text(self.ty(item)));
        }
        if let Some(call) = self.bytes_call(expr) {
            return Some(call);
        }
        let (receiver, method, args) = match expr {
            Expr::Method(path, method, args) => {
                let root = path.split('.').next().unwrap_or(path);
                if !self.locals.contains_key(root) || !is_string(self.place_ty(path)) {
                    return None;
                }
                (Expr::Borrow(path.clone(), false), method, args)
            }
            Expr::Chain(inner, method, args) if is_string(self.ty(inner)) => {
                ((**inner).clone(), method, args)
            }
            Expr::ElementMethod(path, index, field, method, args)
                if is_string(self.element_ty(path, field)) =>
            {
                (
                    Expr::BorrowIndex(path.clone(), index.clone(), field.clone(), false),
                    method,
                    args,
                )
            }
            _ => return None,
        };
        string_method(&self.module.functions, receiver, method, args)
    }
    fn ty(&self, expr: &Expr) -> Type {
        if let Some(call) = self.string_call(expr) {
            return self.ty(&call);
        }
        match expr {
            // `f"{x:.2}"` 的带格式段：拼进 f-string 时是字符串。
            Expr::Formatted(..) => Type::Owned,
            Expr::Invoke(callee, _) => match self.value_type(callee) {
                Type::Func(id) | Type::FuncRef(id) => func_sig(id).1,
                _ => unreachable!("checked function value"),
            },
            Expr::Lambda(..)
                if self
                    .hint
                    .is_some_and(|(at, _)| at == expr as *const Expr as usize) =>
            {
                self.hint.expect("hint").1
            }
            Expr::Lambda(params, result, body) => Type::Func(
                self.lambda_type(params, *result, body, None)
                    .expect("checked anonymous fn"),
            ),
            Expr::FnRef(name) => {
                let function = &self.module.functions[name];
                Type::Func(func_type_full(
                    function.params.iter().map(|(_, ty)| *ty).collect(),
                    function.result,
                    function.error,
                ))
            }
            // 递归调用：函数体里没有自己的局部变量，类型直接从自己的签名取。
            Expr::Apply(path, _) if self.own.as_ref().is_some_and(|(own, ..)| own == path) => {
                func_sig(self.own.as_ref().expect("own").2).1
            }
            Expr::Apply(path, _) => match self.place_ty(path) {
                Type::Func(id) | Type::FuncRef(id) => func_sig(id).1,
                _ => unreachable!("checked function value"),
            },
            Expr::Method(..) if self.field_call(expr).is_some() => {
                let (path, _) = self.field_call(expr).expect("field call");
                match self.place_ty(&path) {
                    Type::Func(id) | Type::FuncRef(id) => func_sig(id).1,
                    _ => unreachable!("checked function field"),
                }
            }
            _ if self.record_call(expr) => {
                let (name, _) = self.call_target(expr).expect("record method");
                self.module.functions[&name].result
            }
            Expr::Record(_, _) | Expr::Variant(_, _, _) | Expr::If(..) | Expr::Match(..)
                if self
                    .hint
                    .is_some_and(|(at, _)| at == expr as *const Expr as usize) =>
            {
                self.hint.expect("hint").1
            }
            Expr::Record(_, _) | Expr::Variant(_, _, _) => {
                Type::Record(self.constructed(expr).expect("checked constructor"))
            }
            // 分支的值可能用到绑定：在带上第一个分支绑定的副本里求类型。
            Expr::Match(scrutinee, arms) => {
                let id = self.scrutinee_id(scrutinee);
                let (pattern, value) = &arms[0];
                let mut locals = self.locals.clone();
                for (name, _, ty) in self.arm_bindings(id, pattern, false) {
                    locals.insert(name, (String::new(), ty));
                }
                Generator {
                    module: self.module,
                    symbols: self.symbols,
                    locals,
                    owners: Vec::new(),
                    loops: Vec::new(),
                    proofs: ranges::Proofs::default(),
                    wrapping: false,
                    mutable_names: Default::default(),
                    call_targets: BTreeMap::new(),
                    code: String::new(),
                    next: 0,
                    result: self.result,
                    error: self.error,
                    containers: Containers::new(),
                    values: false,
                    records: Default::default(),
                    hint: None,
                    funcs: Default::default(),
                    lifted: Vec::new(),
                    prefix: String::new(),
                    own: None,
                    boxes: Default::default(),
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
            // 与检查器一致：整数是定长数组，其他按第一项的类型推断成列表。空列表的类型来自标注，
            // 这里不会用到。
            Expr::Array(values) if values.is_empty() => Type::ArrayInt(0),
            Expr::Array(values) => match self.ty(&values[0]) {
                Type::Int => Type::ArrayInt(values.len()),
                first => {
                    let first = match first.borrowable().map_or(first, |(owned, _)| owned) {
                        Type::ArrayInt(_) => Type::List(Elem::Int),
                        first => first,
                    };
                    crate::compiler::elem_from(first)
                        .map_or(Type::ArrayInt(values.len()), Type::List)
                }
            },
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
            Expr::SetLit(items) => Type::Map(elem_of(self.ty(&items[0])), SET),
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
            Expr::Tuple(items) => Type::Record(linked_tuple(
                items.iter().map(|item| self.ty(item)).collect(),
            )),
            Expr::Chain(receiver, method, _) if crate::compiler::call_field(method).is_some() => {
                let field = crate::compiler::call_field(method).expect("guarded");
                field_path(&self.module.structs, self.ty(receiver), field)
                    .expect("checked field path")
                    .1
            }
            Expr::Chain(receiver, method, _) => {
                method_signature(
                    crate::compiler::literal_list(self.ty(receiver), receiver),
                    method,
                )
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
            Expr::Call(name, args) if name == COPY => self.value_type(&args[0]),
            Expr::Call(name, args) => self.module.functions[&self.callee(name, args)].result,
            Expr::Try(inner) => self.ty(inner),
            Expr::Catch(inner, _, _) => self.ty(inner),
            Expr::Unary('!', _) => Type::Bool,
            Expr::Unary('f', _) => Type::Float,
            Expr::Unary(_, value) => numeric(self.ty(value)),
            Expr::Binary(op, left, _) if op == "+" && is_text(self.ty(left)) => Type::Owned,
            Expr::Binary(op, _, _) if bitwise(op) => Type::Int,
            Expr::Binary(op, left, _) => {
                if matches!(op.as_str(), "+" | "-" | "*" | "/" | "//" | "%") {
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
        if let Some(call) = self.string_call(expr) {
            let value = self.expr(&call);
            return format!("&{value}");
        }
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
        if let Type::Func(id) | Type::FuncRef(id) = ty {
            self.funcs.insert(id);
        }
        if let Some(id) = record_of(ty) {
            self.records.insert(id);
        }
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
        if let Some(borrow) = auto_borrow(expr).filter(|_| !matches!(expr, Expr::Variable(_))) {
            return self.borrowed(&borrow);
        }
        match expr {
            Expr::Variable(name) | Expr::Borrow(name, _) => self.pointer(name),
            Expr::BorrowIndex(_, _, _, _) => self.expr(expr),
            // 字面量与插值借给 `&string` 参数：先生成临时字符串，作用域结束时释放。
            Expr::Text(_) | Expr::Format(_) => {
                let value = self.expr(expr);
                format!("&{value}")
            }
            // 调用结果（临时结构体）借给 `&T` 参数：登记在 owners 里，语句结束时释放。
            _ => {
                let ty = self.ty(expr);
                let value = self.expr_as(expr, ty);
                format!("&{value}")
            }
        }
    }
    /// 按形参类型传值：owned 移交，标量按值，借用传指针。
    fn argument(&mut self, arg: &Expr, ty: Type) -> String {
        // 列表字面量借给 `&[T]`：按形参的元素类型生成临时列表，语句结束时释放。
        if let (Expr::Array(_), Type::ListRef(elem, false)) = (arg, ty) {
            let value = self.expr_as(arg, Type::List(elem));
            return format!("&{value}");
        }
        if owned(ty) {
            let value = self.expr_as(arg, ty);
            self.moved(&value, ty)
        } else if by_value(ty) {
            self.expr_as(arg, ty)
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
        // 要的是纯数据的值、给的是它的借用（`&Date` 参数）：复制一份，和数字一样。
        if let (Type::Record(id), Expr::Variable(name)) = (ty, expr) {
            if by_value(ty)
                && matches!(self.locals.get(name), Some((_, Type::RecordRef(other, _))) if *other == id)
            {
                let pointer = self.locals[name].0.clone();
                return self.scalar(&format!("*{pointer}"), ty);
            }
        }
        // 期望接口值、给的是具体类型：把值移到堆上，配上这个具体类型的虚表。
        if let Type::Record(interface) = ty {
            if self.is_interface(interface) {
                if let Some(Type::Record(concrete)) = self.known_type(expr) {
                    if concrete != interface {
                        let value = self.expr_as(expr, Type::Record(concrete));
                        let value = self.moved(&value, Type::Record(concrete));
                        self.boxes.insert((interface, concrete));
                        self.records.insert(interface);
                        self.records.insert(concrete);
                        self.values = true;
                        let local = self.scalar(&format!("box{interface}_{concrete}({value})"), ty);
                        self.owners.push((local.clone(), ty));
                        return local;
                    }
                }
            }
        }
        match (expr, ty) {
            (Expr::Array(values), Type::List(elem)) => {
                let local = self.scalar("{0}", ty);
                self.owners.push((local.clone(), ty));
                if !values.is_empty() {
                    self.code.push_str(&format!(
                        "reserve{}(&{local}, {});
",
                        list_suffix(elem),
                        values.len()
                    ));
                }
                for value in values {
                    // 元素也按目标类型生成：内层的 `[1, 2]` 是列表而不是定长数组。
                    let value = self.expr_as(value, elem.ty());
                    let value = self.moved(&value, elem.ty());
                    self.code
                        .push_str(&format!("push{}(&{local}, {value});\n", list_suffix(elem)));
                }
                local
            }
            // 元组按目标类型的各项生成：`(1, 2)` 赋给 `(f64, i64)` 时第一项按 f64 处理。
            (Expr::Tuple(items), Type::Record(id)) => {
                let fields = tuple_fields(id).expect("checked tuple");
                let local = self.scalar("{0}", ty);
                self.owners.push((local.clone(), ty));
                for (i, (item, field)) in items.iter().zip(fields).enumerate() {
                    let value = self.expr_as(item, field);
                    let value = self.moved(&value, field);
                    self.code.push_str(&format!("{local}.m{i} = {value};\n"));
                }
                local
            }
            (Expr::SetLit(items), Type::Map(key, SET)) => {
                let local = self.scalar("{0}", ty);
                self.owners.push((local.clone(), ty));
                for item in items {
                    let k = self.expr(item);
                    let k = if key == Elem::Str { format!("&{k}") } else { k };
                    self.code
                        .push_str(&format!("set{}(&{local}, {k}, 1);\n", ctype(ty)));
                }
                local
            }
            (Expr::MapLit(entries), Type::Map(key, value)) => {
                let local = self.scalar("{0}", ty);
                self.owners.push((local.clone(), ty));
                for (k, v) in entries {
                    let k = self.expr(k);
                    let k = if key == Elem::Str { format!("&{k}") } else { k };
                    let v = self.expr_as(v, value.ty());
                    let v = self.moved(&v, value.ty());
                    self.code
                        .push_str(&format!("set{}(&{local}, {k}, {v});\n", ctype(ty)));
                }
                local
            }
            // 匿名函数按目标函数类型确定参数与返回类型（参数类型可以省略）。
            // 泛型函数当值：按目标函数类型生成的副本。
            (Expr::FnRef(name), Type::Func(id) | Type::FuncRef(id))
                if !self.module.functions[name].generics.is_empty() =>
            {
                let instance = generic_value(&self.module.functions, name, id)
                    .expect("checked generic function value");
                self.function_value(&instance)
            }
            (Expr::Lambda(params, _, body), Type::Func(_) | Type::FuncRef(_)) => {
                self.lambda(params, body, ty)
            }
            // 泛型类型的值：记下目标类型，构造（含 if/match 的分支）按它确定类型实参。
            (_, Type::Record(id)) if instance_of(id).is_some() => {
                let outer = self.hint.replace((expr as *const Expr as usize, ty));
                let value = self.expr(expr);
                self.hint = outer;
                value
            }
            _ => self.expr(expr),
        }
    }
    /// 构造表达式的类型编号：泛型结构体与枚举按字段推断类型实参（检查器已确认推断得出）。
    fn constructed(&self, expr: &Expr) -> Option<u64> {
        constructed(&self.module.structs, expr, None, |value| match value {
            Expr::Record(..) | Expr::Variant(..) => self.constructed(value).map(Type::Record),
            _ => Some(self.ty(value)),
        })
        .ok()
    }
    /// 结构体类型（或它的借用）对应的声明。
    fn record_struct(&self, ty: Type) -> &crate::compiler::Struct {
        let (Type::Record(id) | Type::RecordRef(id, _)) = ty else {
            unreachable!("record type")
        };
        self.module
            .structs
            .iter()
            .find(|(name, _)| record_id(name) == id)
            .expect("checked struct")
            .1
    }
    /// 把已生成的 owned 值移交出去（实参、字段、返回值）；标量原样返回。
    fn moved(&self, value: &str, ty: Type) -> String {
        // 纯数据的结构体与枚举按值复制，原来的变量不清零。
        if owned(ty) && !by_value(ty) {
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
    /// 容器元素的指针：区间分析证明列表下标在范围内时直接取，否则经过带检查的 `at`
    /// （负下标换算与越界报错都在里面）。
    fn is_interface(&self, id: u64) -> bool {
        crate::compiler::struct_of(id, &self.module.structs).is_some_and(|(_, s)| s.interface)
    }
    fn slot(&self, container: Type, pointer: &str, key: &str, index: &Expr) -> String {
        if matches!(container, Type::List(_)) && self.proofs.list_index(index) {
            format!("(&({pointer})->data[{key}])")
        } else {
            format!("at{}({pointer}, {key})", ctype(container))
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
            "(*{}){}",
            self.slot(container, &pointer, &key, index),
            self.members(elem, field)
        )
    }
    fn expr(&mut self, expr: &Expr) -> String {
        if let Some(call) = self.string_call(expr) {
            return self.expr(&call);
        }
        match expr {
            // 带格式的段只在 f-string 里出现，由上面 `Expr::Format` 的分支处理；
            // 单独一个 `{x:.2}` 当值用时退化成那个值本身的文本。
            Expr::Formatted(inner, _) => {
                let ty = self.ty(inner);
                let value = self.expr(inner);
                self.text_of(value, ty)
            }
            Expr::FnRef(name) => self.function_value(name),
            Expr::Invoke(callee, args) => {
                let ty = self.value_type(callee);
                let (Type::Func(id) | Type::FuncRef(id)) = ty else {
                    unreachable!("checked function value")
                };
                let function = self.expr_as(callee, ty);
                let (params, result) = func_sig(id);
                let mut values = vec![format!("{function}.env")];
                for (arg, ty) in args.iter().zip(params) {
                    values.push(self.argument(arg, ty));
                }
                self.call_value(&format!("{function}.call({})", values.join(", ")), result)
            }
            Expr::Lambda(params, _, body) => {
                let ty = self.ty(expr);
                self.lambda(params, body, ty)
            }
            Expr::Apply(path, args) => self.apply(path, args),
            Expr::Method(..) if self.field_call(expr).is_some() => {
                let (path, args) = self.field_call(expr).expect("field call");
                self.apply(&path, &args)
            }
            // 结构体的方法：改写成普通调用 `Owner.m(接收者, ...)`。
            _ if self.record_call(expr) => {
                let (name, args) = self.call_target(expr).expect("record method");
                let result = self.module.functions[&name].result;
                let call = self.raw_call(&name, &args, None);
                self.call_value(&call, result)
            }
            Expr::Record(_, fields) => {
                let ty = self.ty(expr);
                let local = self.scalar("{0}", ty);
                self.owners.push((local.clone(), ty));
                let declaration = self.record_struct(ty).clone();
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
            // 字符串字段读出来是副本，结构体里的那份不动。
            Expr::Field(local, field) if self.ty(expr) == Type::Owned => {
                let target = self.field(local, field);
                self.temporary(&format!("copy(&{target})"))
            }
            // 移出列表、字典或结构体字段：原位置清零，整个结构体之后释放时不会重复释放。
            Expr::Field(local, field) if aggregate(self.ty(expr)) && !by_value(self.ty(expr)) => {
                let ty = self.ty(expr);
                let place = self.field(local, field);
                let value = self.scalar(&format!("{}(&{place})", take_fn(ty)), ty);
                self.owners.push((value.clone(), ty));
                value
            }
            Expr::Field(local, field) => self.scalar(&self.field(local, field), self.ty(expr)),
            // 移走结构体、列表、字典；纯数据的结构体与枚举按值复制，原变量不清零（走下面的分支）。
            Expr::Variable(name)
                if aggregate(self.locals[name].1) && !by_value(self.locals[name].1) =>
            {
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
                let slot = self.slot(container, &pointer, &key, index);
                // 字符串元素读出来是副本，列表里的那份不动。
                if self.ty(expr) == Type::Owned {
                    return self.temporary(&format!("copy({slot})"));
                }
                self.scalar(&format!("*{slot}"), self.ty(expr))
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
            Expr::Tuple(_) => {
                let ty = self.ty(expr);
                self.expr_as(expr, ty)
            }
            // 调用结果的字段：结果先放进临时变量（语句结束时释放），字符串字段复制一份。
            Expr::Chain(receiver, method, _) if crate::compiler::call_field(method).is_some() => {
                let field = crate::compiler::call_field(method).expect("guarded");
                let ty = self.ty(receiver);
                let value = self.expr_as(receiver, ty);
                let target = format!("{value}{}", self.members(ty, field));
                let found = self.ty(expr);
                if found == Type::Owned {
                    self.temporary(&format!("copy(&{target})"))
                } else {
                    self.scalar(&target, found)
                }
            }
            Expr::Chain(receiver, method, args) => {
                let ty = crate::compiler::literal_list(self.ty(receiver), receiver);
                let (_, params, result) = method_signature(ty, method).expect("checked method");
                // 调用结果是临时值，登记在 owners 里，语句结束时释放。
                let value = self.expr_as(receiver, ty);
                let mut values = vec![format!("&{value}")];
                for (arg, ty) in args.iter().zip(&params) {
                    values.push(self.argument(arg, *ty));
                }
                self.method_call(ty, method, &values, result)
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
            Expr::MapLit(_) | Expr::SetLit(_) => {
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
                    // `{x:.2}`：先按普通段把值变成文本，再按说明补白/截小数。
                    let (segment, spec) = match segment {
                        Expr::Formatted(inner, spec) => (&**inner, Some(spec.as_str())),
                        other => (other, None),
                    };
                    let part = match segment {
                        Expr::Text(text) => self.literal(text),
                        _ if compound(self.value_type(segment)) => {
                            let ty = self.value_type(segment);
                            let pointer = self.operand(segment);
                            self.values = true;
                            native_values::text_call(ty, &pointer)
                        }
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
                    // 带格式的段：按说明补白与截小数（生成的文本本身已经释放得了）。
                    let part = match spec {
                        Some(spec) => {
                            let parsed =
                                crate::compiler::spec::parse(spec).expect("checked format spec");
                            // C 的字符字面量：单引号包住，反斜杠与单引号转义。
                            let literal = |c: char| match c {
                                '\\' => "'\\\\'".to_owned(),
                                '\'' => "'\\''".to_owned(),
                                c => format!("'{c}'"),
                            };
                            format!(
                                "zam_format({part}, {}, {}, {}, {})",
                                literal(parsed.align.unwrap_or(if parsed.fill == '0' {
                                    '>'
                                } else {
                                    ' '
                                })),
                                literal(parsed.fill),
                                parsed.width,
                                parsed
                                    .precision
                                    .map_or("-1".to_owned(), |digits| digits.to_string())
                            )
                        }
                        None => part,
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
            Expr::Variant(_, variant, args) => {
                let ty = self.ty(expr);
                let Type::Record(id) = ty else {
                    unreachable!("variant type")
                };
                let local = self.scalar("{0}", ty);
                self.owners.push((local.clone(), ty));
                let (tag, first) = self.variant_of(id, variant);
                self.code.push_str(&format!("{local}.m0 = {tag};\n"));
                let fields = self.record_struct(ty).fields.clone();
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
                    let names = self.bind(&handle, id, pattern, false);
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
                    } else if *op == '-' && self.wrapping {
                        format!("(int64_t)(0 - (uint64_t){value})")
                    } else if *op == '-' {
                        format!("sub(0, {value})")
                    } else if *op == '~' {
                        format!("(~{value})")
                    } else if *op == 'f' {
                        format!("((double){value})")
                    } else {
                        format!("!{value}")
                    },
                    ty,
                )
            }
            Expr::Binary(op, left, right)
                if matches!(op.as_str(), "==" | "!=")
                    && self
                        .known_type(left)
                        .or(self.known_type(right))
                        .is_some_and(compound) =>
            {
                // 一边是自己看不出类型的构造（`Option.None`）时按另一边的类型生成。
                let ty = self
                    .known_type(left)
                    .or(self.known_type(right))
                    .expect("checked comparison");
                let a = self.operand_as(left, ty);
                let b = self.operand_as(right, ty);
                self.values = true;
                let test = native_values::equal(ty, &a, &b);
                self.scalar(
                    &if op == "==" { test } else { format!("!{test}") },
                    Type::Bool,
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
                    &match op.as_str() {
                        // `@wrapping`：除以零照 IEEE 754 得到 inf 或 NaN，不报错。
                        "/" if self.wrapping => format!("({a} / {b})"),
                        "//" if self.wrapping => format!("floor({a} / {b})"),
                        "%" if self.wrapping => format!("zam_fmod_ieee({a}, {b})"),
                        "/" => format!("zam_fdiv({a}, {b})"),
                        "//" => format!("zam_ffloordiv({a}, {b})"),
                        "%" => format!("zam_fmod({a}, {b})"),
                        _ => format!("({a} {op} {b})"),
                    },
                    ty,
                )
            }
            Expr::Binary(op, left, right) => {
                let ty = self.ty(expr);
                // A positive constant divisor can neither be zero nor overflow.
                let positive_constant = matches!(right.as_ref(), Expr::Int(value) if *value > 0);
                // Dynamic divisors retain the runtime's guarded 32-bit fast path.
                // Unsigned power-of-two reduction exposes a precise result range;
                // other constants retain signed reduction to avoid slower magic division.
                let unsigned_constant_divisor = matches!(right.as_ref(), Expr::Int(value) if *value > 0 && (*value as u64).is_power_of_two());
                let right_expr = right.as_ref();
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
                if matches!(op.as_str(), "//" | "%")
                    && unsigned_constant_divisor
                    && self.proofs.unsigned_division(expr)
                {
                    // Both operands are nonnegative and the divisor is positive:
                    // floor division and Python's remainder equal C's, and fit i64.
                    let c = if op == "//" { "/" } else { "%" };
                    return self.scalar(
                        &format!("(int64_t)((uint64_t){left} {c} (uint64_t){right})"),
                        ty,
                    );
                }
                if let (true, Expr::Int(divisor)) = (
                    matches!(op.as_str(), "//" | "%") && unsigned_constant_divisor,
                    right_expr,
                ) {
                    // 除数是 2 的正整数次幂：Python 的整除向下取整，等于算术右移；
                    // 余数与除数同号（非负），等于二进制补码的低位。都不会溢出，也不会除零。
                    // 支持的 C 编译器（GCC、Clang、MSVC）对有符号数右移都是算术移位。
                    return self.scalar(
                        &if op == "//" {
                            format!("(({left}) >> {})", divisor.trailing_zeros())
                        } else {
                            format!("(int64_t)((uint64_t)({left}) & {}u)", divisor - 1)
                        },
                        ty,
                    );
                }
                if self.wrapping && matches!(op.as_str(), "+" | "-" | "*") {
                    // 按二进制补码回绕：无符号运算在 C 里有定义，再转回有符号。
                    return self.scalar(
                        &format!("(int64_t)((uint64_t)({left}) {op} (uint64_t)({right}))"),
                        ty,
                    );
                }
                let function = match op.as_str() {
                    "+" if !self.proofs.arithmetic(expr) => Some("add"),
                    "-" if !self.proofs.arithmetic(expr) => Some("sub"),
                    "*" if !self.proofs.arithmetic(expr) => Some("mul"),
                    "//" if positive_constant => Some("zam_floordiv_k"),
                    "%" if positive_constant => Some("zam_mod_k"),
                    "//" => Some("zam_floordiv"),
                    "%" => Some("zam_mod"),
                    "<<" => Some("zam_shl"),
                    ">>" => Some("zam_shr"),
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
            // 借用的字符串在要 owned 的地方复制一份（检查器已放行）。
            Expr::Variable(name) if matches!(self.locals[name].1, Type::Shared | Type::Mutable) => {
                self.temporary(&format!("copy({})", self.pointer(name)))
            }
            Expr::Variable(name) => self.temporary(&format!("take({})", self.pointer(name))),
            Expr::Borrow(name, _) => self.pointer(name),
            // 泛型副本里的 `copy(v)` / `copy(&p.n)`：数值和布尔直接取值。
            Expr::Call(name, args) if name == COPY && by_value(self.value_type(&args[0])) => {
                let ty = self.value_type(&args[0]);
                let value = match &args[0] {
                    Expr::Borrow(path, _) => match path.split_once('.') {
                        Some((root, field)) => self.field(root, field),
                        None => self.locals[path].0.clone(),
                    },
                    other => self.expr(other),
                };
                self.scalar(&value, ty)
            }
            Expr::Call(name, args) if name == COPY => {
                let ty = self.value_type(&args[0]);
                let pointer = self.operand(&args[0]);
                self.values |= compound(ty);
                self.call_value(&native_values::copy(ty, &pointer), ty)
            }
            Expr::Call(name, args) => {
                let name = &self.callee(name, args);
                let result = self.module.functions[name].result;
                let symbol = self
                    .call_targets
                    .get(&(expr as *const Expr as usize))
                    .cloned();
                let call = self.raw_call(name, args, symbol.as_deref());
                self.call_value(&call, result)
            }
            Expr::Try(inner) => {
                let (call, result, error) = match self.value_call(inner) {
                    Some((call, result, error)) => (call, result, Some(error)),
                    None => {
                        let (name, args) = self.call_target(inner).expect("checked ? operand");
                        let function = &self.module.functions[&name];
                        let (result, error) = (function.result, function.error);
                        (self.raw_call(&name, &args, None), result, error)
                    }
                };
                let structure = fallible_name(result, error);
                let temp = format!("v{}", self.next);
                self.next += 1;
                self.code
                    .push_str(&format!("{structure} {temp} = {call};\n"));
                // 失败：先把当前作用域的值都释放掉，再构造同样形状的错误结果返回。
                self.code.push_str(&format!("if (!{temp}.ok) {{\n"));
                let outer = fallible_err(self.result, self.error);
                let take = take_fn(error.expect("checked fallible call"));
                self.cleanup();
                self.code
                    .push_str(&format!("return {outer}({take}(&{temp}.error));\n}}\n"));
                if result == Type::Unit {
                    return String::new();
                }
                self.result_value(&temp, result)
            }
            Expr::Catch(inner, binding, block) => {
                let (call, result, error) = match self.value_call(inner) {
                    Some(found) => found,
                    None => {
                        let (name, args) = self.call_target(inner).expect("checked catch operand");
                        let function = &self.module.functions[&name];
                        let (result, error) = (
                            function.result,
                            function.error.expect("checked fallible call"),
                        );
                        (self.raw_call(&name, &args, None), result, error)
                    }
                };
                let structure = fallible_name(result, Some(error));
                let temp = format!("v{}", self.next);
                self.next += 1;
                self.code
                    .push_str(&format!("{structure} {temp} = {call};\n"));
                self.code.push_str(&format!("if (!{temp}.ok) {{\n"));
                let local = format!("v{}", self.next);
                self.next += 1;
                let previous = self.locals.insert(binding.clone(), (local.clone(), error));
                self.code.push_str(&format!(
                    "{} {local} = {}(&{temp}.error);\n",
                    ctype(error),
                    take_fn(error)
                ));
                note(&mut self.containers, error);
                if let Some(id) = record_of(error) {
                    self.records.insert(id);
                }
                self.owners.push((local, error));
                // `f() catch e: default`：出错时把回退值写进结果，再释放错误值（规则与检查器相同）。
                let fallback = crate::compiler::catch_fallback(block).filter(|value| {
                    result != Type::Unit
                        && (crate::compiler::empty_fallback(value, result)
                            || crate::compiler::fallback_fits(self.ty(value), result))
                });
                let tail = crate::compiler::catch_tail(block).filter(|_| result != Type::Unit);
                let returned = if let Some((prefix, value, _)) = tail {
                    let prefix: Block = prefix.to_vec();
                    self.block_tail(&prefix, self.result, Some((value, result, &temp)))
                } else if let Some(value) = fallback {
                    // 回退值的临时变量声明在 `if` 块里，也要在块里释放（移走后只剩空壳）。
                    let scope = self.owners.len();
                    let value = self.expr_as(value, result);
                    let value = self.moved(&value, result);
                    self.code
                        .push_str(&format!("{temp}.ok = 1;\n{temp}.value = {value};\n"));
                    for (name, ty) in self.owners.split_off(scope) {
                        self.code.push_str(&format!("{}(&{name});\n", drop_fn(ty)));
                    }
                    false
                } else {
                    // 有成功值时代码块在所有路径上离开（检查器保证）；没有成功值时可能走到块尾，
                    // block 会释放块里的值，出块后接着执行后面的语句。
                    self.block(block, self.result)
                };
                let (local, ty) = self.owners.pop().expect("catch binding");
                // 块走到底（没有成功值的调用或单独成句）时，错误值在这里释放。
                if !returned {
                    self.code.push_str(&format!("{}(&{local});\n", drop_fn(ty)));
                }
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
    fn arm_bindings(&self, id: u64, pattern: &Pattern, owned: bool) -> Vec<(String, usize, Type)> {
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
                let bound = if by_value(ty) || owned {
                    ty
                } else {
                    borrow_of(ty, false).expect("owned payload")
                };
                (binding.clone(), first + j, bound)
            })
            .collect()
    }
    /// 被匹配的是不是位置（变量、字段、容器元素）；不是位置的是临时值。与检查器的 `scrutinee` 一致。
    fn place_scrutinee(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Variable(_) | Expr::Field(_, _) => true,
            Expr::Index(path, _) | Expr::IndexField(path, _, _) => {
                self.place_ty(path).container().is_some()
            }
            _ => false,
        }
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
    /// 分支绑定的变量。`owned` 时（匹配的是临时值）把载荷取出来交给绑定：原位置清零，
    /// 绑定登记在 owners 里，分支结束时释放（移走了就是空值）。
    fn bind(&mut self, handle: &str, id: u64, pattern: &Pattern, owned: bool) -> Vec<String> {
        let mut names = Vec::new();
        for (name, member, ty) in self.arm_bindings(id, pattern, owned) {
            let access = format!("{handle}->m{member}");
            let value = if ty.is_reference() {
                self.scalar(&format!("&{access}"), ty)
            } else if owned && !by_value(ty) {
                let value = self.scalar(&format!("{}(&{access})", take_fn(ty)), ty);
                self.owners.push((value.clone(), ty));
                value
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
        // 结构体或嵌套容器列表的切片要深复制元素，复制函数在复合值帮助函数里。
        if let (Type::List(elem), "slice") = (container, method) {
            if !elem.comparable() {
                self.values = true;
            }
        }
        let name = match method {
            "contains" => "has",
            "append" => "push",
            "get" => "getor",
            "$pop_at" => "removeat",
            "remove" if list => "removeval",
            "remove" | "discard" => "del",
            "index_of" => "index",
            "sort_by" => "sortby",
            "$at" => "valat",
            "$put" => "put",
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
    /// 只读使用一个值时它的 owned 类型：借用换成被借用的类型。
    fn value_type(&self, expr: &Expr) -> Type {
        let ty = match expr {
            Expr::Borrow(path, _) => self.place_ty(path),
            Expr::BorrowIndex(path, _, field, _) => self.element_ty(path, field),
            _ => self.ty(expr),
        };
        ty.borrowable().map_or(ty, |(owned, _)| owned)
    }
    /// 只读使用时的类型；泛型类型的构造推断不出类型实参时是 None。
    fn known_type(&self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Record(..) | Expr::Variant(..) => self.constructed(expr).map(Type::Record),
            _ => Some(self.value_type(expr)),
        }
    }
    /// 同 `operand`，推断不出类型的构造按 `ty` 生成。
    fn operand_as(&mut self, expr: &Expr, ty: Type) -> String {
        if self.known_type(expr).is_some() {
            return self.operand(expr);
        }
        let value = self.expr_as(expr, ty);
        format!("&{value}")
    }
    /// 只读使用一个值的指针：变量、字段、元素直接取地址，其他表达式先求值成临时值。
    fn operand(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Variable(name) | Expr::Borrow(name, _) => self.pointer(name),
            Expr::Field(local, field) => format!("&{}", self.field(local, field)),
            Expr::Index(local, index) if self.place_ty(local).container().is_some() => self.expr(
                &Expr::BorrowIndex(local.clone(), index.clone(), String::new(), false),
            ),
            Expr::IndexField(local, index, field) => self.expr(&Expr::BorrowIndex(
                local.clone(),
                index.clone(),
                field.clone(),
                false,
            )),
            Expr::BorrowIndex(_, _, _, _) => self.expr(expr),
            _ => {
                let ty = self.value_type(expr);
                let value = self.expr_as(expr, ty);
                format!("&{value}")
            }
        }
    }
    /// 被调函数的实际名字：`abs`/`min`/`max` 按第一个参数的类型选版本。
    /// 被调函数的实际名字：`abs`/`min`/`max` 按第一个参数的类型选版本，泛型函数按实参类型选副本。
    fn callee(&self, name: &str, args: &[Expr]) -> String {
        // `Maybe.Nothing` 这类实参自己推断不出类型，和检查器一样不参与推断。
        let known = |arg: &&Expr| {
            !matches!(arg, Expr::Record(..) | Expr::Variant(..)) || self.constructed(arg).is_some()
        };
        let typed = |arg: &&Expr| match arg {
            Expr::Lambda(params, _, _) => params.iter().all(|(_, ty)| ty.is_some()),
            _ => known(arg),
        };
        let float_power = name == "std/core:pow"
            && args
                .iter()
                .any(|arg| typed(&arg) && self.ty(arg) == Type::Float);
        let first = if float_power {
            Some(Type::Float)
        } else {
            args.first().filter(typed).map(|arg| self.ty(arg))
        };
        let name = overload(name, first);
        let actuals: Vec<Option<Type>> = args
            .iter()
            .map(|arg| typed(&arg).then(|| self.value_type(arg)))
            .collect();
        let late = |index: usize, known: Vec<Type>| {
            let Expr::Lambda(params, result, body) = &args[index] else {
                return None;
            };
            self.lambda_type(params, *result, body, Some(known))
                .map(Type::FuncRef)
        };
        instance_with(&self.module.functions, name, &actuals, &late).expect("checked generic call")
    }
    /// 调用的目标函数与实参：普通调用原样返回，结构体的方法改写成 `Owner.m(接收者, ...)`。
    fn call_target(&self, expr: &Expr) -> Option<(String, Vec<Expr>)> {
        if let Some(Expr::Call(name, args)) = self.string_call(expr) {
            return Some((self.callee(&name, &args), args));
        }
        let (receiver, ty, method, args) = match expr {
            Expr::Call(name, args) => return Some((self.callee(name, args), args.clone())),
            Expr::Method(path, method, args) => {
                let ty = self.place_ty(path);
                (Receiver::Path(path, ty), ty, method, args)
            }
            Expr::ElementMethod(path, index, field, method, args) => (
                Receiver::Element(path, index, field),
                self.element_ty(path, field),
                method,
                args,
            ),
            Expr::Chain(_, method, _) if crate::compiler::call_field(method).is_some() => {
                return None
            }
            Expr::Chain(receiver, method, args) => {
                (Receiver::Temp(receiver), self.ty(receiver), method, args)
            }
            _ => return None,
        };
        // 方法自己的类型参数（`fn map[U](&self, f: fn(T) -> U)`）要从实参推断，和检查器一致。
        // 只有模板真的带了自己的类型参数才去估实参类型：这一步有副作用（会走到构造器的
        // 期望类型推断上），普通方法不该多算一遍。匿名函数实参也跳过——它的类型由期望
        // 类型决定，生成器单独取会走到 `lambda_type` 的断言上。
        // 只和方法所属的那个结构体比：别的类型（如 `Option[T]`）也叫 T 的类型参数不算。
        let (name, callee) = {
            let own = self
                .module
                .functions
                .iter()
                .filter_map(|(fname, function)| {
                    let (owner, m) = fname.rsplit_once('.')?;
                    (m == method).then_some((owner, function))
                })
                .any(|(owner, function)| {
                    let owner_generics = self
                        .module
                        .structs
                        .get(owner)
                        .map_or(&[][..], |s| s.generics.as_slice());
                    function
                        .generics
                        .iter()
                        .any(|g| !owner_generics.contains(g))
                });
            let actuals: Vec<Option<Type>> = if own {
                args.iter()
                    .map(|arg| match arg {
                        Expr::Lambda(..) => None,
                        // 关键字实参（`stretchy=true`）在归位之前不参与推断，和检查器一致。
                        _ if crate::compiler::keyword_arg(arg).is_some() => None,
                        _ => self.value_type(arg).into(),
                    })
                    .collect()
            } else {
                Vec::new()
            };
            record_method_with(
                &self.module.structs,
                &self.module.functions,
                ty,
                method,
                &actuals,
            )
            .ok()??
        };
        Some((name, method_args(receiver, callee, method, args).ok()?))
    }
    fn record_call(&self, expr: &Expr) -> bool {
        matches!(
            expr,
            Expr::Method(..) | Expr::ElementMethod(..) | Expr::Chain(..)
        ) && self.call_target(expr).is_some()
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
        self.block_tail(body, result_type, None)
    }
    /// 代码块；`tail` 是 catch 块最后那个当作值的表达式（值, 类型, 结果变量）：在块里的变量
    /// 释放之前求值，写进结果变量。
    fn block_tail(
        &mut self,
        body: &Block,
        result_type: Type,
        tail: Option<(&Expr, Type, &str)>,
    ) -> bool {
        let outer = self.locals.clone();
        let scope_start = self.owners.len();
        let mut returned = false;
        let mut skip = false;
        for (at, (instruction, _)) in body.iter().enumerate() {
            if std::mem::take(&mut skip) {
                continue;
            }
            if let Some((name, source, kind, nested)) = self.streamed(body, at) {
                self.stream(name, &source, kind, nested, result_type);
                skip = true;
                continue;
            }
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
                    let owned = !self.place_scrutinee(scrutinee);
                    let handle = self.scrutinee(scrutinee, id);
                    let mut all = true;
                    for (index, (pattern, body)) in arms.iter().enumerate() {
                        let opener =
                            self.arm_opener(&handle, id, pattern, index, index + 1 == arms.len());
                        self.code.push_str(&opener);
                        let owners = self.owners.len();
                        let names = self.bind(&handle, id, pattern, owned);
                        let returns = self.block(body, result_type);
                        if !returns {
                            for (name, ty) in &self.owners[owners..] {
                                self.code.push_str(&format!("{}(&{name});\n", drop_fn(*ty)));
                            }
                        }
                        self.owners.truncate(owners);
                        all &= returns;
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
                // `let x = &place`：局部变量存的是指针。
                Instruction::Let(
                    name,
                    _,
                    None,
                    expr @ (Expr::Borrow(_, false) | Expr::BorrowIndex(_, _, _, false)),
                ) => {
                    let owned = match expr {
                        Expr::Borrow(path, _) => self.place_ty(path),
                        Expr::BorrowIndex(path, _, field, _) => self.element_ty(path, field),
                        _ => unreachable!("borrow expression"),
                    };
                    let owned = owned.borrowable().map_or(owned, |(owned, _)| owned);
                    let ty = borrow_of(owned, false).expect("checked local borrow");
                    let pointer = match expr {
                        Expr::Borrow(path, _) => {
                            let pointer = self.pointer(path);
                            self.scalar(&pointer, ty)
                        }
                        _ => self.expr(expr),
                    };
                    self.locals.insert(name.clone(), (pointer, ty));
                }
                Instruction::Let(name, _, annotation, expr) => {
                    let ty = match (annotation, loop_list(name, expr)) {
                        (Some(ty), _) => *ty,
                        (None, Some(first)) => Type::List(
                            elem_from(crate::compiler::literal_list(self.ty(first), first))
                                .expect("checked loop list"),
                        ),
                        (None, None) => self.ty(expr),
                    };
                    // 函数里定义的 fn：函数体可能调用自己，递归在 `lambda_named` 里直接
                    // 生成对它自己静态函数的调用，不经过这里的局部变量。
                    let value = match expr {
                        Expr::Lambda(params, _, body) => {
                            self.lambda_named(params, body, ty, Some(name))
                        }
                        _ => self.expr_as(expr, ty),
                    };
                    self.locals.insert(name.clone(), (value, ty));
                }
                // `for ch in s:`：在 UTF-8 字节上逐个切出字符。每个字符是一个临时字符串：continue 与
                // 正常的下一轮在 for 的递增部分释放，break 在循环后释放，return 经 owners 释放（drop 可重复）。
                Instruction::Each(name, second, path, _, nested)
                    if is_string(self.place_ty(path)) =>
                {
                    let source = self.scalar(&self.pointer(path), Type::Shared);
                    let [ch, at, end, index] = [0, 1, 2, 3].map(|k| format!("v{}", self.next + k));
                    self.next += 4;
                    self.code.push_str(&format!(
                        "S {ch} = {{0}}; size_t {at}, {end} = 0, {index};\n\
                         for ({at} = 0, {index} = 0; {at} < {source}->len; drop(&{ch}), {at} = {end}, {index}++) {{\n\
                         {end} = {at} + 1;\n\
                         while ({end} < {source}->len && ({source}->data[{end}] & 0xC0) == 0x80) {end}++;\n\
                         {ch} = own({source}->data + {at}, {end} - {at});\n"
                    ));
                    self.owners.push((ch.clone(), Type::Owned));
                    let names: Vec<&String> = std::iter::once(name).chain(second).collect();
                    let mut items = vec![(format!("(&{ch})"), Type::Shared)];
                    if second.is_some() {
                        items.insert(0, (format!("(int64_t){index}"), Type::Int));
                    }
                    for (name, item) in names.iter().zip(items) {
                        self.locals.insert((*name).clone(), item);
                    }
                    self.loops.push(self.owners.len());
                    self.block(nested, result_type);
                    self.loops.pop();
                    for name in names {
                        self.locals.remove(name);
                    }
                    self.owners.pop();
                    self.code.push_str(&format!("}}\ndrop(&{ch});\n"));
                }
                Instruction::Each(name, second, path, mutable, nested) => {
                    let container = self.place_ty(path).container().expect("checked loop");
                    let borrowed = match container {
                        Type::List(elem) => Type::ListRef(elem, *mutable),
                        Type::Map(key, value) => Type::MapRef(key, value, *mutable),
                        _ => unreachable!(),
                    };
                    // 循环体不能修改被遍历的容器（检查器冻结了根绑定），长度与存储整轮不变。
                    let handle = self.scalar(&self.pointer(path), borrowed);
                    let index = format!("v{}", self.next);
                    self.next += 1;
                    self.code.push_str(&format!(
                        "for (size_t {index} = 0; {index} < {handle}->len; {index}++) {{\n"
                    ));
                    // 列表取（下标与）元素；字典按插入顺序取键（与值），跳过已删除的项。
                    let accesses = match container {
                        Type::List(_) => vec![
                            format!("(int64_t){index}"),
                            format!("{handle}->data[{index}]"),
                        ],
                        Type::Map(_, _) => {
                            self.code.push_str(&format!(
                                "if (!{handle}->entries[{index}].live) continue;\n"
                            ));
                            vec![
                                format!("{handle}->entries[{index}].key"),
                                format!("{handle}->entries[{index}].value"),
                            ]
                        }
                        _ => unreachable!(),
                    };
                    let types =
                        each_bindings(container, second.is_some(), *mutable).expect("checked loop");
                    // 一个名字的列表遍历只绑定元素；其余情况按顺序对应。
                    let accesses = if matches!(container, Type::List(_)) && second.is_none() {
                        &accesses[1..]
                    } else {
                        &accesses[..]
                    };
                    let names: Vec<&String> = std::iter::once(name).chain(second).collect();
                    for ((name, (ty, alias)), access) in names.iter().zip(types).zip(accesses) {
                        let item = if alias {
                            // 可赋值的标量别名：读写都经过元素的地址。
                            let pointer = format!("v{}", self.next);
                            self.next += 1;
                            self.code
                                .push_str(&format!("{} *{pointer} = &{access};\n", ctype(ty)));
                            format!("(*{pointer})")
                        } else if ty.is_reference() {
                            self.scalar(&format!("&{access}"), ty)
                        } else {
                            self.scalar(access, ty)
                        };
                        self.locals.insert((*name).clone(), (item, ty));
                    }
                    self.loops.push(self.owners.len());
                    self.block(nested, result_type);
                    self.loops.pop();
                    for name in names {
                        self.locals.remove(name);
                    }
                    self.code.push_str("}\n");
                }
                Instruction::AssignIndex(local, index, expr)
                    if self.map_update(local, index, expr).is_some() =>
                {
                    let (op, default, right) =
                        self.map_update(local, index, expr).expect("guarded");
                    let container = self.place_ty(local).container().expect("guarded");
                    let ty = self.ty(right);
                    let key = self.key(index, container);
                    let default = self.expr(default);
                    let right = self.expr(right);
                    let pointer = self.pointer(local);
                    let slot = format!("v{}", self.next);
                    self.next += 1;
                    let current = format!("(*{slot})");
                    let value = if ty == Type::Float {
                        format!("({current} {op} {right})")
                    } else if self.wrapping {
                        format!("(int64_t)((uint64_t){current} {op} (uint64_t)({right}))")
                    } else {
                        let function = match op {
                            "+" => "add",
                            "-" => "sub",
                            _ => "mul",
                        };
                        format!("{function}({current}, {right})")
                    };
                    self.code.push_str(&format!(
                        "{} *{slot} = slot{}({pointer}, {key}, {default});\n*{slot} = {value};\n",
                        ctype(ty),
                        ctype(container)
                    ));
                }
                Instruction::AssignIndex(local, index, expr)
                    if self.place_ty(local).container().is_some() =>
                {
                    let container = self.place_ty(local).container().expect("guarded");
                    let key = self.key(index, container);
                    let (Type::List(elem) | Type::Map(_, elem)) = container else {
                        unreachable!("container type")
                    };
                    let value = self.expr_as(expr, elem.ty());
                    let pointer = self.pointer(local);
                    let c = ctype(container);
                    let slot = self.slot(container, &pointer, &key, index);
                    self.code.push_str(&match container {
                        Type::Map(_, elem) => format!(
                            "set{c}({pointer}, {key}, {});\n",
                            self.moved(&value, elem.ty())
                        ),
                        Type::List(elem) if owned(elem.ty()) => format!(
                            "{{ {} *slot = {slot}; {}(slot); *slot = {}(&{value}); }}\n",
                            ctype(elem.ty()),
                            drop_fn(elem.ty()),
                            take_fn(elem.ty())
                        ),
                        Type::List(elem) => format!("*{slot} = {};\n", elem_store(elem, &value)),
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
                        "(*{}){}",
                        self.slot(container, &pointer, &key, index),
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
                Instruction::Print(expr) if compound(self.value_type(expr)) => {
                    let ty = self.value_type(expr);
                    let pointer = self.operand(expr);
                    self.values = true;
                    let text = self.temporary(&native_values::text_call(ty, &pointer));
                    self.code.push_str(&format!("print(&{text});\n"));
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
                    // `env.exit(n)` 不会返回：代码块到此结束，后面也不用释放。
                    if terminates(expr) {
                        returned = true;
                    }
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
                        let ok = fallible_ok(result_type, self.error);
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
                Instruction::Unpack(names, _, expr) => {
                    let ty = self.ty(expr);
                    let Type::Record(id) = ty else {
                        unreachable!("checked tuple")
                    };
                    let fields = tuple_fields(id).expect("checked tuple");
                    // 元组是登记在 owners 里的临时值：各项移进新变量后清零，它自己的释放什么也不做。
                    let value = self.expr_as(expr, ty);
                    for (i, (name, field)) in names.iter().zip(fields).enumerate() {
                        if name == "_" {
                            if owned(field) {
                                self.code
                                    .push_str(&format!("{}(&{value}.m{i});\n", drop_fn(field)));
                            }
                            continue;
                        }
                        let local = self.scalar(&format!("{value}.m{i}"), field);
                        if owned(field) {
                            self.owners.push((local.clone(), field));
                        }
                        self.locals.insert(name.clone(), (local, field));
                    }
                    self.code
                        .push_str(&format!("memset(&{value}, 0, sizeof {value});\n"));
                }
                Instruction::Fail(expr) => {
                    let error = self.error.expect("checked fail");
                    let value = self.expr_as(expr, error);
                    self.code.push_str(&format!(
                        "{} failure = {}(&{value});\n",
                        ctype(error),
                        take_fn(error)
                    ));
                    self.cleanup();
                    self.code.push_str(&format!(
                        "return {}(failure);\n",
                        fallible_err(self.result, self.error)
                    ));
                    returned = true;
                }
            }
        }
        if let Some((value, ty, target)) = tail {
            if !returned {
                let value = self.expr_as(value, ty);
                let value = self.moved(&value, ty);
                self.code
                    .push_str(&format!("{target}.ok = 1;\n{target}.value = {value};\n"));
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
                matches!(op.as_str(), "+" | "-" | "*" | "//" | "%")
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
                | Instruction::Each(_, _, _, _, nested) => Self::loop_mutations(nested, mutated),
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
/// 函数里可能被改写的名字：`let mut`、可变解构，以及不是只读借用的参数。
fn mutable_names(function: &crate::compiler::Function) -> std::collections::BTreeSet<String> {
    fn walk(body: &Block, names: &mut std::collections::BTreeSet<String>) {
        for (instruction, _) in body {
            match instruction {
                Instruction::Let(name, true, _, _) => {
                    names.insert(name.clone());
                }
                Instruction::Unpack(items, true, _) => names.extend(items.iter().cloned()),
                Instruction::If(_, yes, no) => {
                    walk(yes, names);
                    walk(no, names);
                }
                Instruction::While(_, nested)
                | Instruction::For(_, _, _, nested)
                | Instruction::Each(_, _, _, _, nested) => walk(nested, names),
                Instruction::Match(_, arms) => {
                    for (_, arm) in arms {
                        walk(arm, names);
                    }
                }
                _ => {}
            }
        }
    }
    let mut names: std::collections::BTreeSet<String> = function
        .params
        .iter()
        .filter(|(_, ty)| *ty != Type::Shared)
        .map(|(name, _)| name.clone())
        .collect();
    walk(&function.body, &mut names);
    names
}
fn visit_exprs<'a>(body: &'a Block, visit: &mut impl FnMut(&'a Expr)) {
    fn expr<'a>(value: &'a Expr, visit: &mut impl FnMut(&'a Expr)) {
        visit(value);
        match value {
            Expr::Lambda(_, _, body) => visit_exprs(body, visit),
            Expr::Apply(_, args) => {
                for arg in args {
                    expr(arg, visit);
                }
            }
            Expr::Invoke(callee, args) => {
                expr(callee, visit);
                for arg in args {
                    expr(arg, visit);
                }
            }
            Expr::Call(_, args)
            | Expr::Method(_, _, args)
            | Expr::Variant(_, _, args)
            | Expr::Array(args)
            | Expr::Tuple(args)
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
            Expr::ElementMethod(_, index, _, _, args) | Expr::Chain(index, _, args) => {
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
            Expr::SetLit(items) => {
                for item in items {
                    expr(item, visit);
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
            | Instruction::Unpack(_, _, value)
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
            Instruction::Each(_, _, _, _, body) => visit_exprs(body, visit),
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
        // 泛型模板不生成，只生成按类型生成的副本。
        if name.split_once(':').unwrap().0 != unit || !function.generics.is_empty() {
            continue;
        }
        let proofs = ranges::analyze(function);
        let mut targets = BTreeMap::new();
        visit_exprs(&function.body, &mut |expr| {
            let Expr::Call(callee, _) = expr else {
                return;
            };
            let Some(target) = module.functions.get(callee) else {
                return;
            };
            if callee.split_once(':').unwrap().0 != unit
                || cyclic.contains(callee)
                || target.result != Type::Int
                || target.error.is_some()
                || target.external
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
#[cfg(test)]
fn generate(module: &Module, entry: &str, unit: &str) -> String {
    generate_with(module, entry, unit, &[], &[])
}

/// `tests` 不为空时生成测试程序：入口模块的 C `main` 按第一个参数的名字只运行那一个测试函数。
pub fn generate_with(
    module: &Module,
    entry: &str,
    unit: &str,
    tests: &[String],
    headers: &[String],
) -> String {
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
                fallible_name(function.result, function.error)
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
    // 用到 C 句柄或 extern fn 的目标文件才包含 `[native] headers` 里的头文件。
    let mut needs_headers = false;
    let mut used_externs = std::collections::BTreeSet::new();
    for function in module.functions.values() {
        if function.generics.is_empty() {
            visit_exprs(&function.body, &mut |expr| {
                if let Expr::Call(name, _) | Expr::FnRef(name) = expr {
                    if module.functions.get(name).is_some_and(|f| f.external) {
                        used_externs.insert(name.clone());
                    }
                }
            });
        }
    }
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
    // 装箱时为具体类型生成转发函数，要调用它与接口同名的方法：程序里有接口时把这些方法的原型都带上。
    let dispatched: std::collections::BTreeSet<&str> = module
        .functions
        .iter()
        .filter(|(_, f)| f.dispatch)
        .filter_map(|(name, _)| name.rsplit_once('.').map(|(_, m)| m))
        .collect();
    for (name, function) in &module.functions {
        if !function.dispatch
            && !function.builtin
            && function.generics.is_empty()
            && name
                .rsplit_once('.')
                .is_some_and(|(owner, m)| owner.contains(':') && dispatched.contains(m))
        {
            needed.insert(name.clone());
        }
    }
    for (name, function) in &module.functions {
        if name.split_once(':').unwrap().0 == unit && function.generics.is_empty() {
            needed.insert(name.clone());
            visit_exprs(&function.body, &mut |expr| match expr {
                // `copy` 不在函数表里；重载的内建函数两个版本都带上，按参数类型调用其中一个。
                // `s[i]` 在生成时改写成 string.char_at：一并带上。
                Expr::Index(..) => {
                    if module.functions.contains_key("std/string:char_at") {
                        needed.insert("std/string:char_at".to_owned());
                    }
                }
                // `x in s` 在生成时改写成 s.contains(x)：字符串的 contains 一并带上。
                Expr::Binary(op, _, _) if op == "in" || op == "not in" => {
                    if module.functions.contains_key("std/string:contains") {
                        needed.insert("std/string:contains".to_owned());
                    }
                }
                Expr::Call(name, _) if name != COPY => {
                    for ty in [Type::Int, Type::Float, Type::Owned] {
                        let callee = overload(name, Some(ty));
                        if module
                            .functions
                            .get(callee)
                            .is_some_and(|f| f.generics.is_empty())
                        {
                            needed.insert(callee.to_owned());
                        }
                    }
                    // 泛型函数：调用的是按类型生成的副本，同名模板的副本都带上原型。
                    let prefix = format!("{name}[");
                    needed.extend(
                        module
                            .functions
                            .keys()
                            .filter(|key| key.starts_with(&prefix))
                            .cloned(),
                    );
                }
                // 方法调用要到生成时才知道接收者类型：同名的结构体方法都带上原型。
                Expr::Method(_, method, args)
                | Expr::ElementMethod(_, _, _, method, args)
                | Expr::Chain(_, method, args) => {
                    // 字符串上的方法就是 std/string 的同名函数：把它的 C 实现一并带上
                    // （按实参个数选的版本，如 find_from，见 compiler::string_function）。
                    let string = crate::compiler::string_function(method, args.len());
                    if module.functions.contains_key(&string) {
                        needed.insert(string);
                    }
                    let bytes = format!("std/bytes:{method}");
                    if module.functions.contains_key(&bytes) {
                        needed.insert(bytes);
                    }
                    // 泛型类型的方法调用的是副本（`Stack.push[Int]`），模板本身不生成。
                    let suffix = format!(".{method}");
                    let instance = format!(".{method}[");
                    needed.extend(
                        module
                            .functions
                            .iter()
                            .filter(|(name, function)| {
                                (name.ends_with(&suffix) || name.contains(&instance))
                                    && !function.builtin
                                    && function.generics.is_empty()
                            })
                            .map(|(name, _)| name.clone()),
                    );
                }
                // 泛型函数当值时用的是按类型生成的副本，模板本身不生成。
                Expr::FnRef(name) if !module.functions[name].generics.is_empty() => {
                    let prefix = format!("{name}[");
                    needed.extend(
                        module
                            .functions
                            .keys()
                            .filter(|key| key.starts_with(&prefix))
                            .cloned(),
                    );
                }
                Expr::FnRef(name) => {
                    needed.insert(name.clone());
                }
                // 泛型模板不生成；用到的实例经生成器登记的类型带进来。
                Expr::Record(name, _) | Expr::Variant(name, _, _)
                    if module.structs[name].generics.is_empty() =>
                {
                    records.insert(name.clone());
                }
                _ => {}
            });
        }
    }
    if entry.split_once(':').unwrap().0 == unit {
        if tests.is_empty() {
            needed.insert(entry.into());
        } else {
            needed.extend(tests.iter().cloned());
        }
    }
    for name in &needed {
        let function = &module.functions[name];
        for ty in function
            .params
            .iter()
            .map(|(_, ty)| ty)
            .chain(std::iter::once(&function.result))
            .chain(function.error.iter())
        {
            if let Some(id) = record_of(*ty) {
                records.insert(record_name(id));
            }
        }
    }
    // 列表与字典的 C 辅助按用到的类型生成：签名、结构体字段，以及函数体里出现过的每个值。
    let mut containers = Containers::new();
    // 入口在这个目标文件里且错误是结构体或枚举：未捕获时要按 println 的格式打印它。
    let mut values = entry.split_once(':').unwrap().0 == unit
        && if tests.is_empty() {
            module.functions[entry].error.is_some_and(compound)
        } else {
            tests
                .iter()
                .any(|test| module.functions[test].error.is_some_and(compound))
        };
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
    let mut funcs = std::collections::BTreeSet::new();
    let mut lifted: Vec<(String, String)> = Vec::new();
    let mut boxes: std::collections::BTreeSet<(u64, u64)> = std::collections::BTreeSet::new();
    for name in &needed {
        let function = &module.functions[name];
        for ty in function
            .params
            .iter()
            .map(|(_, ty)| *ty)
            .chain([function.result])
        {
            if let Type::Func(id) | Type::FuncRef(id) = ty {
                funcs.insert(id);
            }
        }
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
        let prefix = symbol.clone().unwrap_or_else(|| symbols[&name].clone());
        let declaration = if let Some(symbol) = symbol {
            format!(
                "static {}",
                signature(&name).replacen(&symbols[&name], &symbol, 1)
            )
        } else {
            signature(&name)
        };
        if function.dispatch {
            let method = name.rsplit('.').next().unwrap_or(&name);
            let args: String = (1..function.params.len())
                .map(|i| format!(", p{i}"))
                .collect();
            let call = format!("p0->vt->m_{method}(p0->data{args})");
            bodies.push_str(
                &if function.result == Type::Unit && function.error.is_none() {
                    format!("{declaration} {{ {call}; }}\n")
                } else {
                    format!("{declaration} {{ return {call}; }}\n")
                },
            );
            continue;
        }
        if let Some((helpers, body)) = intrinsic(&name, function, &prefix) {
            bodies.push_str(&helpers);
            bodies.push_str(&format!("{declaration} {{\n{body}}}\n"));
            continue;
        }
        if function.external {
            // 没被调用的 extern fn 不生成，用不到的 C 函数就不用链接。
            if used_externs.contains(&name) {
                bodies.push_str(&extern_wrapper(
                    &name,
                    function,
                    &declaration,
                    module,
                    !headers.is_empty(),
                ));
                needs_headers = true;
            }
            continue;
        }
        bodies.push_str(&format!("{declaration} {{\n"));
        let mut g = Generator {
            module,
            symbols: &symbols,
            locals: BTreeMap::new(),
            owners: Vec::new(),
            loops: Vec::new(),
            proofs: plan.proofs,
            wrapping: function.wrapping,
            mutable_names: mutable_names(function),
            call_targets: plan.targets,
            code: String::new(),
            next: 0,
            result: function.result,
            error: function.error,
            containers: Containers::new(),
            values: false,
            records: Default::default(),
            hint: None,
            funcs: Default::default(),
            lifted: Vec::new(),
            prefix,
            own: None,
            boxes: Default::default(),
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
            bodies.push_str(&format!(
                "return {}();\n",
                fallible_ok(Type::Unit, function.error)
            ));
        }
        bodies.push_str("}\n");
        containers.extend(g.containers);
        values |= g.values;
        for id in g.records {
            records.insert(record_name(id));
        }
        funcs.extend(g.funcs);
        for (interface, concrete) in &g.boxes {
            records.insert(record_name(*interface));
            records.insert(record_name(*concrete));
        }
        boxes.extend(g.boxes);
        for (key, definition) in g.lifted {
            if !lifted.iter().any(|(other, _)| *other == key) {
                lifted.push((key, definition));
            }
        }
    }
    // 结构体与容器互相引用（结构体字段是列表，列表元素是结构体），收集到不再增加为止。
    loop {
        let before = (records.len(), containers.len(), funcs.len());
        for name in records.clone() {
            for (_, ty) in &module.structs[&name].fields {
                note(&mut containers, *ty);
                if let Some(id) = record_of(*ty) {
                    records.insert(record_name(id));
                }
                if let Type::Func(id) = ty {
                    funcs.insert(*id);
                }
            }
        }
        // 函数类型的参数与结果也要有定义（按值传的结构体、容器）。
        for id in funcs.clone() {
            let (params, result) = func_sig(id);
            for ty in params.into_iter().chain([result]) {
                note(&mut containers, ty);
                if let Some(id) = record_of(ty) {
                    records.insert(record_name(id));
                }
                if let Type::Func(id) | Type::FuncRef(id) = ty {
                    funcs.insert(id);
                }
            }
        }
        for (key, value) in containers.clone() {
            for elem in std::iter::once(key).chain(value) {
                match elem {
                    Elem::Record(id) => {
                        records.insert(record_name(id));
                    }
                    Elem::Nested(_) => note_nested(&mut containers, elem),
                    _ => {}
                }
            }
        }
        if (records.len(), containers.len(), funcs.len()) == before {
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
    // 失败函数的结果结构体：函数签名与会失败的函数值都要用到，先前置声明。
    let mut fallible: Vec<(Type, Option<Type>)> = Vec::new();
    let failing = needed
        .iter()
        .map(|name| {
            let function = &module.functions[name];
            (function.result, function.error)
        })
        .chain(funcs.iter().map(|id| (func_sig(*id).1, func_error(*id))));
    for (result, error) in failing {
        let key = (result, error.filter(|e| *e != Type::Owned));
        if error.is_some() && !fallible.contains(&key) {
            fallible.push(key);
        }
    }
    for (ty, error) in &fallible {
        let structure = fallible_name(*ty, *error);
        source.push_str(&format!("typedef struct {structure} {structure};\n"));
    }
    // 函数值：函数指针（第一个参数是环境）加环境指针。不捕获变量时环境是 NULL。
    for id in &funcs {
        source.push_str(&format!("typedef struct F{id} F{id};\n"));
    }
    for id in &funcs {
        let (params, _) = func_sig(*id);
        let params: String = params
            .iter()
            .map(|ty| format!(", {}", ctype(*ty)))
            .collect();
        source.push_str(&format!(
            "struct F{id} {{ {} (*call)(void *{params}); void *env; }};\n",
            value_result(*id)
        ));
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
        if structure.interface {
            source.push_str(&format!(
                "struct R{id} {{ void *data; const struct V{id} *vt; }};\n"
            ));
            continue;
        }
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
    // 接口的虚表：方法的返回类型可以是还没定义完的结果结构体（函数指针只要声明）。
    for name in &records {
        if module.structs[name].interface {
            source.push_str(&vtable_definition(module, record_id(name)));
        }
    }
    for name in &records {
        let id = record_id(name);
        source.push_str(&format!("static void dropR{id}(R{id} *p);\n"));
    }
    // 容器的辅助函数会互相调用（列表的列表释放时要释放内层列表），先给出原型。
    for elem in &lists {
        let l = list_suffix(*elem);
        source.push_str(&format!("static void drop{l}({l} *l);\n"));
    }
    for (key, value) in &maps {
        let m = map_name(*key, *value);
        source.push_str(&format!("static void drop{m}({m} *m);\n"));
    }
    if !containers.is_empty() {
        source.push_str(LIST_PRELUDE);
    }
    if !maps.is_empty() {
        source.push_str(MAP_PRELUDE);
    }
    for elem in &lists {
        source.push_str(&list_helpers(*elem));
        // 用到 `sort_by` 的列表（它的比较函数类型出现过）才生成。
        if funcs.contains(&sort_by_type(*elem)) {
            source.push_str(&sort_by_helper(*elem));
        }
    }
    for (key, value) in &maps {
        source.push_str(&map_helpers(MAP_TEMPLATE, *key, *value));
        source.push_str(&map_values(*key, *value));
        source.push_str(&map_slots(*key, *value));
        if *value == SET {
            source.push_str(&map_helpers(SET_TEMPLATE, *key, *value));
        }
    }
    let mut closers = std::collections::BTreeSet::new();
    for name in &records {
        let structure = &module.structs[name];
        let id = record_id(name);
        if structure.interface {
            // 接口值：经虚表释放里面的具体值（连同它占的堆内存）。
            source.push_str(&format!(
                "static void dropR{id}(R{id} *p) {{ if (p->data) p->vt->drop(p->data); p->data = NULL; p->vt = NULL; }}\nstatic R{id} takeR{id}(R{id} *p) {{ R{id} v=*p; p->data = NULL; p->vt = NULL; return v; }}\n"
            ));
            continue;
        }
        if let Some(handle) = &structure.handle {
            // C 句柄：非 NULL 时调用 close，之后清零，重复释放也安全。
            needs_headers = true;
            let close = match &handle.close {
                Some(close) => {
                    if headers.is_empty() && closers.insert(close.clone()) {
                        source.push_str(&format!("void {close}(void *);\n"));
                    }
                    format!(
                        "if (p->m0) {close}(({})(intptr_t)p->m0); ",
                        handle_pointer(handle, !headers.is_empty())
                    )
                }
                None => String::new(),
            };
            source.push_str(&format!(
                "static void dropR{id}(R{id} *p) {{ {close}p->m0 = 0; }}\nstatic R{id} takeR{id}(R{id} *p) {{ R{id} v=*p; p->m0 = 0; return v; }}\n"
            ));
            continue;
        }
        source.push_str(&format!("static void dropR{id}(R{id} *p) {{"));
        for (i, (_, ty)) in structure.fields.iter().enumerate() {
            if owned(*ty) {
                source.push_str(&format!("{}(&p->m{i});", drop_fn(*ty)));
            }
        }
        source.push_str(&format!("memset(p,0,sizeof(*p)); }}\nstatic R{id} takeR{id}(R{id} *p) {{ R{id} v=*p; memset(p,0,sizeof(*p)); return v; }}\n"));
    }
    if values {
        source.push_str(&native_values::functions(module, &records, &lists, &maps));
        // 结构体与嵌套容器列表的切片：逐个深复制，复制函数刚在上面生成。
        for elem in &lists {
            if matches!(elem, Elem::Record(_) | Elem::Nested(_)) {
                let l = list_suffix(*elem);
                let copy = native_values::copy(elem.ty(), "&l->data[i]");
                source.push_str(&format!(
                    "static {l} slice{l}({l} *l, int64_t from, int64_t to) {{ {l} out = {{0}}; int64_t i; zam_clamp(&from, &to, l->len); for (i = from; i < to; i++) push{l}(&out, {copy}); return out; }}\n"
                ));
            }
        }
    }
    // 失败函数的结果结构体与构造器；调用点只读 ok 相符的那一半。
    for (ty, error) in &fallible {
        let (ty, error) = (*ty, *error);
        let structure = fallible_name(ty, error);
        let error_type = ctype(error.unwrap_or(Type::Owned));
        source.push_str(&format!(
            "struct {structure} {{ int ok; {}{error_type} error; }};\n",
            if ty == Type::Unit {
                String::new()
            } else {
                format!("{} value; ", ctype(ty))
            }
        ));
        if ty == Type::Unit {
            source.push_str(&format!(
                "static {structure} {}(void) {{ {structure} r = {{0}}; r.ok = 1; return r; }}\n",
                fallible_ok(ty, error)
            ));
        } else {
            source.push_str(&format!(
                "static {structure} {}({} value) {{ {structure} r = {{0}}; r.ok = 1; r.value = value; return r; }}\n",
                fallible_ok(ty, error),
                ctype(ty)
            ));
        }
        source.push_str(&format!(
            "static {structure} {}({error_type} error) {{ {structure} r = {{0}}; r.error = error; return r; }}\n",
            fallible_err(ty, error)
        ));
    }
    let mut prelude = false;
    let mut module_preludes = std::collections::BTreeSet::new();
    for name in &needed {
        if let Some(builtin) = builtin(name) {
            if !prelude {
                source.push_str(STD_PRELUDE);
                prelude = true;
            }
            for (prefix, code) in MODULE_PRELUDES {
                if name.starts_with(prefix) && module_preludes.insert(*prefix) {
                    source.push_str(code);
                }
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
    for (interface, concrete) in &boxes {
        source.push_str(&box_definition(module, &symbols, *interface, *concrete));
    }
    source.push_str(&specialized_decls);
    for (_, definition) in &lifted {
        source.push_str(definition);
    }
    source.push_str(&bodies);
    if let Some((_, code)) = UNIT_PRELUDES.iter().find(|(id, _)| *id == unit) {
        source.insert_str(RUNTIME.len(), &code.concat());
    }
    if needs_headers && !headers.is_empty() {
        let includes: String = headers
            .iter()
            .map(|header| format!("#include \"{header}\"\n"))
            .collect();
        source.insert_str(RUNTIME.len(), &includes);
    }
    if entry.split_once(':').unwrap().0 == unit && !tests.is_empty() {
        // 测试程序：`程序 模块:test_x` 只运行这一个测试；失败的 assert 与运行时错误都以非零退出。
        source.push_str("int zam_argc;\nchar **zam_argv;\nZAM_THREAD uint64_t zam_random_state[4];\nZAM_THREAD int zam_random_seeded;\nint zam_threaded;\n");
        let mut dispatch = String::new();
        for test in tests {
            let function = &module.functions[test];
            let call = match function.error {
                None => format!("{}();", symbols[test]),
                Some(error) => {
                    let structure = fallible_name(function.result, Some(error));
                    let report = if error == Type::Owned {
                        "uncaught_error(&r.error);".to_owned()
                    } else {
                        format!(
                            "S text = {}; uncaught_error(&text);",
                            native_values::text_call(error, "&r.error")
                        )
                    };
                    format!(
                        "{structure} r = {}(); if (!r.ok) {{ {report} return 1; }}",
                        symbols[test]
                    )
                }
            };
            dispatch.push_str(&format!(
                "if (strcmp(argv[1], \"{test}\") == 0) {{ {call} return fflush(stdout) == EOF ? 1 : 0; }}\n"
            ));
        }
        source.push_str(&format!(
            "int main(int argc, char **argv) {{ if (argc < 2) return 2; zam_argc = argc - 1; zam_argv = argv + 1; output_mode();\n{dispatch}return 2; }}\n"
        ));
    } else if entry.split_once(':').unwrap().0 == unit {
        // argv 留给 std/env（Unix）；Windows 下 std/env 改读 GetCommandLineW 以拿到 UTF-8。
        source.push_str("int zam_argc;\nchar **zam_argv;\nZAM_THREAD uint64_t zam_random_state[4];\nZAM_THREAD int zam_random_seeded;\nint zam_threaded;\n");
        if let Some(error) = module.functions[entry].error {
            let structure = fallible_name(module.functions[entry].result, Some(error));
            // 未捕获的错误写到 stderr：字符串原样，结构体与枚举按 println 的格式。
            let report = if error == Type::Owned {
                "uncaught_error(&r.error); drop(&r.error);".to_owned()
            } else {
                format!(
                    "S text = {}; uncaught_error(&text); drop(&text); {}(&r.error);",
                    native_values::text_call(error, "&r.error"),
                    drop_fn(error)
                )
            };
            source.push_str(&format!(
                "int main(int argc, char **argv) {{ zam_argc = argc; zam_argv = argv; output_mode(); {structure} r = {}(); if (!r.ok) {{ {report} return 1; }} return fflush(stdout) == EOF ? 1 : 0; }}\n",
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

/// `extern fn` 的包装：先给出 C 函数的原型，再按 Zamak 的调用约定包一层。`&string` 参数复制成
/// 以 `\0` 结尾的临时 C 字符串（调用后释放），返回的 `const char *` 复制成 Zamak 字符串（NULL 是空串），
/// C 侧保留它自己的内存。
/// C 句柄在 C 里的指针类型：有库的头文件时用真实类型（`sqlite3 *`），否则用 `void *`。
fn handle_pointer(handle: &crate::compiler::Handle, headers: bool) -> String {
    if headers {
        format!("{} *", handle.c_type)
    } else {
        "void *".into()
    }
}

/// 接口的分派方法：（方法名, 声明），按方法名排序；虚表里的顺序就是这个顺序。
fn dispatch_methods(module: &Module, interface: u64) -> Vec<(String, &crate::compiler::Function)> {
    let mut methods = crate::compiler::interface_methods(&module.functions, interface);
    methods.sort_by(|a, b| a.0.cmp(&b.0));
    methods
}
/// 方法在 C 里的返回类型：会失败时是结果结构体。
fn method_result(function: &crate::compiler::Function) -> String {
    if function.error.is_some() {
        fallible_name(function.result, function.error)
    } else {
        ctype(function.result)
    }
}
/// 接口的虚表：释放、显示，以及每个方法（第一个参数是数据指针）。
fn vtable_definition(module: &Module, interface: u64) -> String {
    let mut slots = vec![
        "void (*drop)(void *);".to_owned(),
        "void (*show)(S *, void *);".to_owned(),
    ];
    for (method, function) in dispatch_methods(module, interface) {
        let params: String = function.params[1..]
            .iter()
            .map(|(_, ty)| format!(", {}", ctype(*ty)))
            .collect();
        slots.push(format!(
            "{} (*m_{method})(void *{params});",
            method_result(function)
        ));
    }
    format!("struct V{interface} {{ {} }};\n", slots.join(" "))
}
/// 把具体类型 `concrete` 装进接口值的代码：转发方法的小函数、虚表、装箱函数。
fn box_definition(
    module: &Module,
    symbols: &BTreeMap<String, String>,
    interface: u64,
    concrete: u64,
) -> String {
    let mut out = format!(
        "static void boxdrop{interface}_{concrete}(void *p) {{ dropR{concrete}((R{concrete} *)p); free(p); }}\n\
static void boxshow{interface}_{concrete}(S *out, void *p) {{ showR{concrete}(out, (R{concrete} *)p); }}\n"
    );
    let mut slots = vec![
        format!("boxdrop{interface}_{concrete}"),
        format!("boxshow{interface}_{concrete}"),
    ];
    for (method, function) in dispatch_methods(module, interface) {
        let target = module
            .functions
            .iter()
            .find(|(name, f)| {
                !f.dispatch
                    && f.generics.is_empty()
                    && name
                        .rsplit_once('.')
                        .is_some_and(|(owner, m)| m == method && record_id(owner) == concrete)
            })
            .map(|(name, _)| symbols[name].clone())
            .expect("checked interface method");
        let params: Vec<String> = function.params[1..]
            .iter()
            .enumerate()
            .map(|(i, (_, ty))| format!("{} a{i}", ctype(*ty)))
            .collect();
        let args: String = (0..params.len()).map(|i| format!(", a{i}")).collect();
        let call = format!("{target}((R{concrete} *)self{args})");
        let body = if function.result == Type::Unit && function.error.is_none() {
            format!("{call};")
        } else {
            format!("return {call};")
        };
        let name = format!("boxm{interface}_{concrete}_{method}");
        out.push_str(&format!(
            "static {} {name}(void *self{}) {{ {body} }}\n",
            method_result(function),
            params.iter().map(|p| format!(", {p}")).collect::<String>()
        ));
        slots.push(name);
    }
    out.push_str(&format!(
        "static const struct V{interface} boxvt{interface}_{concrete} = {{ {} }};\n\
static R{interface} box{interface}_{concrete}(R{concrete} v) {{ R{interface} out; R{concrete} *p = (R{concrete} *)malloc(sizeof *p); if (!p) {{ fputs(\"zam: allocation failed\\n\", stderr); exit(1); }} *p = v; out.data = p; out.vt = &boxvt{interface}_{concrete}; return out; }}\n",
        slots.join(", ")
    ));
    out
}
/// extern fn 的包装：把 Zamak 的值换成 C 的参数，调用 C 函数，再把结果换回来。
/// 有 `[native] headers` 时用库自己的原型（C 编译器按真实类型转换 int、size_t 等参数），
/// 否则按 Zamak 的签名写一个原型。
fn extern_wrapper(
    name: &str,
    function: &crate::compiler::Function,
    declaration: &str,
    module: &Module,
    headers: bool,
) -> String {
    use crate::compiler::{option_handle, struct_of};
    let structs = &module.structs;
    let handle = |id: u64| {
        struct_of(id, structs)
            .and_then(|(_, s)| s.handle.as_ref())
            .expect("checked C handle")
    };
    let c_name = name.rsplit(':').next().unwrap_or(name);
    let c_type = |ty: Type| -> String {
        match ty {
            Type::Int => "int64_t".into(),
            Type::Float => "double".into(),
            Type::Bool => "int".into(),
            Type::Shared | Type::Owned => "const char *".into(),
            Type::ListRef(Elem::Byte, false) => "const uint8_t *".into(),
            Type::ListRef(Elem::Byte, true) => "uint8_t *".into(),
            Type::RecordRef(id, true) if option_handle(id, structs).is_some() => {
                format!(
                    "{}*",
                    handle_pointer(option_handle(id, structs).expect("guarded"), headers)
                )
            }
            Type::RecordRef(id, _) => handle_pointer(handle(id), headers),
            Type::Record(id) => {
                handle_pointer(option_handle(id, structs).expect("checked"), headers)
            }
            _ => "void".into(),
        }
    };
    let mut out = String::new();
    if !headers {
        let params: Vec<String> = function.params.iter().map(|(_, ty)| c_type(*ty)).collect();
        out.push_str(&format!(
            "{} {c_name}({});\n",
            c_type(function.result),
            if params.is_empty() {
                "void".into()
            } else {
                params.join(", ")
            }
        ));
    }
    out.push_str(&format!("{declaration} {{\n"));
    let mut args = Vec::new();
    // 调用之后要做的事：释放临时 C 字符串、把输出参数写回 Option。
    let mut after = Vec::new();
    for (i, (_, ty)) in function.params.iter().enumerate() {
        match *ty {
            Type::Shared => {
                out.push_str(&format!("char *c{i} = zam_cstr(p{i});\n"));
                args.push(format!("c{i}"));
                after.push(format!("free(c{i});"));
            }
            Type::ListRef(Elem::Byte, _) => args.push(format!("p{i}->data")),
            Type::RecordRef(id, true) if option_handle(id, structs).is_some() => {
                // 输出参数 `&mut Option[Db]`：C 写进一个指针，非 NULL 时成为 Some。原来的值先释放。
                let pointer = c_type(Type::Record(id));
                out.push_str(&format!("{pointer} t{i} = NULL;\n"));
                args.push(format!("&t{i}"));
                after.push(format!(
                    "dropR{id}(p{i}); if (t{i}) {{ p{i}->m0 = 0; p{i}->m1.m0 = (int64_t)(intptr_t)t{i}; }} else p{i}->m0 = 1;"
                ));
            }
            Type::RecordRef(id, _) => args.push(format!(
                "({})(intptr_t)p{i}->m0",
                c_type(Type::RecordRef(id, false))
            )),
            _ => args.push(format!("p{i}")),
        }
    }
    let call = format!("{c_name}({})", args.join(", "));
    let after = after.join(" ");
    out.push_str(&match function.result {
        Type::Unit => format!("{call};\n{after}\n"),
        Type::Owned => format!(
            "const char *r = {call};\nS out = r ? own((const unsigned char *)r, strlen(r)) : own((const unsigned char *)\"\", 0);\n{after}\nreturn out;\n"
        ),
        Type::Bool => format!("int r = {call} != 0;\n{after}\nreturn r;\n"),
        // `Option[Db]`：NULL 是 None，其余是 Some(句柄)。
        Type::Record(id) => format!(
            "{} r = {call};\nR{id} out; memset(&out, 0, sizeof out);\nif (r) {{ out.m0 = 0; out.m1.m0 = (int64_t)(intptr_t)r; }} else out.m0 = 1;\n{after}\nreturn out;\n",
            c_type(Type::Record(id))
        ),
        result => format!("{} r = {call};\n{after}\nreturn r;\n", c_type(result)),
    });
    out.push_str("}\n");
    out
}

/// 函数体由生成器按具体类型给出的函数：`thread.spawn` 合成的线程入口，以及 std/thread 里要知道
/// 元素在 C 里的大小与释放方式的队列函数。返回（函数体之前要定义的辅助代码, 函数体）。
/// 参数在 C 里叫 p0、p1……；句柄参数 `&QueueHandle` 是结构体指针，句柄在 m0 里。
fn intrinsic(
    name: &str,
    function: &crate::compiler::Function,
    symbol: &str,
) -> Option<(String, String)> {
    let param = |i: usize| function.params[i].1;
    // 列表参数 `&[T]` / `&mut [T]` 的元素：值的 C 类型、追加函数、释放一个值的语句。
    let element = |i: usize| match param(i) {
        Type::ListRef(elem, _) => elem,
        _ => unreachable!("intrinsic list parameter"),
    };
    let dropper = |elem: Elem| -> (String, String) {
        let ty = ctype(elem.ty());
        match drop_elem(elem, &format!("({ty} *)p")) {
            drop if drop.is_empty() => (String::new(), "NULL".into()),
            drop => (
                format!("static void {symbol}_drop(void *p) {{ {drop} }}\n"),
                format!("{symbol}_drop"),
            ),
        }
    };
    let handle = |i: usize| format!("(void *)(intptr_t)p{i}->m0");
    if name.starts_with(&format!("{}[", crate::compiler::SPAWN)) {
        let Type::Func(id) = param(0) else {
            unreachable!("spawn takes a function value")
        };
        let (_, result) = func_sig(id);
        let error = func_error(id);
        let count = function.params.len();
        let fields: String = (1..count)
            .map(|i| format!(" {} a{i};", ctype(param(i))))
            .collect();
        let args: String = (1..count).map(|i| format!(", job.a{i}")).collect();
        let call = format!("job.f.call(job.f.env{args})");
        let finish = match (result, error) {
            (Type::Unit, None) => format!("{call};"),
            (Type::Unit, Some(_)) => format!(
                "{} r = {call}; if (!r.ok) {{ *failed = 1; *error = r.error; }}",
                value_result(id)
            ),
            (_, None) => format!(
                "{} v = {call}; *result = zam_thread_box(&v, sizeof v);",
                ctype(result)
            ),
            (_, Some(_)) => format!(
                "{} r = {call}; if (!r.ok) {{ *failed = 1; *error = r.error; }} else *result = zam_thread_box(&r.value, sizeof r.value);",
                value_result(id)
            ),
        };
        let (drop_helper, drop_result) = if result == Type::Unit {
            (String::new(), "NULL".to_owned())
        } else {
            dropper(elem_of(result))
        };
        let mut helpers = format!(
            "struct {symbol}_job {{ F{id} f;{fields} }};\n\
             static void {symbol}_run(void *p, void **result, S *error, int *failed) {{\n\
             struct {symbol}_job job = *(struct {symbol}_job *)p;\n\
             free(p); (void)result; (void)error; (void)failed;\n{finish}\n}}\n"
        );
        helpers.push_str(&drop_helper);
        let moves: String = (1..count).map(|i| format!("j->a{i} = p{i}; ")).collect();
        let body = format!(
            "struct {symbol}_job *j = (struct {symbol}_job *)malloc(sizeof *j);\n\
             {} r;\n\
             if (!j) zam_thread_alloc_failed();\n\
             j->f = p0; {moves}\n\
             memset(&r, 0, sizeof r);\n\
             r.m0.m1.m0 = (int64_t)(intptr_t)zam_thread_start({symbol}_run, j, {drop_result});\n\
             return r;\n",
            ctype(function.result)
        );
        return Some((helpers, body));
    }
    let template = name.split('[').next().unwrap_or(name);
    match template {
        "std/thread:_queue_new" => {
            let elem = element(0);
            let Type::RecordRef(option, _) = param(1) else {
                unreachable!("queue output parameter")
            };
            let (helper, drop) = dropper(elem);
            Some((
                helper,
                format!(
                    "void *q = zam_queue_new((int64_t)sizeof({}), {drop});\n(void)p0;\n\
                     dropR{option}(p1); p1->m0 = 0; p1->m1.m0 = (int64_t)(intptr_t)q;\n",
                    ctype(elem.ty())
                ),
            ))
        }
        "std/thread:_queue_put" => Some((
            String::new(),
            format!("return zam_queue_put({}, &p1);\n", handle(0)),
        )),
        "std/thread:_queue_get" => {
            let elem = element(1);
            Some((
                String::new(),
                format!(
                    "{} v;\nif (!zam_queue_get({}, &v)) return 0;\npush{}(p1, v);\nreturn 1;\n",
                    ctype(elem.ty()),
                    handle(0),
                    list_suffix(elem)
                ),
            ))
        }
        // 运行主循环：状态 p2 留在这个栈帧里，回调经指针读写它，结束后原样返回。
        "std/ui:_run" => Some((
            String::new(),
            format!("zam_ui_run({}, p1, &p2);\nreturn p2;\n", handle(0)),
        )),
        // 回调：保存函数值，按 T 生成调用器，把 C 里存的状态当成 &mut T 交给它。
        // 画布的回调：第二个参数是画笔或事件，C 里传来的是地址，包成只有一个字段的结构体交给它。
        "std/ui:_connect_event" => {
            let Type::Func(id) = param(3) else {
                unreachable!("kept handler")
            };
            let (params, _) = func_sig(id);
            let state = ctype(params[0]);
            let Type::RecordRef(event, _) = params[1] else {
                unreachable!("event parameter is a borrowed struct")
            };
            Some((
                format!(
                    "static void {symbol}_event(void *f, void *state, int64_t arg) {{ F{id} *g = (F{id} *)f; R{event} e; memset(&e, 0, sizeof e); e.m0 = arg; g->call(g->env, ({state})state, &e); }}\n"
                ),
                format!(
                    "zam_ui_connect_event({}, p1, p2, &p3, sizeof p3, {symbol}_event);\n",
                    handle(0)
                ),
            ))
        }
        "std/ui:_connect" => {
            let Type::Func(id) = param(3) else {
                unreachable!("kept handler")
            };
            let (params, _) = func_sig(id);
            let state = ctype(params[0]);
            Some((
                format!(
                    "static void {symbol}_invoke(void *f, void *state) {{ F{id} *g = (F{id} *)f; g->call(g->env, ({state})state); }}\n"
                ),
                format!(
                    "zam_ui_connect({}, p1, p2, &p3, sizeof p3, {symbol}_invoke);\n",
                    handle(0)
                ),
            ))
        }
        "std/thread:_task_take" => {
            let elem = element(1);
            Some((
                String::new(),
                format!(
                    "{ty} *v = ({ty} *)zam_thread_take({});\nif (v) {{ push{}(p1, *v); free(v); }}\n",
                    handle(0),
                    list_suffix(elem),
                    ty = ctype(elem.ty())
                ),
            ))
        }
        _ => None,
    }
}

pub fn uses_msvc(tool: &str) -> bool {
    cfg!(windows)
        && Path::new(tool)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("bat"))
}

pub fn tool() -> Result<(String, String)> {
    // `CC=`（空串）和没设一样，同 make 等工具的习惯。
    select(
        std::env::var("CC")
            .ok()
            .filter(|cc| !cc.is_empty())
            .as_deref(),
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
    // 身份里带上命令本身：`CC=tsancc`（`exec gcc -fsanitize=thread "$@"` 这样的包装）报的版本与
    // gcc 一样，只看版本会让两者共用缓存键，得到不同的目标文件。
    Ok((
        compiler.to_owned(),
        format!("{compiler}\n{}", String::from_utf8_lossy(&output.stdout)),
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
            // 并发写入时可能读到不完整的文件：没有 INCLUDE 就当没有缓存，重新装载。
            if first == stamp && rest.lines().any(|line| line.starts_with("INCLUDE=")) {
                return Ok(parse_environment(rest));
            }
        }
    }
    // 命令写进 .bat 再执行：`cmd /c` 只认自己的引号规则，而 Rust 把参数里的引号转义成
    // `\"`（cmd 不认），带空格的 vcvars 路径就永远找不到。
    // 脚本按进程区分：多个 zam 同时首次装载时，不能互相改写对方正在执行的脚本。
    let script = std::env::temp_dir().join(format!("zam-msvc-{name}-{}.bat", std::process::id()));
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
    let _ = std::fs::remove_file(&script);
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    if !text.lines().any(|line| line.starts_with("INCLUDE=")) {
        return Err("MSVC setup produced no INCLUDE".into());
    }
    // 先写进本进程的临时文件再改名，别的进程只会读到完整的缓存。
    // 写不进去也无所谓，只是下次还要重新装载一遍。
    let partial = path.with_extension(format!("{}.tmp", std::process::id()));
    if std::fs::write(&partial, format!("{stamp}\n{text}")).is_ok()
        && std::fs::rename(&partial, &path).is_err()
    {
        let _ = std::fs::remove_file(&partial);
    }
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
    compile_with(tool, dir, source, &[])
}

/// 带头文件目录编译一段 C 源码（`[native]` 的源码用）。
pub fn compile_with(
    tool: &str,
    dir: &Path,
    source: &str,
    includes: &[std::path::PathBuf],
) -> Result<Vec<u8>> {
    compile_source(tool, dir, source, includes, Language::C, &[])
}

/// `[native]` 源码的语言，按扩展名决定。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    C,
    Cpp,
    ObjectiveC,
}

impl Language {
    /// `.c`、`.cpp`/`.cc`/`.cxx`、`.m`；其他扩展名返回 None。
    pub fn of(path: &Path) -> Option<Language> {
        match path.extension()?.to_str()? {
            "c" => Some(Language::C),
            "cpp" | "cc" | "cxx" => Some(Language::Cpp),
            "m" => Some(Language::ObjectiveC),
            _ => None,
        }
    }
    fn file(self) -> &'static str {
        match self {
            Language::C => "program.c",
            Language::Cpp => "program.cpp",
            Language::ObjectiveC => "program.m",
        }
    }
}

/// 编译一个 C、C++ 或 Objective-C 源文件。`flags` 是额外的编译参数（如 pkg-config 给出的）。
pub fn compile_source(
    tool: &str,
    dir: &Path,
    source: &str,
    includes: &[std::path::PathBuf],
    language: Language,
    flags: &[String],
) -> Result<Vec<u8>> {
    let file = language.file();
    std::fs::write(dir.join(file), source).map_err(|e| e.to_string())?;
    // 编译器记录的是工作目录的写法；临时目录经过 8.3 短名（CI 的 `RUNNER~1`）、联接或
    // 符号链接（macOS 的 /var → /private/var）时，它和规范路径不同，路径映射就会失配，
    // 随机的临时目录名漏进目标文件，破坏「同键 ⇒ 同字节」。统一用规范路径做工作目录与映射源。
    let mapped = std::fs::canonicalize(dir)
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_owned();
    // 头文件目录同理：头文件里的 `__FILE__`（如 Mbed TLS 的调试宏）会带上目录的绝对路径，
    // 换个缓存目录或机器就变。MSVC 用 `/d1trimfile` 把目录前缀裁掉（`/d2pathmap` 管不到 `__FILE__`），
    // GCC/Clang 把每个目录映射成固定的 `I0`、`I1`……
    let includes: Vec<String> = includes
        .iter()
        .map(|dir| {
            std::fs::canonicalize(dir)
                .map(|dir| dir.to_string_lossy().trim_start_matches(r"\\?\").to_owned())
                .unwrap_or_else(|_| dir.display().to_string())
        })
        .collect();
    let output = if uses_msvc(tool) {
        let environment = msvc_environment(tool)?;
        if std::iter::once(&mapped)
            .chain(&includes)
            .any(|path| path.contains(['"', '%', '\r', '\n']))
        {
            return Err("unsupported native cache path".into());
        }
        if language == Language::ObjectiveC {
            return Err("Objective-C sources compile with clang on macOS, not with MSVC".into());
        }
        let cpp: &[&str] = if language == Language::Cpp {
            &["/EHsc", "/std:c++17"]
        } else {
            &[]
        };
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
                file,
                "/Fo:program.obj",
            ])
            .args(cpp)
            .args(includes.iter().map(|dir| format!("/I{dir}")))
            .args(includes.iter().map(|dir| format!("/d1trimfile:{dir}\\")))
            .args(flags)
            .output()
    } else {
        // ponytail: `-g`（用户经 CC 传入）会把编译目录写进 DWARF，同一个缓存键在不同临时目录
        // 就会编出不同字节，破坏「同键 ⇒ 同字节」并让并发构建报 corrupt cache entry。
        // 把临时目录前缀映射掉即可，与 Windows 的 /d2pathmap 对应。
        let map = format!("-ffile-prefix-map={mapped}=.");
        let standard = match language {
            Language::C => "-std=c11",
            Language::Cpp => "-std=c++17",
            Language::ObjectiveC => "-std=gnu11",
        };
        Command::new(tool)
            .current_dir(&mapped)
            .args([
                standard,
                "-O2",
                "-funroll-loops",
                map.as_str(),
                "-c",
                file,
                "-o",
                "program.obj",
            ])
            .args(includes.iter().map(|dir| format!("-I{dir}")))
            .args(
                includes
                    .iter()
                    .enumerate()
                    .map(|(n, dir)| format!("-ffile-prefix-map={dir}=I{n}")),
            )
            .args(flags)
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

/// 把目标文件链接成可执行文件。`extra` 是额外的链接参数（如 macOS 的 `-framework Cocoa`、
/// Windows 的清单），`cpp` 表示有 C++ 目标文件，GCC/Clang 要带上 C++ 运行库。
pub fn link_with(
    tool: &str,
    dir: &Path,
    objects: &[Vec<u8>],
    libraries: &[String],
    library_paths: &[std::path::PathBuf],
    extra: &[String],
    cpp: bool,
) -> Result<Vec<u8>> {
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
            .args(
                library_paths
                    .iter()
                    .map(|path| format!("/LIBPATH:{}", path.display())),
            )
            .args(libraries.iter().map(|name| format!("{name}.lib")))
            // std/net 用 Winsock；没用到网络的程序不会引用它，产物不变。
            .arg("ws2_32.lib")
            .args(extra)
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
        let winsock = if cfg!(windows) {
            &["-lws2_32"][..]
        } else {
            &[][..]
        };
        command
            .args(
                library_paths
                    .iter()
                    .map(|path| format!("-L{}", path.display())),
            )
            .args(libraries.iter().map(|name| format!("-l{name}")))
            .args(winsock)
            .args(extra);
        if cpp {
            command.arg(if cfg!(target_os = "macos") {
                "-lc++"
            } else {
                "-lstdc++"
            });
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
