#include <stdio.h>
#include <stdlib.h>

typedef struct Node { struct Node *left, *right; } Node;

static Node *make(int depth) {
    Node *node = malloc(sizeof(Node));
    if (depth == 0) { node->left = node->right = NULL; }
    else { node->left = make(depth - 1); node->right = make(depth - 1); }
    return node;
}

static long check(const Node *node) {
    return 1 + (node->left ? check(node->left) + check(node->right) : 0);
}

static void release(Node *node) {
    if (node->left) { release(node->left); release(node->right); }
    free(node);
}

int main(int argc, char **argv) {
    int n = atoi(argv[1]);
    int max_depth = n > 6 ? n : 6, stretch = max_depth + 1;
    Node *tree = make(stretch);
    printf("stretch tree of depth %d\t check: %ld\n", stretch, check(tree));
    release(tree);
    Node *long_lived = make(max_depth);
    for (int depth = 4; depth <= max_depth; depth += 2) {
        long iterations = 1L << (max_depth - depth + 4), total = 0;
        for (long i = 0; i < iterations; i++) {
            Node *t = make(depth);
            total += check(t);
            release(t);
        }
        printf("%ld\t trees of depth %d\t check: %ld\n", iterations, depth, total);
    }
    printf("long lived tree of depth %d\t check: %ld\n", max_depth, check(long_lived));
    release(long_lived);
    return 0;
}
