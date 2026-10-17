# 编写测试

Zamak 内置了测试。测试就是名字以 `test_` 开头的函数，`zam test` 会找出并运行所有这样的函数。不需要测试框架，也不需要配置文件。

## 编写测试

把测试放在被测代码旁边。每个测试用 `assert` 写出应该成立的事情：

```zamak,test
fn slugify(title: &string) -> string:
    title.strip().lower().replace(" ", "-")

fn test_lowercases():
    assert slugify("Hello") == "hello"

fn test_replaces_spaces():
    assert slugify("Hello World") == "hello-world"

fn test_trims():
    assert slugify("  Hi  ") == "hi"

fn main():
    println(slugify("The Zamak Book"))
```

用 `zam test` 运行，参数是文件或项目目录：

```sh
zam test slugify.zm
```

```text
test test_lowercases ... ok
test test_replaces_spaces ... ok
test test_trims ... ok

test result: ok. 3 passed; 0 failed
```

测试函数没有参数，也不返回值。它也可以声明成 `-> !string`，这样里面就能用 `?`；返回错误的测试算失败。

## 测试失败时

断言不成立时，报告会显示文件、行号、条件和你写的消息：

```zamak,test
fn slugify(title: &string) -> string:
    title.strip().lower().replace(" ", "-")

fn test_spaces():
    assert slugify("A B") == "a_b", "spaces become underscores"

fn main():
    println(slugify("A B"))
```

```text
test test_spaces ... FAILED

---- test_spaces ----
main.zm:5: assertion failed: slugify("A B") == "a_b": spaces become underscores
(exit status 1)

test result: FAILED. 0 passed; 1 failed
```

只要有测试失败，`zam test` 就以状态码 1 退出，所以可以直接用在持续集成里。

每个测试都在自己的进程里运行。一个测试崩溃了（例如整数溢出或下标越界），只有它自己算失败，不会影响其他测试。

## 只运行一部分测试

在文件或目录后面加一个词，只运行名字里包含这个词的测试：

```sh
zam test slugify.zm trims
```

```text
test test_trims ... ok

test result: ok. 1 passed; 0 failed
```

## 测试出错的路径

要检查函数在该失败时确实失败，就用 `catch` 调用它，记下发生了什么：

```zamak,test
fn parse_percent(text: &string) -> i64!string:
    let n = int(text)?
    if n < 0 or n > 100:
        fail f"out of range: {n}"
    n

fn test_accepts_valid_values() -> !string:
    assert parse_percent("42")? == 42

fn test_rejects_out_of_range():
    let mut message = ""
    parse_percent("150") catch e:
        message = e
    assert message == "out of range: 150"

fn main() -> !string:
    println(parse_percent("7")?)
```

```text
test test_accepts_valid_values ... ok
test test_rejects_out_of_range ... ok

test result: ok. 2 passed; 0 failed
```

## 小结

- 测试是名字为 `test_...`、没有参数的函数，可以返回 `!string`。
- `assert cond, "message"` 写出必须成立的条件。
- `zam test` 让每个测试在自己的进程里运行并打印报告；加一个词只运行匹配的测试。
