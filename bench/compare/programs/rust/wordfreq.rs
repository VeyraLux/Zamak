use std::collections::HashMap;
use std::io::Read;

fn main() {
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text).unwrap();
    let mut counts: HashMap<&str, i64> = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;
    }
    let best = counts.values().copied().max().unwrap_or(0);
    println!("{}", counts.len());
    println!("{best}");
}
