use crate::card::Card;
use crate::comb::{Comb, MIN_MULTI, MIN_SEQ};
use crate::player::Player;
use crate::validator::Validator;
use itertools::Itertools;

pub struct MinNpc {
    name: String,
    hands: Vec<Card>,
}

impl MinNpc {
    pub fn new(name: String) -> Self {
        Self {
            name,
            hands: vec![],
        }
    }

    fn remove_hands(&mut self, indices: &[usize]) {
        // 手札からカードを除く
        for i in indices.iter().rev() {
            self.hands.remove(*i);
        }
    }
}

impl Player for MinNpc {
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
        match validator.get_prev_comb() {
            Some(comb) => match comb {
                Comb::Single(_) => {
                    // 場に出せる最小のカードのインデックスを探す
                    (0..self.hands.len()).find_map(|i| {
                        let new_comb = Comb::Single(self.hands[i]);
                        validator.is_valid(&new_comb).then(|| {
                            self.hands.remove(i);
                            new_comb
                        })
                    })
                }
                Comb::Multi(cards) => {
                    let len = cards.len();
                    get_indices_grouped_by_rank(&self.hands, len)
                        .into_iter()
                        .find_map(|indices| {
                            // 場に出せる最小のカードの組み合わせを探す
                            let cards = get_cards(&self.hands, &indices[0..len]);
                            let new_comb = Comb::try_from(cards).ok()?;
                            validator.is_valid(&new_comb).then(|| {
                                self.remove_hands(&indices[0..len]);
                                new_comb
                            })
                        })
                }
                Comb::Seq(cards) => {
                    let len = cards.len();
                    get_indices_grouped_by_suit(&self.hands, len)
                        .into_iter()
                        .find_map(|indices| {
                            // 場に出せる最小のカードの組み合わせを探す
                            let (new_comb, indices) = find_seq(&self.hands, &indices, len)?;
                            validator.is_valid(&new_comb).then(|| {
                                self.remove_hands(&indices[0..len]);
                                new_comb
                            })
                        })
                }
            },
            None => {
                // 複数のカードを出す
                let new_comb: Option<Comb> = get_indices_grouped_by_rank(&self.hands, MIN_MULTI)
                    .into_iter()
                    .find_map(|indices| {
                        let cards = get_cards(&self.hands, &indices);
                        let comb = Comb::try_from(cards).ok()?;
                        self.remove_hands(&indices);
                        Some(comb)
                    });
                if new_comb.is_some() {
                    return new_comb;
                }
                // 階段を出す
                let new_comb: Option<Comb> = get_indices_grouped_by_suit(&self.hands, MIN_SEQ)
                    .into_iter()
                    .find_map(|indices| {
                        // 階段となる組み合わせを探す(枚数の多い順に探す)
                        let (comb, indices) = (MIN_SEQ..indices.len() + 1)
                            .rev()
                            .find_map(|len| find_seq(&self.hands, &indices, len))?;
                        self.remove_hands(&indices);
                        Some(comb)
                    });
                if new_comb.is_some() {
                    return new_comb;
                }
                //1枚のカードを出す
                (!self.hands.is_empty()).then(|| Comb::Single(self.hands.remove(0)))
            }
        }
    }

    fn get_needless_cards(&mut self, cards_count: usize) -> Vec<Card> {
        (0..cards_count).map(|_| self.hands.remove(0)).collect()
    }
}

fn get_cards(cards: &[Card], indices: &[usize]) -> Vec<Card> {
    indices.iter().map(|i| cards[*i]).collect()
}

fn get_indices_grouped_by_rank(cards: &[Card], len: usize) -> Vec<Vec<usize>> {
    // 数字毎にグループ分けしたインデックスのベクタを取得する
    (0..cards.len())
        .group_by(|i| match cards[*i] {
            Card::Normal(_, r) => Some(r),
            _ => None,
        })
        .into_iter()
        .map(|(_, grp)| grp.collect::<Vec<usize>>())
        .filter(|indices| indices.len() >= len)
        .collect()
}

fn get_indices_grouped_by_suit(cards: &[Card], len: usize) -> Vec<Vec<usize>> {
    // スート毎にグループ分けしたインデックスのベクタを取得する
    (0..cards.len())
        .into_group_map_by(|i| match cards[*i] {
            Card::Normal(s, _) => Some(s),
            _ => None,
        })
        .into_iter()
        .filter(|(k, indices)| k.is_some() && (indices.len() >= len))
        .sorted_by(|x, y| {
            let s1 = x.0.unwrap();
            let s2 = y.0.unwrap();
            s1.cmp(&s2)
        })
        .map(|(_, indices)| indices)
        .collect()
}

fn find_seq(cards: &[Card], indices: &[usize], len: usize) -> Option<(Comb, Vec<usize>)> {
    // 階段となる組み合わせのカードを探す
    (0..indices.len() + 1 - len).find_map(|i| {
        let seq_cards = get_cards(cards, &indices[i..len + i]);
        let comb = Comb::try_from(seq_cards).ok()?;
        Some((comb, indices[i..len + i].to_vec()))
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::card::{cmp_rank, cmp_rank_reversely, Rank, Suit};

    struct TestValidator {
        is_revolution: bool,
        prev_comb: Option<Comb>,
    }

    impl TestValidator {
        pub fn new(is_revolution: bool) -> Self {
            TestValidator {
                is_revolution,
                prev_comb: None,
            }
        }
    }

    impl Validator for TestValidator {
        fn get_prev_comb(&self) -> Option<&Comb> {
            self.prev_comb.as_ref()
        }

        fn is_valid(&self, comb: &Comb) -> bool {
            match &self.prev_comb {
                Some(prev_comb) => {
                    let comparator = match self.is_revolution {
                        true => cmp_rank_reversely,
                        false => cmp_rank,
                    };
                    comb.is_greater(prev_comb, comparator)
                }
                None => true,
            }
        }
    }

    #[test]
    fn test_get_indices_grouped_by_rank() {
        let cards = vec![
            Card::Normal(Suit::Spade, Rank::Three),
            Card::Normal(Suit::Heart, Rank::Four),
            Card::Normal(Suit::Spade, Rank::Four),
            Card::Normal(Suit::Club, Rank::Five),
            Card::Normal(Suit::Diamond, Rank::Five),
            Card::Normal(Suit::Heart, Rank::Five),
            Card::Normal(Suit::Spade, Rank::Five),
            Card::Normal(Suit::Club, Rank::Six),
            Card::Normal(Suit::Diamond, Rank::Six),
            Card::Normal(Suit::Heart, Rank::Six),
            Card::Joker,
        ];
        let expected = vec![vec![1, 2], vec![3, 4, 5, 6], vec![7, 8, 9]];
        assert_eq!(get_indices_grouped_by_rank(&cards, 2), expected);
    }

    #[test]
    fn test_get_indices_grouped_by_suit() {
        let cards = vec![
            Card::Normal(Suit::Spade, Rank::Three),
            Card::Normal(Suit::Heart, Rank::Four),
            Card::Normal(Suit::Spade, Rank::Four),
            Card::Normal(Suit::Club, Rank::Five),
            Card::Normal(Suit::Diamond, Rank::Five),
            Card::Normal(Suit::Heart, Rank::Five),
            Card::Normal(Suit::Spade, Rank::Five),
            Card::Normal(Suit::Club, Rank::Six),
            Card::Normal(Suit::Diamond, Rank::Six),
            Card::Normal(Suit::Heart, Rank::Six),
            Card::Normal(Suit::Spade, Rank::Six),
            Card::Joker,
        ];
        let expected = vec![vec![3, 7], vec![4, 8], vec![1, 5, 9], vec![0, 2, 6, 10]];
        assert_eq!(get_indices_grouped_by_suit(&cards, 2), expected);
    }

    #[test]
    fn test_find_seq() {
        let cards = vec![
            Card::Normal(Suit::Spade, Rank::Three),
            Card::Normal(Suit::Spade, Rank::Five),
            Card::Normal(Suit::Spade, Rank::Seven),
            Card::Normal(Suit::Spade, Rank::Eight),
            Card::Normal(Suit::Spade, Rank::Nine),
            Card::Normal(Suit::Spade, Rank::Ten),
            Card::Normal(Suit::Spade, Rank::Queen),
            Card::Normal(Suit::Spade, Rank::King),
            Card::Normal(Suit::Spade, Rank::Ace),
            Card::Normal(Suit::Spade, Rank::Two),
        ];
        let actual = find_seq(&cards, &(0..cards.len()).collect::<Vec<usize>>(), 4);
        let expected = Some((
            Comb::Seq(vec![
                Card::Normal(Suit::Spade, Rank::Seven),
                Card::Normal(Suit::Spade, Rank::Eight),
                Card::Normal(Suit::Spade, Rank::Nine),
                Card::Normal(Suit::Spade, Rank::Ten),
            ]),
            vec![2, 3, 4, 5],
        ));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_min_npc_play_single() {
        let mut validator = TestValidator::new(false);
        let cards = vec![
            Card::Normal(Suit::Heart, Rank::Three),
            Card::Normal(Suit::Spade, Rank::Five),
            Card::Normal(Suit::Club, Rank::Ten),
            Card::Normal(Suit::Heart, Rank::Ten),
            Card::Normal(Suit::Diamond, Rank::King),
        ];
        let mut player = MinNpc::new("A".to_owned());
        player.init(cards);
        for (prev_comb, expected) in [
            (
                Some(Comb::Single(Card::Normal(Suit::Spade, Rank::Three))),
                Some(Comb::Single(Card::Normal(Suit::Spade, Rank::Five))),
            ),
            (
                Some(Comb::Single(Card::Normal(Suit::Spade, Rank::Eight))),
                Some(Comb::Single(Card::Normal(Suit::Club, Rank::Ten))),
            ),
            (
                Some(Comb::Single(Card::Normal(Suit::Club, Rank::Jack))),
                Some(Comb::Single(Card::Normal(Suit::Diamond, Rank::King))),
            ),
            (
                Some(Comb::Single(Card::Normal(Suit::Diamond, Rank::Two))),
                None,
            ),
        ] {
            validator.prev_comb = prev_comb.clone();
            let actual = player.play(&validator);
            assert_eq!(actual, expected);
        }
        assert_eq!(player.count_hands(), 2);
        let mut validator = TestValidator::new(true);
        let cards = vec![
            Card::Normal(Suit::Diamond, Rank::King),
            Card::Normal(Suit::Club, Rank::Ten),
            Card::Normal(Suit::Heart, Rank::Ten),
            Card::Normal(Suit::Spade, Rank::Five),
            Card::Normal(Suit::Heart, Rank::Three),
        ];
        let mut player = MinNpc::new("A".to_owned());
        player.init(cards);
        for (prev_comb, expected) in [
            (
                Some(Comb::Single(Card::Normal(Suit::Spade, Rank::Queen))),
                Some(Comb::Single(Card::Normal(Suit::Club, Rank::Ten))),
            ),
            (
                Some(Comb::Single(Card::Normal(Suit::Spade, Rank::Seven))),
                Some(Comb::Single(Card::Normal(Suit::Spade, Rank::Five))),
            ),
        ] {
            validator.prev_comb = prev_comb.clone();
            let actual = player.play(&validator);
            assert_eq!(expected, actual);
        }
        assert_eq!(player.count_hands(), 3);
    }

    #[test]
    fn test_min_npc_play_multi() {
        let mut validator = TestValidator::new(false);
        let cards = vec![
            Card::Normal(Suit::Heart, Rank::Four),
            Card::Normal(Suit::Spade, Rank::Four),
            Card::Normal(Suit::Club, Rank::Ten),
            Card::Normal(Suit::Heart, Rank::Ten),
            Card::Normal(Suit::Spade, Rank::Ten),
            Card::Normal(Suit::Club, Rank::King),
            Card::Normal(Suit::Diamond, Rank::King),
            Card::Normal(Suit::Heart, Rank::King),
            Card::Normal(Suit::Spade, Rank::King),
        ];
        let mut player = MinNpc::new("A".to_owned());
        player.init(cards);
        for (prev_comb, expected) in [
            (
                Some(Comb::Multi(vec![
                    Card::Normal(Suit::Heart, Rank::Three),
                    Card::Normal(Suit::Spade, Rank::Three),
                ])),
                Some(Comb::Multi(vec![
                    Card::Normal(Suit::Heart, Rank::Four),
                    Card::Normal(Suit::Spade, Rank::Four),
                ])),
            ),
            (
                Some(Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Eight),
                    Card::Normal(Suit::Diamond, Rank::Eight),
                    Card::Normal(Suit::Heart, Rank::Eight),
                ])),
                Some(Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Ten),
                    Card::Normal(Suit::Heart, Rank::Ten),
                    Card::Normal(Suit::Spade, Rank::Ten),
                ])),
            ),
            (
                Some(Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Two),
                    Card::Normal(Suit::Diamond, Rank::Two),
                    Card::Normal(Suit::Heart, Rank::Two),
                    Card::Normal(Suit::Heart, Rank::Two),
                ])),
                None,
            ),
        ] {
            validator.prev_comb = prev_comb.clone();
            let actual = player.play(&validator);
            assert_eq!(actual, expected);
        }
        assert_eq!(player.count_hands(), 4);
        let mut validator = TestValidator::new(true);
        let cards = vec![
            Card::Normal(Suit::Heart, Rank::Four),
            Card::Normal(Suit::Spade, Rank::Four),
            Card::Normal(Suit::Club, Rank::Ten),
            Card::Normal(Suit::Heart, Rank::Ten),
            Card::Normal(Suit::Spade, Rank::Ten),
            Card::Normal(Suit::Club, Rank::King),
            Card::Normal(Suit::Diamond, Rank::King),
            Card::Normal(Suit::Heart, Rank::King),
            Card::Normal(Suit::Spade, Rank::King),
        ];
        let mut player = MinNpc::new("A".to_owned());
        player.init(cards);
        for (prev_comb, expected) in [
            (
                Some(Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Two),
                    Card::Normal(Suit::Diamond, Rank::Two),
                    Card::Normal(Suit::Heart, Rank::Two),
                    Card::Normal(Suit::Spade, Rank::Two),
                ])),
                Some(Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::King),
                    Card::Normal(Suit::Diamond, Rank::King),
                    Card::Normal(Suit::Heart, Rank::King),
                    Card::Normal(Suit::Spade, Rank::King),
                ])),
            ),
            (
                Some(Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Jack),
                    Card::Normal(Suit::Diamond, Rank::Jack),
                    Card::Normal(Suit::Heart, Rank::Jack),
                ])),
                Some(Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Ten),
                    Card::Normal(Suit::Heart, Rank::Ten),
                    Card::Normal(Suit::Spade, Rank::Ten),
                ])),
            ),
            (
                Some(Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Ace),
                    Card::Normal(Suit::Diamond, Rank::Ace),
                    Card::Normal(Suit::Heart, Rank::Ace),
                    Card::Normal(Suit::Heart, Rank::Ace),
                ])),
                None,
            ),
        ] {
            validator.prev_comb = prev_comb.clone();
            let actual = player.play(&validator);
            assert_eq!(actual, expected);
        }
        assert_eq!(player.count_hands(), 2);
    }

    #[test]
    fn test_min_npc_play_seq() {
        let mut validator = TestValidator::new(false);
        let cards = vec![
            Card::Normal(Suit::Diamond, Rank::Three),
            Card::Normal(Suit::Diamond, Rank::Five),
            Card::Normal(Suit::Diamond, Rank::Six),
            Card::Normal(Suit::Spade, Rank::Six),
            Card::Normal(Suit::Diamond, Rank::Seven),
            Card::Normal(Suit::Heart, Rank::Ten),
            Card::Normal(Suit::Diamond, Rank::King),
        ];
        let mut player = MinNpc::new("A".to_owned());
        player.init(cards);
        for (prev_comb, expected) in [
            (
                Some(Comb::Seq(vec![
                    Card::Normal(Suit::Spade, Rank::Three),
                    Card::Normal(Suit::Spade, Rank::Four),
                    Card::Normal(Suit::Spade, Rank::Five),
                ])),
                Some(Comb::Seq(vec![
                    Card::Normal(Suit::Diamond, Rank::Five),
                    Card::Normal(Suit::Diamond, Rank::Six),
                    Card::Normal(Suit::Diamond, Rank::Seven),
                ])),
            ),
            (
                Some(Comb::Seq(vec![
                    Card::Normal(Suit::Heart, Rank::Queen),
                    Card::Normal(Suit::Heart, Rank::King),
                    Card::Normal(Suit::Heart, Rank::Ace),
                    Card::Normal(Suit::Heart, Rank::Two),
                ])),
                None,
            ),
        ] {
            validator.prev_comb = prev_comb.clone();
            let actual = player.play(&validator);
            assert_eq!(actual, expected);
        }
        assert_eq!(player.count_hands(), 4);
        let mut validator = TestValidator::new(true);
        let cards = vec![
            Card::Normal(Suit::Heart, Rank::King),
            Card::Normal(Suit::Heart, Rank::Queen),
            Card::Normal(Suit::Spade, Rank::Queen),
            Card::Normal(Suit::Heart, Rank::Jack),
            Card::Normal(Suit::Club, Rank::Ten),
            Card::Normal(Suit::Heart, Rank::Ten),
            Card::Normal(Suit::Club, Rank::Nine),
            Card::Normal(Suit::Club, Rank::Eight),
        ];
        let mut player = MinNpc::new("A".to_owned());
        player.init(cards);
        for (prev_comb, expected) in [
            (
                Some(Comb::Seq(vec![
                    Card::Normal(Suit::Spade, Rank::Two),
                    Card::Normal(Suit::Spade, Rank::Ace),
                    Card::Normal(Suit::Spade, Rank::King),
                    Card::Normal(Suit::Spade, Rank::Queen),
                ])),
                Some(Comb::Seq(vec![
                    Card::Normal(Suit::Heart, Rank::King),
                    Card::Normal(Suit::Heart, Rank::Queen),
                    Card::Normal(Suit::Heart, Rank::Jack),
                    Card::Normal(Suit::Heart, Rank::Ten),
                ])),
            ),
            (
                Some(Comb::Seq(vec![
                    Card::Normal(Suit::Diamond, Rank::Queen),
                    Card::Joker,
                    Card::Normal(Suit::Diamond, Rank::Ten),
                ])),
                Some(Comb::Seq(vec![
                    Card::Normal(Suit::Club, Rank::Ten),
                    Card::Normal(Suit::Club, Rank::Nine),
                    Card::Normal(Suit::Club, Rank::Eight),
                ])),
            ),
            (
                Some(Comb::Seq(vec![
                    Card::Normal(Suit::Diamond, Rank::Five),
                    Card::Normal(Suit::Diamond, Rank::Four),
                    Card::Normal(Suit::Diamond, Rank::Three),
                ])),
                None,
            ),
        ] {
            validator.prev_comb = prev_comb.clone();
            let actual = player.play(&validator);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_min_npc_play_first_comb() {
        let validator = TestValidator::new(false);
        for (cards, expected_comb, expected_len) in [
            (
                vec![
                    Card::Normal(Suit::Club, Rank::Three),
                    Card::Normal(Suit::Club, Rank::Six),
                    Card::Normal(Suit::Spade, Rank::Six),
                    Card::Normal(Suit::Diamond, Rank::Eight),
                    Card::Normal(Suit::Heart, Rank::Eight),
                ],
                Some(Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Six),
                    Card::Normal(Suit::Spade, Rank::Six),
                ])),
                3,
            ),
            (
                vec![
                    Card::Normal(Suit::Club, Rank::Five),
                    Card::Normal(Suit::Heart, Rank::Seven),
                    Card::Normal(Suit::Diamond, Rank::Nine),
                    Card::Normal(Suit::Diamond, Rank::Ten),
                    Card::Normal(Suit::Diamond, Rank::Jack),
                    Card::Normal(Suit::Diamond, Rank::Queen),
                    Card::Normal(Suit::Spade, Rank::King),
                    Card::Normal(Suit::Spade, Rank::Ace),
                    Card::Normal(Suit::Spade, Rank::Two),
                ],
                Some(Comb::Seq(vec![
                    Card::Normal(Suit::Diamond, Rank::Nine),
                    Card::Normal(Suit::Diamond, Rank::Ten),
                    Card::Normal(Suit::Diamond, Rank::Jack),
                    Card::Normal(Suit::Diamond, Rank::Queen),
                ])),
                5,
            ),
            (
                vec![
                    Card::Normal(Suit::Heart, Rank::Eight),
                    Card::Normal(Suit::Club, Rank::Queen),
                    Card::Normal(Suit::Diamond, Rank::Two),
                ],
                Some(Comb::Single(Card::Normal(Suit::Heart, Rank::Eight))),
                2,
            ),
        ] {
            let mut player = MinNpc::new("A".to_owned());
            player.init(cards);
            let actual = player.play(&validator);
            assert_eq!(actual, expected_comb);
            assert_eq!(player.count_hands(), expected_len);
        }
    }
}
