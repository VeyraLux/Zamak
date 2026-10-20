#include <math.h>
#include <time.h>
#include <stdio.h>
#include <stdlib.h>

#define PI 3.141592653589793
#define SOLAR_MASS (4 * PI * PI)
#define DAYS_PER_YEAR 365.24

typedef struct { double x, y, z, vx, vy, vz, mass; } Body;

static Body bodies[5] = {
    {0, 0, 0, 0, 0, 0, SOLAR_MASS},
    {4.84143144246472090e+00, -1.16032004402742839e+00, -1.03622044471123109e-01,
     1.66007664274403694e-03 * DAYS_PER_YEAR, 7.69901118419740425e-03 * DAYS_PER_YEAR,
     -6.90460016972063023e-05 * DAYS_PER_YEAR, 9.54791938424326609e-04 * SOLAR_MASS},
    {8.34336671824457987e+00, 4.12479856412430479e+00, -4.03523417114321381e-01,
     -2.76742510726862411e-03 * DAYS_PER_YEAR, 4.99852801234917238e-03 * DAYS_PER_YEAR,
     2.30417297573763929e-05 * DAYS_PER_YEAR, 2.85885980666130812e-04 * SOLAR_MASS},
    {1.28943695621391310e+01, -1.51111514016986312e+01, -2.23307578892655734e-01,
     2.96460137564761618e-03 * DAYS_PER_YEAR, 2.37847173959480950e-03 * DAYS_PER_YEAR,
     -2.96589568540237556e-05 * DAYS_PER_YEAR, 4.36624404335156298e-05 * SOLAR_MASS},
    {1.53796971148509165e+01, -2.59193146099879641e+01, 1.79258772950371181e-01,
     2.68067772490389322e-03 * DAYS_PER_YEAR, 1.62824170038242295e-03 * DAYS_PER_YEAR,
     -9.51592254519715870e-05 * DAYS_PER_YEAR, 5.15138902046611451e-05 * SOLAR_MASS},
};

static double energy(void) {
    double e = 0;
    for (int i = 0; i < 5; i++) {
        Body *b = &bodies[i];
        e += 0.5 * b->mass * (b->vx * b->vx + b->vy * b->vy + b->vz * b->vz);
        for (int j = i + 1; j < 5; j++) {
            Body *c = &bodies[j];
            double dx = b->x - c->x, dy = b->y - c->y, dz = b->z - c->z;
            e -= b->mass * c->mass / sqrt(dx * dx + dy * dy + dz * dz);
        }
    }
    return e;
}

static void advance(double dt) {
    for (int i = 0; i < 5; i++) {
        Body *b = &bodies[i];
        for (int j = i + 1; j < 5; j++) {
            Body *c = &bodies[j];
            double dx = b->x - c->x, dy = b->y - c->y, dz = b->z - c->z;
            double d2 = dx * dx + dy * dy + dz * dz;
            double mag = dt / (d2 * sqrt(d2));
            double mj = c->mass * mag, mi = b->mass * mag;
            b->vx -= dx * mj; b->vy -= dy * mj; b->vz -= dz * mj;
            c->vx += dx * mi; c->vy += dy * mi; c->vz += dz * mi;
        }
    }
    for (int i = 0; i < 5; i++) {
        Body *b = &bodies[i];
        b->x += dt * b->vx; b->y += dt * b->vy; b->z += dt * b->vz;
    }
}

int main(int argc, char **argv) {
    int steps = atoi(argv[1]);
    double px = 0, py = 0, pz = 0;
    for (int i = 0; i < 5; i++) {
        px += bodies[i].vx * bodies[i].mass;
        py += bodies[i].vy * bodies[i].mass;
        pz += bodies[i].vz * bodies[i].mass;
    }
    bodies[0].vx = -px / SOLAR_MASS;
    bodies[0].vy = -py / SOLAR_MASS;
    bodies[0].vz = -pz / SOLAR_MASS;
    printf("%.9f\n", energy());
    /* 只计计算本身：起点放在第一行输出之后，终点放在第二行输出之前，
       这样外部计时（含启动）与这里能对得上去。写到 stderr，不干扰输出比对。 */
    struct timespec t0, t1;
    clock_gettime(CLOCK_MONOTONIC, &t0);
    for (int i = 0; i < steps; i++) advance(0.01);
    clock_gettime(CLOCK_MONOTONIC, &t1);
    double e2 = energy();
    double ms = (t1.tv_sec - t0.tv_sec) * 1000.0 + (t1.tv_nsec - t0.tv_nsec) / 1e6;
    fprintf(stderr, "TIME %.3f\n", ms);
    printf("%.9f\n", e2);
    return 0;
}
