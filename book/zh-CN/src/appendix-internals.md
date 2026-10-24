# 附录 E：Zamak 是怎么工作的

这篇附录讲从 `zam run main.zm` 到程序输出之间发生了什么。写 Zamak 用不到这些知识，但它能回答“内存在哪里释放”“为什么第二次构建这么快”这类问题。

## 从源码到可执行文件

`zam` 自己不生成机器码。它把你的程序翻译成 C，剩下的交给 C 编译器：

1. **解析。** `zam` 读入每个 `.zm` 文件，建出语法树。缩进变成代码块，和 Python 一样。
2. **检查。** 推断并检查类型、展开泛型、检查所有权：每个值归谁所有，有没有在移走之后又被使用。`zam check` 报的每一个错误都来自这一步。
3. **生成 C。** 每个模块写成一个 C11 文件，再加上一个小运行时，里面有字符串、列表、字典，以及本附录后面讲到的各种检查。
4. **编译和链接。** 调用系统里找到的 C 编译器：Windows 上是 MSVC，Linux 和 macOS 上是 GCC 或 Clang。链接器生成一个独立的可执行文件，运行时不需要 Zamak、运行时库或虚拟机。

第 2 到第 4 步都有缓存（见[后文](#构建缓存)），所以大多数构建都能跳过大部分工作。

## 代码会变成什么

下面这个程序用到了字符串列表、会失败的函数、泛型函数和接口：

```zamak
interface Shape:
    fn area(&self) -> f64

struct Square:
    side: f64

    fn area(&self) -> f64:
        self.side * self.side

fn parse(text: &string) -> i64!string:
    if text == "":
        fail "empty"
    int(text)?

fn largest[T](items: &[T]) -> T:
    let mut best = items[0]
    for x in items:
        if x > best:
            best = x
    best

fn main():
    let names = ["ada", "grace"]
    let nums: [i64] = [3, 9, 4]
    let n = parse("42") catch e: 0
    let shapes: [Shape] = [Square(side: 2.0)]
    println(f"{len(names)} {n + 1} {largest(nums)} {shapes[0].area()}")
```

```text
2 43 9 4.0
```

下面看每一部分在生成的 C 里是什么样子。实际输出里的名字是编码过的，例如 `main:parse` 写作 `f6d61696e3a7061727365`；下面的片段换成了好读的名字。

**字符串**是一个指向 UTF-8 字节的指针、按字节算的长度和按字符算的长度。列表是指针、长度和容量，和 Python 的列表或 Rust 的 `Vec` 一样：

```c
typedef struct { unsigned char *data; size_t len; size_t chars; } S;
```

**会失败的函数**返回一个小结构体，里面要么是值，要么是错误。`fail` 填入错误，`?` 在 `ok` 为 0 时提前返回，`catch` 把值换掉：

```c
struct Result_i64 { int ok; int64_t value; S error; };

Result_i64 r = parse(&text);
if (!r.ok) {          // catch e: 0
    drop(&r.error);
    r.ok = 1;
    r.value = 0;
}
```

这里没有异常，也没有栈展开，所以出错的开销和返回一个普通值一样。

**泛型函数**按用到的每种类型各编译一份。`largest(nums)` 调用的是为 `i64` 生成的 C 函数，名字是 `main:largest[Int]`。每一份都和手写的专用代码一样快。

**接口值**里有一个指向数据的指针，和一个指向函数表的指针，表里每个方法占一项。`shapes[0].area()` 就是在这张表里找到 `area` 再调用。只有通过接口或函数值的调用才经过指针；其他地方，C 编译器都知道调用的是哪个函数，可以把它内联。

```c
struct Shape_table { void (*drop)(void *); void (*show)(S *, void *); double (*area)(void *); };
double Shape_area(Shape *p) { return p->table->area(p->data); }
```

## 不用垃圾回收怎么管内存

每个值都只有一个所有者：一个变量、一个字段或一个列表元素。所有者离开作用域时，`zam` 会插入释放它的代码。上面例子里 `main` 的末尾，生成的 C 是这样的：

```c
drop_list_of_strings(&names);
drop_list_of_i64(&nums);
drop_list_of_shapes(&shapes);
```

靠三条规则保证安全，检查器在生成任何 C 代码之前就把它们全部查一遍：

- **赋值就是移动。** `let b = a` 把值交给 `b`，之后再用 `a` 是编译错误，所以同一个值不可能被释放两次。数字、布尔值，以及只由这类简单值组成的结构体，赋值时直接复制，因为复制它们很便宜。
- **借用只持续一次调用。** `&` 或 `&mut` 参数把值借给函数，函数返回时借用就结束了。函数不能把借用存起来，也不能返回借用，所以不会有指针活得比它指向的值更久，你也永远不用写生命周期标注。
- **同一时间只能有一个可变借用。** 函数拿着 `&mut x` 时，别的地方不能读也不能改 `x`。

代价是可以预期的：内存在确定的位置释放，没有停顿，也没有后台回收线程。

## 让程序停下的检查

整数运算和下标在运行时检查。`+`、`-`、`*` 溢出时，调用的是 C 编译器自带的溢出检查函数；下标超出列表范围时，程序带着说明停下：

```c
if (__builtin_add_overflow(a, b, &result)) arithmetic_error();
if (index < 0 || (uintmax_t)index >= length) { panic("list index out of bounds"); }
```

程序会停下，而不是带着错误的数字继续算，或者去读不属于它的内存。能证明一个值不会越界时，`zam` 就省掉这次检查。例如对一个 4 个元素的数组写 `for i in 0..4`，`a[i]` 和 `i + 1` 都不做检查。如果某个函数需要整数回绕（例如哈希函数），给它加上 `@wrapping`。

## 构建缓存

`zam` 的缓存在 `~/.zam/cache`，电脑上所有项目共用。缓存分三层，`zam` 每层都会打印一行：

| 输出行 | 缓存的是什么 | 键里包含什么 |
| --- | --- | --- |
| `CACHE` | 检查过的模块 | 模块的源码，以及它用到的模块 |
| `OBJECT` | 编译好的 C 目标文件 | 生成的 C、C 编译器及其版本、编译选项 |
| `LINK` | 可执行文件 | 所有目标文件和链接选项 |

键是内容的哈希，不是文件名或修改时间。因此：

- 同样的代码构建两次，即使在另一个文件夹或 Git worktree 里，也只编译一次。
- 改了一个模块，只重新构建这个模块和用到它的模块，别的都不动。
- 升级 `zam` 或 C 编译器会改变键，旧的结果不会被误用。

`zam` 先把缓存内容写进临时文件，再改名放到位，所以两个同时运行的构建永远看不到写了一半的内容。它还会记住找到的 C 编译器和版本，所以什么都没改的构建几毫秒就结束了。

## 内置的 C 库

`zam` 的可执行文件里带着三个 C 库的源码：用于 HTTPS 的 Mbed TLS、用于 `std/ui` 的 libui-ng，以及用于 `std/sqlite` 的 SQLite。程序第一次用到某个库时，`zam` 把它的源码写进缓存并在那里编译。之后的程序直接复用编译好的目标文件，所以一台电脑上只有第一个 HTTPS 程序要花大约 13 秒编译 Mbed TLS。

这些库会成为你的可执行文件的一部分。分发程序时，请附上它们的许可声明，这些声明列在 Zamak 仓库的 `THIRD-PARTY-NOTICES.md` 里。

## 线程

`thread.spawn` 启动一个操作系统线程。参数会移动到新线程，调用 `spawn` 之后，原来的线程就碰不到它们了。线程之间通过 `thread.Queue` 交换值，每个值也是移动过去的。因为任何两个线程都不可能同时拿到同一个可变的值，Zamak 程序不会有数据竞争，你自己的数据也不需要加锁。
