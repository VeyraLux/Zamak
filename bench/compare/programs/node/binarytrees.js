const make = (depth) => (depth === 0 ? { left: null, right: null } : { left: make(depth - 1), right: make(depth - 1) });
const check = (node) => (node.left === null ? 1 : 1 + check(node.left) + check(node.right));

const n = Number(process.argv[2]);
const maxDepth = Math.max(6, n), stretch = maxDepth + 1;
console.log(`stretch tree of depth ${stretch}\t check: ${check(make(stretch))}`);
const longLived = make(maxDepth);
for (let depth = 4; depth <= maxDepth; depth += 2) {
  const iterations = 1 << (maxDepth - depth + 4);
  let total = 0;
  for (let i = 0; i < iterations; i++) total += check(make(depth));
  console.log(`${iterations}\t trees of depth ${depth}\t check: ${total}`);
}
console.log(`long lived tree of depth ${maxDepth}\t check: ${check(longLived)}`);
