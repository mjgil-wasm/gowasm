package seed

var ready bool

func init() {
    ready = true
}

func First() string {
    return "alpha"
}

func Ready() bool {
    return ready
}
