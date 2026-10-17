# 写第一个程序

在这一章，你会写一个程序：统计一段文字里的单词，打印出现最多的几个。一路上会碰到变量、循环、函数、字典、结构体和排序。后面每个概念都有专门的一章，所以现在不必纠结细节。

## 第 1 步：统计单词

新建文件 `words.zm`：

```zamak
fn main():
    let text = "the quick brown fox jumps over the lazy dog and the fox runs"
    let mut counts: {string: i64} = {}
    for word in text.words():
        counts[word] = counts.get(word, 0) + 1
    println(counts)
```

用 `zam run words.zm` 运行：

```text
{"the": 3, "quick": 1, "brown": 1, "fox": 2, "jumps": 1, "over": 1, "lazy": 1, "dog": 1, "and": 1, "runs": 1}
```

每一行的作用：

- `let text = "..."` 给一个字符串起名字。用 `let` 绑定的名字不能再改变。
- `let mut counts: {string: i64} = {}` 创建一个从字符串到整数的空字典。`mut` 表示这个变量可以改变；空字典看不出要装什么，所以需要写类型标注。
- `text.words()` 按空白切分文字，`for word in ...` 对每个单词执行一遍缩进的代码块。
- `counts.get(word, 0)` 读出当前计数；单词还不在字典里时得到 `0`。`counts[word] = ...` 存入新的计数。
- `println(counts)` 打印整个字典。字典按插入顺序保存键。

## 第 2 步：把统计挪进函数

程序变长以后，用函数保持代码清晰。把统计部分挪进 `count_words`：

```zamak
fn count_words(text: &string) -> {string: i64}:
    let mut counts: {string: i64} = {}
    for word in text.words():
        counts[word] = counts.get(word, 0) + 1
    counts

fn main():
    let counts = count_words("the quick brown fox jumps over the lazy dog and the fox runs")
    println(f"{len(counts)} different words")
    let n = counts["the"]
    println(f"'the' appears {n} times")
```

```text
10 different words
'the' appears 3 times
```

- `text: &string` 表示 `count_words` 借用这个字符串：它可以读，但字符串仍归调用方所有。[借用](borrowing.md)一章会详细讲。
- `-> {string: i64}` 是返回类型。函数里最后一个表达式（这里是 `counts`）就是返回值。
- `f"..."` 构造字符串，里面的每个 `{...}` 会被替换成表达式的值。`len(counts)` 是字典里的条目数。
- `counts["the"]` 按键读取值。键不存在时程序会报错停下；键可能不存在时用 `get`。

## 第 3 步：打印最常见的单词

要打印排名靠前的单词，就把每个单词和它的次数放进一个结构体，排序后只保留前几个：

```zamak
const TEXT = "the quick brown fox jumps over the lazy dog and the fox runs"

struct WordCount:
    word: string
    count: i64

fn count_words(text: &string) -> {string: i64}:
    let mut counts: {string: i64} = {}
    for word in text.words():
        counts[word] = counts.get(word, 0) + 1
    counts

fn top(counts: &{string: i64}, n: i64) -> [WordCount]:
    let mut all: [WordCount] = []
    for word, count in counts:
        all.push(WordCount(word: word, count: count))
    all.sort_by(fn(a, b): a.count > b.count)
    all[:n]

fn main():
    let counts = count_words(TEXT)
    println(f"{len(counts)} different words")
    for entry in top(counts, 3):
        println(f"{entry.word}: {entry.count}")
```

```text
10 different words
the: 3
fox: 2
quick: 1
```

新出现的内容：

- `const TEXT = ...` 声明一个常量。常量名全部大写。
- `struct WordCount:` 声明一个有两个字段的类型，`WordCount(word: word, count: count)` 创建这个类型的值。
- `for word, count in counts:` 同时遍历字典的键和值。
- `all.sort_by(fn(a, b): a.count > b.count)` 给列表排序。这个匿名函数在 `a` 应该排在 `b` 前面时返回 `true`，所以次数大的排在前面。排序是稳定的：次数相同的单词保持原来的顺序。
- `all[:n]` 复制前 `n` 项，与 Python 的切片相同。

## 动手试试

- 把 `top(counts, 3)` 改成打印前五个单词。
- 统计 `word.lower()` 而不是 `word`，让大小写不同的同一个词算作一个。
- 按照[标准库速览](standard-library.md)里的写法，用 `io.read_all()` 从标准输入读取文字。

## 小结

你写出了一个完整的程序，它用到了日常 Zamak 代码里的大部分内容。接下来的几章会逐一详细介绍，先从[变量与类型](variables.md)开始。
