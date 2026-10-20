public class Nbody {
    static final double PI = 3.141592653589793, SOLAR_MASS = 4 * PI * PI, DAYS_PER_YEAR = 365.24;

    static final class Body {
        double x, y, z, vx, vy, vz, mass;
        Body(double x, double y, double z, double vx, double vy, double vz, double mass) {
            this.x = x; this.y = y; this.z = z; this.vx = vx; this.vy = vy; this.vz = vz; this.mass = mass;
        }
    }

    static Body[] planets() {
        Body[] bodies = {
            new Body(0, 0, 0, 0, 0, 0, SOLAR_MASS),
            new Body(4.84143144246472090e+00, -1.16032004402742839e+00, -1.03622044471123109e-01,
                1.66007664274403694e-03 * DAYS_PER_YEAR, 7.69901118419740425e-03 * DAYS_PER_YEAR,
                -6.90460016972063023e-05 * DAYS_PER_YEAR, 9.54791938424326609e-04 * SOLAR_MASS),
            new Body(8.34336671824457987e+00, 4.12479856412430479e+00, -4.03523417114321381e-01,
                -2.76742510726862411e-03 * DAYS_PER_YEAR, 4.99852801234917238e-03 * DAYS_PER_YEAR,
                2.30417297573763929e-05 * DAYS_PER_YEAR, 2.85885980666130812e-04 * SOLAR_MASS),
            new Body(1.28943695621391310e+01, -1.51111514016986312e+01, -2.23307578892655734e-01,
                2.96460137564761618e-03 * DAYS_PER_YEAR, 2.37847173959480950e-03 * DAYS_PER_YEAR,
                -2.96589568540237556e-05 * DAYS_PER_YEAR, 4.36624404335156298e-05 * SOLAR_MASS),
            new Body(1.53796971148509165e+01, -2.59193146099879641e+01, 1.79258772950371181e-01,
                2.68067772490389322e-03 * DAYS_PER_YEAR, 1.62824170038242295e-03 * DAYS_PER_YEAR,
                -9.51592254519715870e-05 * DAYS_PER_YEAR, 5.15138902046611451e-05 * SOLAR_MASS),
        };
        double px = 0, py = 0, pz = 0;
        for (Body b : bodies) { px += b.vx * b.mass; py += b.vy * b.mass; pz += b.vz * b.mass; }
        bodies[0].vx = -px / SOLAR_MASS; bodies[0].vy = -py / SOLAR_MASS; bodies[0].vz = -pz / SOLAR_MASS;
        return bodies;
    }

    static double energy(Body[] bodies) {
        double e = 0;
        for (int i = 0; i < bodies.length; i++) {
            Body b = bodies[i];
            e += 0.5 * b.mass * (b.vx * b.vx + b.vy * b.vy + b.vz * b.vz);
            for (int j = i + 1; j < bodies.length; j++) {
                Body c = bodies[j];
                double dx = b.x - c.x, dy = b.y - c.y, dz = b.z - c.z;
                e -= b.mass * c.mass / Math.sqrt(dx * dx + dy * dy + dz * dz);
            }
        }
        return e;
    }

    static void advance(Body[] bodies, double dt) {
        for (int i = 0; i < bodies.length; i++) {
            Body b = bodies[i];
            for (int j = i + 1; j < bodies.length; j++) {
                Body c = bodies[j];
                double dx = b.x - c.x, dy = b.y - c.y, dz = b.z - c.z;
                double d2 = dx * dx + dy * dy + dz * dz;
                double mag = dt / (d2 * Math.sqrt(d2));
                double mj = c.mass * mag, mi = b.mass * mag;
                b.vx -= dx * mj; b.vy -= dy * mj; b.vz -= dz * mj;
                c.vx += dx * mi; c.vy += dy * mi; c.vz += dz * mi;
            }
        }
        for (Body b : bodies) { b.x += dt * b.vx; b.y += dt * b.vy; b.z += dt * b.vz; }
    }

    public static void main(String[] args) {
        int steps = Integer.parseInt(args[0]);
        Body[] bodies = planets();
        System.out.printf("%.9f%n", energy(bodies));
        // Time the compute loop only, and write to stderr so the output stays comparable.
        long t0 = System.nanoTime();
        for (int i = 0; i < steps; i++) advance(bodies, 0.01);
        double ms = (System.nanoTime() - t0) / 1e6;
        double e2 = energy(bodies);
        System.err.printf("TIME %.3f%n", ms);
        System.out.printf("%.9f%n", e2);
    }
}
