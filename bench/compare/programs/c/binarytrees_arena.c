// binary-trees 的对照实现：与 binarytrees.c 做同样的计算，但节点从一个
// 连续 arena 里按块分配，用完一次性释放，不做逐节点 malloc/free。
//
// 用途：把「Zamak 的代码生成更快」和「Zamak 的分配策略更省」分开。
// Zamak 版本用子节点列表 + 小缓冲复用，本质上是 arena 风格的分配；
// 这个文件让 C 用同样的策略，两者再比才接近比较语言本身。
//
// 输出与 binarytrees.c 完全一致，校验和可直接对照。
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct Node { struct Node *left, *right; } Node;

// 每次 arena_reset 后从头分配；块不够就再要一块更大的。
// 单次 make 的最大节点数是 2^(depth+1)-1，depth 最大为 max_depth+1，
// 所以一块能容纳整棵树就够了，重置时机是「每棵短命树用完即重置」。
static Node *arena = NULL;
static size_t arena_used = 0;
static size_t arena_cap = 0;

static void arena_reset(void) { arena_used = 0; }

static void arena_ensure(size_t bytes) {
    if (bytes <= arena_cap) return;
    free(arena);
    arena = malloc(bytes);
    if (!arena) { fprintf(stderr, "allocation failed\n"); exit(1); }
    arena_cap = bytes;
    arena_used = 0;
}

static Node *alloc_node(void) {
    Node *node = &arena[arena_used++];
    return node;
}

static Node *make(int depth) {
    Node *node = alloc_node();
    if (depth == 0) { node->left = node->right = NULL; }
    else { node->left = make(depth - 1); node->right = make(depth - 1); }
    return node;
}

static long check(const Node *node) {
    return 1 + (node->left ? check(node->left) + check(node->right) : 0);
}

int main(int argc, char **argv) {
    int n = atoi(argv[1]);
    int max_depth = n > 6 ? n : 6, stretch = max_depth + 1;

    // 一棵深度 d 的树有 2^(d+1)-1 个节点；按最大那棵预留，之后只重置不复用。
    size_t nodes = ((size_t)1 << (stretch + 1)) - 1;
    arena_ensure(nodes * sizeof(Node));

    arena_reset();
    Node *tree = make(stretch);
    printf("stretch tree of depth %d\t check: %ld\n", stretch, check(tree));

    // 长命树与短命树要同时存在，所以长命树建在 arena 尾部单独一块。
    size_t long_nodes = ((size_t)1 << (max_depth + 1)) - 1;
    Node *long_arena = malloc(long_nodes * sizeof(Node));
    if (!long_arena) { fprintf(stderr, "allocation failed\n"); exit(1); }
    Node *saved = arena;
    size_t saved_cap = arena_cap, saved_used = arena_used;
    arena = long_arena;
    arena_cap = long_nodes;
    arena_used = 0;
    Node *long_lived = make(max_depth);
    arena = saved;
    arena_cap = saved_cap;
    arena_used = saved_used;

    for (int depth = 4; depth <= max_depth; depth += 2) {
        long iterations = 1L << (max_depth - depth + 4), total = 0;
        for (long i = 0; i < iterations; i++) {
            arena_reset();
            Node *t = make(depth);
            total += check(t);
        }
        printf("%ld\t trees of depth %d\t check: %ld\n", iterations, depth, total);
    }
    printf("long lived tree of depth %d\t check: %ld\n", max_depth, check(long_lived));

    free(long_arena);
    free(arena);
    return 0;
}
