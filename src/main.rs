use std::{cmp::Ordering, collections::VecDeque};

use fastrand;
use itertools::*;

/// Represents ants on a string. Each ant has a position, and a direction (up means
/// higher index and down means lower index). `dists` represents the distance from one
/// ant to the next. If there are `N` ants, there are `N - 1` distances.
#[derive(Debug, PartialEq)]
struct Ants {
    pos: VecDeque<f64>,
    dists: Vec<f64>,
    facing_up: VecDeque<bool>,
    time_taken: f64,
}

impl Ants {
    /// Generate `n_ants` on a string, placed randomly
    fn new_rand(n_ants: usize) -> Ants {
        // Generate n_ants between 0.0 and 1.0
        let pos: VecDeque<f64> = std::iter::repeat_with(|| fastrand::f64())
            .take(n_ants)
            .sorted_by(|a, b| a.partial_cmp(b).expect("Received a non-valid ant position"))
            .collect();

        // What are the distances between the ants?
        let dists: Vec<f64> = diff(&pos);

        let facing_up: VecDeque<bool> = std::iter::repeat_with(|| fastrand::bool())
            .take(n_ants)
            .collect();

        Ants {
            pos,
            dists,
            facing_up,
            time_taken: 0.0,
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
            .iter()
            .tuple_windows::<(_, _)>()
            .enumerate()
            // Will only collide if lower is facing up, and upper is facing down
            .filter(|(_, ant_pair)| ant_pair.0 & !ant_pair.1)
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

        panic!("Messed up the if-else logic")
    }

    /// Once the next event has been found using `find_next_event`, this function takes
    /// the step and ensures that the state of `self` is fully up to date.
    /// If taking this step means that all the ants are off the string, return true,
    /// else false
    fn take_step(&mut self, event: Event) -> bool {
        match event {
            Event::AntDropsOffBottom => {
                // If there is one ant left, then taking this step will empty the string
                if self.pos.len() == 1 {
                    // Update time taken
                    self.time_taken += 100.0 * self.pos[0];

                    return true;
                }

                // The distance all ants will move this step
                let d = self.pos[0];

                // Update time spent on this step
                self.time_taken += 100.0 * d;

                // The one on the bottom gets removed.
                self.pos.pop_front();
                self.facing_up.pop_front();

                // The rest move
                self.pos
                    .iter_mut()
                    .zip(&self.facing_up)
                    .for_each(|(p, facing_up)| {
                        if *facing_up {
                            *p += d;
                        } else {
                            *p -= d;
                        }
                    });

                // Recalculate distances
                self.dists = diff(&self.pos);

                // We aren't done, so return false
                return false;
            }
            Event::AntDropsOffTop => {
                // If there is one ant left, then taking this step will empty the string
                if self.pos.len() == 1 {
                    // Update time taken
                    self.time_taken += 100.0 * self.pos[0];

                    return true;
                }

                // The distance all ants will move this step
                let d = 1.0 - self.pos[self.pos.len() - 1];

                // Update time spent on this step
                self.time_taken += 100.0 * d;

                // The one of the top gets removed
                self.pos.pop_back();
                self.facing_up.pop_back();

                // The rest move
                self.pos
                    .iter_mut()
                    .zip(&self.facing_up)
                    .for_each(|(p, facing_up)| {
                        if *facing_up {
                            *p += d;
                        } else {
                            *p -= d;
                        }
                    });

                // Recalculate distances
                self.dists = diff(&self.pos);

                // We aren't done, so return false
                return false;
            }
            Event::AntsCollide { idx_of_first } => {
                // The distance all ants will move this step
                let d = (self.pos[idx_of_first + 1] - self.pos[idx_of_first]) / 2.0;

                // Update time spent on this step
                self.time_taken += 100.0 * d;

                // All ants move
                self.pos
                    .iter_mut()
                    .zip(&self.facing_up)
                    .for_each(|(p, facing_up)| {
                        if *facing_up {
                            *p += d;
                        } else {
                            *p -= d;
                        }
                    });

                // Ants that collide swap directions
                self.facing_up[idx_of_first] = !self.facing_up[idx_of_first];
                self.facing_up[idx_of_first + 1] = !self.facing_up[idx_of_first + 1];

                // Recalculate distances
                self.dists = diff(&self.pos);

                // We aren't done, so return false
                return false;
            }
        }
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

fn diff(x: &VecDeque<f64>) -> Vec<f64> {
    if x.len() == 0 {
        return vec![];
    }

    let mut res = vec![0.0; x.len() - 1];
    for idx in 1..x.len() {
        res[idx - 1] = x[idx] - x[idx - 1];
    }
    res
}

fn main() {
    let ants = Ants::new_rand(10_000);
    let next = ants.find_next_event();

    println!("{:?}", next);
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    impl Ants {
        fn from_positions(pos: Vec<f64>, facing_up: Vec<bool>) -> Ants {
            // This is just a helper function for testing. Should usually be run with small
            // number of ants, so just clone it
            let p: VecDeque<f64> = pos.into();
            let dists: Vec<f64> = diff(&p);
            Ants {
                pos: p,
                dists,
                facing_up: facing_up.into(),
                time_taken: 0.0,
            }
        }
    }

    #[test]
    fn test_find_next_event_btm() {
        let p1 = 0.01;
        let p2 = 0.5;
        let ants = Ants::from_positions(vec![p1, p2], vec![false, true]);

        let got = ants.find_next_event();
        let want = Event::AntDropsOffBottom;
        assert_eq!(want, got);
    }

    #[test]
    fn test_find_next_event_top() {
        let p1 = 0.25;
        let p2 = 0.99;
        let ants = Ants::from_positions(vec![p1, p2], vec![false, true]);

        let got = ants.find_next_event();
        let want = Event::AntDropsOffTop;
        assert_eq!(want, got);
    }

    #[test]
    fn test_find_next_event_collide() {
        let p1 = 0.50;
        let p2 = 0.52;
        let ants = Ants::from_positions(vec![p1, p2], vec![true, false]);

        let got = ants.find_next_event();
        let want = Event::AntsCollide { idx_of_first: 0 };
        assert_eq!(want, got);
    }

    #[test]
    fn test_find_next_event_ants_close_but_no_collide() {
        let p1 = 0.50;
        let p2 = 0.52;
        let ants = Ants::from_positions(vec![p1, p2], vec![false, false]);

        let got = ants.find_next_event();
        let want = Event::AntDropsOffBottom;
        assert_eq!(want, got);
    }

    #[test]
    fn test_find_next_event_top_and_btm_eql() {
        // When the distances are equal, top should go over first
        let p1 = 0.25;
        let p2 = 0.75;
        let ants = Ants::from_positions(vec![p1, p2], vec![false, true]);

        let got = ants.find_next_event();
        let want = Event::AntDropsOffTop;
        assert_eq!(want, got);
    }

    #[test]
    fn test_3_ants() {
        let p1 = 0.1;
        let p2 = 0.2;
        let p3 = 0.9;
        let ants = Ants::from_positions(vec![p1, p2, p3], vec![false, false, false]);

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
        let ants = Ants::from_positions(vec![p1, p2, p3], vec![true, false, true]);

        let got = ants.find_next_event();
        let want = Event::AntsCollide { idx_of_first: 0 };
        assert_eq!(want, got);
    }

    #[test]
    fn test_next_step_easy_off_bottom() {
        // This tests for correct state update after an ant drops off the bottom
        let p1 = 0.01;
        let p2 = 0.5;
        let mut ants = Ants::from_positions(vec![p1, p2], vec![false, true]);

        let event = ants.find_next_event();
        ants.take_step(event);

        let want_pos = vec![p2 + p1];
        let want_dists = vec![];
        let want_facing = vec![true];
        let want_ants = Ants {
            pos: want_pos.into(),
            dists: want_dists,
            facing_up: want_facing.into(),
            time_taken: p1 * 100.0,
        };
        assert_eq!(want_ants, ants);
    }

    #[test]
    fn test_next_step_easy_off_top() {
        // This tests for correct state update after an ant drops off the bottom
        let p1 = 0.01;
        let p2 = 0.95;
        let mut ants = Ants::from_positions(vec![p1, p2], vec![true, true]);

        let event = ants.find_next_event();
        ants.take_step(event);

        let want_pos = vec![p1 + (1.0 - p2)];
        let want_dists = vec![];
        let want_facing = vec![true];
        let want_ants = Ants {
            pos: want_pos.into(),
            dists: want_dists,
            facing_up: want_facing.into(),
            time_taken: (1.0 - p2) * 100.0,
        };
        assert_eq!(want_ants, ants);
    }

    #[test]
    fn test_next_step_easy_collide() {
        // This tests for correct state update after an ant drops off the bottom
        let p1 = 0.10;
        let p2 = 0.20;
        let mut ants = Ants::from_positions(vec![p1, p2], vec![true, false]);

        let event = ants.find_next_event();
        ants.take_step(event);

        let d = (p2 - p1) / 2.0;

        let want_pos = vec![p1 + d, p2 - d];
        let want_dists = vec![(p2 - d) - (p1 + d)];
        let want_facing = vec![false, true];
        let want_ants = Ants {
            pos: want_pos.into(),
            dists: want_dists,
            facing_up: want_facing.into(),
            time_taken: d * 100.0,
        };
        assert_eq!(want_ants, ants);
    }
}
