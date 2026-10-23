/* 一个很小的 C 库，演示 Zamak 怎样调用带句柄的 C 接口。
   函数用的是普通的 C 类型（int、size_t、unsigned char *）：zam.toml 列出了这个头文件，
   Zamak 按这里的原型调用，参数由 C 编译器转换。 */
#ifndef COUNTER_H
#define COUNTER_H
#include <stddef.h>

typedef struct counter counter;

/* 名字为空时失败，返回 NULL。 */
counter *counter_new(const char *name);
/* 成功返回 0 并把句柄写进 *out；失败返回 1，*out 为 NULL。 */
int counter_open(const char *name, counter **out);
void counter_add(counter *c, int amount);
int counter_value(const counter *c);
/* 把名字写进 buf（最多 n 个字节），返回写了多少。 */
size_t counter_name(const counter *c, unsigned char *buf, size_t n);
void counter_free(counter *c);
/* 已经释放了多少个 counter：用来确认每个句柄正好释放一次。 */
int counter_freed(void);
/* 字节之和对 65521 取余。 */
int checksum(const unsigned char *data, size_t n);

#endif
