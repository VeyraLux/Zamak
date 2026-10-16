/* 被 Zamak 调用的 C 函数：参数和返回值只用 int64_t、double、int 与 const char *。 */
#include <stdio.h>
#include <string.h>
#include "textutil.h"

int64_t text_vowels(const char *text) {
    int64_t count = 0;
    for (; *text; text++) {
        if (strchr("aeiouAEIOU", *text)) count++;
    }
    return count;
}

/* 返回的字符串由 C 这边管理：Zamak 会复制一份，所以用静态缓冲区就行。 */
const char *text_greeting(const char *name) {
    static char buffer[256];
    snprintf(buffer, sizeof buffer, "hello, %s", name);
    return buffer;
}

double text_scale(double value, int64_t times) { return value * (double)times; }

int text_is_even(int64_t n) { return n % 2 == 0; }

void text_log(const char *message) {
    fflush(stdout);
    fprintf(stderr, "[c] %s\n", message);
}
