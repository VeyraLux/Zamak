package main

import (
	"fmt"
	"io"
	"os"
	"strings"
)

func main() {
	data, _ := io.ReadAll(os.Stdin)
	counts := map[string]int{}
	for _, word := range strings.Fields(string(data)) {
		counts[word]++
	}
	best := 0
	for _, n := range counts {
		if n > best {
			best = n
		}
	}
	fmt.Println(len(counts))
	fmt.Println(best)
}
