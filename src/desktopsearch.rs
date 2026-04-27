use freedesktop_desktop_entry::{desktop_entries, get_languages_from_env};
use std::env::var;
use crate::cache::*;

pub fn getdesktopfiles() -> Vec<(String, String, String, i32)> {
    let cdesktop_bind = var("XDG_CURRENT_DESKTOP").unwrap();
    let current_desktop = cdesktop_bind.as_str();
    let cache_count = read_cache();
    let mut desktopfiles = desktop_entries(&get_languages_from_env())
        .clone()
        .into_iter()
        .filter_map(|entry| {
            let desktop_group = entry.groups.0.get("Desktop Entry")?;
            if let Some((value, _)) = desktop_group.0.get("NoDisplay") {
                if value == "true" {
                    return None;
                }
            }
            if let Some((value, _)) = desktop_group.0.get("Terminal") {
                if value == "true" {
                    return None;
                }
            }
            if let Some((value, _)) = desktop_group.0.get("OnlyShowIn") {
                let envs: Vec<&str> = value.split(';').filter(|s| !s.is_empty()).collect();
                if !envs.contains(&current_desktop) {
                    return None;
                }
            }
            if let Some((value, _)) = desktop_group.0.get("NotShowIn") {
                let envs: Vec<&str> = value.split(';').filter(|s| !s.is_empty()).collect();
                if envs.contains(&current_desktop) {
                    return None;
                }
            }
            let name = desktop_group.0.get("Name")?.0.clone();
            let icon = desktop_group.0.get("Icon")?.0.clone();
            let exec = desktop_group.0.get("Exec")?.0.clone();
            let score = cache_count.get(&name).unwrap_or(&0).clone();
            Some((name, icon, exec, score))
        }).collect::<Vec<_>>();

    desktopfiles.sort_by(|a, b| a.0.cmp(&b.0)); // 0 is name, lower means sorted alphabetically
    desktopfiles.sort_by(|a, b| b.3.cmp(&a.3)); // 3 is score, higher means more relevant
    desktopfiles
}


pub fn reeval(searchterm: &str, desktopfiles: Vec<(String, String, String, i32)>) -> Vec<(String, String, String, i32)> {
    if searchterm.is_empty() {
        return desktopfiles;
    }
    let mut desktopfiles2 = desktopfiles.into_iter().map(|entry| reeval_single(searchterm, entry)).collect::<Vec<_>>();
    desktopfiles2.sort_by(|a, b| a.3.cmp(&b.3)); // 3 is score, lower means more relevant
    desktopfiles2
}


fn reeval_single(searchterm: &str, toevaluate: (String, String, String, i32)) -> (String, String, String, i32) {
    let mut finalmult = 0;
    let mut find_from = 0;
    let mut found_first = false;
    let mut score:i32 = 1;
    let searchedstring = toevaluate.0.clone().to_lowercase();
    for letter in searchterm.chars() {
        let count = searchedstring.match_indices(letter).find_map(|(i, _)| (i >= find_from).then(|| i));
        if !found_first {
            finalmult += 1;
            if !count.is_none() {
                found_first = true;
                score += (count.unwrap() - find_from) as i32;
            }
            else {
                score *= 4;
            }
        }
        else {
            if !count.is_none(){
                score += (count.unwrap() - find_from).pow(2) as i32;
            }
            else {
                score *= 4;
            }
        }
        find_from += 1;
    }
    score = score * finalmult;
    (toevaluate.0, toevaluate.1, toevaluate.2, score)
}