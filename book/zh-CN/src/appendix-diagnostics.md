# 附录 D：诊断码

`zam check` 和 `zam build` 报告的每个错误都有一个错误码。错误码的含义在各个版本之间保持不变，工具和 AI 助手可以依赖它。消息文字可能逐步改进，错误码不会变。

| 错误码 | 分类 | 含义 |
| --- | --- | --- |
| `ZM100` | syntax | Zamak 源码里不允许出现的字符。 |
| `ZM101` | syntax | 字符串里不支持的转义序列。 |
| `ZM110` | syntax | 语法错误，包括缩进问题，以及该写 `==` 的地方写成了 `=`。 |
| `ZM200` | type | 类型不匹配。 |
| `ZM201` | name | 未定义的变量、函数、结构体、类型或枚举变体。 |
| `ZM202` | name | 未定义的结构体字段。 |
| `ZM203` | name | 缺少导入或导入有歧义。按 help 的提示加上 `use` 行。 |
| `ZM204` | name | 重复声明：导入、字段、函数或变量。 |
| `ZM301` | ownership | 值被移动之后又被使用。 |
| `ZM302` | ownership | 给不是 `let mut` 的值赋值。 |
| `ZM406` | bounds | 定长数组的下标越界。 |
| `ZM421` | ownership | 借用冲突，例如在循环读取列表时修改它。 |
| `ZM422` | ownership | 借用比它借用的值活得更久，例如返回引用。 |
| `ZM423` | ownership | 借用的种类或类型不对。 |
| `ZM424` | ownership | 对不是 `let mut` 的值做可变借用。 |
| `ZM425` | ownership | 调用需要显式写 `&` 或 `&mut`。 |
| `ZM500` | error_handling | 可能失败的调用没有用 `?` 或 `catch` 处理。 |
| `ZM509` | error_handling | 错误类型不是 `string`、结构体或枚举。 |
| `ZM510` | control_flow | 函数并非在每条路径上都返回了值。 |
| `ZM511` | control_flow | `match` 没有覆盖所有变体，或者 `_` 之后还有分支。 |
| `ZM600` | arithmetic | 整数字面量或常量表达式溢出。 |
| `ZM900` | unsupported | Zamak 暂不支持的功能。 |
| `ZM999` | general | 尚未归类的错误。请附上一个小例子报告它。 |

## 运行时错误

下面这些消息来自正在运行的程序，而不是编译器。程序把消息打印到标准错误，并以状态码 1 退出。

| 消息 | 原因 |
| --- | --- |
| `zam: integer arithmetic error` | 整数运算溢出，或者 `i64` 的最小值除以 -1。 |
| `zam: division by zero` | 除以零或对零取余。 |
| `zam: list index out of bounds` | 下标超出列表范围。 |
| `zam: key not found` | 用 `m[k]` 读取字典里不存在的键。 |
| `zam: pop from empty list` | 对空列表调用 `pop`。 |
| `assertion failed: ...` | `assert` 的条件不成立。 |
