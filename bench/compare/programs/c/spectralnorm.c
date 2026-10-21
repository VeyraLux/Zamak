#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

static double now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec / 1e6;
}

static double a(long i, long j) { return 1.0 / ((i + j) * (i + j + 1) / 2 + i + 1); }

static void times(const double *u, double *out, long n) {
    for (long i = 0; i < n; i++) {
        double s = 0;
        for (long j = 0; j < n; j++) s += a(i, j) * u[j];
        out[i] = s;
    }
}

static void times_transposed(const double *u, double *out, long n) {
    for (long i = 0; i < n; i++) {
        double s = 0;
        for (long j = 0; j < n; j++) s += a(j, i) * u[j];
        out[i] = s;
    }
}

static void at_times(const double *u, double *out, double *tmp, long n) {
    times(u, tmp, n);
    times_transposed(tmp, out, n);
}

int main(int argc, char **argv) {
    long n = atol(argv[1]);
    double *u = malloc(n * sizeof(double)), *v = malloc(n * sizeof(double)), *tmp = malloc(n * sizeof(double));
    for (long i = 0; i < n; i++) u[i] = 1;
    double t0 = now_ms();
    for (int i = 0; i < 10; i++) {
        at_times(u, v, tmp, n);
        at_times(v, u, tmp, n);
    }
    double vbv = 0, vv = 0;
    for (long i = 0; i < n; i++) { vbv += u[i] * v[i]; vv += v[i] * v[i]; }
    double ms = now_ms() - t0;
    fprintf(stderr, "TIME %.3f\n", ms);
    printf("%.9f\n", sqrt(vbv / vv));
    return 0;
}
