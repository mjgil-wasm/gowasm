package report

import "example.com/remote/cards"

func First() cards.Box[string] {
    return cards.Box[string]{Label: "Ada"}
}
