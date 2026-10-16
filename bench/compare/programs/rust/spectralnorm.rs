fn a(i: usize, j: usize) -> f64 {
    1.0 / ((i + j) * (i + j + 1) / 2 + i + 1) as f64
}

fn times(u: &[f64], out: &mut [f64]) {
    for i in 0..u.len() {
        out[i] = (0..u.len()).map(|j| a(i, j) * u[j]).sum();
    }
}

fn times_transposed(u: &[f64], out: &mut [f64]) {
    for i in 0..u.len() {
        out[i] = (0..u.len()).map(|j| a(j, i) * u[j]).sum();
    }
}

fn at_times(u: &[f64], out: &mut [f64], tmp: &mut [f64]) {
    times(u, tmp);
    times_transposed(tmp, out);
}

fn main() {
    let n: usize = std::env::args().nth(1).unwrap().parse().unwrap();
    let (mut u, mut v, mut tmp) = (vec![1.0; n], vec![0.0; n], vec![0.0; n]);
    for _ in 0..10 {
        at_times(&u, &mut v, &mut tmp);
        at_times(&v, &mut u, &mut tmp);
    }
    let vbv: f64 = u.iter().zip(&v).map(|(a, b)| a * b).sum();
    let vv: f64 = v.iter().map(|b| b * b).sum();
    println!("{:.9}", (vbv / vv).sqrt());
}
