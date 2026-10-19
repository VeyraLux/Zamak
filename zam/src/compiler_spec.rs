//! `f"{x:.2}"` 里冒号后面的格式说明。
//!
//! 写法跟 Python 的 format spec 一样，但只支持 Zamak 用得到的部分：
//!
//! ```text
//! [[fill]align][width][.precision]
//! align: '<' 左对齐、'>' 右对齐、'^' 居中
//! width: 最少占几列（不够时用 fill 补，默认空格）
//! .precision: 小数位数（只对 f64 有意义）
//! ```
//!
//! 数字还可以在前面写 `0`（等价于 fill='0'、align='>'），用于 `f"{n:03}"`。

use crate::compiler::Type;

/// 解析好的说明。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    /// 补白字符，默认空格。
    pub fill: char,
    /// 对齐方式；`None` 表示数字靠右、其他靠左。
    pub align: Option<char>,
    /// 最少占几列。
    pub width: usize,
    /// 小数位数。
    pub precision: Option<usize>,
}

/// 解析说明；`Err` 里是给用户看的话。
pub fn parse(text: &str) -> Result<Spec, String> {
    let mut spec = Spec {
        fill: ' ',
        align: None,
        width: 0,
        precision: None,
    };
    let chars: Vec<char> = text.chars().collect();
    let mut at = 0;
    // 对齐：`[fill]align`。fill 只能是单个字符，后面紧跟 `<`、`>`、`^`。
    if chars.len() >= 2 && matches!(chars[1], '<' | '>' | '^') {
        if !chars[0].is_ascii() {
            return Err(format!(
                "unsupported format fill {:?}; the fill must be an ASCII character",
                chars[0]
            ));
        }
        spec.fill = chars[0];
        spec.align = Some(chars[1]);
        at = 2;
    } else if chars.first().is_some_and(|c| matches!(c, '<' | '>' | '^')) {
        spec.align = Some(chars[0]);
        at = 1;
    }
    // 数字前面可以写 0：补零、靠右。
    if chars.get(at) == Some(&'0') {
        if spec.align.is_none() {
            spec.fill = '0';
            spec.align = Some('>');
        }
        at += 1;
    }
    let start = at;
    while chars.get(at).is_some_and(|c| c.is_ascii_digit()) {
        at += 1;
    }
    if at > start {
        let digits: String = chars[start..at].iter().collect();
        spec.width = digits
            .parse()
            .map_err(|_| format!("format width {digits} is too large"))?;
        if spec.width > 1024 {
            return Err(format!(
                "format width {digits} is too large; the limit is 1024"
            ));
        }
    }
    if chars.get(at) == Some(&'.') {
        at += 1;
        let start = at;
        while chars.get(at).is_some_and(|c| c.is_ascii_digit()) {
            at += 1;
        }
        if at == start {
            return Err(
                "write the number of decimals after the dot, such as {x:.2} for two decimals"
                    .into(),
            );
        }
        let digits: String = chars[start..at].iter().collect();
        let precision: usize = digits
            .parse()
            .map_err(|_| format!("format precision {digits} is too large"))?;
        if precision > 17 {
            return Err(format!(
                "format precision {precision} is too large; an f64 has at most 17 digits"
            ));
        }
        spec.precision = Some(precision);
    }
    if at != chars.len() {
        let rest: String = chars[at..].iter().collect();
        return Err(format!(
            "unsupported format {rest:?}; write a width such as {{x:5}}, an alignment such as {{x:>5}}, or decimals such as {{x:.2}}"
        ));
    }
    Ok(spec)
}

/// 检查说明与值的类型是否搭：精度只能给 f64，对齐/宽度对所有类型都可以。
pub fn check(text: &str, ty: Type) -> Result<Spec, String> {
    let spec = parse(text)?;
    if spec.precision.is_some() && ty != Type::Float {
        return Err(
            "only an f64 takes decimals in a format, such as {x:.2}; drop the precision for other values"
                .into(),
        );
    }
    Ok(spec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specs_parse_width_alignment_and_precision() {
        assert_eq!(
            parse(".2").unwrap(),
            Spec {
                fill: ' ',
                align: None,
                width: 0,
                precision: Some(2)
            }
        );
        assert_eq!(
            parse(">5").unwrap(),
            Spec {
                fill: ' ',
                align: Some('>'),
                width: 5,
                precision: None
            }
        );
        assert_eq!(
            parse("*^7").unwrap(),
            Spec {
                fill: '*',
                align: Some('^'),
                width: 7,
                precision: None
            }
        );
        // `03` 是补零、宽 3。
        assert_eq!(
            parse("03").unwrap(),
            Spec {
                fill: '0',
                align: Some('>'),
                width: 3,
                precision: None
            }
        );
        assert_eq!(
            parse("08.3").unwrap(),
            Spec {
                fill: '0',
                align: Some('>'),
                width: 8,
                precision: Some(3)
            }
        );
        assert_eq!(
            parse("").unwrap(),
            Spec {
                fill: ' ',
                align: None,
                width: 0,
                precision: None
            }
        );
    }

    #[test]
    fn bad_specs_say_what_to_write() {
        let error = parse("d").unwrap_err();
        assert!(error.contains("unsupported format"), "{error}");
        let error = parse(".").unwrap_err();
        assert!(error.contains("decimals after the dot"), "{error}");
        let error = parse("x").unwrap_err();
        assert!(error.contains("unsupported format"), "{error}");
    }

    #[test]
    fn precision_is_only_for_floats() {
        assert!(check(".2", Type::Float).is_ok());
        let error = check(".2", Type::Int).unwrap_err();
        assert!(error.contains("only an f64"), "{error}");
        assert!(check(">5", Type::Int).is_ok());
        assert!(check(">5", Type::Owned).is_ok());
    }
}
