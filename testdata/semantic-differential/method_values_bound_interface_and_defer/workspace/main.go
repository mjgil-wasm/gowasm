package main

import "fmt"

type Greeter struct {
	Name string
}

func (g Greeter) Speak(prefix string) string {
	return prefix + g.Name
}

type Named interface {
	Name() string
}

type Person struct {
	Label string
}

func (p Person) Name() string {
	return p.Label
}

type Logger struct {
	Ch chan string
}

func (l Logger) Send(label string) {
	l.Ch <- label
}

func (l Logger) Done() {
	fmt.Println("done")
}

func main() {
	greeter := Greeter{Name: "ada"}
	speak := greeter.Speak
	fmt.Println(speak("hi:"))

	var named Named = Person{Label: "turing"}
	getName := named.Name
	fmt.Println(getName())

	logger := Logger{Ch: make(chan string, 1)}
	send := logger.Send
	done := logger.Done
	defer done()
	go send("go")
	fmt.Println(<-logger.Ch)
}
