# Modules and packages

As a program grows, split it into modules. Each `.zm` file is a module, and a module chooses what other modules can use. Packages let you share modules between projects.

## Split a project into modules

In a project, the files in `src/` are modules named after the file. This project has two:

```zamak,project=shapes,file=src/geometry.zm
pub struct Rect:
    w: f64
    h: f64

    pub fn area(&self) -> f64:
        self.w * self.h

pub fn square(side: f64) -> Rect:
    Rect(w: side, h: side)

pub const UNIT = 1.0

fn helper() -> f64: 0.0
```

```zamak,project=shapes,file=src/main.zm
use geometry

fn main():
    let r = geometry.square(3)
    println(r.area())
    let big = geometry.Rect(w: 4, h: 2.5)
    println(big.area())
    println(geometry.UNIT)
```

```text
9.0
10.0
1.0
```

- `use geometry` imports the module in `src/geometry.zm`.
- Use an imported item by its module name and a dot: `geometry.square(3)`, `geometry.Rect(...)`, `geometry.UNIT`.
- Only items marked `pub` are visible outside their module. `helper` isn't, so `main.zm` can't call it, and you can change it freely without breaking other modules.

Modules can live in subfolders. `src/net/http.zm` is imported as `use net/http` and used as `http.get(...)`.

## The standard library

The standard library is a set of modules under `std/`. Import them the same way:

```zamak
use std/math
use std/fmt

fn main():
    println(fmt.fixed(math.pi, 3))
```

```text
3.142
```

A few functions need no import at all, such as `println`, `len`, `int`, `min`, and `sorted`. [Standard library tour](standard-library.md) lists the rest.

## Why modules make builds fast

`zam` compiles each module separately and caches the result by its content. When you change one module, only that module and the modules that depend on its public items are rebuilt. A one-line change in a large project rebuilds in about a second.

## Use a package

A package is a project that other projects can depend on. Find packages in the [package index](https://github.com/VeyraLux/zamak-index):

```sh
zam search json
```

Add one to your project:

```sh
zam add mathx
```

`zam add` writes the dependency to `zam.toml`:

```toml
[dependencies]
mathx = "^1.4.0"
```

Then import its modules like your own: `use mathx` or `use mathx/geo`.

Version requirements work like Cargo's: `^1.2` accepts any compatible 1.x version, `~1.2` accepts 1.2.x, and `=1.2.3` pins one version. `zam` picks the highest version that every package accepts and records the choice in `zam.lock`, so every build uses the same code, even offline.

You can also depend on a Git repository or a local folder:

```sh
zam add mathx https://github.com/you/mathx v1.0
zam add util ../util
```

To remove a dependency, run `zam remove mathx`. To move to the newest allowed versions, run `zam update`.

## Publish a package

To share your own library:

1. Give `zam.toml` a `version` and a `description`. A library has no `entry`.
2. Commit, tag the commit `v<version>`, such as `v1.0.0`, and push the tag.
3. Run `zam publish`.

`zam publish` checks the project, makes sure the tag matches the version, and prints the index entry to submit to the package index.

## Summary

- Each file is a module. `use name` imports it, and only `pub` items are visible outside.
- Standard modules live under `std/`, such as `use std/math`.
- `zam add`, `zam remove`, and `zam update` manage dependencies in `zam.toml`, and `zam.lock` pins them.
- `zam publish` shares a library through the package index.
