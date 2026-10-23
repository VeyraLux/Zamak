# 函数

函数有名字、带类型的参数，以及可选的结果类型。参数和结果的类型必须写：它们说明了函数的用法，也让编译器能检查每一次调用。

## 定义和调用函数

```zamak
fn area(width: f64, height: f64) -> f64:
    width * height

fn greet(name: &string):
    println(f"Hello, {name}!")

fn main():
    println(area(3, 4.5))
    greet("Ada")
```

```text
13.5
Hello, Ada!
```

- `-> f64` 是结果类型。没有 `->` 的函数不返回值。
- 函数体里最后一个表达式就是结果，所以 `area` 不需要 `return`。也可以写 `return value`，提前返回时很方便。
- `name: &string` 借用字符串，而不是把它拿走。只读取字符串、列表或字典的函数，大多这样接收参数。[借用](borrowing.md)一章会解释原因。

函数体只有一个表达式时，可以写成一行：

```zamak
fn square(n: i64) -> i64: n * n

fn main():
    println(square(12))
```

```text
144
```

## 默认参数与关键字参数

参数可以有默认值，调用时可以按名字传，写法与 Python 相同：

```zamak
fn connect(host: &string, port: i64 = 80, timeout: f64 = 5.0) -> string:
    f"{host}:{port} ({timeout}s)"

fn main():
    println(connect("example.com"))
    println(connect("example.com", 8080))
    println(connect("example.com", timeout=1.5))
```

```text
example.com:80 (5.0s)
example.com:8080 (5.0s)
example.com:80 (1.5s)
```

- 默认值只能是常量（字面量、`const` 常量和它们的运算）或 `Option.None`。默认值在每次调用时重新放进去，不会像 Python 那样被多次调用共用。
- 有默认值的参数后面，参数都要有默认值。
- 调用时先写位置实参，再写 `名字=值`。方法也一样：`p.moved(dy=5)`。
- 结构体也可以这样构造：`Point(x=1, y=2)` 和 `Point(x: 1, y: 2)` 相同。

## 提前返回

`return` 立即离开函数。先处理特殊情况时很有用：

```zamak
fn grade(score: i64) -> string:
    if score < 0 or score > 100:
        return "invalid"
    if score >= 90:
        return "A"
    if score >= 60:
        return "pass"
    "fail"

fn main():
    println(grade(95))
    println(grade(70))
    println(grade(30))
    println(grade(120))
```

```text
A
pass
fail
invalid
```

## 返回多个值

要返回不止一个值，就返回一个元组，再用 `let` 拆开：

```zamak
fn divmod(a: i64, b: i64) -> (i64, i64):
    (a // b, a % b)

fn main():
    let (quotient, remainder) = divmod(17, 5)
    println(f"{quotient} remainder {remainder}")
```

```text
3 remainder 2
```

## 递归

函数可以调用自己：

```zamak
fn factorial(n: i64) -> i64:
    if n <= 1: return 1
    n * factorial(n - 1)

fn main():
    println(factorial(20))
```

```text
2432902008176640000
```

`factorial(21)` 超出了 `i64` 的范围，所以程序会带着溢出错误停下，而不是打印一个错误的数。

## 泛型函数

泛型函数适用于多种类型。在函数名后面的方括号里写类型参数：

```zamak
fn largest[T](items: &[T]) -> T:
    let mut best = items[0]
    for item in items:
        if item > best: best = item
    best

fn main():
    let numbers: [i64] = [3, 9, 4]
    let prices: [f64] = [2.5, 1.25]
    println(largest(numbers))
    println(largest(prices))
```

```text
9
2.5
```

调用时不用写出类型，Zamak 会根据实参推断 `T`。`T` 只出现在返回类型里时，按接收结果的变量的标注推断：`let b: Box[i64] = empty()`。然后它为每种类型各编译一份函数，所以泛型代码和只为一种类型写的代码一样快。编译器针对你用到的每种类型检查函数体，所以对不支持 `>` 的类型调用 `largest` 会在编译时报错。

## 函数作为值

函数是值，可以传给另一个函数。函数类型写作 `fn(i64) -> bool`：

```zamak
fn is_even(n: i64) -> bool: n % 2 == 0

fn count_if(numbers: &[i64], keep: fn(i64) -> bool) -> i64:
    let mut count = 0
    for n in numbers:
        if keep(n): count += 1
    count

fn main():
    let numbers: [i64] = [1, 2, 3, 4, 5, 6]
    println(count_if(numbers, is_even))
    println(count_if(numbers, fn(n): n > 4))
```

```text
3
2
```

`fn(n): n > 4` 是匿名函数。它的函数体是一个表达式，参数类型由 Zamak 根据使用的位置推断。也可以写出类型：`fn(n: i64) -> bool: n > 4`。

## 闭包

匿名函数和在函数里声明的函数可以使用周围的变量，还可以修改用 `let mut` 声明的变量：

```zamak
fn main():
    let mut total = 0
    fn add(n: i64):
        total += n
    add(5)
    add(7)
    println(total)

    let limit = 10
    let is_small = fn(x: i64) -> bool: x < limit
    let numbers: [i64] = [3, 12, 7, 20]
    let mut kept = 0
    for n in numbers:
        if is_small(n): kept += 1
    println(kept)
```

```text
12
2
```

用到周围变量的函数叫闭包。闭包可以调用、用 `let` 绑定、作为实参传递，但不能存进结构体或列表，也不能从函数返回。这条规则保证闭包不会比它用到的变量活得更久，Zamak 正是靠这一点在没有垃圾回收器的情况下支持闭包。不用外部变量的函数没有这些限制。

## 函数里定义的函数

在一个函数里定义的函数可以调用自己，适合只给这个函数用的帮手。返回类型要写出来，因为没有别的地方能推断它：

```zamak
fn main():
    fn fact(n: i64) -> i64:
        if n < 2: return 1
        return n * fact(n - 1)

    fn stars(n: i64) -> string:
        if n < 1: return ""
        return "*" + stars(n - 1)

    println(fact(5))
    println(stars(4))
```

```text
120
****
```

递归和其他调用一样检查：实参按这个函数自己的参数类型核对，结果就是它声明的类型。函数里定义的函数照样能读周围的变量，所以递归的帮手也可以用它们。它不能调用在它后面才定义的函数，需要互相调用时按从上到下的顺序写。

## 小结

- 参数可以有默认值（`port: i64 = 80`），调用时可以按名字传（`connect(host, timeout=1.5)`）。
- `fn name(param: Type) -> Result:` 定义函数，最后一个表达式就是结果。
- 用元组返回多个值，用 `let (a, b) = ...` 拆开。
- `fn name[T](...)` 定义泛型函数；Zamak 推断 `T`，并为每种类型编译一份高效的版本。
- 函数是值，类型形如 `fn(i64) -> bool`。闭包可以使用并修改周围的变量。
