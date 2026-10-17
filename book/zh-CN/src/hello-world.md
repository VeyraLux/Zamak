# Hello, world

在这一章，你会运行一个单文件程序、创建一个项目，并生成一个可以分享给别人的可执行文件。

## 运行单个文件

新建文件 `hello.zm`，写入以下内容：

```zamak
fn main():
    println("Hello, world!")
```

然后运行：

```sh
zam run hello.zm
```

```text
Hello, world!
```

`zam run` 检查程序、编译，然后运行。第一次构建要稍等片刻，因为 `zam` 要调用 C 编译器；之后再运行会用上缓存，几乎立即启动。

`zam` 还会打印 `CACHE MISS`、`OUTPUT` 这样的进度行。它们写到标准错误，所以标准输出里只有你的程序打印的内容，可以照常重定向到文件。

## 程序里有什么

- 每个程序都从 `fn main()` 开始执行。
- 以 `:` 结尾的行开始一个代码块。块里的行多缩进 4 个空格，遇到第一行缩进变少的行，块就结束了，与 Python 相同。缩进用空格，不用 Tab。
- `println` 打印一个值并换行。字符串用双引号。

只有一条语句的块可以写在同一行：

```zamak
fn main(): println("Hello on one line")
```

```text
Hello on one line
```

注释以 `#` 开头，一直到行尾：

```zamak
# 用两种语言打招呼。
fn main():
    println("Hello")  # 英文
    println("你好")    # 中文
```

```text
Hello
你好
```

## 创建项目

项目把多个文件组织在一起，并且有自己的名字。创建项目：

```sh
zam new hello
```

`zam new` 会生成下面的结构：

```text
hello/
├── .gitignore
├── zam.toml
└── src/
    └── main.zm
```

`zam.toml` 描述这个项目：

```toml
[project]
name = "hello"
entry = "src/main.zm"
modules = "src"
```

- `name` 是可执行文件的名字。
- `entry` 是包含 `fn main()` 的文件。
- `modules` 是存放项目其他文件的目录。在[模块与包](modules.md)一章里，你会把程序拆成多个模块。

在项目目录里运行，或者把目录名传给 `zam`：

```sh
cd hello
zam run
```

```console
hello from zamak
```

## 生成可执行文件

`zam build` 只编译不运行，并在 `OUTPUT` 行打印可执行文件的路径：

```sh
zam build
```

```console
OUTPUT ./build/hello-3e93e504925813ae...
```

文件名末尾是内容的哈希值，所以不同版本的构建结果不会互相覆盖。生成的可执行文件是独立的：复制到另一台操作系统相同的电脑上，不装 Zamak 也能运行。

## 传递参数

`--` 后面的参数会交给你的程序：

```sh
zam run hello.zm -- one two
```

在[标准库速览](standard-library.md)一章里，你会用 `env.args()` 读取它们。

## 小结

- `zam run file.zm` 检查、编译并运行一个文件。
- `zam new name` 创建项目，`zam run` 运行项目。
- `zam build` 在 `build/` 里生成独立的可执行文件。
- 代码块靠缩进界定，注释以 `#` 开头，`println` 打印一行。
