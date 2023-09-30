use crate::{card::Card, comb::Comb, input::get_input, player::Player, validator::Validator};
use itertools::Itertools;

pub struct Pc {
    name: String,
    hands: Vec<Card>,
}

impl Pc {
    pub fn new(name: String) -> Self {
        Self {
            name,
            hands: vec![],
        }
    }
}

impl Player for Pc {
    fn init(&mut self, hands: Vec<Card>) {
        self.hands = hands;
    }

    fn count_hands(&self) -> usize {
        self.hands.len()
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_hands(&mut self) -> &mut Vec<Card> {
        &mut self.hands
    }

    fn play(&mut self, validator: &dyn Validator) -> Option<Comb> {
        let prev_comb = validator.get_prev_comb();
        let comb_str = match prev_comb {
            Some(Comb::Single(card)) => format!("({}) ", String::from(card)),
            Some(Comb::Multi(cards) | Comb::Seq(cards)) => {
                let str = cards.iter().map(String::from).join(" ");
                format!("({}) ", str)
            }
            None => "".to_owned(),
        };
        println!("{}", get_cards_with_indices(&self.hands));
        loop {
            let input = get_input(format!("カードの番号{}: ", comb_str));
            if input.is_empty() && prev_comb.is_some() {
                return None;
            }
            let result = parse_idx(&input);
            if result.is_err() {
                continue;
            }
            let indices = result.unwrap();
            let result = get_cards(&indices, &self.hands);
            if result.is_err() {
                continue;
            }
            match conver_to_comb(result.unwrap()) {
                Ok(comb) if validator.is_valid(&comb) => {
                    // 手札からカードを除く
                    for i in indices.iter().rev() {
                        self.hands.remove(*i);
                    }
                    return Some(comb);
                }
                _ => {
                    println!("無効な組み合わせ");
                }
            }
        }
    }

    fn get_needless_cards(&mut self, cards_count: usize) -> Vec<Card> {
        println!("{}", get_cards_with_indices(&self.hands));
        loop {
            let input = get_input(format!("不要なカードを{}枚選択: ", cards_count));
            let result = parse_idx(&input);
            if result.is_err() {
                continue;
            }
            let indices = result.unwrap();
            let result = get_cards(&indices, &self.hands);
            if result.is_err() {
                continue;
            }
            // 手札からカードを除く
            for i in indices.iter().rev() {
                self.hands.remove(*i);
            }
            return result.unwrap();
        }
    }
}

fn get_cards_with_indices(cards: &[Card]) -> String {
    cards
        .iter()
        .enumerate()
        .map(|(idx, card)| format!("{:2}:{}", idx, String::from(card)))
        .join("\n")
}

fn conver_to_comb(cards: Vec<Card>) -> Result<Comb, ()> {
    if cards.len() == 1 {
        Ok(Comb::Single(cards[0]))
    } else {
        Comb::try_from(cards)
    }
}

fn parse_idx(input: &str) -> Result<Vec<usize>, ()> {
    let results: Vec<_> = input.split(' ').map(|s| s.parse::<usize>()).collect();
    match results.iter().all(|r| r.is_ok()) {
        true => Ok(results.into_iter().map(|r| r.unwrap()).sorted().collect()),
        false => Err(()),
    }
}

fn get_cards(indices: &[usize], cards: &[Card]) -> Result<Vec<Card>, ()> {
    let cards: Vec<Option<&Card>> = indices.iter().map(|idx| cards.get(*idx)).collect();
    match cards.iter().any(|card| card.is_none()) {
        true => Err(()),
        false => Ok(cards.iter().map(|card| *card.unwrap()).collect()),
    }
}

#[cfg(test)]
mod test {
    use crate::{
        card::{Card, Rank, Suit},
        comb::Comb,
        pc::{conver_to_comb, get_cards, get_cards_with_indices, parse_idx},
    };

    #[test]
    fn test_get_cards_with_indices() {
        let cards = vec![
            Card::Normal(Suit::Heart, Rank::Three),
            Card::Normal(Suit::Spade, Rank::Five),
        ];
        assert_eq!(get_cards_with_indices(&cards), " 0:♥3\n 1:♠️5");
    }

    #[test]
    fn test_conver_to_comb() {
        for (cards, expected) in [
            (
                vec![Card::Normal(Suit::Spade, Rank::Three)],
                Ok(Comb::Single(Card::Normal(Suit::Spade, Rank::Three))),
            ),
            (
                vec![
                    Card::Normal(Suit::Diamond, Rank::Four),
                    Card::Normal(Suit::Heart, Rank::Four),
                ],
                Ok(Comb::Multi(vec![
                    Card::Normal(Suit::Diamond, Rank::Four),
                    Card::Normal(Suit::Heart, Rank::Four),
                ])),
            ),
            (
                vec![
                    Card::Normal(Suit::Club, Rank::Five),
                    Card::Normal(Suit::Club, Rank::Six),
                    Card::Normal(Suit::Club, Rank::Seven),
                ],
                Ok(Comb::Seq(vec![
                    Card::Normal(Suit::Club, Rank::Five),
                    Card::Normal(Suit::Club, Rank::Six),
                    Card::Normal(Suit::Club, Rank::Seven),
                ])),
            ),
            (vec![], Err(())),
        ] {
            assert_eq!(conver_to_comb(cards), expected);
        }
    }

    #[test]
    fn test_parse_idx() {
        for (input, expected) in [
            ("1 2 3 4", Ok(vec![1, 2, 3, 4])),
            ("0 2 1", Ok(vec![0, 1, 2])),
            ("1 2 a 4", Err(())),
            ("xyz", Err(())),
        ] {
            assert_eq!(parse_idx(input), expected);
        }
    }

    #[test]
    fn test_get_cards() {
        let cards = vec![
            Card::Normal(Suit::Heart, Rank::Three),
            Card::Normal(Suit::Heart, Rank::Four),
            Card::Normal(Suit::Spade, Rank::Five),
        ];
        for (indices, expected) in [
            (
                vec![0, 2],
                Ok(vec![
                    Card::Normal(Suit::Heart, Rank::Three),
                    Card::Normal(Suit::Spade, Rank::Five),
                ]),
            ),
            (
                vec![0, 2, 1],
                Ok(vec![
                    Card::Normal(Suit::Heart, Rank::Three),
                    Card::Normal(Suit::Spade, Rank::Five),
                    Card::Normal(Suit::Heart, Rank::Four),
                ]),
            ),
            (vec![1, 4], Err(())),
        ] {
            assert_eq!(get_cards(&indices, &cards), expected);
        }
    }
}
