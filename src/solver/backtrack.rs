use std::cmp::{max, Ordering, Reverse};
use std::collections::{BinaryHeap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::io::{stdout, Write};
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering as SyncOrdering};
use std::thread::spawn;
use std::time::{Duration, Instant};

use crossbeam_channel::{unbounded, Receiver, Sender};

use super::pruning::*;
use crate::constants::*;
use crate::game::instructions::*;
use crate::game::{puzzle::Puzzle, state::State, Source};
use crate::web::encode_program;

const BACKTRACK_STACK_SIZE: usize = 2200;
const PHI: f64 = 1.61803398875;
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

#[derive(Clone, Copy, Default)]
struct Limit {
    cost: f64,
    old_steps: usize,
    steps: usize,
    touches: usize,
    instructions: usize,
    increased: Increased,
}

impl Hash for Limit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.steps.hash(state);
        self.touches.hash(state);
        self.instructions.hash(state);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
enum Increased {
    #[default]
    Steps,
    Touches,
    Instructions,
}

impl Eq for Limit {}

impl PartialEq for Limit {
    fn eq(&self, other: &Self) -> bool {
        self.steps == other.steps
            && self.touches == other.touches
            && self.instructions == other.instructions
    }
}

impl PartialOrd for Limit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}
impl Ord for Limit {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost
            .total_cmp(&other.cost)
            .then_with(|| self.steps.cmp(&other.steps))
            .then_with(|| self.touches.cmp(&other.touches))
            .then_with(|| self.instructions.cmp(&other.instructions))
    }
}

pub fn backtrack(puzzle: Puzzle, timeout: Option<u128>) -> Vec<(usize, Source)> {
    let instruction_set_length = puzzle.get_ins_set(INS_COLOR_MASK, true).len();
    //let mut max_instructions = puzzle.methods.iter().sum();
    let mut step_cap = [usize::MAX; 50];
    let mut checked = [0; 50];
    let mut touch_cap = [usize::MAX; 50];
    let mut instruction_cap = puzzle.methods.iter().sum();

    let reachable_tiles = puzzle.board.count_tiles();

    // ------------------------------------------------------------
    const PRINT_STUFF: bool = false;
    // ------------------------------------------------------------

    let mut priorities = BinaryHeap::new();
    priorities.push(Reverse(Limit {
        cost: 2.0,
        old_steps: 1,
        steps: 8 * reachable_tiles,
        touches: 8,
        instructions: 1,
        increased: Increased::Steps,
    }));
    // priorities.push(Reverse(Limit {
    //     cost: 2.0,
    //     old_steps: 1,
    //     steps: 1000000,
    //     touches: 800,
    //     instructions: 9,
    //     increased: Increased::Steps,
    // }));
    let mut done_limits = HashSet::new();
    let mut last_outer_steps = 0;
    let mut solved = false;
    let mut result: Vec<(usize, Source)> = vec![];
    'outer: while let Some(Reverse(mut limit)) = priorities.pop() {
        limit.instructions = limit.instructions.min(instruction_cap);
        if solved {
            limit.steps = (step_cap[limit.instructions]);
            limit.touches = (touch_cap[limit.instructions]);
        }
        if !done_limits.insert(limit) {
            continue;
        }
        let now = Instant::now();
        let mut outer_frame = Frame::new(&puzzle);
        outer_frame.max_steps = limit.steps;
        outer_frame.max_touches = limit.touches;
        outer_frame.max_instructions = limit.instructions;
        outer_frame.candidate.shade(outer_frame.max_instructions);

        if PRINT_STUFF {
            print!(
                "{{s: {:>4}, t: {:>4}, i: {:>2}}}, o {:>10}, c {:>10.1}",
                limit.steps, limit.touches, limit.instructions, limit.old_steps, limit.cost,
            );
            stdout().flush().unwrap();
        }

        let mut candidates = VecDeque::new();
        candidates.push_back(outer_frame);
        let mut branches: u64 = 0;
        let mut outer_steps = 0;
        let mut step_deaths = 0;
        let mut touch_deaths = 0;
        let mut both_deaths = 0;
        while let Some(mut frame) = candidates.pop_back() {
            frame.max_steps = frame.max_steps.min(step_cap[limit.instructions] - 1);

            branches += 1;
            let (is_solution, after_steps, step_death, touch_death) =
                search(&puzzle, &mut frame, &mut candidates);
            outer_steps += after_steps;
            step_deaths += (step_death & !touch_death) as usize;
            touch_deaths += (touch_death & !step_death) as usize;
            both_deaths += (step_death & touch_death) as usize;

            if is_solution {
                let mut solution = frame.candidate.clone();
                solution.sanitize();
                let max_touches = frame.state.board.max_touches();
                result.push((frame.state.steps, solution));
                for incnt in 1..=limit.instructions {
                    step_cap[incnt] = frame.state.steps * (4 * (limit.instructions - incnt) + 1);
                    touch_cap[incnt] = max_touches * (4 * (limit.instructions - incnt) + 1);
                }
                instruction_cap = limit.instructions - 1;
                solved = true;
                if PRINT_STUFF {
                    println!();
                    println!(
                        "solved! candidates: {}, current: {}, ins: {}, steps: {}, touches: {}, code: {}",
                        candidates.len(),
                        frame.candidate,
                        frame.candidate.count_ins(),
                        frame.state.steps,
                        max_touches,
                        encode_program(&frame.candidate, &puzzle)
                    );
                }
                //candidates.clear();
                coz::progress!("backtrack frame");
            }
            coz::progress!("backtrack frame");
            if let Some(timeout) = timeout {
                if now.elapsed().as_millis() > timeout {
                    break 'outer;
                }
            }
            if true || branches % (1 << 8) == 0 || solved {
                // println!(
                //     "candidates: {}, current: {}, ins: {}",
                //     candidates.len(),
                //     frame.candidate,
                //     frame.candidate.count_ins()
                // );
                // for c in candidates.iter().rev().take(10) {
                //     println!("queued: {}, ins: {}", c.candidate, c.candidate.count_ins());
                // }
            }
        }
        let deaths = step_deaths + touch_deaths + both_deaths;
        let death_ratio = deaths as f64 / branches as f64;

        if PRINT_STUFF {
            print!(", took {:>7} branches, {:>11} steps", branches, outer_steps);
            print!(
                ", s {}, t {}, b {}, r {:.3}",
                step_deaths, touch_deaths, both_deaths, death_ratio
            );
            println!();
        }

        if deaths > 0 {
            let scale = 2.0;
            let cost = outer_steps as f64
                + (outer_steps as f64 * death_ratio * scale) * (1.3_f64.powf(limit.touches as f64));
            let next_touches = (limit.touches as f64 * scale).ceil() as usize;
            let next_steps = limit.touches * (reachable_tiles);
            let limit = Limit {
                cost,
                old_steps: outer_steps,
                steps: next_steps,
                touches: next_touches,
                increased: Increased::Steps,
                ..limit
            };
            // println!(
            //     "adding steps {{s: {:>3}, t: {:>2}, i: {:>2}}}, c {:>10}",
            //     limit.steps, limit.touches, limit.instructions, limit.cost,
            // );
            priorities.push(Reverse(limit));
        }
        if death_ratio < 0.9 {
            let cost = outer_steps as f64 * (instruction_set_length as f64 / 2.0);
            let limit = Limit {
                cost,
                old_steps: outer_steps,
                instructions: limit.instructions + 1,
                increased: Increased::Instructions,
                ..limit
            };
            // println!(
            //     "adding instr {{s: {:>3}, t: {:>2}, i: {:>2}}}, c {:>10}",
            //     limit.steps, limit.touches, limit.instructions, limit.cost,
            // );
            priorities.push(Reverse(limit));
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
) -> (bool, usize, bool, bool) {
    let mut preferred = [true; 5];
    for i in 1..frame.candidate.0.len() {
        for j in (i + 1)..frame.candidate.0.len() {
            if frame.candidate.0[i] == frame.candidate.0[j] {
                preferred[j] = false;
            }
        }
    }
    let pre_steps = frame.state.steps;
    let mut running = true;
    while running
        && frame.state.steps < frame.max_steps
        && frame.state.current_tile().touches() <= frame.max_touches
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
            for &instruction in instructions.rev() {
                let mut temp = frame.candidate.clone();
                temp[method_index][ins_index] = instruction;
                //temp.shade(frame.max_instructions);
                if true
                    && !snip_around(puzzle, &temp, *ins_pointer, false)
                    && !deny(puzzle, &temp, false)
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

            frame.candidate[method_index][ins_index] = *replacement_instruction;
            //break;
        }

        coz::progress!("search state step");
        // running = frame.state.step(&frame.candidate, puzzle);
        running = frame.state.steps(
            &frame.candidate,
            puzzle,
            frame.max_steps - frame.state.steps,
            // 1,
            frame.max_touches,
        );
    }
    return (
        frame.state.stars == 0,
        frame.state.steps - pre_steps,
        frame.state.steps >= frame.max_steps,
        frame.state.current_tile().touches() > frame.max_touches,
    );
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
            HALT, // --including halt is questionable--
                 // including halt is necessary for puzzle 26
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
