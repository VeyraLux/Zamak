public class Spectralnorm {
    static double a(int i, int j) { return 1.0 / ((long) (i + j) * (i + j + 1) / 2 + i + 1); }

    static void times(double[] u, double[] out) {
        for (int i = 0; i < u.length; i++) {
            double s = 0;
            for (int j = 0; j < u.length; j++) s += a(i, j) * u[j];
            out[i] = s;
        }
    }

    static void timesTransposed(double[] u, double[] out) {
        for (int i = 0; i < u.length; i++) {
            double s = 0;
            for (int j = 0; j < u.length; j++) s += a(j, i) * u[j];
            out[i] = s;
        }
    }

    static void atTimes(double[] u, double[] out, double[] tmp) {
        times(u, tmp);
        timesTransposed(tmp, out);
    }

    public static void main(String[] args) {
        int n = Integer.parseInt(args[0]);
        double[] u = new double[n], v = new double[n], tmp = new double[n];
        java.util.Arrays.fill(u, 1);
        for (int i = 0; i < 10; i++) {
            atTimes(u, v, tmp);
            atTimes(v, u, tmp);
        }
        double vbv = 0, vv = 0;
        for (int i = 0; i < n; i++) { vbv += u[i] * v[i]; vv += v[i] * v[i]; }
        System.out.printf("%.9f%n", Math.sqrt(vbv / vv));
    }
}
