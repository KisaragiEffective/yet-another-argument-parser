use std::cell::Cell;
use std::num::NonZeroU8;
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
    use crate::{Arg, CommandLineArgumentsDefinition};

    #[test]
    fn simplest() {
        let def = CommandLineArgumentsDefinition {
            args: vec![
                Arg {
                    name: "foo".to_string()
                }
            ]
        };

        let x = def.parse("--foo").unwrap();
        assert!(x.rest.is_none());
        assert_eq!(x.detected[0].name, "foo".to_string());

    }
}
