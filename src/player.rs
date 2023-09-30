use crate::card::Card;
use crate::comb::Comb;
use crate::validator::Validator;

pub trait Player {
    fn init(&mut self, hands: Vec<Card>);
    fn count_hands(&self) -> usize;
    fn get_name(&self) -> &str;
    fn get_hands(&mut self) -> &mut Vec<Card>;
    fn play(&mut self, validator: &dyn Validator) -> Option<Comb>;
    fn get_needless_cards(&mut self, cards_count: usize) -> Vec<Card>;
}
