#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suit {
    Club,
    Diamond,
    Heart,
    Spade,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rank {
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
    Two,
}

impl From<&Rank> for i32 {
    fn from(rank: &Rank) -> Self {
        match rank {
            Rank::Three => 0,
            Rank::Four => 1,
            Rank::Five => 2,
            Rank::Six => 3,
            Rank::Seven => 4,
            Rank::Eight => 5,
            Rank::Nine => 6,
            Rank::Ten => 7,
            Rank::Jack => 8,
            Rank::Queen => 9,
            Rank::King => 10,
            Rank::Ace => 11,
            Rank::Two => 12,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Card {
    Normal(Suit, Rank),
    Joker,
}

impl From<&Card> for String {
    fn from(card: &Card) -> Self {
        match card {
            Card::Normal(suit, rank) => {
                let s = match suit {
                    Suit::Spade => "♠️",
                    Suit::Club => "♣️",
                    Suit::Diamond => "♦︎",
                    Suit::Heart => "♥",
                };
                let r = match rank {
                    Rank::Three => "3",
                    Rank::Four => "4",
                    Rank::Five => "5",
                    Rank::Six => "6",
                    Rank::Seven => "7",
                    Rank::Eight => "8",
                    Rank::Nine => "9",
                    Rank::Ten => "10",
                    Rank::Jack => "J",
                    Rank::Queen => "Q",
                    Rank::King => "K",
                    Rank::Ace => "A",
                    Rank::Two => "2",
                };
                format!("{s}{r}")
            }
            Card::Joker => "Joker".to_owned(),
        }
    }
}

pub fn create_deck() -> Vec<Card> {
    let mut deck = Vec::<Card>::new();
    for suit in [Suit::Spade, Suit::Club, Suit::Diamond, Suit::Heart] {
        for rank in [
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
            Rank::Ace,
            Rank::Two,
        ] {
            deck.push(Card::Normal(suit, rank));
        }
    }
    deck.push(Card::Joker);
    deck
}

pub fn cmp_order(c1: &Card, c2: &Card) -> std::cmp::Ordering {
    match (c1, c2) {
        (Card::Normal(s1, r1), Card::Normal(s2, r2)) => r1.cmp(r2).then(s1.cmp(s2)),
        (_, _) => c1.cmp(c2),
    }
}

pub fn cmp_order_reversely(c1: &Card, c2: &Card) -> std::cmp::Ordering {
    match (c1, c2) {
        (Card::Normal(s1, r1), Card::Normal(s2, r2)) => r2.cmp(r1).then(s1.cmp(s2)),
        (_, _) => c1.cmp(c2),
    }
}

pub fn cmp_rank(c1: &Card, c2: &Card) -> std::cmp::Ordering {
    match (c1, c2) {
        (Card::Normal(_, r1), Card::Normal(_, r2)) => r1.cmp(r2),
        (_, _) => c1.cmp(c2),
    }
}

pub fn cmp_rank_reversely(c1: &Card, c2: &Card) -> std::cmp::Ordering {
    match (c1, c2) {
        (Card::Normal(_, r1), Card::Normal(_, r2)) => r2.cmp(r1),
        (_, _) => c1.cmp(c2),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cmp_order() {
        for (c1, c2, expected) in [
            (
                Card::Normal(Suit::Spade, Rank::Three),
                Card::Normal(Suit::Diamond, Rank::Five),
                std::cmp::Ordering::Less,
            ),
            (
                Card::Normal(Suit::Club, Rank::Ten),
                Card::Normal(Suit::Spade, Rank::Ten),
                std::cmp::Ordering::Less,
            ),
            (
                Card::Normal(Suit::Diamond, Rank::Ace),
                Card::Normal(Suit::Diamond, Rank::Ace),
                std::cmp::Ordering::Equal,
            ),
            (
                Card::Normal(Suit::Heart, Rank::Seven),
                Card::Normal(Suit::Club, Rank::Four),
                std::cmp::Ordering::Greater,
            ),
            (
                Card::Normal(Suit::Spade, Rank::Ten),
                Card::Joker,
                std::cmp::Ordering::Less,
            ),
            (Card::Joker, Card::Joker, std::cmp::Ordering::Equal),
            (
                Card::Joker,
                Card::Normal(Suit::Club, Rank::Four),
                std::cmp::Ordering::Greater,
            ),
        ] {
            assert_eq!(cmp_order(&c1, &c2), expected);
        }
        let mut cards = vec![
            Card::Normal(Suit::Heart, Rank::Jack),
            Card::Normal(Suit::Spade, Rank::Four),
            Card::Joker,
            Card::Normal(Suit::Diamond, Rank::Jack),
            Card::Normal(Suit::Club, Rank::Four),
            Card::Normal(Suit::Spade, Rank::Seven),
            Card::Normal(Suit::Club, Rank::Jack),
        ];
        cards.sort_by(cmp_order);
        let expected = vec![
            Card::Normal(Suit::Club, Rank::Four),
            Card::Normal(Suit::Spade, Rank::Four),
            Card::Normal(Suit::Spade, Rank::Seven),
            Card::Normal(Suit::Club, Rank::Jack),
            Card::Normal(Suit::Diamond, Rank::Jack),
            Card::Normal(Suit::Heart, Rank::Jack),
            Card::Joker,
        ];
        assert_eq!(cards, expected);
    }

    #[test]
    fn test_cmp_order_reversely() {
        for (c1, c2, expected) in [
            (
                Card::Normal(Suit::Spade, Rank::Three),
                Card::Normal(Suit::Diamond, Rank::Five),
                std::cmp::Ordering::Greater,
            ),
            (
                Card::Normal(Suit::Club, Rank::Ten),
                Card::Normal(Suit::Spade, Rank::Ten),
                std::cmp::Ordering::Less,
            ),
            (
                Card::Normal(Suit::Diamond, Rank::Ace),
                Card::Normal(Suit::Diamond, Rank::Ace),
                std::cmp::Ordering::Equal,
            ),
            (
                Card::Normal(Suit::Heart, Rank::Seven),
                Card::Normal(Suit::Club, Rank::Four),
                std::cmp::Ordering::Less,
            ),
            (
                Card::Normal(Suit::Spade, Rank::Ten),
                Card::Joker,
                std::cmp::Ordering::Less,
            ),
            (Card::Joker, Card::Joker, std::cmp::Ordering::Equal),
            (
                Card::Joker,
                Card::Normal(Suit::Club, Rank::Four),
                std::cmp::Ordering::Greater,
            ),
        ] {
            assert_eq!(cmp_order_reversely(&c1, &c2), expected);
        }
        let mut cards = vec![
            Card::Normal(Suit::Heart, Rank::Jack),
            Card::Normal(Suit::Spade, Rank::Four),
            Card::Joker,
            Card::Normal(Suit::Diamond, Rank::Jack),
            Card::Normal(Suit::Club, Rank::Four),
            Card::Normal(Suit::Spade, Rank::Seven),
            Card::Normal(Suit::Club, Rank::Jack),
        ];
        cards.sort_by(cmp_order_reversely);
        let expected = vec![
            Card::Normal(Suit::Club, Rank::Jack),
            Card::Normal(Suit::Diamond, Rank::Jack),
            Card::Normal(Suit::Heart, Rank::Jack),
            Card::Normal(Suit::Spade, Rank::Seven),
            Card::Normal(Suit::Club, Rank::Four),
            Card::Normal(Suit::Spade, Rank::Four),
            Card::Joker,
        ];
        assert_eq!(cards, expected);
    }

    #[test]
    fn test_cmp_rank() {
        for (c1, c2, expected) in [
            (
                Card::Normal(Suit::Spade, Rank::Three),
                Card::Normal(Suit::Diamond, Rank::Five),
                std::cmp::Ordering::Less,
            ),
            (
                Card::Normal(Suit::Club, Rank::Ten),
                Card::Normal(Suit::Spade, Rank::Ten),
                std::cmp::Ordering::Equal,
            ),
            (
                Card::Normal(Suit::Diamond, Rank::Ace),
                Card::Normal(Suit::Diamond, Rank::Ace),
                std::cmp::Ordering::Equal,
            ),
            (
                Card::Normal(Suit::Heart, Rank::Seven),
                Card::Normal(Suit::Club, Rank::Four),
                std::cmp::Ordering::Greater,
            ),
            (
                Card::Normal(Suit::Spade, Rank::Ten),
                Card::Joker,
                std::cmp::Ordering::Less,
            ),
            (Card::Joker, Card::Joker, std::cmp::Ordering::Equal),
            (
                Card::Joker,
                Card::Normal(Suit::Club, Rank::Four),
                std::cmp::Ordering::Greater,
            ),
        ] {
            assert_eq!(cmp_rank(&c1, &c2), expected);
        }
    }

    #[test]
    fn test_cmp_rank_reversely() {
        for (c1, c2, expected) in [
            (
                Card::Normal(Suit::Spade, Rank::Three),
                Card::Normal(Suit::Diamond, Rank::Five),
                std::cmp::Ordering::Greater,
            ),
            (
                Card::Normal(Suit::Club, Rank::Ten),
                Card::Normal(Suit::Spade, Rank::Ten),
                std::cmp::Ordering::Equal,
            ),
            (
                Card::Normal(Suit::Diamond, Rank::Ace),
                Card::Normal(Suit::Diamond, Rank::Ace),
                std::cmp::Ordering::Equal,
            ),
            (
                Card::Normal(Suit::Heart, Rank::Seven),
                Card::Normal(Suit::Club, Rank::Four),
                std::cmp::Ordering::Less,
            ),
            (
                Card::Normal(Suit::Spade, Rank::Ten),
                Card::Joker,
                std::cmp::Ordering::Less,
            ),
            (Card::Joker, Card::Joker, std::cmp::Ordering::Equal),
            (
                Card::Joker,
                Card::Normal(Suit::Club, Rank::Four),
                std::cmp::Ordering::Greater,
            ),
        ] {
            assert_eq!(cmp_rank_reversely(&c1, &c2), expected);
        }
    }
}
