package main

import (
	"fmt"
	"os"
	"strconv"
)

type Node struct{ left, right *Node }

func make_(depth int) *Node {
	if depth == 0 {
		return &Node{}
	}
	return &Node{make_(depth - 1), make_(depth - 1)}
}

func check(node *Node) int {
	if node.left == nil {
		return 1
	}
	return 1 + check(node.left) + check(node.right)
}

func main() {
	n, _ := strconv.Atoi(os.Args[1])
	maxDepth := n
	if maxDepth < 6 {
		maxDepth = 6
	}
	stretch := maxDepth + 1
	fmt.Printf("stretch tree of depth %d\t check: %d\n", stretch, check(make_(stretch)))
	longLived := make_(maxDepth)
	for depth := 4; depth <= maxDepth; depth += 2 {
		iterations := 1 << (maxDepth - depth + 4)
		total := 0
		for i := 0; i < iterations; i++ {
			total += check(make_(depth))
		}
		fmt.Printf("%d\t trees of depth %d\t check: %d\n", iterations, depth, total)
	}
	fmt.Printf("long lived tree of depth %d\t check: %d\n", maxDepth, check(longLived))
}
