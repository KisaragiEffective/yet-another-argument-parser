use std::cell::Cell;
use std::str::FromStr;

#[derive(Eq, PartialEq, Clone, Debug)]
struct ArgProp;

#[derive(Eq, PartialEq, Clone, Debug)]
struct LongArg {
    name: String,
    settings: ArgProp,
}

#[derive(Eq, PartialEq, Clone, Debug)]
struct ShortArg {
    name: char,
    settings: ArgProp,
}

#[derive(Eq, PartialEq, Clone, Debug)]
struct CommandLineArgumentsDefinition {
    long_args: Vec<LongArg>,
    short_args: Vec<ShortArg>,
}

impl CommandLineArgumentsDefinition {
    fn parse(&self, s: &str) -> Result<UntypedArgs, ()> {
        UntypedArgs::from_str(s)
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
struct UntypedArgs {
    detected_long: Vec<LongArg>,
    detected_short: Vec<ShortArg>,
    rest: Option<String>,
}

impl FromStr for UntypedArgs {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let index = Cell::new(0);
        use crate::ParserState::*;
        let mut state = CaptureNewFlag;
        let mut name_buf = String::new();
        let mut rest = None;
        let mut found_long_arg_name = vec![];
        let mut found_short_arg_name = vec![];
        loop {
            let cc = s.chars().nth(index.get()).expect("index overflow!!!");
            println!("{state:?} {index} {cc}", index = index.get());
            let forward = || {
                index.set(index.get() + 1);
            };

            match state {
                CaptureNewFlag => {
                    if cc == '-' {
                        forward();
                        state = CaptureLongFlag;
                    } else {
                        return Err(())
                    }
                }
                CaptureLongFlag => {
                    if cc == '-' {
                        forward();
                        state = ParseNameFirst;
                    } else {
                        state = CaptureShortFlags;
                    }
                }
                CaptureShortFlags => {
                    if cc == ' ' {
                        forward();
                        state = CaptureNewFlag;
                    } else if cc == '-' {
                        return Err(())
                    } else {
                        forward();
                        found_short_arg_name.push(cc);
                        if index.get() == s.len() {
                            state = Complete;
                        }
                    }
                }
                ParseName => {
                    index.set(index.get() + 1);
                    if cc == ' ' {
                        state = CaptureNewFlag;
                        found_long_arg_name.push(name_buf.clone());
                        name_buf.clear();
                    } else {
                        name_buf.push(cc);
                    }

                    if index.get() == s.len() {
                        state = Complete;
                        found_long_arg_name.push(name_buf.clone());
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

            if state == Complete {
                break
            }
        }

        let detected_long = found_long_arg_name.into_iter().map(|a| LongArg {
            name: a,
            settings: ArgProp
        }).collect();

        let detected_short = found_short_arg_name.into_iter().map(|a| ShortArg {
            name: a,
            settings: ArgProp
        }).collect();

        let ret = Self {
            detected_long,
            detected_short,
            rest
        };

        Ok(ret)
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum ParserState {
    CaptureNewFlag,
    CaptureLongFlag,
    CaptureShortFlags,
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
    use crate::{LongArg, CommandLineArgumentsDefinition, ShortArg, ArgProp};

    #[test]
    fn simplest() {
        arbitrary_long_flags(&["foo"]);
    }

    #[test]
    fn two_flags() {
        arbitrary_long_flags(&["foo", "bar"]);
    }

    fn arbitrary_long_flags<'slice: 'e, 'e>(long_flags: &'slice [&'e str]) {
        let long_args = long_flags.iter().map(|a| LongArg {
            name: a.to_string(),
            settings: ArgProp,
        }).collect::<Vec<_>>();

        let def = CommandLineArgumentsDefinition {
            long_args,
            short_args: vec![],
        };

        let x = def.parse(long_flags.iter().map(|a| format!("--{a}")).join(" ").as_str()).unwrap();
        assert!(x.rest.is_none());
        long_flags.iter().enumerate().for_each(|(i, e)| {
            assert_eq!(x.detected_long[i].name, e.to_string());
        })
    }

    #[test]
    fn short_flag() {
        arbitrary_short_flags(&['a'])
    }

    #[test]
    fn short_flags() {
        arbitrary_short_flags(&['a', 'b', 'c', 'd', 'e']);
    }

    fn arbitrary_short_flags(short: &[char]) {
        let def = CommandLineArgumentsDefinition {
            short_args: short.iter().map(|a| ShortArg {
                name: *a,
                settings: ArgProp,
            }).collect(),
            long_args: vec![]
        };

        let x = def.parse(format!("-{flags}", flags = short.iter().join("")).as_str()).unwrap();
        assert!(x.rest.is_none());
        short.iter().enumerate().for_each(|(i, e)| {
            assert_eq!(x.detected_short[i].name, *e);
        })
    }
}
