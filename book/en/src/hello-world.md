# Hello, world

In this chapter, you run a one-file program, create a project, and build an executable you can share.

## Run a single file

Create a file named `hello.zm` with this content:

```zamak
fn main():
    println("Hello, world!")
```

Then run it:

```sh
zam run hello.zm
```

```text
Hello, world!
```

`zam run` checks the program, compiles it, and runs it. The first build takes a moment because `zam` calls the C compiler; later runs reuse the cache and start almost at once.

`zam` also prints progress lines such as `CACHE MISS` and `OUTPUT`. They go to standard error, so standard output contains only what your program prints, and you can redirect it to a file as usual.

## What's in the program

- Every program starts at `fn main()`.
- A line that ends with `:` opens a block. The lines of the block are indented 4 spaces deeper, and the block ends at the first line that's indented less, as in Python. Indent with spaces, not tabs.
- `println` prints a value followed by a newline. Strings use double quotes.

A block that holds one statement can stay on the same line:

```zamak
fn main(): println("Hello on one line")
```

```text
Hello on one line
```

Comments start with `#` and run to the end of the line:

```zamak
# Print a greeting in two languages.
fn main():
    println("Hello")  # English
    println("你好")    # Chinese
```

```text
Hello
你好
```

## Create a project

A project groups several files and has a name. To create one, run:

```sh
zam new hello
```

`zam new` creates this layout:

```text
hello/
├── .gitignore
├── zam.toml
└── src/
    └── main.zm
```

`zam.toml` describes the project:

```toml
[project]
name = "hello"
entry = "src/main.zm"
modules = "src"
```

- `name` names the executable.
- `entry` is the file that contains `fn main()`.
- `modules` is the folder that holds the project's other files. You'll split a program into modules in [Modules and packages](modules.md).

Run the project from its folder, or pass the folder name:

```sh
cd hello
zam run
```

```console
hello from zamak
```

## Build an executable

`zam build` compiles the project without running it and prints the path of the executable on the `OUTPUT` line:

```sh
zam build
```

```console
OUTPUT ./build/hello-3e93e504925813ae...
```

The file name ends with a hash of its contents, so builds of different versions never overwrite each other. The executable is self-contained: copy it to another computer with the same operating system, and it runs without Zamak installed.

## Pass arguments

Arguments after `--` go to your program:

```sh
zam run hello.zm -- one two
```

You'll read them with `env.args()` in [Standard library tour](standard-library.md).

## Summary

- `zam run file.zm` checks, compiles, and runs one file.
- `zam new name` creates a project, and `zam run` runs it.
- `zam build` writes a standalone executable to `build/`.
- Blocks use indentation, comments start with `#`, and `println` prints a line.
