package main

import "fmt"

func main() {
	count := 0
	for n := int64(2); n < 1000000; n++ {
		prime := true
		for d := int64(2); d*d <= n; d++ {
			if n%d == 0 {
				prime = false
				break
			}
		}
		if prime {
			count++
		}
	}
	fmt.Println(count)
}
