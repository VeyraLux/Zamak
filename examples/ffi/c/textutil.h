#ifndef TEXTUTIL_H
#define TEXTUTIL_H
#include <stdint.h>

int64_t text_vowels(const char *text);
const char *text_greeting(const char *name);
double text_scale(double value, int64_t times);
int text_is_even(int64_t n);
void text_log(const char *message);

#endif
