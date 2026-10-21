#include <stdio.h>
#include <stdlib.h>
#include <time.h>

static double now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec / 1e6;
}

int main(int argc, char **argv) {
    int n = atoi(argv[1]);
    double t0 = now_ms();
    int perm1[32], perm[32], count[32];
    for (int i = 0; i < n; i++) perm1[i] = i;
    int max_flips = 0, checksum = 0, permutation = 0, r = n;
    for (;;) {
        while (r != 1) { count[r - 1] = r; r--; }
        for (int i = 0; i < n; i++) perm[i] = perm1[i];
        int flips = 0, k = perm[0];
        while (k) {
            for (int lo = 0, hi = k; lo < hi; lo++, hi--) { int t = perm[lo]; perm[lo] = perm[hi]; perm[hi] = t; }
            flips++;
            k = perm[0];
        }
        if (flips > max_flips) max_flips = flips;
        checksum += permutation % 2 == 0 ? flips : -flips;
        for (;;) {
            if (r == n) {
                fprintf(stderr, "TIME %.3f\n", now_ms() - t0);
                printf("%d\nPfannkuchen(%d) = %d\n", checksum, n, max_flips);
                return 0;
            }
            int first = perm1[0];
            for (int i = 0; i < r; i++) perm1[i] = perm1[i + 1];
            perm1[r] = first;
            if (--count[r] > 0) break;
            r++;
        }
        permutation++;
    }
}
