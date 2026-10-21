#include <stdio.h>
#include <stdint.h>
#include <time.h>

static double now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec / 1e6;
}

int main(void) {
    double t0 = now_ms();
    int count = 0;
    for (int64_t n = 2; n < 1000000; n++) {
        int prime = 1;
        for (int64_t d = 2; d * d <= n; d++) {
            if (n % d == 0) { prime = 0; break; }
        }
        if (prime) count++;
    }
    double ms = now_ms() - t0;
    fprintf(stderr, "TIME %.3f\n", ms);
    printf("%d\n", count);
    return 0;
}
