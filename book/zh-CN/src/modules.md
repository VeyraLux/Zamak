# 模块与包

程序变大以后，把它拆成模块。每个 `.zm` 文件就是一个模块，由模块自己决定哪些东西给别的模块用。包让你在不同项目之间共享模块。

## 把项目拆成模块

在项目里，`src/` 下的每个文件都是一个以文件名命名的模块。这个项目有两个模块：

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

- `use geometry` 导入 `src/geometry.zm` 这个模块。
- 使用导入的内容时，写模块名加点：`geometry.square(3)`、`geometry.Rect(...)`、`geometry.UNIT`。
- 只有标了 `pub` 的内容在模块外可见。`helper` 没有标，所以 `main.zm` 调用不了它；你可以随意修改它，不会影响其他模块。

模块也可以放在子目录里。`src/net/http.zm` 用 `use net/http` 导入，用 `http.get(...)` 调用。

## 标准库

标准库是 `std/` 下的一组模块，导入方式相同：

```zamak
use std/math
use std/fmt

fn main():
    println(fmt.fixed(math.pi, 3))
```

```text
3.142
```

少数函数完全不用导入，例如 `println`、`len`、`int`、`min` 和 `sorted`。其余的见[标准库速览](standard-library.md)。

## 为什么模块让构建更快

`zam` 分别编译每个模块，并按内容缓存结果。改动一个模块时，只重新构建这个模块，以及依赖它公开内容的模块。在大项目里改一行代码，大约一秒就能构建完。

## 使用包

包是可以被别的项目依赖的项目。在[包索引](https://github.com/VeyraLux/zamak-index)里查找包：

```sh
zam search json
```

把它加到你的项目里：

```sh
zam add mathx
```

`zam add` 会把依赖写进 `zam.toml`：

```toml
[dependencies]
mathx = "^1.4.0"
```

然后像导入自己的模块一样导入它：`use mathx` 或 `use mathx/geo`。

版本要求与 Cargo 相同：`^1.2` 接受任何兼容的 1.x 版本，`~1.2` 接受 1.2.x，`=1.2.3` 固定一个版本。`zam` 选出所有包都接受的最高版本，并记在 `zam.lock` 里，所以每次构建用的都是同一份代码，离线也一样。

也可以依赖一个 Git 仓库或本地目录：

```sh
zam add mathx https://github.com/you/mathx v1.0
zam add util ../util
```

删除依赖运行 `zam remove mathx`。要升级到允许范围内的最新版本，运行 `zam update`。

## 发布包

分享你自己的库：

1. 在 `zam.toml` 里写上 `version` 和 `description`。库没有 `entry`。
2. 提交，给这次提交打上 `v<版本号>` 标签（例如 `v1.0.0`），并推送标签。
3. 运行 `zam publish`。

`zam publish` 会检查项目、确认标签与版本一致，然后打印要提交到包索引的条目。

## 小结

- 每个文件是一个模块。`use name` 导入它，只有 `pub` 的内容在外部可见。
- 标准模块在 `std/` 下，例如 `use std/math`。
- `zam add`、`zam remove`、`zam update` 管理 `zam.toml` 里的依赖，`zam.lock` 固定它们的版本。
- `zam publish` 通过包索引分享库。
