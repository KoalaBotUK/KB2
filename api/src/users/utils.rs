use regex::Regex;
use crate::users::models::Link;

pub fn link_arr_match(links: &[Link], pattern: &str) -> bool {
    let regex = Regex::new(pattern).unwrap();
    links.iter().any(|l| l.active && regex.is_match(&l.link_address))
}

pub fn link_match(link: &Link, pattern: &str) -> bool {
    let regex = Regex::new(pattern).unwrap();
    link.active && regex.is_match(&link.link_address)
}