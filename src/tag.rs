// use crate::LINE_ENDING;
// use regex::Regex;

// pub fn insert_into_tags(content: &str, value: &str, tag: &str) -> String {
//     let pattern = format!(r"{}((.|{})*){}", tag, LINE_ENDING, tag);
//     let re = Regex::new(pattern.as_str()).unwrap();
//     let new_string = format!("{}{}{}", tag, value, tag);
//     String::from(re.replace(content, new_string.as_str()))
// }
//
// pub fn extract_from_tags(content: &str, tag: &str) -> String {
//     let pattern = format!(r"{}(.*){}", tag, tag);
//     let re = Regex::new(pattern.as_str()).unwrap();
//     let caps = re.captures(content).unwrap();
//     String::from(caps.get(1).unwrap().as_str())
// }