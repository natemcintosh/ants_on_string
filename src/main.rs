use std::cmp::Ordering;

use fastrand;
use itertools::*;

/// Represents ants on a string. Each ant has a position, and a direction (up means
/// higher index and down means lower index). `dists` represents the distance from one
/// ant to the next. If there are `N` ants, there are `N - 1` distances.
#[derive(Debug)]
struct Ants {
    pos: Vec<f64>,
    dists: Vec<f64>,
    facing_up: Vec<bool>,
}

impl Ants {
    fn new_rand(n_ants: usize) -> Ants {
        // Generate n_ants between 0.0 and 1.0
        let pos: Vec<f64> = std::iter::repeat_with(|| fastrand::f64())
            .take(n_ants)
            .sorted_by(|a, b| a.partial_cmp(b).expect("Received a non-valid ant position"))
            .collect();

        // What are the distances between the ants?
        let dists: Vec<f64> = pos.windows(2).map(|pair| pair[1] - pair[0]).collect();

        let facing_up: Vec<bool> = std::iter::repeat_with(|| fastrand::bool())
            .take(n_ants)
            .collect();

        Ants {
            pos,
            dists,
            facing_up,
        }
    }

    /// Look for what event happens next:
    /// - ant steps off low end (low index)
    /// - ant steps off high end (high index)
    /// - ant collides with another ant
    fn find_next_event(&self) -> Event {
        // Will an ant step off low end, and if so, when?
        let mut btm_dist = f64::MAX;
        if !self.facing_up[0] {
            btm_dist = self.pos[0];
        }

        let mut top_dist = f64::MAX;
        if self.facing_up[self.facing_up.len() - 1] {
            top_dist = 1.0 - self.pos[self.pos.len() - 1];
        }

        // Find the places where ants are headed towards eachother. These are
        // the indices of an ant at `i` and `i+1` that are going in opposite
        // directions
        let facing_eachother: Vec<usize> = self
            .facing_up
            .windows(2)
            .enumerate()
            // Will only collide if lower is facing up, and upper is facing down
            .filter(|(_, ant_pair)| ant_pair[0] & !ant_pair[1])
            .map(|(idx, _)| idx)
            .collect();

        // Then filter down to just those distances
        let relevant_dists: Vec<f64> = facing_eachother
            .iter()
            .map(|idx| self.dists[*idx] / 2.0)
            .collect();
        let mut ant_dist = f64::MAX;
        let mut idx = 0;
        if !relevant_dists.is_empty() {
            (ant_dist, idx) = min_and_pos(&relevant_dists);
        }

        // Figure out which distance is shortest
        if (top_dist <= btm_dist) & (top_dist <= ant_dist) {
            return Event::AntDropsOffTop;
        } else if (btm_dist <= top_dist) & (btm_dist <= ant_dist) {
            return Event::AntDropsOffBottom;
        } else if (ant_dist <= top_dist) & (ant_dist <= btm_dist) {
            return Event::AntsCollide { idx_of_first: idx };
        }

        unreachable!("Messed up the if-else logic")
    }
}

#[derive(Debug, PartialEq)]
enum Event {
    AntDropsOffBottom,
    AntDropsOffTop,
    AntsCollide { idx_of_first: usize },
}

/// Find both the minimum value and the position of the minimum value.
/// If they are all the same, the value and position of item 0 will be returned
fn min_and_pos(v: &[f64]) -> (f64, usize) {
    let mut pos: usize = 0;
    let mut min: f64 = v[0];

    for (idx, x) in v.iter().enumerate() {
        if x.total_cmp(&min) == Ordering::Less {
            pos = idx;
            min = *x;
        }
    }

    (min, pos)
}

fn main() {
    let ants = Ants::new_rand(10);

    println!("{:?}", ants);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_next_event_btm() {
        let p1 = 0.01;
        let p2 = 0.5;
        let ants = Ants {
            pos: vec![p1, p2],
            dists: vec![p2 - p1],
            facing_up: vec![false, true],
        };

        let got = ants.find_next_event();
        let want = Event::AntDropsOffBottom;
        assert_eq!(want, got);
    }

    #[test]
    fn test_find_next_event_top() {
        let p1 = 0.25;
        let p2 = 0.99;
        let ants = Ants {
            pos: vec![p1, p2],
            dists: vec![p2 - p1],
            facing_up: vec![false, true],
        };

        let got = ants.find_next_event();
        let want = Event::AntDropsOffTop;
        assert_eq!(want, got);
    }

    #[test]
    fn test_find_next_event_collide() {
        let p1 = 0.50;
        let p2 = 0.52;
        let ants = Ants {
            pos: vec![p1, p2],
            dists: vec![p2 - p1],
            facing_up: vec![true, false],
        };

        let got = ants.find_next_event();
        let want = Event::AntsCollide { idx_of_first: 0 };
        assert_eq!(want, got);
    }

    #[test]
    fn test_find_next_event_ants_close_but_no_collide() {
        let p1 = 0.50;
        let p2 = 0.52;
        let ants = Ants {
            pos: vec![p1, p2],
            dists: vec![p2 - p1],
            facing_up: vec![false, false],
        };

        let got = ants.find_next_event();
        let want = Event::AntDropsOffBottom;
        assert_eq!(want, got);
    }

    #[test]
    fn test_find_next_event_top_and_btm_eql() {
        // When the distances are equal, top should go over first
        let p1 = 0.25;
        let p2 = 0.75;
        let ants = Ants {
            pos: vec![p1, p2],
            dists: vec![p2 - p1],
            facing_up: vec![false, true],
        };

        let got = ants.find_next_event();
        let want = Event::AntDropsOffTop;
        assert_eq!(want, got);
    }

    #[test]
    fn test_3_ants() {
        let p1 = 0.1;
        let p2 = 0.2;
        let p3 = 0.9;
        let ants = Ants {
            pos: vec![p1, p2, p3],
            dists: vec![p2 - p1, p3 - p2],
            facing_up: vec![false, false, false],
        };

        let got = ants.find_next_event();
        let want = Event::AntDropsOffBottom;
        assert_eq!(want, got);
    }

    #[test]
    fn test_3_ants_collision() {
        // In this case, ants 1 and 2 should collide before 3 makes it off the top
        // In particular, this tests the case that two ants facing eachother that
        // are a certain distance apart will collide before an ant the same distance
        // from an end will get off the end.
        let p1 = 0.1;
        let p2 = 0.2;
        let p3 = 0.9;
        let ants = Ants {
            pos: vec![p1, p2, p3],
            dists: vec![p2 - p1, p3 - p2],
            facing_up: vec![true, false, true],
        };

        let got = ants.find_next_event();
        let want = Event::AntsCollide { idx_of_first: 0 };
        assert_eq!(want, got);
    }
}
