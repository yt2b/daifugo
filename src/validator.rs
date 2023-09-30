use crate::comb::Comb;

pub trait Validator {
    fn get_prev_comb(&self) -> Option<&Comb>;
    fn is_valid(&self, comb: &Comb) -> bool;
}
