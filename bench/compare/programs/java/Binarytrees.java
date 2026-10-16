public class Binarytrees {
    static final class Node {
        final Node left, right;
        Node(Node left, Node right) { this.left = left; this.right = right; }
    }

    static Node make(int depth) {
        return depth == 0 ? new Node(null, null) : new Node(make(depth - 1), make(depth - 1));
    }

    static int check(Node node) {
        return node.left == null ? 1 : 1 + check(node.left) + check(node.right);
    }

    public static void main(String[] args) {
        int n = Integer.parseInt(args[0]);
        int maxDepth = Math.max(6, n), stretch = maxDepth + 1;
        System.out.println("stretch tree of depth " + stretch + "\t check: " + check(make(stretch)));
        Node longLived = make(maxDepth);
        for (int depth = 4; depth <= maxDepth; depth += 2) {
            int iterations = 1 << (maxDepth - depth + 4), total = 0;
            for (int i = 0; i < iterations; i++) total += check(make(depth));
            System.out.println(iterations + "\t trees of depth " + depth + "\t check: " + total);
        }
        System.out.println("long lived tree of depth " + maxDepth + "\t check: " + check(longLived));
    }
}
