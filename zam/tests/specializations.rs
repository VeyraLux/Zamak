mod common;

use common::Case;

#[test]
fn repeated_call_site_joins_all_visits_before_removing_checks() {
    let case = Case::new("specializations", "joined-overflow");
    let result = case.run("fn add(n: i64) -> i64: n + 1\nfn main():\n    let mut n = 0\n    let mut i = 0\n    while i < 2:\n        println(add(n))\n        n = 9223372036854775807\n        i = i + 1");
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("integer arithmetic error"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), "1\n");
}

#[test]
fn negative_modulus_and_nested_generic_fallback_keep_behavior() {
    let case = Case::new("specializations", "negative-unknown");
    let result = case.run("fn rem(n: i64) -> i64: (n * 31 + 1) % 7\nfn unknown() -> i64: -5\nfn nested(n: i64) -> i64: rem(n)\nfn main():\n    println(rem(-5))\n    println(rem(unknown()))\n    println(nested(-5))");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), "0\n0\n0\n");
    let remainder_inputs = case.run("fn twice(n: i64) -> i64: n * 2\nfn main():\n    let mut n = -9\n    while n < 0:\n        println(twice(n % 8))\n        n = n + 1");
    assert!(
        remainder_inputs.status.success(),
        "{}",
        String::from_utf8_lossy(&remainder_inputs.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&remainder_inputs.stdout),
        "14\n0\n2\n4\n6\n8\n10\n12\n14\n"
    );
    let failed = case.run("fn unknown() -> i64: 9223372036854775807\nfn add(n: i64) -> i64: n + 1\nfn main(): println(add(unknown()))");
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("integer arithmetic error"));
}

#[test]
fn specialized_call_evaluates_arguments_in_original_order() {
    let case = Case::new("specializations", "argument-order");
    let result = case.run("fn first() -> i64:\n    println(1)\n    3\nfn second() -> i64:\n    println(2)\n    4\nfn sum(a: i64, b: i64, seed: i64) -> i64: a + b + seed\nfn main(): println(sum(first(), second(), 0))");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), "1\n2\n7\n");
}

#[test]
fn bounded_accumulator_and_recursive_generic_calls_keep_results() {
    let case = Case::new("specializations", "accumulator");
    let result = case.run("fn accum(seed: i64) -> i64:\n    let mut acc = seed\n    let mut i = 0\n    while i < 512:\n        acc = (acc * 31 + i) % 1000003\n        i = i + 1\n    acc\nfn recurse(n: i64) -> i64:\n    if n < 1: return 0\n    1 + recurse(n - 1)\nfn main():\n    let mut seed = 0\n    while seed < 8:\n        println(accum(seed))\n        seed = seed + 1\n    println(recurse(4))");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut expected = String::new();
    for seed in 0_i64..8 {
        let mut acc = seed;
        for i in 0..512 {
            acc = (acc * 31 + i) % 1000003;
        }
        expected.push_str(&format!("{acc}\n"));
    }
    expected.push_str("4\n");
    assert_eq!(String::from_utf8_lossy(&result.stdout), expected);
}

/// 运行源码，并用 ZAMAK_DUMP_C 取回入口模块生成的 C。
fn run_and_dump(case: &Case, source: &str) -> (std::process::Output, String) {
    let file = case.path("main.zm");
    let dump = case.path("dump");
    let _ = std::fs::remove_dir_all(&dump);
    std::fs::write(&file, source).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&file)
        .env("ZAMAK_CACHE_DIR", case.path("cache"))
        .env("ZAMAK_DUMP_C", &dump)
        .output()
        .unwrap();
    // 命中缓存时不会重新生成 C；每次用新的缓存目录保证能取到。
    let c = std::fs::read_to_string(dump.join("main.c")).unwrap_or_default();
    (output, c)
}

#[test]
fn list_indexes_bounded_by_a_stable_length_skip_checks() {
    // 长度不变的列表：for i in range(len(xs)) 与 let n = len(xs) 后的 range(a, n) 都直接访问元素。
    let case = Case::new("specializations", "list-index-proof");
    let (output, c) = run_and_dump(
        &case,
        "fn total(xs: &[i64]) -> i64:\n    let mut s = 0\n    for i in range(len(xs)):\n        s += xs[i]\n    s\n\nfn pairs(xs: &mut [i64]):\n    let n = len(xs)\n    for i in range(n):\n        for j in range(i + 1, n):\n            xs[j] += xs[i]\n\nfn main():\n    let mut xs: [i64] = [1, 2, 3]\n    println(total(xs))\n    pairs(&mut xs)\n    println(xs)\n",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        "6\n[1, 3, 7]\n"
    );
    assert!(c.contains("(&(p0)->data["), "{c}");
}

#[test]
fn list_indexes_keep_checks_when_the_length_can_change() {
    // 循环里改长度、以 &mut 传出、整体重新赋值、下标不是循环变量：都保留检查（负下标照常换算）。
    let case = Case::new("specializations", "list-index-unproven");
    for source in [
        "fn grow(xs: &mut [i64]):\n    for i in range(len(xs)):\n        if xs[i] == 1: xs.push(9)\n\nfn main():\n    let mut xs: [i64] = [1, 2]\n    grow(&mut xs)\n    println(xs)\n",
        "fn other(xs: &mut [i64]): xs.clear()\n\nfn shrink(xs: &mut [i64]):\n    for i in range(len(xs)):\n        println(xs[i])\n        other(&mut xs)\n\nfn main():\n    let mut xs: [i64] = [1, 2]\n    shrink(&mut xs)\n",
        "fn main():\n    let mut xs: [i64] = [1, 2, 3]\n    for i in range(len(xs)):\n        println(xs[i])\n        xs = [4]\n",
        "fn main():\n    let xs: [i64] = [1, 2, 3]\n    for i in range(len(xs)):\n        println(xs[i - 1])\n",
    ] {
        let (output, c) = run_and_dump(&case, source);
        // 直接访问只会写成 `(&(指针)->data[下标])`；这里一处都不该有。
        assert!(!c.contains(")->data[v"), "{source}\n{c}");
        assert!(!c.is_empty(), "{source}: {}", String::from_utf8_lossy(&output.stderr));
    }
    // 第二个程序在 clear 之后越界：照常报错。
    let (output, _) = run_and_dump(&case, "fn other(xs: &mut [i64]): xs.clear()\n\nfn shrink(xs: &mut [i64]):\n    for i in range(len(xs)):\n        println(xs[i])\n        other(&mut xs)\n\nfn main():\n    let mut xs: [i64] = [1, 2]\n    shrink(&mut xs)\n");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("list index out of bounds"));
    // 下标 i - 1 在第一轮是 -1：与 Python 一样取最后一个元素。
    let (output, _) = run_and_dump(&case, "fn main():\n    let xs: [i64] = [1, 2, 3]\n    for i in range(len(xs)):\n        println(xs[i - 1])\n");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        "3\n1\n2\n"
    );
}

#[test]
fn generic_type_methods_can_have_their_own_type_parameters() {
    let case = Case::new("specializations", "method-generics");
    // 方法自己带的类型参数从实参推断：`U` 来自传进来的函数值。
    let output = case.run("struct Box[T]:\n    value: T\n\n    fn map[U](&self, f: fn(T) -> U) -> U:\n        f(self.value)\n\nfn double(n: i64) -> i64: n * 2\n\nfn main():\n    let b = Box(value: 21)\n    println(b.map(double))\n");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "42\n");

    // 两个方法自己的类型参数，一个由函数值给出，一个由普通实参给出。
    let output = case.run("struct Box[T]:\n    value: T\n\n    fn combine[U, V](&self, f: fn(T, U) -> V, extra: U) -> V:\n        f(self.value, extra)\n\nfn add(n: i64, m: i64) -> i64: n + m\n\nfn main():\n    let b = Box(value: 10)\n    println(b.combine(add, 5))\n");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "15\n");

    // 同一份模板按不同的 U 各生成一份副本。
    let output = case.run("struct Box[T]:\n    value: T\n\n    fn map[U](&self, f: fn(T) -> U) -> U:\n        f(self.value)\n\nfn twice(n: i64) -> i64: n * 2\nfn wide(n: i64) -> f64: 1.5\n\nfn main():\n    let b = Box(value: 4)\n    println(b.map(twice))\n    println(b.map(wide))\n");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "8\n1.5\n");

    // 推不出来时要报错，并说清楚怎么改。
    let output = case.run("struct Box[T]:\n    value: T\n\n    fn make[U](&self) -> U:\n        self.value\n\nfn main():\n    let b = Box(value: 1)\n    println(b.make())\n");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot infer U"), "{stderr}");
}
