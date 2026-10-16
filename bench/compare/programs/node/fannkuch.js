const n = Number(process.argv[2]);
const perm1 = Array.from({ length: n }, (_, i) => i), perm = new Array(n).fill(0), count = new Array(n).fill(0);
let maxFlips = 0, checksum = 0, permutation = 0, r = n;
for (;;) {
  while (r !== 1) { count[r - 1] = r; r--; }
  for (let i = 0; i < n; i++) perm[i] = perm1[i];
  let flips = 0;
  for (let k = perm[0]; k !== 0; k = perm[0]) {
    for (let lo = 0, hi = k; lo < hi; lo++, hi--) { const t = perm[lo]; perm[lo] = perm[hi]; perm[hi] = t; }
    flips++;
  }
  maxFlips = Math.max(maxFlips, flips);
  checksum += permutation % 2 === 0 ? flips : -flips;
  let done = false;
  for (;;) {
    if (r === n) { done = true; break; }
    const first = perm1[0];
    for (let i = 0; i < r; i++) perm1[i] = perm1[i + 1];
    perm1[r] = first;
    if (--count[r] > 0) break;
    r++;
  }
  if (done) break;
  permutation++;
}
console.log(checksum);
console.log(`Pfannkuchen(${n}) = ${maxFlips}`);
