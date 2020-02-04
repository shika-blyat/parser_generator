use mini_rparsec::{
    character::{identifier, label, string},
    multi::many,
    Error, Parser, ParserError, Remaining,
};
use std::collections::HashMap;
use std::ops::Range;

pub type Grammar = HashMap<String, Rule>;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Rule {
    RRule(String),
    RString(String),
    RRange(Range<char>),
    ROr(Vec<Rule>),
    RRuleList(Vec<Rule>),
}

pub fn ws_no_newline<'a>() -> impl Parser<'a, ()> {
    |s: Remaining<'a>| match s.rem.find(|c: char| !c.is_whitespace() || c == '\n') {
        Some(index) => Ok((Remaining::new(&s.rem[index..], s.pos + index), ())),
        None => Ok((
            Remaining::new(&s.rem[s.rem.len()..s.rem.len()], s.pos + s.rem.len()),
            (),
        )),
    }
}
pub fn newline_or_eof<'a>() -> impl Parser<'a, ()> {
    |s| {
        label("\n")(s)
            .map(|(remaining, _)| (remaining, ()))
            .or_else(|err| {
                if err.rem().rem.is_empty() {
                    Ok((err.rem(), ()))
                } else {
                    Err(Error::Unsavable(
                        err.rem().pos,
                        ParserError::new(
                            0..1,
                            format!(
                                "Expected a newline or an EOF, found {:#?}",
                                &s.rem[0..s
                                    .rem
                                    .find(|c: char| c.is_whitespace())
                                    .unwrap_or(s.rem.len())]
                            ),
                        ),
                    ))
                }
            })
    }
}
pub fn rule<'a>() -> impl Parser<'a, Rule> {
    |s| {
        string()(s)
            .map(|(remaining, string)| (remaining, Rule::RString(string.to_string())))
            .or_else(|error| {
                identifier()(error.rem())
                    .map(|(remaining, string)| (remaining, Rule::RRule(string.to_string())))
            })
            .or_else(|_| {
                Err(Error::Failure(
                    s,
                    ParserError::new(
                        0..1,
                        format!(
                            "Expected a valid rule, found {:#?}",
                            &s.rem[0..s
                                .rem
                                .find(|c: char| c.is_whitespace())
                                .unwrap_or(s.rem.len())]
                        ),
                    ),
                ))
            })
    }
}
pub fn rules<'a>() -> impl Parser<'a, Rule> {
    |s| {
        many(|remaining| ws_no_newline()(remaining).and_then(|(remaining, _)| rule()(remaining)))(s)
            .map(|(remaining, rules)| (remaining, Rule::RRuleList(rules)))
    }
}
pub fn or_rules<'a>() -> impl Parser<'a, Rule> {
    |s| {
        ws_no_newline()(s)
            .and_then(|(remaining, _)| rules()(remaining))
            .and_then(|(remaining, rule)| {
                let (remaining, mut rules) = many(|remaining| {
                    ws_no_newline()(remaining)
                        .and_then(|(remaining, _)| label("|")(remaining))
                        .and_then(|(remaining, _)| {
                            ws_no_newline()(remaining).and_then(|(remaining, _)| rules()(remaining))
                        })
                        .map(|(remaining, rule)| (remaining, rule))
                })(remaining)?;
                rules.push(rule);
                Ok((remaining, rules))
            })
            .map(|(remaining, rules)| (remaining, Rule::ROr(rules)))
    }
}
pub fn rule_def<'a>() -> impl Parser<'a, (String, Rule)> {
    |s| {
        identifier()(s).and_then(|(remaining, rule_name)| {
            ws_no_newline()(remaining)
                .and_then(|(remaining, _)| label("=")(remaining))
                .and_then(|(remaining, _)| ws_no_newline()(remaining))
                .and_then(|(remaining, _)| or_rules()(remaining))
                .and_then(|(remaining, rule)| {
                    ws_no_newline()(remaining)
                        .and_then(|(remaining, _)| newline_or_eof()(remaining))
                        .map(|(remaining, _)| (remaining, (rule_name.to_string(), rule)))
                })
        })
    }
}
pub fn grammar<'a>(grammar: &'a str) -> Result<Grammar, Error> {
    many(|remaining| rule_def()(remaining))(Remaining::new(grammar.trim(), 0)).and_then(
        |(remaining, rules)| {
            if remaining.rem.is_empty() {
                let mut grammar = HashMap::new();
                for (rule_name, rule) in rules {
                    grammar.insert(rule_name, rule);
                }
                Ok(grammar)
            } else {
                println!("{:#?}", remaining);
                let eol = remaining
                    .rem
                    .find(|c: char| c == '\n')
                    .unwrap_or(remaining.rem.len());
                Err(Error::Unsavable(
                    remaining.pos,
                    ParserError::new(0..eol, format!("Unexpected {:#?}", &remaining.rem[0..eol])),
                ))
            }
        },
    )
}
