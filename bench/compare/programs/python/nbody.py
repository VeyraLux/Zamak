import math
import sys

PI = 3.141592653589793
SOLAR_MASS = 4 * PI * PI
DAYS_PER_YEAR = 365.24


def planets():
    bodies = [
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, SOLAR_MASS],
        [4.84143144246472090e+00, -1.16032004402742839e+00, -1.03622044471123109e-01,
         1.66007664274403694e-03 * DAYS_PER_YEAR, 7.69901118419740425e-03 * DAYS_PER_YEAR,
         -6.90460016972063023e-05 * DAYS_PER_YEAR, 9.54791938424326609e-04 * SOLAR_MASS],
        [8.34336671824457987e+00, 4.12479856412430479e+00, -4.03523417114321381e-01,
         -2.76742510726862411e-03 * DAYS_PER_YEAR, 4.99852801234917238e-03 * DAYS_PER_YEAR,
         2.30417297573763929e-05 * DAYS_PER_YEAR, 2.85885980666130812e-04 * SOLAR_MASS],
        [1.28943695621391310e+01, -1.51111514016986312e+01, -2.23307578892655734e-01,
         2.96460137564761618e-03 * DAYS_PER_YEAR, 2.37847173959480950e-03 * DAYS_PER_YEAR,
         -2.96589568540237556e-05 * DAYS_PER_YEAR, 4.36624404335156298e-05 * SOLAR_MASS],
        [1.53796971148509165e+01, -2.59193146099879641e+01, 1.79258772950371181e-01,
         2.68067772490389322e-03 * DAYS_PER_YEAR, 1.62824170038242295e-03 * DAYS_PER_YEAR,
         -9.51592254519715870e-05 * DAYS_PER_YEAR, 5.15138902046611451e-05 * SOLAR_MASS],
    ]
    px = sum(b[3] * b[6] for b in bodies)
    py = sum(b[4] * b[6] for b in bodies)
    pz = sum(b[5] * b[6] for b in bodies)
    bodies[0][3] = -px / SOLAR_MASS
    bodies[0][4] = -py / SOLAR_MASS
    bodies[0][5] = -pz / SOLAR_MASS
    return bodies


def energy(bodies):
    e = 0.0
    for i, (x, y, z, vx, vy, vz, m) in enumerate(bodies):
        e += 0.5 * m * (vx * vx + vy * vy + vz * vz)
        for x2, y2, z2, _, _, _, m2 in bodies[i + 1:]:
            dx, dy, dz = x - x2, y - y2, z - z2
            e -= m * m2 / math.sqrt(dx * dx + dy * dy + dz * dz)
    return e


def advance(bodies, dt):
    n = len(bodies)
    for i in range(n):
        bi = bodies[i]
        for j in range(i + 1, n):
            bj = bodies[j]
            dx, dy, dz = bi[0] - bj[0], bi[1] - bj[1], bi[2] - bj[2]
            d2 = dx * dx + dy * dy + dz * dz
            mag = dt / (d2 * math.sqrt(d2))
            mj, mi = bj[6] * mag, bi[6] * mag
            bi[3] -= dx * mj
            bi[4] -= dy * mj
            bi[5] -= dz * mj
            bj[3] += dx * mi
            bj[4] += dy * mi
            bj[5] += dz * mi
    for b in bodies:
        b[0] += dt * b[3]
        b[1] += dt * b[4]
        b[2] += dt * b[5]


bodies = planets()
print(f"{energy(bodies):.9f}")
for _ in range(int(sys.argv[1])):
    advance(bodies, 0.01)
print(f"{energy(bodies):.9f}")
