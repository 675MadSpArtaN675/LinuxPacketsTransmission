use std::ops::Deref;
use log::{info, debug};

use regex::Regex;

struct PatternCheckup {
    pattern_list: Vec<String>,
    blacklist: bool
}

impl PatternCheckup {
    pub fn new_whitelist(patterns: Vec<String>) -> PatternCheckup {
        return PatternCheckup { pattern_list: patterns, blacklist: false };
    }

    pub fn new_blacklist(patterns: Vec<String>) -> PatternCheckup {
        return PatternCheckup { pattern_list: patterns, blacklist: true };
    }

    pub fn get_pattern_list(&self) -> &Vec<String> {
        return &self.pattern_list;
    }

    pub fn add_pattern(&mut self, pattern: String) {
        self.pattern_list.push(pattern);
    }

    pub fn pattern_contains(&self, pattern: String) -> bool {
        return self.pattern_list.contains(&pattern);
    }
    pub fn remove(&mut self, index: usize) -> String {
        return self.pattern_list.remove(index);
    }
    pub fn remove_by_value(&mut self, pattern: String) -> String {
        let found_index = self.pattern_list.iter().position(|c| c.clone().deref() == pattern);

        if let Some(index) = found_index {
            return self.pattern_list.remove(index);
        }

        return String::new();
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        return self.pattern_list.get(index);
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut String> {
        return self.pattern_list.get_mut(index);
    }

    pub fn set_is_blacklist(&mut self, value: bool) {
        self.blacklist = value;
    }

    pub fn is_blacklist(&self) -> bool
    {
        return self.blacklist;
    }

    pub fn clear(&mut self) {
        self.pattern_list.clear();
    }

    pub fn check(&self, text: &String) -> bool {
        info!("Checking text: {}", text);
        for pattern_str in self.pattern_list.iter() {
            let pattern_result = Regex::new(pattern_str);

            if let Ok(pattern) = pattern_result {
                let flag: bool = pattern.is_match(&text);

                if flag {
                    debug!("Text: {}, Flag: {}", text, flag);
                    return flag;
                }
            }
        }

        info!("Text: {}, Flag: {}", text, false);
        return false;
    }
}

impl Clone for PatternCheckup {
    fn clone(&self) -> Self {
        return PatternCheckup { pattern_list: self.pattern_list.clone(), blacklist: self.blacklist }
    }
}

pub struct NameCheckuper {
    whitelist: PatternCheckup,
    blacklist: PatternCheckup
}

impl NameCheckuper {
    pub fn new_empty() -> NameCheckuper {
        return NameCheckuper {
            whitelist: PatternCheckup::new_whitelist(vec![]),
            blacklist: PatternCheckup::new_blacklist(vec![])
        };
    }

    pub fn new(whitelist_patterns: Vec<String>, blacklist_patterns: Vec<String>) -> NameCheckuper {
        info!("Whitelist len: {}. Blacklist len: {}", whitelist_patterns.len(), blacklist_patterns.len());
        return NameCheckuper {
            whitelist: PatternCheckup::new_whitelist(whitelist_patterns),
            blacklist: PatternCheckup::new_blacklist(blacklist_patterns)
        };
    }

    pub fn check(&self, value: String) -> bool {
        let whitelist_flag: bool = self.whitelist.check(&value);
        let blacklist_flag: bool = self.blacklist.check(&value);

        info!("Text: {} Whitelist flag: {}; Blacklist flag: {}", value, whitelist_flag, blacklist_flag);

        return (whitelist_flag == blacklist_flag) || whitelist_flag
    }

    pub fn add_to_whitelist(&mut self, pattern: String) {
        if !pattern.is_empty() {
            self.whitelist.add_pattern(pattern);
        }
    }

    pub fn add_to_blacklist(&mut self, pattern: String) {
        if !pattern.is_empty() {
            self.blacklist.add_pattern(pattern);
        }
    }

    pub fn remove_from_whitelist_index(&mut self, index: usize) {
        self.whitelist.remove(index);
    }

    pub fn remove_from_blacklist_index(&mut self, index: usize) {
        self.blacklist.remove(index);
    }

    pub fn remove_from_whitelist(&mut self, pattern: String) {
        if !pattern.is_empty() {
            self.whitelist.remove_by_value(pattern);
        }
    }

    pub fn remove_from_blacklist(&mut self, pattern: String) {
        if !pattern.is_empty() {
            self.blacklist.remove_by_value(pattern);
        }
    }

    pub fn clear_whitelist(&mut self) {
        self.whitelist.clear();
    }

    pub fn clear_blacklist(&mut self) {
        self.blacklist.clear();
    }
}

impl Clone for NameCheckuper {
    fn clone(&self) -> Self {
        return NameCheckuper { whitelist: self.whitelist.clone(), blacklist: self.blacklist.clone() }
    }
}