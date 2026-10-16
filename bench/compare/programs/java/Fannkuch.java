public class Fannkuch {
    public static void main(String[] args) {
        int n = Integer.parseInt(args[0]);
        int[] perm1 = new int[n], perm = new int[n], count = new int[n];
        for (int i = 0; i < n; i++) perm1[i] = i;
        int maxFlips = 0, checksum = 0, permutation = 0, r = n;
        while (true) {
            while (r != 1) { count[r - 1] = r; r--; }
            System.arraycopy(perm1, 0, perm, 0, n);
            int flips = 0;
            for (int k = perm[0]; k != 0; k = perm[0]) {
                for (int lo = 0, hi = k; lo < hi; lo++, hi--) { int t = perm[lo]; perm[lo] = perm[hi]; perm[hi] = t; }
                flips++;
            }
            maxFlips = Math.max(maxFlips, flips);
            checksum += permutation % 2 == 0 ? flips : -flips;
            while (true) {
                if (r == n) {
                    System.out.println(checksum);
                    System.out.println("Pfannkuchen(" + n + ") = " + maxFlips);
                    return;
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
}
