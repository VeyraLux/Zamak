public class Primes {
    public static void main(String[] args) {
        int count = 0;
        for (long n = 2; n < 1000000; n++) {
            boolean prime = true;
            for (long d = 2; d * d <= n; d++) {
                if (n % d == 0) { prime = false; break; }
            }
            if (prime) count++;
        }
        System.out.println(count);
    }
}
