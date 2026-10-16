package main

import (
	"fmt"
	"math"
	"os"
	"strconv"
)

func a(i, j int) float64 { return 1.0 / float64((i+j)*(i+j+1)/2+i+1) }

func times(u, out []float64) {
	for i := range u {
		s := 0.0
		for j := range u {
			s += a(i, j) * u[j]
		}
		out[i] = s
	}
}

func timesTransposed(u, out []float64) {
	for i := range u {
		s := 0.0
		for j := range u {
			s += a(j, i) * u[j]
		}
		out[i] = s
	}
}

func atTimes(u, out, tmp []float64) {
	times(u, tmp)
	timesTransposed(tmp, out)
}

func main() {
	n, _ := strconv.Atoi(os.Args[1])
	u, v, tmp := make([]float64, n), make([]float64, n), make([]float64, n)
	for i := range u {
		u[i] = 1
	}
	for i := 0; i < 10; i++ {
		atTimes(u, v, tmp)
		atTimes(v, u, tmp)
	}
	var vbv, vv float64
	for i := range u {
		vbv += u[i] * v[i]
		vv += v[i] * v[i]
	}
	fmt.Printf("%.9f\n", math.Sqrt(vbv/vv))
}
