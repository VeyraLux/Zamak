import java.nio.charset.StandardCharsets;
import java.util.HashMap;

public class Wordfreq {
    public static void main(String[] args) throws Exception {
        String text = new String(System.in.readAllBytes(), StandardCharsets.UTF_8);
        HashMap<String, Integer> counts = new HashMap<>();
        for (String word : text.split("\\s+")) {
            if (!word.isEmpty()) counts.merge(word, 1, Integer::sum);
        }
        int best = 0;
        for (int n : counts.values()) if (n > best) best = n;
        System.out.println(counts.size());
        System.out.println(best);
    }
}
