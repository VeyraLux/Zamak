let count = 0;
for (let n = 2; n < 1000000; n++) {
  let prime = true;
  for (let d = 2; d * d <= n; d++) {
    if (n % d === 0) { prime = false; break; }
  }
  if (prime) count++;
}
console.log(count);
