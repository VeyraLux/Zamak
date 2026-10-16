struct Node {
    children: Option<(Box<Node>, Box<Node>)>,
}

fn make(depth: u32) -> Box<Node> {
    if depth == 0 {
        Box::new(Node { children: None })
    } else {
        Box::new(Node { children: Some((make(depth - 1), make(depth - 1))) })
    }
}

fn check(node: &Node) -> i64 {
    1 + node.children.as_ref().map_or(0, |(l, r)| check(l) + check(r))
}

fn main() {
    let n: u32 = std::env::args().nth(1).unwrap().parse().unwrap();
    let max_depth = n.max(6);
    let stretch = max_depth + 1;
    println!("stretch tree of depth {stretch}\t check: {}", check(&make(stretch)));
    let long_lived = make(max_depth);
    for depth in (4..=max_depth).step_by(2) {
        let iterations = 1i64 << (max_depth - depth + 4);
        let total: i64 = (0..iterations).map(|_| check(&make(depth))).sum();
        println!("{iterations}\t trees of depth {depth}\t check: {total}");
    }
    println!("long lived tree of depth {max_depth}\t check: {}", check(&long_lived));
}
