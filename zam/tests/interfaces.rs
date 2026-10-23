mod common;

use common::Case;

fn stdout(output: &std::process::Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

#[test]
fn interface_values_dispatch_and_bounds_specialize() {
    let case = Case::new("interfaces", "example");
    let output = case.run(include_str!("../../examples/structs/interfaces.zm"));
    assert_eq!(
        stdout(&output),
        "circle: 3.0\nlabel hi: 4.0\ncircle: 0.75\nlabel hi\n24.0\n\
[Circle(r: 1.0), Label(text: \"hi\", size: 12.0), Circle(r: 0.5)]\n15.0\n27.75\n"
    );
}

#[test]
fn interface_values_in_fields_returns_and_fallible_methods() {
    let case = Case::new("interfaces", "values");
    let output = case.run(
        "interface Source:\n    fn read(&mut self) -> string!string\n    fn label(&self) -> string\n\n\
struct Fixed:\n    items: [string]\n\n    fn read(&mut self) -> string!string:\n        if len(self.items) == 0:\n            fail \"empty\"\n\n        self.items.pop()\n\n    fn label(&self) -> string: \"fixed\"\n\n\
struct Counter:\n    n: i64\n\n    fn read(&mut self) -> string!string:\n        self.n += 1\n        str(self.n)\n\n    fn label(&self) -> string: f\"counter at {self.n}\"\n\n\
struct Pipeline:\n    name: string\n    input: Source\n\n\
fn pick(fixed: bool) -> Source:\n    if fixed:\n        return Fixed(items=[\"b\", \"a\"])\n\n    Counter(n=10)\n\n\
fn drain[T: Source](source: &mut T, times: i64) -> [string]:\n    let mut out: [string] = []\n\n    for i in range(times):\n        let value = source.read() catch e:\n            out.push(f\"<{e}>\")\n            continue\n\n        out.push(value)\n\n    out\n\n\
fn main() -> !string:\n    let mut p = Pipeline(name=\"demo\", input=pick(true))\n    println(p.input.read()?)\n    println(p.input.label())\n    let mut counter = pick(false)\n    println(drain(&mut counter, 3))\n    println(counter.label())\n    let mut fixed = Fixed(items=[\"x\"])\n    println(drain(&mut fixed, 2))\n    println(p)\n",
    );
    assert_eq!(
        stdout(&output),
        "a\nfixed\n[\"11\", \"12\", \"13\"]\ncounter at 13\n[\"x\", \"<empty>\"]\n\
Pipeline(name: \"demo\", input: Fixed(items: [\"b\"]))\n"
    );
}

#[test]
fn interfaces_cross_modules() {
    let case = Case::new("interfaces", "modules");
    let project = case.path("app");
    std::fs::create_dir_all(project.join("src")).unwrap();
    std::fs::write(
        project.join("zam.toml"),
        "[project]\nname = \"app\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/geo.zm"),
        "pub interface Shape:\n    fn area(&self) -> f64\n\npub fn total(shapes: &[Shape]) -> f64:\n    let mut t = 0.0\n\n    for s in shapes:\n        t += s.area()\n\n    t\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/main.zm"),
        "use geo\n\nstruct Sq:\n    s: f64\n\n    fn area(&self) -> f64: self.s * self.s\n\nfn main():\n    let items: [geo.Shape] = [Sq(s=2.0), Sq(s=3.0)]\n    println(geo.total(items))\n",
    )
    .unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&project)
        .env("ZAMAK_CACHE_DIR", case.path("cache"))
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap();
    assert_eq!(stdout(&output), "13.0\n");
}

#[test]
fn interface_mistakes_are_reported() {
    let case = Case::new("interfaces", "errors");
    let shape = "interface Shape:\n    fn area(&self) -> f64\n\nstruct Dot:\n    x: i64\n\nstruct Sq:\n    s: f64\n\n    fn area(&self) -> f64: self.s\n\nstruct Bad:\n    x: i64\n\n    fn area(&self) -> i64: self.x\n\n";
    for (source, message) in [
        (
            format!("{shape}fn main():\n    let s: Shape = Dot(x=1)\n"),
            "Dot does not satisfy Shape: it has no method area",
        ),
        (
            format!("{shape}fn total[T: Shape](item: &T) -> f64: item.area()\n\nfn main(): println(total(Bad(x=1)))\n"),
            "total needs T to satisfy the bound: Bad.area has a different signature from Shape.area",
        ),
        (
            format!("{shape}fn main():\n    let s = Shape()\n"),
            "Shape is an interface",
        ),
        (
            format!("{shape}fn main():\n    let s: [Shape] = [Sq(s=1.0)]\n    let t = copy(&s)\n"),
            "an interface value cannot be copied",
        ),
        (
            "interface Bad:\n    fn eat(self) -> i64\n\nfn main(): pass\n".to_owned(),
            "interface method Bad.eat takes &self or &mut self first",
        ),
        (
            "interface Bad:\n    fn eat(&self) -> i64: 1\n\nfn main(): pass\n".to_owned(),
            "interface method Bad.eat has no body",
        ),
    ] {
        let output = case.run(&source);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "accepted {source}");
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
}
