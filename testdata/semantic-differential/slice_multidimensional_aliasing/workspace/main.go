package main

import "fmt"

func main() {
    matrix := [][]int{
        []int{1, 2},
        []int{3, 4},
    }

    row := matrix[0][:]
    row[1] = 9

    matrix[1] = append(matrix[1][:1], 8)

    fmt.Println(matrix[0], row)
    fmt.Println(matrix[1], len(matrix[1]), cap(matrix[1]))
}
