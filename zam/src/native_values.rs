//! 复合值（字符串、结构体、枚举、列表、字典）的深复制、相等与打印。
//! 每种类型生成四个 C 函数：`copyX`（深复制）、`eqX`（逐项相等）、`showX`（追加文本）、
//! `textX`（返回新字符串）。一个目标文件只要用到其中任何一种，就为它的全部结构体与容器生成。
use super::{ctype, list_suffix, map_name};
use crate::compiler::{record_id, Elem, Module, Type};

/// 文本拼接与基本值的显示。字符串在容器与结构体里加引号并转义，与字面量写法一致。
pub const PRELUDE: &str = r#"static size_t zam_room(size_t n) { size_t c = 16; while (c < n) { if (c > SIZE_MAX / 2) return n; c *= 2; } return c; }
// 只用于从空串开始拼出来的文本：容量总是 zam_room(len)，按 2 倍增长。
static void zam_append(S *out, const void *p, size_t n) {
    unsigned char *q;
    if (!n) return;
    if (n > SIZE_MAX - out->len) { fputs("zam: allocation failed\n", stderr); exit(1); }
    if (!out->data || zam_room(out->len + n) != zam_room(out->len)) {
        q = (unsigned char*)realloc(out->data, zam_room(out->len + n));
        if (!q) { fputs("zam: allocation failed\n", stderr); exit(1); }
        out->data = q;
    }
    memcpy(out->data + out->len, p, n);
    out->len += n;
}
static void zam_show_int(S *out, int64_t v) { char b[32]; int n = snprintf(b, sizeof b, "%" PRId64, v); zam_append(out, b, (size_t)n); }
static void zam_show_f64(S *out, double v) { char b[48]; size_t n = zam_f64_text(v, b); zam_append(out, b, n); }
static void zam_show_bool(S *out, int v) { if (v) zam_append(out, "true", 4); else zam_append(out, "false", 5); }
static void zam_show_str(S *out, S *s) {
    size_t i;
    zam_append(out, "\"", 1);
    for (i = 0; i < s->len; i++) {
        unsigned char c = s->data[i];
        if (c == '"') zam_append(out, "\\\"", 2);
        else if (c == '\\') zam_append(out, "\\\\", 2);
        else if (c == '\n') zam_append(out, "\\n", 2);
        else if (c == '\r') zam_append(out, "\\r", 2);
        else if (c == '\t') zam_append(out, "\\t", 2);
        else zam_append(out, &c, 1);
    }
    zam_append(out, "\"", 1);
}
"#;

/// 复制一份 `*ptr`：标量按值，字符串与复合值深复制。
pub fn copy(ty: Type, ptr: &str) -> String {
    match ty {
        Type::Int | Type::Float | Type::Bool => format!("(*{ptr})"),
        Type::Owned => format!("copy({ptr})"),
        _ => format!("copy{}({ptr})", ctype(ty)),
    }
}

/// `*a` 与 `*b` 是否相等。浮点按 IEEE 754，NaN 与自己不相等。
pub fn equal(ty: Type, a: &str, b: &str) -> String {
    match ty {
        Type::Int | Type::Float | Type::Bool => format!("(*{a} == *{b})"),
        Type::Owned => format!("string_equal({a}, {b})"),
        _ => format!("eq{}({a}, {b})", ctype(ty)),
    }
}

/// 把 `*ptr` 的显示文本追加到 `out`。
fn show(ty: Type, out: &str, ptr: &str) -> String {
    match ty {
        Type::Int => format!("zam_show_int({out}, *{ptr});"),
        Type::Float => format!("zam_show_f64({out}, *{ptr});"),
        Type::Bool => format!("zam_show_bool({out}, *{ptr});"),
        Type::Owned => format!("zam_show_str({out}, {ptr});"),
        _ => format!("show{}({out}, {ptr});", ctype(ty)),
    }
}

/// 追加一段固定文本；每个字节都写成 `\xNN`，名字里有非 ASCII 字符也不受源码编码影响。
fn text(out: &str, value: &str) -> String {
    let bytes: String = value.bytes().map(|b| format!("\\x{b:02x}")).collect();
    format!("zam_append({out}, \"{bytes}\", {});", value.len())
}

/// 显示文本（新字符串）的 C 表达式：容器与结构体里的字符串加引号，顶层字符串原样。
pub fn text_call(ty: Type, ptr: &str) -> String {
    format!("text{}({ptr})", ctype(ty))
}

fn shared(ty: Type, protos: &mut String, defs: &mut String, body: (String, String, String)) {
    let t = ctype(ty);
    protos.push_str(&format!(
        "static {t} copy{t}({t} *p);\nstatic int eq{t}({t} *a, {t} *b);\nstatic void show{t}(S *out, {t} *p);\nstatic S text{t}({t} *p);\n"
    ));
    let (copy_body, eq_body, show_body) = body;
    defs.push_str(&format!(
        "static {t} copy{t}({t} *p) {{ {copy_body} }}\nstatic int eq{t}({t} *a, {t} *b) {{ {eq_body} }}\nstatic void show{t}(S *out, {t} *p) {{ {show_body} }}\nstatic S text{t}({t} *p) {{ S out = {{0}}; show{t}(&out, p); return out; }}\n"
    ));
}

/// 结构体或枚举的四个函数。枚举只复制、比较和显示选中变体的数据。
fn record(module: &Module, name: &str, protos: &mut String, defs: &mut String) {
    let structure = &module.structs[name];
    let id = record_id(name);
    let short = name.rsplit(':').next().unwrap_or(name);
    // 泛型实例 `Maybe[Int]` 显示成模板名 `Maybe`，与源码里的写法一致。
    let short = if short.starts_with('(') {
        short
    } else {
        short.split('[').next().unwrap_or(short)
    };
    let fields = &structure.fields;
    let mut copy_body = format!("R{id} out; memset(&out, 0, sizeof out); ");
    let mut eq_body = String::new();
    let mut show_body = String::new();
    if short.starts_with('(') {
        // 元组按位置显示：`(1, "a")`。
        eq_body.push_str("return 1");
        show_body.push_str(&text("out", "("));
        for (i, (_, ty)) in fields.iter().enumerate() {
            copy_body.push_str(&format!("out.m{i} = {}; ", copy(*ty, &format!("&p->m{i}"))));
            eq_body.push_str(&format!(
                " && {}",
                equal(*ty, &format!("&a->m{i}"), &format!("&b->m{i}"))
            ));
            if i > 0 {
                show_body.push_str(&text("out", ", "));
            }
            show_body.push_str(&show(*ty, "out", &format!("&p->m{i}")));
        }
        eq_body.push(';');
        show_body.push_str(&text("out", ")"));
    } else if structure.variants.is_empty() {
        eq_body.push_str("return 1");
        show_body.push_str(&text("out", &format!("{short}(")));
        for (i, (field, ty)) in fields.iter().enumerate() {
            copy_body.push_str(&format!("out.m{i} = {}; ", copy(*ty, &format!("&p->m{i}"))));
            eq_body.push_str(&format!(
                " && {}",
                equal(*ty, &format!("&a->m{i}"), &format!("&b->m{i}"))
            ));
            let label = if i == 0 {
                format!("{field}: ")
            } else {
                format!(", {field}: ")
            };
            show_body.push_str(&text("out", &label));
            show_body.push_str(&show(*ty, "out", &format!("&p->m{i}")));
        }
        eq_body.push(';');
        show_body.push_str(&text("out", ")"));
    } else {
        copy_body.push_str("out.m0 = p->m0; ");
        eq_body.push_str("if (a->m0 != b->m0) return 0; ");
        for (tag, (variant, first, count)) in structure.variants.iter().enumerate() {
            let payload = *first..first + count;
            copy_body.push_str(&format!("if (p->m0 == {tag}) {{ "));
            eq_body.push_str(&format!("if (a->m0 == {tag}) return 1"));
            show_body.push_str(&format!("if (p->m0 == {tag}) {{ "));
            show_body.push_str(&text("out", &format!("{short}.{variant}")));
            if *count > 0 {
                show_body.push_str(&text("out", "("));
            }
            for i in payload {
                let ty = fields[i].1;
                copy_body.push_str(&format!("out.m{i} = {}; ", copy(ty, &format!("&p->m{i}"))));
                eq_body.push_str(&format!(
                    " && {}",
                    equal(ty, &format!("&a->m{i}"), &format!("&b->m{i}"))
                ));
                if i > *first {
                    show_body.push_str(&text("out", ", "));
                }
                show_body.push_str(&show(ty, "out", &format!("&p->m{i}")));
            }
            if *count > 0 {
                show_body.push_str(&text("out", ")"));
            }
            copy_body.push_str("} ");
            eq_body.push_str("; ");
            show_body.push_str("} ");
        }
        eq_body.push_str("return 1;");
    }
    copy_body.push_str("return out;");
    shared(
        Type::Record(id),
        protos,
        defs,
        (copy_body, eq_body, show_body),
    );
}

fn list(elem: Elem, protos: &mut String, defs: &mut String) {
    let l = list_suffix(elem);
    let ty = elem.ty();
    let copy_body = format!(
        "{l} out = {{0}}; size_t i; for (i = 0; i < p->len; i++) push{l}(&out, {}); return out;",
        copy(ty, "&p->data[i]")
    );
    let eq_body = format!(
        "size_t i; if (a->len != b->len) return 0; for (i = 0; i < a->len; i++) if (!{}) return 0; return 1;",
        equal(ty, "&a->data[i]", "&b->data[i]")
    );
    let show_body = format!(
        "size_t i; {} for (i = 0; i < p->len; i++) {{ if (i) {} {} }} {}",
        text("out", "["),
        text("out", ", "),
        show(ty, "out", "&p->data[i]"),
        text("out", "]")
    );
    shared(
        Type::List(elem),
        protos,
        defs,
        (copy_body, eq_body, show_body),
    );
}

/// 字典按插入顺序复制与显示；相等与顺序无关（同 Python 的 dict）：条目数相同、每个键的值相等。
fn map(key: Elem, value: Elem, protos: &mut String, defs: &mut String) {
    let m = map_name(key, value);
    let key_arg = if key == Elem::Str {
        "&e->key"
    } else {
        "e->key"
    };
    let copy_body = format!(
        "{m} out = {{0}}; size_t i; for (i = 0; i < p->len; i++) {{ E{m} *e = &p->entries[i]; if (e->live) set{m}(&out, {key_arg}, {}); }} return out;",
        copy(value.ty(), "&e->value")
    );
    let eq_body = format!(
        "size_t i; int64_t at; if (a->count != b->count) return 0; for (i = 0; i < a->len; i++) {{ E{m} *e = &a->entries[i]; if (!e->live) continue; at = find{m}(b, {key_arg}, e->hash); if (at < 0 || !{}) return 0; }} return 1;",
        equal(value.ty(), "&e->value", "&b->entries[at].value")
    );
    let show_body = format!(
        "size_t i; int first = 1; {} for (i = 0; i < p->len; i++) {{ E{m} *e = &p->entries[i]; if (!e->live) continue; if (!first) {} first = 0; {} {} {} }} {}",
        text("out", "{"),
        text("out", ", "),
        show(key.ty(), "out", "&e->key"),
        text("out", ": "),
        show(value.ty(), "out", "&e->value"),
        text("out", "}")
    );
    shared(
        Type::Map(key, value),
        protos,
        defs,
        (copy_body, eq_body, show_body),
    );
}

/// 一个目标文件里全部结构体与容器的复制、相等与显示函数：先给原型（它们互相调用），再给定义。
pub fn functions(
    module: &Module,
    records: &[String],
    lists: &[Elem],
    maps: &[(Elem, Elem)],
) -> String {
    let mut protos = String::new();
    let mut defs = String::new();
    for name in records {
        record(module, name, &mut protos, &mut defs);
    }
    for elem in lists {
        list(*elem, &mut protos, &mut defs);
    }
    for (key, value) in maps {
        map(*key, *value, &mut protos, &mut defs);
    }
    format!("{PRELUDE}{protos}{defs}")
}
