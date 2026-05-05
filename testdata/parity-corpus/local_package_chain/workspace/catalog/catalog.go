package catalog

import "example.com/app/seed"

func Title() string {
    return "local-imports"
}

func Items() []string {
    return []string{seed.First(), "beta"}
}

func Ready() bool {
    return seed.Ready()
}
