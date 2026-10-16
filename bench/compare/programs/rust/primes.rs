fn main() {
    let mut count = 0;
    for n in 2i64..1_000_000 {
        let mut prime = true;
        let mut d = 2i64;
        while d * d <= n {
            if n % d == 0 {
                prime = false;
                break;
            }
            d += 1;
        }
        if prime {
            count += 1;
        }
    }
    println!("{count}");
}
