use std::{collections::HashMap, env, ffi::OsString, iter};

use clap::Parser;
use itertools::Itertools;

pub fn parse<S: Parser>() -> (S, Vec<(String, u32)>) {
    parse_from::<S, _, _>(env::args())
}

pub fn parse_from<S, I, T>(argv: I) -> (S, Vec<(String, u32)>)
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
    S: Parser,
{
    let iter = argv.into_iter();

    let (iter, iter_new) = iter.tee();
    let argv_arg = iter_new.skip(1).map(|x| x.into()).collect_vec();

    let (iter, iter_new) = iter.tee();
    let arg_struct = S::parse_from(iter_new);

    let cmd = S::command();
    let all_flags: HashMap<String, (String, String)> = cmd
        .get_arguments()
        .filter_map(|a| {
            if a.get_action().takes_values() {
                None
            } else {
                Some((
                    a.get_id().to_string(),
                    (
                        format!("-{}", a.get_short().unwrap()),
                        format!("--{}", a.get_long().unwrap()),
                    ),
                ))
            }
        })
        .collect();

    let matches = cmd.get_matches_from(iter);

    let optarg_map: HashMap<usize, String> = matches
        .ids()
        .map(|id| id.as_str().to_string())
        .filter(|x| !all_flags.contains_key(x))
        .filter_map(|id| matches.indices_of(&id).zip(Some(id)))
        .flat_map(|(idc, id)| idc.zip(iter::repeat(id)))
        .collect();

    let mut args: Vec<Option<String>> = Vec::with_capacity(argv_arg.len());
    args.resize(argv_arg.len(), None);

    for (key, val) in optarg_map {
        args[key - 1] = Some(val);
    }

    for pos in args.iter().positions(|x| x.is_none()).collect_vec() {
        for (id, (short, long)) in &all_flags {
            if argv_arg[pos] == Into::<OsString>::into(short)
                || argv_arg[pos] == Into::<OsString>::into(long)
            {
                args[pos] = Some(id.to_owned());
            }
        }
    }

    let mut oprs: Vec<(String, u32)> = vec![];
    let mut id: Option<String> = None;
    let mut args_iter = args.into_iter();

    loop {
        let Some(arg) = args_iter.next() else {
            break
        };

        match arg {
            Some(arg) => match &id {
                Some(val) => {
                    if val == &arg {
                        let (id, mut count) = oprs.pop().unwrap();
                        count += 1;
                        oprs.push((id, count));
                    } else {
                        id = None;
                        oprs.push((arg, 0))
                    }
                }
                None => {
                    oprs.push((arg, 0));
                }
            },
            None => {
                let Some(next) = args_iter.next() else {
                    break
                };
                id = next.clone();
                oprs.push((next.unwrap(), 1))
            }
        }
    }

    (arg_struct, oprs)
}
