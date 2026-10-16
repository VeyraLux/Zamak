import sys


def make(depth):
    return (None, None) if depth == 0 else (make(depth - 1), make(depth - 1))


def check(node):
    left, right = node
    return 1 if left is None else 1 + check(left) + check(right)


n = int(sys.argv[1])
max_depth = max(6, n)
stretch = max_depth + 1
print(f"stretch tree of depth {stretch}\t check: {check(make(stretch))}")
long_lived = make(max_depth)
for depth in range(4, max_depth + 1, 2):
    iterations = 1 << (max_depth - depth + 4)
    total = sum(check(make(depth)) for _ in range(iterations))
    print(f"{iterations}\t trees of depth {depth}\t check: {total}")
print(f"long lived tree of depth {max_depth}\t check: {check(long_lived)}")
