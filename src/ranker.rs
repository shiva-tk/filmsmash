use std::collections::VecDeque;
use std::fmt::Display;
use std::fs::File;
use std::mem;
use std::io::{self, Write};
use colored::*;

pub trait Ranker<T : Display> {
    fn new(to_rank: Vec<T>) -> Self where Self : Sized;
    fn left(&mut self) -> Option<&mut T>;
    fn right(&mut self) -> Option<&mut T>;
    fn lt(&mut self);
    fn gt(&mut self);
    fn is_ranked(&self) -> bool;
    fn print_top_10(&self);
    fn write_ranking(&self) -> io::Result<()>;
}

pub struct MergeSortRanker<T : Display> {
    to_merge: VecDeque<VecDeque<T>>,
    merged: VecDeque<T>
}

impl<T : Display> Ranker<T> for MergeSortRanker<T> {

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

    fn print_top_10(&self) {
        if self.is_ranked() {
            let top_10 = self.to_merge.iter().flatten().take(10);
            for (i, item) in top_10.enumerate() {
                let output = format!("{}. {}", i + 1, item);
                match i {
                    0 => println!("{}", output.bold().yellow()),
                    1 => println!("{}", output.bold().white()),
                    2 => println!("{}", output.bold().bright_black()),
                    _ => println!("{}", output),
                }
            }
        }
    }

    fn write_ranking(&self) -> io::Result<()> {
        let mut file = File::create("ranking.txt")?;
        let ranking = self.to_merge.iter().flatten();
        for item in ranking {
            writeln!(file, "{}", item);
        }

        Ok(())
    }

    fn left(&mut self) -> Option<&mut T> {
        if !self.is_ranked() {
            let to_merge_left = &mut self.to_merge[0];
            let left = &mut to_merge_left[0];
            Some(left)
        } else {
            None
        }
    }

    fn right(&mut self) -> Option<&mut T> {
        if !self.is_ranked() {
            let to_merge_right = &mut self.to_merge[1];
            let right = &mut to_merge_right[0];
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
