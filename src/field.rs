use crate::card::{cmp_order, cmp_order_reversely, cmp_rank, cmp_rank_reversely, Card, Rank};
use crate::comb::Comb;
use crate::indexer::Indexer;
use crate::suit_binder::SuitBinder;
use crate::validator::Validator;
use bitflags::bitflags;
use std::cmp::Ordering;

bitflags! {
    pub struct Flags: u32 {
        const BIND  =  0b00000001;
        const EIGHT =  0b00000010;
        const REV   =  0b00000100;
        const OUT   =  0b00001000;
        const LOSE  =  0b00010000;
    }
}

pub struct Field {
    prev_comb: Option<Comb>,
    indexer: Indexer,
    binder: SuitBinder,
    pass_counter: usize,
    is_rev: bool,
}

impl Field {
    pub fn new(players_count: usize, start_idx: usize) -> Self {
        Self {
            prev_comb: None,
            indexer: Indexer::new(players_count, start_idx),
            binder: SuitBinder::new(),
            pass_counter: 0,
            is_rev: false,
        }
    }

    pub fn get_idx(&self) -> usize {
        self.indexer.get_idx()
    }

    pub fn get_player_rank(&self) -> Vec<usize> {
        self.indexer.get_player_rank()
    }

    pub fn count_active_players(&self) -> usize {
        self.indexer.count_active_players()
    }

    pub fn put(&mut self, new_comb: Option<Comb>, hands_count: usize) -> Flags {
        let mut flags = Flags::empty();
        match new_comb {
            Some(comb) => {
                self.pass_counter = self.indexer.count_active_players() - 1;
                let eight_flag = contains_eight(&comb);
                if hands_count > 0 {
                    if eight_flag {
                        // 8切り
                        flags.insert(Flags::EIGHT);
                        self.binder.clear();
                    } else {
                        // 次のプレイヤーのターンに移る
                        self.indexer.next();
                    }
                } else if contains_especial_card(&comb, self.is_rev) {
                    // 反則上がり
                    self.indexer.set_rank_back();
                    flags.insert(Flags::LOSE);
                } else {
                    // 上がり
                    self.indexer.set_rank_front();
                    flags.insert(Flags::OUT);
                }
                if !eight_flag && !self.binder.is_activate() && self.binder.push(&comb) {
                    flags.insert(Flags::BIND);
                }
                if is_rev_comb(&comb) {
                    // カードの強さが逆転する
                    self.is_rev = !self.is_rev;
                    flags.insert(Flags::REV);
                }
                // 8を含むなら場を流す
                self.prev_comb = if eight_flag { None } else { Some(comb) }
            }
            None => {
                // カウントが0なら場を流す
                self.pass_counter -= 1;
                if self.pass_counter == 0 {
                    self.prev_comb = None;
                    self.binder.clear();
                }
                self.indexer.next();
            }
        }
        flags
    }

    pub fn get_order_comparator(&self) -> impl Fn(&Card, &Card) -> Ordering {
        match self.is_rev {
            true => cmp_order_reversely,
            false => cmp_order,
        }
    }
}

impl Validator for Field {
    fn get_prev_comb(&self) -> Option<&Comb> {
        self.prev_comb.as_ref()
    }

    fn is_valid(&self, comb: &Comb) -> bool {
        match &self.prev_comb {
            Some(prev_comb) => {
                let comparator = match self.is_rev {
                    true => cmp_rank_reversely,
                    false => cmp_rank,
                };
                self.binder.is_valid(comb) && comb.is_greater(prev_comb, comparator)
            }
            None => true,
        }
    }
}

fn get_rank(cards: &[Card]) -> Option<&Rank> {
    cards.iter().find_map(|card| match card {
        Card::Normal(_, r) => Some(r),
        Card::Joker => None,
    })
}

fn contains_eight(comb: &Comb) -> bool {
    // 組み合わせに8のカードを含むか
    // 階段の場合は無視する
    match comb {
        Comb::Single(Card::Normal(_, Rank::Eight)) => true,
        Comb::Multi(cards) => matches!(get_rank(cards), Some(&Rank::Eight)),
        _ => false,
    }
}

fn contains_especial_card(comb: &Comb, is_rev: bool) -> bool {
    let especial_ranks = if is_rev {
        &[Rank::Eight, Rank::Three]
    } else {
        &[Rank::Eight, Rank::Two]
    };
    match comb {
        Comb::Single(card) => match card {
            Card::Normal(_, r) => especial_ranks.contains(r),
            Card::Joker => true,
        },
        Comb::Multi(cards) => match get_rank(cards) {
            Some(r) => especial_ranks.contains(r),
            None => false,
        },
        _ => false,
    }
}

fn is_rev_comb(comb: &Comb) -> bool {
    match comb {
        Comb::Multi(cards) => cards.len() >= 4,
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::card::{Card, Rank, Suit};

    #[test]
    fn test_is_valid_single() {
        let comb = Comb::Single(Card::Normal(Suit::Heart, Rank::Eight));
        let mut field = Field::new(4, 0);
        field.prev_comb = Some(comb.clone());
        let mut field_rev = Field::new(4, 0);
        field_rev.is_rev = true;
        field.prev_comb = Some(comb.clone());
        for (c, expected) in [
            (Card::Normal(Suit::Diamond, Rank::Three), false),
            (Card::Normal(Suit::Club, Rank::Eight), false),
            (Card::Normal(Suit::Spade, Rank::Jack), true),
            (Card::Joker, true),
        ] {
            assert_eq!(field.is_valid(&Comb::Single(c)), expected);
        }
    }

    #[test]
    fn test_is_valid_multi() {
        let comb = Comb::try_from(vec![
            Card::Normal(Suit::Heart, Rank::Eight),
            Card::Normal(Suit::Club, Rank::Eight),
        ])
        .unwrap();
        let mut field = Field::new(4, 0);
        field.prev_comb = Some(comb);
        for (cards, expected) in [
            (
                vec![
                    Card::Normal(Suit::Spade, Rank::Three),
                    Card::Normal(Suit::Diamond, Rank::Three),
                ],
                false,
            ),
            (
                vec![
                    Card::Normal(Suit::Heart, Rank::Eight),
                    Card::Normal(Suit::Club, Rank::Eight),
                ],
                false,
            ),
            (
                vec![
                    Card::Normal(Suit::Spade, Rank::Jack),
                    Card::Normal(Suit::Heart, Rank::Jack),
                ],
                true,
            ),
            (
                vec![Card::Normal(Suit::Spade, Rank::Ace), Card::Joker],
                true,
            ),
        ] {
            let comb = Comb::try_from(cards).unwrap();
            assert_eq!(field.is_valid(&comb), expected);
        }
    }

    #[test]
    fn test_contains_eight() {
        for (comb, expected) in [
            (Comb::Single(Card::Normal(Suit::Club, Rank::Three)), false),
            (Comb::Single(Card::Normal(Suit::Club, Rank::Eight)), true),
            (Comb::Single(Card::Joker), false),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Four),
                    Card::Normal(Suit::Heart, Rank::Four),
                ]),
                false,
            ),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Eight),
                    Card::Normal(Suit::Heart, Rank::Eight),
                ]),
                true,
            ),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Eight),
                    Card::Normal(Suit::Heart, Rank::Eight),
                    Card::Joker,
                ]),
                true,
            ),
        ] {
            assert_eq!(contains_eight(&comb), expected);
        }
    }

    #[test]
    fn test_contains_especial_card() {
        for (comb, is_rev, expected) in [
            (
                Comb::Single(Card::Normal(Suit::Spade, Rank::Three)),
                false,
                false,
            ),
            (
                Comb::Single(Card::Normal(Suit::Spade, Rank::Eight)),
                false,
                true,
            ),
            (
                Comb::Single(Card::Normal(Suit::Spade, Rank::Two)),
                false,
                true,
            ),
            (Comb::Single(Card::Joker), false, true),
            (
                Comb::Single(Card::Normal(Suit::Spade, Rank::Three)),
                true,
                true,
            ),
            (
                Comb::Single(Card::Normal(Suit::Spade, Rank::Eight)),
                true,
                true,
            ),
            (
                Comb::Single(Card::Normal(Suit::Spade, Rank::Two)),
                true,
                false,
            ),
            (Comb::Single(Card::Joker), true, true),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Heart, Rank::Three),
                    Card::Normal(Suit::Spade, Rank::Three),
                ]),
                false,
                false,
            ),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Heart, Rank::Eight),
                    Card::Normal(Suit::Spade, Rank::Eight),
                ]),
                false,
                true,
            ),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Heart, Rank::Two),
                    Card::Normal(Suit::Spade, Rank::Two),
                ]),
                false,
                true,
            ),
            (
                Comb::Multi(vec![Card::Normal(Suit::Heart, Rank::Five), Card::Joker]),
                false,
                false,
            ),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Heart, Rank::Three),
                    Card::Normal(Suit::Spade, Rank::Three),
                ]),
                true,
                true,
            ),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Heart, Rank::Eight),
                    Card::Normal(Suit::Spade, Rank::Eight),
                ]),
                true,
                true,
            ),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Heart, Rank::Two),
                    Card::Normal(Suit::Spade, Rank::Two),
                ]),
                true,
                false,
            ),
            (
                Comb::Multi(vec![Card::Normal(Suit::Heart, Rank::Five), Card::Joker]),
                true,
                false,
            ),
        ] {
            assert_eq!(contains_especial_card(&comb, is_rev), expected);
        }
    }

    #[test]
    fn test_is_rev_comb() {
        for (comb, expected) in [
            (Comb::Single(Card::Normal(Suit::Spade, Rank::Three)), false),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Diamond, Rank::Four),
                    Card::Normal(Suit::Spade, Rank::Four),
                ]),
                false,
            ),
            (
                Comb::Multi(vec![
                    Card::Normal(Suit::Club, Rank::Five),
                    Card::Normal(Suit::Diamond, Rank::Five),
                    Card::Normal(Suit::Heart, Rank::Five),
                    Card::Normal(Suit::Spade, Rank::Five),
                ]),
                true,
            ),
            (
                Comb::Seq(vec![
                    Card::Normal(Suit::Club, Rank::Three),
                    Card::Normal(Suit::Club, Rank::Four),
                    Card::Normal(Suit::Club, Rank::Five),
                ]),
                false,
            ),
        ] {
            assert_eq!(is_rev_comb(&comb), expected);
        }
    }
}
