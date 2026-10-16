// 读入标准输入，按空白切词，开放寻址哈希表计数（FNV-1a，容量满一半时翻倍）。
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct { const char *word; size_t len; long count; } Entry;
static Entry *table;
static size_t cap = 1 << 16, used;

static uint64_t hash(const char *p, size_t n) {
    uint64_t h = 1469598103934665603ULL;
    for (size_t i = 0; i < n; i++) { h ^= (unsigned char)p[i]; h *= 1099511628211ULL; }
    return h;
}

static void insert(const char *p, size_t n, long add);

static void grow(void) {
    Entry *old = table; size_t old_cap = cap;
    cap *= 2; used = 0;
    table = calloc(cap, sizeof(Entry));
    for (size_t i = 0; i < old_cap; i++) if (old[i].word) insert(old[i].word, old[i].len, old[i].count);
    free(old);
}

static void insert(const char *p, size_t n, long add) {
    if (2 * (used + 1) > cap) grow();
    for (size_t i = hash(p, n) & (cap - 1);; i = (i + 1) & (cap - 1)) {
        Entry *e = &table[i];
        if (!e->word) { e->word = p; e->len = n; e->count = add; used++; return; }
        if (e->len == n && memcmp(e->word, p, n) == 0) { e->count += add; return; }
    }
}

int main(void) {
    size_t size = 0, room = 1 << 20;
    char *text = malloc(room);
    size_t got;
    while ((got = fread(text + size, 1, room - size, stdin)) > 0) {
        size += got;
        if (size == room) { room *= 2; text = realloc(text, room); }
    }
    table = calloc(cap, sizeof(Entry));
    for (size_t i = 0; i < size;) {
        while (i < size && (text[i] == ' ' || text[i] == '\n' || text[i] == '\r' || text[i] == '\t')) i++;
        size_t start = i;
        while (i < size && !(text[i] == ' ' || text[i] == '\n' || text[i] == '\r' || text[i] == '\t')) i++;
        if (i > start) insert(text + start, i - start, 1);
    }
    long best = 0;
    for (size_t i = 0; i < cap; i++) if (table[i].word && table[i].count > best) best = table[i].count;
    printf("%zu\n%ld\n", used, best);
    return 0;
}
