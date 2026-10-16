const text = require("fs").readFileSync(0, "utf8");
const counts = new Map();
for (const word of text.split(/\s+/)) {
  if (word) counts.set(word, (counts.get(word) || 0) + 1);
}
let best = 0;
for (const n of counts.values()) if (n > best) best = n;
console.log(counts.size);
console.log(best);
