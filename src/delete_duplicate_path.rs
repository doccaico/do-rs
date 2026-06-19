use anyhow::{Context, Result};
use std::collections::HashSet;
use std::env;
// use std::process::ExitCode;

pub fn run() -> Result<()> {
    let paths = env::var("PATH").context("not found 'PATH' in env variable")?;

    let path_list: Vec<_> = paths.split(';').filter(|p| !p.is_empty()).collect();

    let len = path_list.len();
    let mut path_set = HashSet::with_capacity(len);
    let mut path_vec = Vec::with_capacity(len);

    for path in path_list {
        if path_set.insert(path) {
            path_vec.push(path);
        }
    }

    print!("{}", path_vec.join(";"));

    Ok(())
}
