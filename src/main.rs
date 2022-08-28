use std::cell::Cell;
use std::str::FromStr;

struct Arg {
    name: String,
}

struct CommandLineArgumentsDefinition {
    args: Vec<Arg>,
}

impl CommandLineArgumentsDefinition {
    fn parse(&self, s: &str) -> Result<UntypedArgs, ()> {
        UntypedArgs::from_str(s)
    }
}

struct UntypedArgs {
    detected: Vec<Arg>,
    rest: Option<String>,
}

impl FromStr for UntypedArgs {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let index = Cell::new(0);
        use crate::ParserState::*;
        let mut state = ExpectTwoDash;
        let mut name_buf = String::new();
        let mut rest = None;
        let mut found_arg_name = vec![];
        loop {
            println!("{state:?} {index}", index = index.get());
            let cc = s.chars().nth(index.get()).unwrap();
            let forward = || {
                index.set(index.get() + 1);
            };

            match state {
                ExpectTwoDash => {
                    if cc == '-' {
                        forward();
                        state = ExpectOneDash;
                    } else {
                        return Err(())
                    }
                }
                ExpectOneDash => {
                    if cc == '-' {
                        forward();
                        state = ParseNameFirst;
                    } else {
                        return Err(())
                    }
                }
                ParseName => {
                    index.set(index.get() + 1);
                    if cc == ' ' {
                        state = ExpectTwoDash;
                        found_arg_name.push(name_buf.clone());
                        name_buf.clear();
                    } else {
                        name_buf.push(cc);
                    }

                    if index.get() == s.len() {
                        state = Complete;
                        found_arg_name.push(name_buf.clone());
                        name_buf.clear();
                        break
                    }
                }
                ParseNameFirst => {
                    forward();
                    if cc == ' ' {
                        // `-- ...`
                        state = RestIsExplicitRawForm;
                    } else {
                        name_buf.push(cc);
                        state = ParseName;
                    }
                }
                RestIsExplicitRawForm => {
                    rest = Some(s[index.get()..].to_string());
                    state = Complete;
                }
                Complete => {
                    break
                }
            }
        }

        let args = found_arg_name.into_iter().map(|a| Arg {
            name: a
        }).collect();

        Ok(Self {
            detected: args,
            rest
        })
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum ParserState {
    ExpectTwoDash,
    ExpectOneDash,
    ParseName,
    ParseNameFirst,
    RestIsExplicitRawForm,
    Complete,
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use crate::{Arg, CommandLineArgumentsDefinition};

    #[test]
    fn simplest() {
        arbitrary_flags(&["foo"]);
    }

    #[test]
    fn two_flags() {
        arbitrary_flags(&["foo", "bar"]);
    }

    fn arbitrary_flags<'slice: 'e, 'e>(flags: &'slice [&'e str]) {
        let args = flags.iter().map(|a| Arg {
            name: a.to_string()
        }).collect::<Vec<_>>();

        let def = CommandLineArgumentsDefinition {
            args
        };

        let x = def.parse(flags.iter().map(|a| format!("--{a}")).join(" ").as_str()).unwrap();
        assert!(x.rest.is_none());
        flags.iter().enumerate().for_each(|(i, e)| {
            assert_eq!(x.detected[i].name, e.to_string());
        })
    }
}
