use std::fs;
use std::io::{Read, Write};
use std::str::FromStr;
use std::{fs::File, path::PathBuf};

use crate::game::Source;
use crate::web::errors::SolverError;

pub fn read_solution_from_file(puzzle_id: u64) -> Result<Vec<Source>, SolverError> {
    let mut path = PathBuf::from_str("data/solutions").expect("unable to create puzzle pathbuf");
    path.push(puzzle_id.to_string());
    File::options()
        .read(true)
        .open(path)
        .map_err(SolverError::IOError)
        .and_then(|mut file| {
            let mut string = String::new();
            file.read_to_string(&mut string)?;
            let level_json: Result<Vec<Source>, _> =
                serde_json::from_str::<Option<Vec<Source>>>(&string)
                    .map_err(SolverError::Serde)
                    .and_then(|opt| opt.ok_or(SolverError::NoPuzzleForId));
            return level_json;
        })
}

pub fn store_solutions_locally(solution: &Vec<Source>, puzzle_id: u64) {
    let mut path = PathBuf::from_str("data/solutions").expect("unable to create solution pathbuf");
    path.push(puzzle_id.to_string());
    File::options()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|err| {
            eprintln!(
                "unable to store solution {} locally.\nerror: {}\ndata: {:?}",
                puzzle_id, err, solution
            );
        })
        .map(|mut file| {
            file.write_all(
                serde_json::to_string(solution)
                    .expect("should be able to convert soluiton to string")
                    .as_bytes(),
            )
            .expect("unable to write solution json to file");
        })
        .unwrap();
}

pub fn remove_solution_file(puzzle_id: u64) {
    let mut path = PathBuf::from_str("data/solutions").expect("unable to create solution pathbuf");
    path.push(puzzle_id.to_string());
    fs::remove_file(path).expect("should remove the file");
}
