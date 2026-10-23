#include "counter.h"
#include <stdlib.h>
#include <string.h>

struct counter {
    char name[32];
    int value;
};

static int freed;

counter *counter_new(const char *name) {
    counter *c;
    if (!name[0]) return NULL;
    c = (counter *)calloc(1, sizeof *c);
    if (!c) return NULL;
    strncpy(c->name, name, sizeof c->name - 1);
    return c;
}

int counter_open(const char *name, counter **out) {
    *out = counter_new(name);
    return *out ? 0 : 1;
}

void counter_add(counter *c, int amount) { c->value += amount; }

int counter_value(const counter *c) { return c->value; }

size_t counter_name(const counter *c, unsigned char *buf, size_t n) {
    size_t len = strlen(c->name);
    if (len > n) len = n;
    memcpy(buf, c->name, len);
    return len;
}

void counter_free(counter *c) {
    freed++;
    free(c);
}

int counter_freed(void) { return freed; }

int checksum(const unsigned char *data, size_t n) {
    size_t i;
    int sum = 0;
    for (i = 0; i < n; i++) sum = (sum + data[i]) % 65521;
    return sum;
}
