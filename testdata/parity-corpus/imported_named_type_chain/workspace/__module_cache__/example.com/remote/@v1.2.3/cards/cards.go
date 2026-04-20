package cards

type Card struct {
    Label string
}

func (card Card) Message() string {
    return "remote:" + card.Label
}
