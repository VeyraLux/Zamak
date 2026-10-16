import sys

counts = {}
for word in sys.stdin.read().split():
    counts[word] = counts.get(word, 0) + 1
print(len(counts))
print(max(counts.values()))
