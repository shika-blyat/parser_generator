mod grammar;
use grammar::{grammar, Grammar, Rule};

const EXEMPLE_GRAMMAR: &str = r#"
word = letter | letter word
letter = "a""#;
const EXEMPLE_STRING: &str = "aaaa";

pub fn eval_orrule(rule_list: &Vec<Rule>, val: &str, grammar: &Grammar) {}

pub fn eval_rule(rule_name: &str, val: &str, grammar: &Grammar) {
    match grammar.get(rule_name).unwrap() {
        Rule::ROr(rule_list) => {
            eval_orrule(rule_list, val, grammar);
        }
        _ => unreachable!(),
    }
}
fn main() {
    println!("{:#?}", grammar(EXEMPLE_GRAMMAR));
}
