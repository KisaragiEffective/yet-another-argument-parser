use std::cell::Cell;
use std::ops::ControlFlow;
use std::str::FromStr;
use itertools::Itertools;

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
    chars_after_single_dash: SingleDashFlagSolver
}

/// You may choose how `-flto` can be parsed, in two way.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum SingleDashFlagSolver {
    /// `-flto` is `-f -l -t -o`, not a long flag. This is commonly used on GNU coreutils.
    ShortFlagSequence,
    /// `-flto` is `--flto`, not short flags. This is commonly used on GCC or Clang.
    OneLongFlag,
}

impl CommandLineArgumentsDefinition {
    fn parse(&self, s: &str) -> Result<UntypedArgs, ()> {
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
                        if self.chars_after_single_dash == SingleDashFlagSolver::ShortFlagSequence {
                            forward();
                            state = ParseNameFirst;
                        } else {
                            return Err(())
                        }
                    } else {
                        if self.chars_after_single_dash == SingleDashFlagSolver::ShortFlagSequence {
                            state = CaptureShortFlags;
                        } else {
                            state = ParseName;
                        }
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

                        if index.get() == s.len() - 1 {
                            break
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
                    break
                }
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

        let ret = UntypedArgs {
            detected_long,
            detected_short,
            rest
        };

        Ok(ret)
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
struct UntypedArgs {
    detected_long: Vec<LongArg>,
    detected_short: Vec<ShortArg>,
    rest: Option<String>,
}


#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum ParserState {
    CaptureNewFlag,
    CaptureLongFlag,
    CaptureShortFlags,
    ParseName,
    ParseNameFirst,
    RestIsExplicitRawForm,
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use crate::{LongArg, CommandLineArgumentsDefinition, ShortArg, ArgProp, SingleDashFlagSolver};

    #[test]
    fn simplest() {
        arbitrary_long_flags(&["foo"]);
    }

    #[test]
    fn two_flags() {
        arbitrary_long_flags(&["foo", "bar"]);
    }

    #[inline]
    fn arbitrary_long_flags<'slice: 'e, 'e>(long_flags: &'slice [&'e str]) {
        arbitrary_short_and_long(&[], long_flags);
    }

    #[test]
    fn short_flag() {
        arbitrary_short_flags(&['a'])
    }

    #[test]
    fn short_flags() {
        arbitrary_short_flags(&['a', 'b', 'c', 'd', 'e']);
    }

    #[inline]
    fn arbitrary_short_flags(short: &[char]) {
        arbitrary_short_and_long(short, &[]);
    }

    #[test]
    fn mixed() {
        arbitrary_short_and_long(&['a', 'b', 'c'], &["foo", "bar", "baz"]);
    }

    fn arbitrary_short_and_long<'ss, 'ls: 'le, 'le>(short: &'ss [char], long: &'ls [&'le str]) {
        let def = CommandLineArgumentsDefinition {
            short_args: short.iter().map(|a| ShortArg {
                name: *a,
                settings: ArgProp,
            }).collect(),
            long_args: long.iter().map(|a| LongArg {
                name: a.to_string(),
                settings: ArgProp,
            }).collect(),
            chars_after_single_dash: SingleDashFlagSolver::ShortFlagSequence
        };

        let short_flags = short.iter().join("");
        let long_flags = long.iter().map(|a| format!("--{a}")).join(" ");
        let x = def.parse(format!("-{short_flags} {long_flags}").as_str()).unwrap();
        assert!(x.rest.is_none());
        short.iter().enumerate().for_each(|(i, e)| {
            assert_eq!(x.detected_short[i].name, *e);
        });
        long.iter().enumerate().for_each(|(i, e)| {
            assert_eq!(x.detected_long[i].name, e.to_string());
        });
    }

    #[test]
    fn single_dash_long_flag() {
        let def = CommandLineArgumentsDefinition {
            long_args: vec![
                LongArg {
                    name: "foobarbaz".to_string(),
                    settings: ArgProp
                }
            ],
            short_args: vec![],
            chars_after_single_dash: SingleDashFlagSolver::OneLongFlag
        };

        let x = def.parse("-foobarbaz").unwrap();
        assert!(x.rest.is_none());
        assert!(x.detected_short.is_empty());
        assert_eq!(x.detected_long.len(), 1);
        assert_eq!(x.detected_long[0].name, "foobarbaz".to_string());
    }

    #[test]
    fn single_dash_long_flags() {
        let def = CommandLineArgumentsDefinition {
            long_args: vec![
                LongArg {
                    name: "foobarbaz".to_string(),
                    settings: ArgProp
                },
                LongArg {
                    name: "quux".to_string(),
                    settings: ArgProp
                }
            ],
            short_args: vec![],
            chars_after_single_dash: SingleDashFlagSolver::OneLongFlag
        };

        let x = def.parse("-foobarbaz -quux").unwrap();
        assert!(x.rest.is_none());
        assert!(x.detected_short.is_empty());
        assert_eq!(x.detected_long.len(), 2);
        assert_eq!(x.detected_long[0].name, "foobarbaz".to_string());
        assert_eq!(x.detected_long[1].name, "quux".to_string());
    }
}
