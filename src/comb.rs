use crate::card::Card;
use itertools::Itertools;
use std::{cmp::Ordering, collections::HashSet};

pub const MIN_MULTI: usize = 2;
pub const MIN_SEQ: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comb {
    Single(Card),
    Multi(Vec<Card>),
    Seq(Vec<Card>),
}

impl Comb {
    pub fn is_greater<F>(&self, comb: &Comb, comparator: F) -> bool
    where
        F: Fn(&Card, &Card) -> Ordering,
    {
        match (self, comb) {
            (Comb::Single(card1), Comb::Single(card2)) => {
                comparator(card1, card2) == Ordering::Greater
            }
            (Comb::Multi(cards1), Comb::Multi(cards2)) | (Comb::Seq(cards1), Comb::Seq(cards2)) => {
                // カードの枚数が同じか
                if cards1.len() != cards2.len() {
                    return false;
                }
                // cards1の全てのカードがcards2のカードより大きいか
                cards1
                    .iter()
                    .zip(cards2.iter())
                    .all(|(c1, c2)| match (c1, c2) {
                        (Card::Normal(_, _), Card::Normal(_, _)) => {
                            comparator(c1, c2) == Ordering::Greater
                        }
                        // どちらかのカードがジョーカーならtrue
                        (_, _) => true,
                    })
            }
            (_, _) => false,
        }
    }
}

impl TryFrom<Vec<Card>> for Comb {
    type Error = ();

    fn try_from(cards: Vec<Card>) -> Result<Self, Self::Error> {
        let len = cards.len();
        if len < MIN_MULTI {
            return Err(());
        }
        if is_same_ranks(&cards) {
            return Ok(Comb::Multi(cards));
        }
        if len >= MIN_SEQ && is_same_suits(&cards) && is_seq(&cards) {
            return Ok(Comb::Seq(cards));
        }
        Err(())
    }
}

// 全てのカードが同じ数字か判定する
fn is_same_ranks(cards: &[Card]) -> bool {
    cards
        .iter()
        .filter_map(|c| match c {
            Card::Normal(_, r) => Some(r),
            Card::Joker => None,
        })
        .tuple_windows()
        .all(|(v1, v2)| v1 == v2)
}

// 全てのカードが同じスートか判定する
fn is_same_suits(cards: &[Card]) -> bool {
    cards
        .iter()
        .filter_map(|c| match c {
            Card::Normal(s, _) => Some(s),
            Card::Joker => None,
        })
        .tuple_windows()
        .all(|(v1, v2)| v1 == v2)
}

// カードの数字が連続しているか判定する
fn is_seq(cards: &[Card]) -> bool {
    if cards.len() < MIN_SEQ {
        return false;
    }
    let joker_idx = cards.iter().position(|c| matches!(*c, Card::Joker));
    match joker_idx {
        // ジョーカーを含む
        Some(idx) => {
            let mut nums: Vec<Option<i32>> = cards
                .iter()
                .map(|c| match c {
                    // カードの数字をi32に変換
                    Card::Normal(_, r) => Some(i32::from(r)),
                    Card::Joker => None,
                })
                .collect();
            // ジョーカーを数字に置き換える
            match idx {
                _ if idx == 0 => {
                    let x = *nums[idx + 1].as_ref().unwrap();
                    let y = *nums[idx + 2].as_ref().unwrap();
                    nums[idx] = Some(2 * x - y);
                }
                _ if idx == nums.len() - 1 => {
                    let x = *nums[idx - 2].as_ref().unwrap();
                    let y = *nums[idx - 1].as_ref().unwrap();
                    nums[idx] = Some(2 * y - x);
                }
                _ => {
                    let v1 = *nums[idx - 1].as_ref().unwrap();
                    let v2 = *nums[idx + 1].as_ref().unwrap();
                    nums[idx] = Some((v1 + v2) / 2)
                }
            };
            let diffs = nums
                .into_iter()
                .flatten()
                .tuple_windows()
                .map(|(v1, v2)| v2 - v1) // 隣同士の数字の差分を計算する
                .collect::<HashSet<i32>>() // 差分の重複を排除する
                .into_iter()
                .collect::<Vec<i32>>();
            (diffs.len() == 1) && (diffs[0].abs() == 1)
        }
        // ジョーカーなし
        None => {
            // カードから数字を抽出する
            let diffs = cards
                .iter()
                .filter_map(|c| match c {
                    // カードの数字をi32に変換
                    Card::Normal(_, r) => Some(i32::from(r)),
                    Card::Joker => None,
                })
                .tuple_windows()
                .map(|(v1, v2)| v2 - v1) // 隣同士の数字の差分を計算する
                .collect::<HashSet<i32>>() // 差分の重複を排除する
                .into_iter()
                .collect::<Vec<i32>>();
            (diffs.len() == 1) && (diffs[0].abs() == 1)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::card::{cmp_rank, cmp_rank_reversely, Rank, Suit};

    #[test]
    fn test_create_multi() {
        let cards = [
            Card::Normal(Suit::Club, Rank::Three),
            Card::Normal(Suit::Diamond, Rank::Three),
            Card::Normal(Suit::Heart, Rank::Three),
            Card::Normal(Suit::Spade, Rank::Three),
        ];
        let joker = Card::Joker;
        for cds in [
            vec![cards[0], cards[1]],
            vec![cards[0], cards[1], cards[2]],
            vec![cards[0], cards[1], cards[2], cards[3]],
            vec![cards[0], cards[1], cards[2], cards[3], joker],
        ] {
            let expected = Ok::<Comb, ()>(Comb::Multi(cds.clone()));
            assert_eq!(Comb::try_from(cds), expected);
        }
        for cds in [
            vec![],
            vec![joker],
            vec![cards[0], Card::Normal(Suit::Diamond, Rank::Six)],
        ] {
            assert_eq!(Comb::try_from(cds), Err::<Comb, ()>(()));
        }
    }

    #[test]
    fn test_create_seq() {
        let cards = [
            Card::Normal(Suit::Spade, Rank::Three),
            Card::Normal(Suit::Spade, Rank::Four),
            Card::Normal(Suit::Spade, Rank::Five),
            Card::Normal(Suit::Spade, Rank::Six),
        ];
        let joker = Card::Joker;
        for cds in [
            vec![cards[0], cards[1], cards[2]],
            vec![cards[2], cards[1], cards[0]],
            vec![cards[0], cards[1], cards[2], cards[3]],
            vec![cards[3], cards[2], cards[1], cards[0]],
            vec![joker, cards[1], cards[2], cards[3]],
            vec![cards[0], joker, cards[2], cards[3]],
            vec![cards[0], cards[1], joker, cards[3]],
            vec![cards[0], cards[1], cards[2], joker],
            vec![joker, cards[2], cards[1], cards[0]],
            vec![cards[3], joker, cards[1], cards[0]],
            vec![cards[3], cards[2], joker, cards[0]],
            vec![cards[3], cards[2], cards[1], joker],
        ] {
            let expected = Ok::<Comb, ()>(Comb::Seq(cds.clone()));
            assert_eq!(Comb::try_from(cds), expected);
        }
        for cds in [
            vec![],
            vec![cards[0]],
            vec![cards[0], cards[1]],
            vec![cards[1], cards[0], cards[2]],
            vec![cards[3], cards[1], cards[2], cards[0]],
            vec![joker, cards[1], cards[2], cards[0]],
            vec![cards[3], joker, cards[2], cards[0]],
            vec![cards[3], cards[1], joker, cards[0]],
            vec![cards[3], cards[1], cards[2], joker],
        ] {
            assert_eq!(Comb::try_from(cds), Err::<Comb, ()>(()));
        }
    }

    #[test]
    fn test_is_greater_single() {
        for (comb1, comb2, expected) in [
            (
                Comb::Single(Card::Normal(Suit::Spade, Rank::King)),
                Comb::Single(Card::Normal(Suit::Diamond, Rank::Seven)),
                true,
            ),
            (
                Comb::Single(Card::Normal(Suit::Diamond, Rank::Seven)),
                Comb::Single(Card::Normal(Suit::Spade, Rank::King)),
                false,
            ),
        ] {
            assert_eq!(comb1.is_greater(&comb2, cmp_rank), expected);
            assert_eq!(comb1.is_greater(&comb2, cmp_rank_reversely), !expected);
        }
        for (comb1, comb2, expected) in [
            (
                Comb::Single(Card::Joker),
                Comb::Single(Card::Normal(Suit::Diamond, Rank::Seven)),
                true,
            ),
            (
                Comb::Single(Card::Normal(Suit::Diamond, Rank::Seven)),
                Comb::Single(Card::Joker),
                false,
            ),
        ] {
            assert_eq!(comb1.is_greater(&comb2, cmp_rank), expected);
            assert_eq!(comb1.is_greater(&comb2, cmp_rank_reversely), expected);
        }
    }

    #[test]
    fn test_is_greater_multi() {
        let comb1 = Comb::Multi(vec![
            Card::Normal(Suit::Spade, Rank::Nine),
            Card::Normal(Suit::Heart, Rank::Nine),
            Card::Normal(Suit::Club, Rank::Nine),
        ]);
        let comb2 = Comb::Multi(vec![
            Card::Normal(Suit::Spade, Rank::Seven),
            Card::Normal(Suit::Heart, Rank::Seven),
        ]);
        assert_eq!(comb1.is_greater(&comb2, cmp_rank), false);
        assert_eq!(comb1.is_greater(&comb2, cmp_rank_reversely), false);
        for (cards, expected) in [
            (
                vec![
                    Card::Normal(Suit::Club, Rank::Five),
                    Card::Normal(Suit::Diamond, Rank::Five),
                    Card::Normal(Suit::Heart, Rank::Five),
                ],
                true,
            ),
            (
                vec![
                    Card::Normal(Suit::Club, Rank::King),
                    Card::Normal(Suit::Heart, Rank::King),
                    Card::Normal(Suit::Spade, Rank::King),
                ],
                false,
            ),
            (
                vec![
                    Card::Normal(Suit::Club, Rank::Four),
                    Card::Normal(Suit::Diamond, Rank::Four),
                    Card::Joker,
                ],
                true,
            ),
            (
                vec![
                    Card::Normal(Suit::Club, Rank::Ace),
                    Card::Normal(Suit::Diamond, Rank::Ace),
                    Card::Joker,
                ],
                false,
            ),
        ] {
            let comb2 = Comb::try_from(cards).unwrap();
            assert_eq!(comb1.is_greater(&comb2, cmp_rank), expected);
            assert_eq!(comb1.is_greater(&comb2, cmp_rank_reversely), !expected);
        }
        // 4枚の10(ジョーカーを含む)
        let comb1 = Comb::Multi(vec![
            Card::Normal(Suit::Club, Rank::Ten),
            Card::Normal(Suit::Heart, Rank::Ten),
            Card::Normal(Suit::Spade, Rank::Ten),
            Card::Joker,
        ]);
        for (cards, expected) in [
            (
                vec![
                    Card::Normal(Suit::Club, Rank::Three),
                    Card::Normal(Suit::Diamond, Rank::Three),
                    Card::Normal(Suit::Heart, Rank::Three),
                    Card::Normal(Suit::Spade, Rank::Three),
                ],
                true,
            ),
            (
                vec![
                    Card::Normal(Suit::Club, Rank::Two),
                    Card::Normal(Suit::Diamond, Rank::Two),
                    Card::Normal(Suit::Heart, Rank::Two),
                    Card::Normal(Suit::Spade, Rank::Two),
                ],
                false,
            ),
        ] {
            let comb2 = Comb::try_from(cards).unwrap();
            assert_eq!(comb1.is_greater(&comb2, cmp_rank), expected);
            assert_eq!(comb1.is_greater(&comb2, cmp_rank_reversely), !expected);
        }
    }

    #[test]
    fn test_is_greater_seq() {
        let comb1 = Comb::Seq(vec![
            Card::Normal(Suit::Spade, Rank::Nine),
            Card::Normal(Suit::Spade, Rank::Ten),
            Card::Normal(Suit::Spade, Rank::Jack),
        ]);
        for (cards, expected) in [
            (
                vec![
                    Card::Normal(Suit::Club, Rank::Three),
                    Card::Normal(Suit::Club, Rank::Four),
                    Card::Normal(Suit::Club, Rank::Five),
                ],
                true,
            ),
            (
                vec![
                    Card::Normal(Suit::Club, Rank::Six),
                    Card::Joker,
                    Card::Normal(Suit::Club, Rank::Eight),
                ],
                true,
            ),
            (
                vec![
                    Card::Normal(Suit::Heart, Rank::Eight),
                    Card::Normal(Suit::Heart, Rank::Nine),
                    Card::Normal(Suit::Heart, Rank::Ten),
                ],
                true,
            ),
            (
                vec![
                    Card::Normal(Suit::Diamond, Rank::Ten),
                    Card::Normal(Suit::Diamond, Rank::Jack),
                    Card::Normal(Suit::Diamond, Rank::Queen),
                ],
                false,
            ),
            (
                vec![
                    Card::Normal(Suit::Diamond, Rank::Jack),
                    Card::Normal(Suit::Diamond, Rank::Queen),
                    Card::Joker,
                ],
                false,
            ),
            (
                vec![
                    Card::Normal(Suit::Spade, Rank::King),
                    Card::Normal(Suit::Spade, Rank::Ace),
                    Card::Normal(Suit::Spade, Rank::Two),
                ],
                false,
            ),
        ] {
            let comb2 = Comb::try_from(cards).unwrap();
            assert_eq!(comb1.is_greater(&comb2, cmp_rank), expected);
            assert_eq!(comb1.is_greater(&comb2, cmp_rank_reversely), !expected);
        }
    }

    #[test]
    fn test_is_same_ranks() {
        for (cards, expected) in [
            (vec![], true),
            (vec![Card::Normal(Suit::Spade, Rank::Five)], true),
            (
                vec![
                    Card::Normal(Suit::Diamond, Rank::Ace),
                    Card::Normal(Suit::Spade, Rank::Ace),
                ],
                true,
            ),
            (
                vec![
                    Card::Normal(Suit::Diamond, Rank::Jack),
                    Card::Normal(Suit::Heart, Rank::Jack),
                    Card::Normal(Suit::Spade, Rank::Jack),
                ],
                true,
            ),
            (
                vec![
                    Card::Normal(Suit::Club, Rank::Seven),
                    Card::Normal(Suit::Diamond, Rank::Seven),
                    Card::Normal(Suit::Heart, Rank::Seven),
                    Card::Normal(Suit::Spade, Rank::Seven),
                ],
                true,
            ),
            (
                vec![
                    Card::Normal(Suit::Club, Rank::Two),
                    Card::Normal(Suit::Diamond, Rank::Two),
                    Card::Normal(Suit::Heart, Rank::Two),
                    Card::Joker,
                ],
                true,
            ),
            (
                vec![
                    Card::Normal(Suit::Diamond, Rank::Queen),
                    Card::Normal(Suit::Spade, Rank::Ace),
                ],
                false,
            ),
            (
                vec![
                    Card::Normal(Suit::Diamond, Rank::Jack),
                    Card::Normal(Suit::Heart, Rank::Six),
                    Card::Joker,
                ],
                false,
            ),
        ] {
            assert_eq!(is_same_ranks(&cards), expected);
        }
    }

    #[test]
    fn test_is_same_suit() {
        for (cards, expected) in [
            (vec![], true),
            (vec![Card::Normal(Suit::Spade, Rank::Five)], true),
            (
                vec![
                    Card::Normal(Suit::Diamond, Rank::Three),
                    Card::Normal(Suit::Diamond, Rank::Ace),
                ],
                true,
            ),
            (
                vec![
                    Card::Normal(Suit::Heart, Rank::Five),
                    Card::Normal(Suit::Heart, Rank::Eight),
                    Card::Normal(Suit::Heart, Rank::Jack),
                ],
                true,
            ),
            (
                vec![
                    Card::Normal(Suit::Club, Rank::Jack),
                    Card::Normal(Suit::Club, Rank::Queen),
                    Card::Normal(Suit::Club, Rank::King),
                    Card::Normal(Suit::Club, Rank::Two),
                ],
                true,
            ),
            (
                vec![
                    Card::Normal(Suit::Diamond, Rank::Four),
                    Card::Normal(Suit::Diamond, Rank::Seven),
                    Card::Normal(Suit::Diamond, Rank::Ten),
                    Card::Joker,
                ],
                true,
            ),
            (
                vec![
                    Card::Normal(Suit::Diamond, Rank::Queen),
                    Card::Normal(Suit::Spade, Rank::Ace),
                ],
                false,
            ),
            (
                vec![
                    Card::Normal(Suit::Diamond, Rank::Jack),
                    Card::Normal(Suit::Heart, Rank::Six),
                    Card::Joker,
                ],
                false,
            ),
        ] {
            assert_eq!(is_same_suits(&cards), expected);
        }
    }

    #[test]
    fn test_is_seq() {
        let cards = [
            Card::Normal(Suit::Club, Rank::Jack),
            Card::Normal(Suit::Club, Rank::Queen),
            Card::Normal(Suit::Club, Rank::King),
            Card::Normal(Suit::Club, Rank::Ace),
            Card::Normal(Suit::Club, Rank::Two),
        ];
        let joker = Card::Joker;
        for (cards, expected) in [
            (vec![cards[0], cards[1], cards[2]], true),
            (vec![cards[2], cards[1], cards[0]], true),
            (vec![cards[0], cards[1], cards[2], cards[3]], true),
            (vec![cards[3], cards[2], cards[1], cards[0]], true),
            (vec![joker, cards[1], cards[2], cards[3]], true),
            (vec![cards[0], joker, cards[2], cards[3]], true),
            (vec![cards[0], cards[1], joker, cards[3]], true),
            (vec![cards[0], cards[1], cards[2], joker], true),
            (vec![joker, cards[2], cards[1], cards[0]], true),
            (vec![cards[3], joker, cards[1], cards[0]], true),
            (vec![cards[3], cards[2], joker, cards[0]], true),
            (vec![cards[3], cards[2], cards[1], joker], true),
            (vec![], false),
            (vec![cards[0]], false),
            (vec![cards[0], cards[1]], false),
            (vec![cards[1], cards[0]], false),
            (vec![cards[0], cards[2], cards[1]], false),
            (vec![cards[1], cards[2], cards[0]], false),
            (vec![cards[1], cards[3], cards[0], cards[2]], false),
            (vec![joker, cards[3], cards[0], cards[2]], false),
            (vec![cards[1], joker, cards[0], cards[2]], false),
            (vec![cards[1], cards[3], joker, cards[2]], false),
            (vec![cards[1], cards[3], cards[0], joker], false),
        ] {
            assert_eq!(is_seq(&cards), expected);
        }
    }
}
