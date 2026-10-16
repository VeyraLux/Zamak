import math
import sys


def a(i, j):
    return 1.0 / ((i + j) * (i + j + 1) // 2 + i + 1)


def times(u):
    n = len(u)
    return [sum(a(i, j) * u[j] for j in range(n)) for i in range(n)]


def times_transposed(u):
    n = len(u)
    return [sum(a(j, i) * u[j] for j in range(n)) for i in range(n)]


def at_times(u):
    return times_transposed(times(u))


n = int(sys.argv[1])
u = [1.0] * n
for _ in range(10):
    v = at_times(u)
    u = at_times(v)
vbv = sum(x * y for x, y in zip(u, v))
vv = sum(y * y for y in v)
print(f"{math.sqrt(vbv / vv):.9f}")
