use std::cmp::{max, Ordering};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering as SyncOrdering};
use std::thread::spawn;
use std::time::{Duration, Instant};

use crossbeam_channel::{unbounded, Receiver, Sender};

use super::pruning::*;
use crate::constants::*;
use crate::game::instructions::*;
use crate::game::{Puzzle, Source, State};
use crate::web::encode_program;

const BACKTRACK_STACK_SIZE: usize = 2200;
// 44 * 50
static MAX_INS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
pub struct Frame {
    pub candidate: Source,
    pub state: State,
    pub max_steps: usize,
    pub max_touches: usize,
    pub max_instructions: usize,
}

impl Frame {
    fn new(puzzle: &Puzzle) -> Frame {
        Frame {
            candidate: puzzle.empty_source(),
            state: puzzle.initial_state(&NOGRAM),
            max_steps: usize::MAX,
            max_touches: usize::MAX,
            max_instructions: usize::MAX,
        }
    }
}

pub fn backtrack(puzzle: Puzzle, timeout: Option<u128>) -> Vec<(usize, Source)> {
    //let mut max_instructions = puzzle.methods.iter().sum();
    let mut max_steps = 8;
    let mut max_touches = 400;
    let mut max_instructions = 3;

    let mut last_duration = Duration::ZERO;
    //let mut costs = [Duration::ZERO, Duration::ZERO, Duration::ZERO];
    let mut costs: [f64; 3] = [0.0, 0.0, f64::MAX];
    let mut pick = 0;

    let now = Instant::now();

    let mut last_outer_steps = 0;
    let mut solved = false;
    let mut result: Vec<(usize, Source)> = vec![];
    'outer: for i in 0..50 {
        if max_instructions >= puzzle.methods.iter().sum() {
            costs[0] = f64::MAX;
        }

        let (pock, pickers_steps) = costs
            .iter()
            .enumerate()
            .fold((0, f64::MAX), |(mini, min), (i, &steps)| {
                (if steps < min { (i, steps) } else { (mini, min) })
            });
        pick = pock;
        if pick == 0 {
            //max_instructions = (max_instructions + 1).min(7);
            max_instructions += 1;
        } else if pick == 1 {
            max_steps = (max_steps as f64 * 1.61803398875).ceil() as usize;
        } else if pick == 2 {
            max_touches += 1;
        }
        // println!(
        //     "increased {} to {}, dur: {}",
        //     ["max_instructions", "max_steps", "max_touches"][pick],
        //     [max_instructions, max_steps, max_touches][pick],
        //     pickers_steps,
        // );

        let now = Instant::now();
        let mut outer_frame = Frame::new(&puzzle);
        outer_frame.max_steps = max_steps;
        outer_frame.max_instructions = max_instructions;
        outer_frame.candidate.shade(max_instructions);

        let mut candidates = VecDeque::new();
        let mut considered: u64 = 0;
        if result.len() == 0 {
            candidates.push_back(outer_frame);
        }
        let mut outer_steps = 0;
        while let Some(mut frame) = candidates.pop_back() {
            let candidate = frame.candidate;
            let steps = frame.state.steps;

            frame.max_steps = frame.max_steps.min(max_steps);

            considered += 1;
            let (is_solution, after_steps) = search(&puzzle, &mut frame, &mut candidates);
            outer_steps += (after_steps);

            if is_solution {
                let mut solution = candidate.clone();
                solution.sanitize();
                result.push((frame.state.steps, solution));
                max_steps = max_steps.min(steps);
                solved = true;
                coz::progress!("backtrack frame");
            }
            coz::progress!("backtrack frame");
            if let Some(timeout) = timeout {
                if now.elapsed().as_millis() > timeout {
                    break 'outer;
                }
            }
            if true || considered % (1 << 8) == 0 || solved {
                // println!(
                //     "max_ins: {}, after_steps: {}, max_steps: {}",
                //     max_instructions, after_steps, max_steps
                // );
                // println!(
                //     "candidates: {}, current: {}, ins: {}",
                //     candidates.len(),
                //     candidate,
                //     candidate.count_ins()
                // );
                // if candidate.0[0][0] == BLUE_F2
                //     && candidate.0[0][1] == RED_F1
                //     && candidate.0[1][0] == BLUE_FORWARD
                //     && candidate.0[1][1] == BLUE_F2
                //     && candidate.0[1][2] == RED_RIGHT
                // {
                //     println!("This is it");
                //     break;
                // }
                // for c in candidates.iter().rev().take(10) {
                //     println!("queued: {}, ins: {}", c.candidate, c.candidate.count_ins());
                // }
                // if solved {
                //     break;
                // }
            }
        }
        if costs[pick] == 0.0 {
            costs[pick] = outer_steps as f64;
            costs[pick] = 2.0;
        } else if pick == 0 {
            costs[pick] += outer_steps as f64 / last_outer_steps as f64;
        } else if pick == 1 {
            costs[pick] += 2.0 * outer_steps as f64 / last_outer_steps as f64;
        }
        last_outer_steps = outer_steps;

        coz::progress!("outer frame");
        // println!(
        //     "outer {} took {} seconds {}",
        //     i,
        //     now.elapsed().as_secs_f64(),
        //     solved
        // );
        if solved {
            break 'outer;
        }
    }
    result.sort();
    result.dedup();
    return result;
}

fn search(
    puzzle: &Puzzle,
    mut frame: &mut Frame,
    mut candidates: &mut VecDeque<Frame>,
) -> (bool, usize) {
    // let frame = Frame {
    //     candidate,
    //     state,
    //     max_steps,
    //     max_touches,
    //     max_instructions,
    // }
    let mut preferred = [true; 5];
    for i in 1..frame.candidate.0.len() {
        for j in (i + 1)..frame.candidate.0.len() {
            if frame.candidate.0[i] == frame.candidate.0[j] {
                //preferred[j] = false;
            }
        }
    }
    let mut steps = 0;
    let mut running = true;
    while running && frame.state.steps < frame.max_steps
    //&& frame.state.current_tile().touches() <= frame.max_touches
    {
        let ins = frame.state.current_ins(&frame.candidate);
        let ins_pointer = frame.state.ins_pointer();
        let method_index = ins_pointer.get_method_index();
        let ins_index = ins_pointer.get_ins_index();
        let nop_branch = ins.is_nop();
        let probe_branch = ins.is_probe() && frame.state.current_tile().clone().executes(ins);
        let loosening_branch = !ins.is_debug()
            && !ins.is_loosened()
            && !frame
                .state
                .current_tile()
                .to_condition()
                .is_cond(ins.get_cond());
        if nop_branch || probe_branch || loosening_branch {
            // instructions for branches of current program
            let instructions = get_instructions(
                puzzle,
                &frame,
                nop_branch,
                probe_branch,
                loosening_branch,
                preferred,
                method_index,
                ins_index,
                ins,
            );
            let mut instructions = instructions.iter();
            let replacement_instruction = instructions.next().unwrap();
            for &instruction in instructions {
                let mut temp = frame.candidate.clone();
                temp[method_index][ins_index] = instruction;
                if !snip_around(puzzle, &temp, *ins_pointer, false)
                    && !deny(puzzle, &frame.candidate, false)
                    && temp.count_ins() <= frame.max_instructions
                {
                    let mut branch = Frame {
                        candidate: temp.to_owned(),
                        state: frame.state.clone(),
                        ..*frame
                    };
                    branch.candidate.shade(branch.max_instructions);
                    candidates.push_back(branch);
                    coz::progress!("branching");
                }
            }

            steps += 1;
            frame.candidate[method_index][ins_index] = *replacement_instruction;
            //break;
        }

        coz::progress!("search state step");
        running = frame.state.step(&frame.candidate, puzzle);
    }
    return (frame.state.stars == 0, steps);
}

fn get_instructions(
    puzzle: &Puzzle,
    frame: &Frame,
    nop_branch: bool,
    probe_branch: bool,
    loosening_branch: bool,
    preferred: [bool; 5],
    method_index: usize,
    ins_index: usize,
    ins: Ins,
) -> Vec<Ins> {
    if nop_branch {
        // Noop (unallocated) instruction hit, branches are all puzzle-legal
        // commands of the color of the current tile and a probe instruction.
        [
            //HALT, // including halt is questionable
        ]
        .iter()
        .chain(
            puzzle
                .get_ins_set(frame.state.current_tile().to_condition(), false)
                .iter()
                .filter(|&ins| !ins.is_function() || preferred[ins.source_index()]),
        )
        .chain(
            puzzle
                .get_cond_mask()
                .get_probes(frame.state.current_tile().to_condition())
                .iter(),
        )
        .cloned()
        .collect()
    } else if probe_branch {
        // probe instruction hit(a nop with a color),
        // add a branch for each command of the current tile.
        puzzle
            .get_ins_set(frame.state.current_tile().to_condition(), false)
            .iter()
            .map(|i| i.as_loosened())
            .chain(
                frame.candidate[method_index][ins_index]
                    .remove_cond(frame.state.current_tile().to_condition())
                    .is_probe()
                    .then(|| {
                        frame.candidate[method_index][ins_index]
                            .remove_cond(frame.state.current_tile().to_condition())
                    }),
            )
            .collect()
    } else if loosening_branch {
        // try to make current instruction gray.
        //vec![ins.as_loosened(), ins.get_ins().as_loosened()]
        vec![ins.get_ins().as_loosened(), ins.as_loosened()]
        //vec![ins.get_ins().as_loosened()]
    } else {
        vec![]
    }
}

impl Ord for Frame {
    fn cmp(&self, other: &Self) -> Ordering {
        self.state.steps.cmp(&other.state.steps)
    }
}

impl PartialOrd for Frame {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Frame {}
