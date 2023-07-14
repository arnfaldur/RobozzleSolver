use crate::constants::NOGRAM;
use crate::game::{instructions::*, Puzzle, Source};
use crate::solver::carlo::{carlo, score};
use crate::solver::pruning::{deny, snip_around};
use std::f64::consts::SQRT_2;
use std::f64::INFINITY;
use std::fmt::{Display, Error, Formatter};
use std::intrinsics::{logf64, sqrtf64};

const EXPLORATION: f64 = SQRT_2;

struct Node {
    pub source: Source,
    scores: f64,
    rollouts: u64,
    children: Vec<Node>,
}

impl Node {
    fn new(source: Source) -> Self {
        Node {
            source,
            scores: 0.0,
            rollouts: 0,
            children: vec![],
        }
    }
    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
    fn uct(&self, parent_visits: u64) -> f64 {
        return if self.rollouts == 0 {
            INFINITY
        } else {
            self.scores / self.rollouts as f64
                + EXPLORATION
                    * unsafe { sqrtf64(logf64(parent_visits as f64) / self.rollouts as f64) }
        };
    }
    fn have_children(&mut self, puzzle: &Puzzle) -> bool {
        let mut state = puzzle.initial_state(&self.source);
        let mut preferred = [true; 5];
        for i in 1..self.source.0.len() {
            for j in (i + 1)..self.source.0.len() {
                if self.source.0[i] == self.source.0[j] {
                    preferred[j] = false;
                }
            }
        }
        let mut running = true;
        while running {
            let ins_pointer = state.ins_pointer();
            let ins = state.current_ins(&self.source);
            let method_index = ins_pointer.get_method_index();
            let ins_index = ins_pointer.get_ins_index();
            let nop_branch = ins.is_nop();
            let probe_branch = ins.is_probe() && state.current_tile().clone().executes(ins);
            let loosening_branch = !ins.is_debug()
                && !ins.is_loosened()
                && !state.current_tile().to_condition().is_cond(ins.get_cond());
            if nop_branch || probe_branch || loosening_branch {
                let mut instructions: Vec<Ins> = if nop_branch {
                    [HALT]
                        .iter()
                        .chain(
                            puzzle
                                .get_ins_set(state.current_tile().to_condition(), false)
                                .iter()
                                .filter(|&ins| !ins.is_function() || preferred[ins.source_index()]),
                        )
                        .chain(
                            puzzle
                                .get_cond_mask()
                                .get_probes(state.current_tile().to_condition())
                                .iter(),
                        )
                        .cloned()
                        .collect()
                } else if probe_branch {
                    puzzle
                        .get_ins_set(state.current_tile().to_condition(), false)
                        .iter()
                        .map(|i| i.as_loosened())
                        .chain(
                            (if self.source[method_index][ins_index]
                                .remove_cond(state.current_tile().to_condition())
                                .is_probe()
                            {
                                vec![self.source[method_index][ins_index]
                                    .remove_cond(state.current_tile().to_condition())]
                            } else {
                                vec![]
                            }),
                        )
                        .collect()
                } else if loosening_branch {
                    vec![ins.as_loosened(), ins.get_ins().as_loosened()]
                } else {
                    vec![]
                };
                for &instruction in instructions.iter().rev() {
                    let mut temp = self.source.clone();
                    temp[method_index][ins_index] = instruction;
                    if !snip_around(puzzle, &temp, *ins_pointer, false)
                        && !deny(puzzle, &self.source, false)
                    {
                        if instruction == HALT {
                            for i in ins_index..puzzle.methods[method_index] {
                                temp[method_index][i] = HALT;
                            }
                        }
                        let mut node = Node::new(puzzle.empty_source());
                        node.source = temp;
                        self.children.push(node);
                    }
                }
                return !self.children.is_empty();
            }
            running = state.step(&self.source, puzzle);
        }
        return false;
    }
}

pub fn monte_carlo(puzzle: &Puzzle) -> Vec<Source> {
    let mut root = Node::new(puzzle.empty_source());
    let mut best = NOGRAM;
    fn recurse(puzzle: &Puzzle, node: &mut Node, best: &mut Source) -> f64 {
        node.rollouts += 1;
        let mut next: &mut Node = &mut Node::new(puzzle.empty_source());
        let mut score = 0.0;
        if node.is_leaf() {
            if node.rollouts > 0 && node.have_children(puzzle) {
                node.children[0].rollouts += 1;
                score = rollout(puzzle, &mut node.children[0]);
                node.children[0].scores += score;
            } else {
                score = rollout(puzzle, node);
                if score > 1.0 {
                    *best = node.source;
                }
            }
        } else {
            let mut maximum = -INFINITY;
            for child in &mut node.children {
                let contender = child.uct(node.rollouts);
                if contender > maximum {
                    maximum = contender;
                    next = child;
                }
            }
            score = recurse(puzzle, next, best);
        }
        node.scores += score;
        return score;
    };
    let mut score = 0.0;
    let mut checks = 0;
    while score < 1.0 {
        score = recurse(puzzle, &mut root, &mut best);
        checks += 1;
        if checks % (1 << 20) == 0 {
            println!("{}", root);
        }
    }
    println!("{}", best);
    vec![best]
}

fn rollout(puzzle: &Puzzle, node: &mut Node) -> f64 {
    let mut candidate = node.source;
    let mut preferred = [true; 5];
    for i in 1..candidate.0.len() {
        for j in (i + 1)..candidate.0.len() {
            if candidate.0[i] == candidate.0[j] {
                preferred[j] = false;
            }
        }
    }
    let mut state = puzzle.initial_state(&node.source);
    let mut running = true;
    while running {
        let ins_pointer = state.ins_pointer();
        let ins = state.current_ins(&candidate);
        let method_index = ins_pointer.get_method_index();
        let ins_index = ins_pointer.get_ins_index();
        let nop_branch = ins.is_nop();
        let probe_branch = ins.is_probe() && state.current_tile().clone().executes(ins);
        let loosening_branch = !ins.is_debug()
            && !ins.is_loosened()
            && !state.current_tile().to_condition().is_cond(ins.get_cond());
        if nop_branch || probe_branch || loosening_branch {
            let mut instructions: Vec<Ins> = if nop_branch {
                [HALT]
                    .iter()
                    .chain(
                        puzzle
                            .get_ins_set(state.current_tile().to_condition(), false)
                            .iter()
                            .filter(|&ins| !ins.is_function() || preferred[ins.source_index()]),
                    )
                    .chain(
                        puzzle
                            .get_cond_mask()
                            .get_probes(state.current_tile().to_condition())
                            .iter(),
                    )
                    .cloned()
                    .collect()
            } else if probe_branch {
                puzzle
                    .get_ins_set(state.current_tile().to_condition(), false)
                    .iter()
                    .map(|i| i.as_loosened())
                    .chain(
                        (if candidate[method_index][ins_index]
                            .remove_cond(state.current_tile().to_condition())
                            .is_probe()
                        {
                            vec![candidate[method_index][ins_index]
                                .remove_cond(state.current_tile().to_condition())]
                        } else {
                            vec![]
                        }),
                    )
                    .collect()
            } else if loosening_branch {
                vec![ins.as_loosened(), ins.get_ins().as_loosened()]
            } else {
                vec![]
            };
            for &instruction in instructions.iter().rev() {
                candidate[method_index][ins_index] = instruction;
                if !snip_around(puzzle, &candidate, *ins_pointer, false)
                    && !deny(puzzle, &candidate, false)
                {
                    if instruction == HALT {
                        for i in ins_index..puzzle.methods[method_index] {
                            candidate[method_index][i] = HALT;
                        }
                    }
                    break;
                }
            }
        }
        running = state.step(&candidate, puzzle);
    }
    return score(&state, puzzle);
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        //        writeln!(f, "score: {}, rollouts: {}, \nnode score: {}, children: {}",
        //                 self.scores, self.rollouts, self.scores / self.rollouts as f64, self.children.len())?;
        writeln!(f, "{}", self.source)?;
        for child in &self.children {
            write!(f, "{}", child)?;
        }
        write!(f, "")
    }
}
