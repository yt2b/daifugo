use crate::{
    card::{Card, Suit},
    comb::Comb,
};

pub struct SuitBinder {
    suits: Option<Vec<Suit>>,
    prev_suits: Option<Vec<Suit>>,
}

impl SuitBinder {
    pub fn new() -> Self {
        SuitBinder {
            suits: None,
            prev_suits: None,
        }
    }

    pub fn is_activate(&self) -> bool {
        self.suits.is_some()
    }

    pub fn push(&mut self, comb: &Comb) -> bool {
        match comb {
            Comb::Single(Card::Normal(s, _)) => match &self.prev_suits {
                Some(suits) if s == &suits[0] => {
                    self.suits = self.prev_suits.take();
                }
                _ => {
                    self.prev_suits = Some(vec![*s]);
                }
            },
            Comb::Multi(cards) | Comb::Seq(cards) if !cards.contains(&Card::Joker) => {
                match &self.prev_suits {
                    Some(suits) if suits == &get_suits(cards) => {
                        self.suits = self.prev_suits.take();
                    }
                    _ => {
                        self.prev_suits = Some(get_suits(cards));
                    }
                }
            }
            _ => {
                self.prev_suits = None;
            }
        }
        self.is_activate()
    }

    pub fn clear(&mut self) {
        self.suits = None;
        self.prev_suits = None;
    }

    pub fn is_valid(&self, comb: &Comb) -> bool {
        match &self.suits {
            Some(suits) => match comb {
                Comb::Single(card) => {
                    (suits.len() == 1)
                        && match card {
                            Card::Normal(s, _) => s == &suits[0],
                            Card::Joker => true,
                        }
                }
                Comb::Multi(cards) => {
                    (cards.len() == suits.len())
                        && cards.iter().zip(suits).all(|(card, suit)| match card {
                            Card::Normal(s, _) => s == suit,
                            Card::Joker => true,
                        })
                }
                Comb::Seq(cards) => {
                    (cards.len() == suits.len())
                        && cards.iter().all(|card| match card {
                            Card::Normal(s, _) => s == &suits[0],
                            Card::Joker => true,
                        })
                }
            },
            None => true,
        }
    }
}

fn get_suits(cards: &[Card]) -> Vec<Suit> {
    cards
        .iter()
        .filter_map(|card| match card {
            Card::Normal(s, _) => Some(*s),
            Card::Joker => None,
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::card::{Rank, Suit};

    fn create_suit_binder(suits: Vec<Suit>) -> SuitBinder {
        let mut binder = SuitBinder::new();
        binder.suits = Some(suits);
        binder
    }

    #[test]
    fn test_push() {
        for (combs, expected_suits, expected_prev_suits) in [
            (
                vec![
                    Comb::Single(Card::Normal(Suit::Club, Rank::Four)),
                    Comb::Single(Card::Normal(Suit::Heart, Rank::Six)),
                ],
                None,
                Some(vec![Suit::Heart]),
            ),
            (
                vec![
                    Comb::Single(Card::Normal(Suit::Diamond, Rank::Four)),
                    Comb::Single(Card::Normal(Suit::Diamond, Rank::Six)),
                ],
                Some(vec![Suit::Diamond]),
                None,
            ),
            (
                vec![
                    Comb::Single(Card::Normal(Suit::Heart, Rank::Four)),
                    Comb::Single(Card::Joker),
                ],
                None,
                None,
            ),
            (
                vec![
                    Comb::Multi(vec![
                        Card::Normal(Suit::Heart, Rank::Four),
                        Card::Normal(Suit::Spade, Rank::Four),
                    ]),
                    Comb::Multi(vec![
                        Card::Normal(Suit::Club, Rank::Five),
                        Card::Normal(Suit::Diamond, Rank::Five),
                    ]),
                ],
                None,
                Some(vec![Suit::Club, Suit::Diamond]),
            ),
            (
                vec![
                    Comb::Multi(vec![
                        Card::Normal(Suit::Heart, Rank::Four),
                        Card::Normal(Suit::Spade, Rank::Four),
                    ]),
                    Comb::Multi(vec![
                        Card::Normal(Suit::Heart, Rank::Five),
                        Card::Normal(Suit::Spade, Rank::Five),
                    ]),
                ],
                Some(vec![Suit::Heart, Suit::Spade]),
                None,
            ),
            (
                vec![
                    Comb::Multi(vec![
                        Card::Normal(Suit::Heart, Rank::Four),
                        Card::Normal(Suit::Spade, Rank::Four),
                    ]),
                    Comb::Multi(vec![Card::Normal(Suit::Heart, Rank::Five), Card::Joker]),
                ],
                None,
                None,
            ),
            (
                vec![
                    Comb::Seq(vec![
                        Card::Normal(Suit::Spade, Rank::Four),
                        Card::Normal(Suit::Spade, Rank::Five),
                        Card::Normal(Suit::Spade, Rank::Six),
                    ]),
                    Comb::Seq(vec![
                        Card::Normal(Suit::Heart, Rank::Seven),
                        Card::Normal(Suit::Heart, Rank::Eight),
                        Card::Normal(Suit::Heart, Rank::Nine),
                    ]),
                ],
                None,
                Some(vec![Suit::Heart, Suit::Heart, Suit::Heart]),
            ),
            (
                vec![
                    Comb::Seq(vec![
                        Card::Normal(Suit::Spade, Rank::Four),
                        Card::Normal(Suit::Spade, Rank::Five),
                        Card::Normal(Suit::Spade, Rank::Six),
                    ]),
                    Comb::Seq(vec![
                        Card::Normal(Suit::Spade, Rank::Seven),
                        Card::Normal(Suit::Spade, Rank::Eight),
                        Card::Normal(Suit::Spade, Rank::Nine),
                    ]),
                ],
                Some(vec![Suit::Spade, Suit::Spade, Suit::Spade]),
                None,
            ),
            (
                vec![
                    Comb::Seq(vec![
                        Card::Normal(Suit::Diamond, Rank::Four),
                        Card::Normal(Suit::Diamond, Rank::Five),
                        Card::Normal(Suit::Diamond, Rank::Six),
                    ]),
                    Comb::Seq(vec![
                        Card::Joker,
                        Card::Normal(Suit::Spade, Rank::Eight),
                        Card::Normal(Suit::Spade, Rank::Nine),
                    ]),
                ],
                None,
                None,
            ),
        ] {
            let mut binder = SuitBinder::new();
            for comb in combs {
                binder.push(&comb);
            }
            assert_eq!(binder.suits, expected_suits);
            assert_eq!(binder.prev_suits, expected_prev_suits);
        }
    }

    #[test]
    fn test_is_valid() {
        // ♣︎縛り
        let binder = create_suit_binder(vec![Suit::Club]);
        for (comb, expected) in [
            (Comb::Single(Card::Normal(Suit::Heart, Rank::Six)), false),
            (Comb::Single(Card::Normal(Suit::Club, Rank::Ten)), true),
            (Comb::Single(Card::Joker), true),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Three),
                    Card::Normal(Suit::Diamond, Rank::Three),
                ]),
                false,
            ),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Two),
                    Card::Normal(Suit::Diamond, Rank::Two),
                    Card::Normal(Suit::Heart, Rank::Two),
                    Card::Joker,
                ]),
                false,
            ),
            (
                Comb::Seq(vec![
                    Card::Normal(Suit::Club, Rank::Jack),
                    Card::Normal(Suit::Club, Rank::Queen),
                    Card::Normal(Suit::Club, Rank::King),
                ]),
                false,
            ),
            (
                Comb::Seq(vec![
                    Card::Normal(Suit::Club, Rank::Seven),
                    Card::Joker,
                    Card::Normal(Suit::Club, Rank::Nine),
                ]),
                false,
            ),
            (
                Comb::Seq(vec![
                    Card::Normal(Suit::Heart, Rank::Jack),
                    Card::Normal(Suit::Heart, Rank::Queen),
                    Card::Normal(Suit::Heart, Rank::King),
                ]),
                false,
            ),
            (
                Comb::Seq(vec![
                    Card::Normal(Suit::Spade, Rank::Nine),
                    Card::Normal(Suit::Spade, Rank::Ten),
                    Card::Joker,
                ]),
                false,
            ),
        ] {
            assert_eq!(binder.is_valid(&comb), expected);
        }
        // ♣︎3枚縛り
        let binder = create_suit_binder(vec![Suit::Club, Suit::Club, Suit::Club]);
        for (comb, expected) in [
            (Comb::Single(Card::Normal(Suit::Heart, Rank::Six)), false),
            (Comb::Single(Card::Normal(Suit::Club, Rank::Ten)), false),
            (Comb::Single(Card::Joker), false),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Three),
                    Card::Normal(Suit::Diamond, Rank::Three),
                ]),
                false,
            ),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Two),
                    Card::Normal(Suit::Diamond, Rank::Two),
                    Card::Normal(Suit::Heart, Rank::Two),
                    Card::Joker,
                ]),
                false,
            ),
            (
                Comb::Seq(vec![
                    Card::Normal(Suit::Club, Rank::Jack),
                    Card::Normal(Suit::Club, Rank::Queen),
                    Card::Normal(Suit::Club, Rank::King),
                ]),
                true,
            ),
            (
                Comb::Seq(vec![
                    Card::Normal(Suit::Club, Rank::Seven),
                    Card::Joker,
                    Card::Normal(Suit::Club, Rank::Nine),
                ]),
                true,
            ),
            (
                Comb::Seq(vec![
                    Card::Normal(Suit::Heart, Rank::Jack),
                    Card::Normal(Suit::Heart, Rank::Queen),
                    Card::Normal(Suit::Heart, Rank::King),
                ]),
                false,
            ),
            (
                Comb::Seq(vec![
                    Card::Normal(Suit::Spade, Rank::Nine),
                    Card::Normal(Suit::Spade, Rank::Ten),
                    Card::Joker,
                ]),
                false,
            ),
        ] {
            assert_eq!(binder.is_valid(&comb), expected);
        }
        // ♦︎、❤︎、♠️縛り
        let binder = create_suit_binder(vec![Suit::Diamond, Suit::Heart, Suit::Spade]);
        for (comb, expected) in [
            (Comb::Single(Card::Normal(Suit::Heart, Rank::Six)), false),
            (Comb::Single(Card::Normal(Suit::Club, Rank::Ten)), false),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Diamond, Rank::Ace),
                    Card::Normal(Suit::Heart, Rank::Ace),
                    Card::Normal(Suit::Spade, Rank::Ace),
                ]),
                true,
            ),
            (
                Comb::Multi(vec![
                    Card::Joker,
                    Card::Normal(Suit::Heart, Rank::Six),
                    Card::Normal(Suit::Spade, Rank::Six),
                ]),
                true,
            ),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Five),
                    Card::Normal(Suit::Heart, Rank::Five),
                    Card::Normal(Suit::Spade, Rank::Five),
                ]),
                false,
            ),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Four),
                    Card::Normal(Suit::Diamond, Rank::Four),
                    Card::Normal(Suit::Heart, Rank::Four),
                    Card::Normal(Suit::Spade, Rank::Four),
                ]),
                false,
            ),
            (
                Comb::Seq(vec![
                    Card::Normal(Suit::Heart, Rank::Jack),
                    Card::Normal(Suit::Heart, Rank::Queen),
                    Card::Normal(Suit::Heart, Rank::King),
                ]),
                false,
            ),
            (
                Comb::Seq(vec![
                    Card::Normal(Suit::Spade, Rank::Nine),
                    Card::Normal(Suit::Spade, Rank::Ten),
                    Card::Joker,
                    Card::Normal(Suit::Spade, Rank::Queen),
                ]),
                false,
            ),
        ] {
            assert_eq!(binder.is_valid(&comb), expected);
        }
    }
}
