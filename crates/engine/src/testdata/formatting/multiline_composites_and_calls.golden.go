package main

func main() {
	value := map[string][]int{
		"numbers": []int{
			1,
			2,
		},
	}
	println(
		value["numbers"][0],
	)
}
