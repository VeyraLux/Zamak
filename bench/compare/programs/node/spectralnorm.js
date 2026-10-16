const a = (i, j) => 1 / (((i + j) * (i + j + 1)) / 2 + i + 1);

function times(u, out) {
  for (let i = 0; i < u.length; i++) {
    let s = 0;
    for (let j = 0; j < u.length; j++) s += a(i, j) * u[j];
    out[i] = s;
  }
}

function timesTransposed(u, out) {
  for (let i = 0; i < u.length; i++) {
    let s = 0;
    for (let j = 0; j < u.length; j++) s += a(j, i) * u[j];
    out[i] = s;
  }
}

function atTimes(u, out, tmp) {
  times(u, tmp);
  timesTransposed(tmp, out);
}

const n = Number(process.argv[2]);
const u = new Float64Array(n).fill(1), v = new Float64Array(n), tmp = new Float64Array(n);
for (let i = 0; i < 10; i++) {
  atTimes(u, v, tmp);
  atTimes(v, u, tmp);
}
let vbv = 0, vv = 0;
for (let i = 0; i < n; i++) { vbv += u[i] * v[i]; vv += v[i] * v[i]; }
console.log(Math.sqrt(vbv / vv).toFixed(9));
