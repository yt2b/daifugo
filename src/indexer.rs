pub struct Indexer {
    idx: usize,
    active_players: Vec<usize>,
    player_rank: Vec<Option<usize>>,
}

impl Indexer {
    pub fn new(players_count: usize, idx: usize) -> Self {
        Self {
            idx,
            active_players: (0..players_count).collect(),
            player_rank: (0..players_count).map(|_| None).collect(),
        }
    }

    pub fn get_idx(&self) -> usize {
        self.active_players[self.idx]
    }

    pub fn count_active_players(&self) -> usize {
        self.active_players.len()
    }

    pub fn get_player_rank(&self) -> Vec<usize> {
        self.player_rank.iter().filter_map(|p| *p).collect()
    }

    pub fn next(&mut self) {
        self.idx = (self.idx + 1) % self.active_players.len();
    }

    pub fn set_player_rank(&mut self, player: usize) {
        if let Some((i, _)) = self
            .player_rank
            .iter()
            .enumerate()
            .find(|(_, player)| player.is_none())
        {
            self.player_rank[i] = Some(player);
        }
    }

    pub fn set_rank_front(&mut self) {
        // 現在のプレイヤーをアクティブリストから除く
        let player = self.active_players.remove(self.idx);
        self.set_player_rank(player);
        // インデックスがアクティブリストの範囲内になるように調整
        self.idx = if self.idx > self.active_players.len() - 1 {
            0
        } else {
            self.idx
        };
        // 残りのプレイヤーが1人なら最下位に追加
        if self.active_players.len() == 1 {
            let player = self.active_players.remove(0);
            self.set_player_rank(player);
        }
    }

    pub fn set_rank_back(&mut self) {
        // 現在のプレイヤーをアクティブリストから除く
        let player = self.active_players.remove(self.idx);
        if let Some((i, _)) = self
            .player_rank
            .iter()
            .enumerate()
            .rev()
            .find(|(_, player)| player.is_none())
        {
            self.player_rank[i] = Some(player);
        }
        // インデックスがアクティブリストの範囲内になるように調整
        self.idx = if self.idx > self.active_players.len() - 1 {
            0
        } else {
            self.idx
        };
        // 残りのプレイヤーが1人なら最下位に追加
        if self.active_players.len() == 1 {
            let player = self.active_players.remove(0);
            self.set_player_rank(player);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_next() {
        let mut indexer = Indexer::new(4, 2);
        for expected in [3, 0, 1, 2] {
            indexer.next();
            assert_eq!(indexer.idx, expected);
        }
    }

    #[test]
    fn test_set_rank_front() {
        let mut indexer = Indexer::new(4, 0);
        for _ in 0..3 {
            indexer.set_rank_front();
        }
        assert_eq!(
            indexer.player_rank,
            vec![Some(0), Some(1), Some(2), Some(3)]
        );
    }

    #[test]
    fn test_set_rank_back() {
        let mut indexer = Indexer::new(4, 0);
        for _ in 0..3 {
            indexer.set_rank_back();
        }
        assert_eq!(
            indexer.player_rank,
            vec![Some(3), Some(2), Some(1), Some(0)]
        );
    }
}
