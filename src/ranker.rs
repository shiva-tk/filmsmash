use std::collections::VecDeque;
use std::mem;

pub trait Ranker<T> {
    fn new(to_rank: Vec<T>) -> Self;
    fn left(&self) -> Option<&T>;
    fn right(&self) -> Option<&T>;
    fn lt(&mut self);
    fn gt(&mut self);
    fn is_ranked(&self) -> bool;
    fn into_ranking(self) -> Option<Vec<T>>;
}

pub struct MergeSortRanker<T> {
    to_merge: VecDeque<VecDeque<T>>,
    merged: VecDeque<T>
}

impl<T> Ranker<T> for MergeSortRanker<T> {

    fn new(to_rank: Vec<T>) -> MergeSortRanker<T> {
        let mut to_merge: VecDeque<VecDeque<T>> = VecDeque::new();

        for item in to_rank {
            let mut single = VecDeque::new();
            single.push_back(item);
            to_merge.push_back(single);
        }

        MergeSortRanker {
            to_merge,
            merged: VecDeque::new(),
        }
    }

    fn is_ranked(&self) -> bool {
        self.to_merge.len() <= 1
    }

    fn into_ranking(self) -> Option<Vec<T>> {
        if self.is_ranked() {
            Some(self.to_merge.into_iter().flatten().collect())
        } else {
            None
        }
    }

    fn left(&self) -> Option<&T> {
        if !self.is_ranked() {
            let to_merge_left = &self.to_merge[0];
            let left = &to_merge_left[0];
            Some(left)
        } else {
            None
        }
    }

    fn right(&self) -> Option<&T> {
        if !self.is_ranked() {
            let to_merge_right = &self.to_merge[1];
            let right = &to_merge_right[0];
            Some(right)
        } else {
            None
        }
    }

    fn lt(&mut self) {
        if !self.is_ranked() {
            let to_merge_right = &mut self.to_merge[1];
            let right = to_merge_right.pop_front().expect("MergeSortRanker broke invariant");
            self.merged.push_back(right);

            if to_merge_right.is_empty() {
                let to_merge_left = &mut self.to_merge[0];
                self.merged.append(to_merge_left);
                self.to_merge.pop_front();
                self.to_merge.pop_front();
                self.to_merge.push_back(mem::take(&mut self.merged));
            }
        }
    }

    fn gt(&mut self) {
        if !self.is_ranked() {
            let to_merge_left = &mut self.to_merge[0];
            let left = to_merge_left.pop_front().expect("MergeSortRanker broke invariant");
            self.merged.push_back(left);

            if to_merge_left.is_empty() {
                let to_merge_right = &mut self.to_merge[1];
                self.merged.append(to_merge_right);
                self.to_merge.pop_front();
                self.to_merge.pop_front();
                self.to_merge.push_back(mem::take(&mut self.merged));
            }
        }
    }

}
