use crate::field::Flags;
use card::cmp_order;
use card::Card;
use comb::Comb;
use core::time;
use field::Field;
use input::get_input;
use itertools::Itertools;
use npc::MinNpc;
use pc::Pc;
use player::Player;
use rand::seq::SliceRandom;
use std::thread;
mod card;
mod comb;
mod field;
mod indexer;
mod input;
mod npc;
mod pc;
mod player;
mod suit_binder;
mod validator;

const PLAYERS_COUNT: usize = 4;

fn get_split_deck() -> Vec<Vec<Card>> {
    let mut deck = card::create_deck();
    deck.shuffle(&mut rand::thread_rng());
    let d1 = deck.split_off(deck.len() - 13);
    let d2 = deck.split_off(deck.len() - 13);
    let d3 = deck.split_off(deck.len() - 13);
    let mut hands = vec![d1, d2, d3, deck];
    hands.iter_mut().for_each(|d| d.sort_by(cmp_order));
    hands
}

fn create_players() -> Vec<Box<dyn Player>> {
    let mut players: Vec<Box<dyn Player>> = vec![
        Box::new(Pc::new("User".to_owned())),
        Box::new(MinNpc::new("NpcA".to_owned())),
        Box::new(MinNpc::new("NpcB".to_owned())),
        Box::new(MinNpc::new("NpcC".to_owned())),
    ];
    players
        .iter_mut()
        .zip(get_split_deck())
        .for_each(|(player, hands)| player.init(hands));
    players.shuffle(&mut rand::thread_rng());
    players
}

fn print_comb(comb: &Comb) -> String {
    match comb {
        Comb::Single(card) => String::from(card),
        Comb::Multi(cards) | Comb::Seq(cards) => cards.iter().map(String::from).join(" "),
    }
}

fn exchange_cards(
    players: &mut [Box<dyn Player>],
    winner_idx: usize,
    loser_idx: usize,
    cards_count: usize,
) {
    let needless_cards = players[winner_idx].get_needless_cards(cards_count);
    let max_cards: Vec<Card> = (0..cards_count)
        .filter_map(|_| players[loser_idx].get_hands().pop())
        .collect();
    players[winner_idx].get_hands().extend(max_cards);
    players[winner_idx].get_hands().sort_by(cmp_order);
    players[loser_idx].get_hands().extend(needless_cards);
    players[loser_idx].get_hands().sort_by(cmp_order);
}

fn main() {
    let mut players = create_players();
    let mut field = Field::new(PLAYERS_COUNT, 0);
    let duration = time::Duration::from_millis(300);
    loop {
        while field.count_active_players() > 0 {
            let idx = field.get_idx();
            // 場に出すカードを取得
            let played_comb = players[idx].play(&field);
            let hands_count = players[idx].count_hands();
            let c = match &played_comb {
                Some(comb) => print_comb(comb),
                None => "パス".to_owned(),
            };
            println!("{} [{:2}]: {}", players[idx].get_name(), hands_count, c);
            // カードを場に出すかパス
            let flags = field.put(played_comb, hands_count);
            if flags.contains(Flags::EIGHT) {
                println!("8切り");
            }
            if flags.contains(Flags::BIND) {
                println!("縛り");
            }
            if flags.contains(Flags::REV) {
                println!("カードの強さが逆転");
                // 全プレイヤーの手札をソート
                players
                    .iter_mut()
                    .for_each(|player| player.get_hands().sort_by(field.get_order_comparator()));
            }
            if flags.contains(Flags::OUT) {
                println!("{} 上がり", players[idx].get_name());
            }
            if flags.contains(Flags::LOSE) {
                println!("{} 反則上がり", players[idx].get_name());
            }
            thread::sleep(duration);
        }
        println!("結果発表");
        let player_rank = field.get_player_rank();
        for (i, idx) in player_rank.iter().enumerate() {
            println!("{}位: {}", i + 1, players[*idx].get_name());
        }
        if get_input("もう一度遊びますか? (y/n): ".to_string()) != "y" {
            break;
        }
        // 新しいカードを配る
        get_split_deck()
            .into_iter()
            .zip(players.iter_mut())
            .for_each(|(hands, player)| player.init(hands));
        // カードを交換
        exchange_cards(&mut players, player_rank[0], player_rank[3], 2);
        exchange_cards(&mut players, player_rank[1], player_rank[2], 1);
        println!("強いカードと不要なカードを交換");
        // フィールドをリセット、大貧民のプレイヤーから開始
        field = Field::new(PLAYERS_COUNT, player_rank[3]);
    }
}
