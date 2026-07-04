use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Character {
    pub name: String,
    pub identity: String,
    pub personality: String,
    pub abilities: String,
    pub relations: String,
    pub speech_style: String,
}

pub fn read_characters(story_path: &Path) -> Result<Vec<Character>> {
    let char_dir = story_path.join("characters");
    if !char_dir.exists() {
        return Ok(Vec::new());
    }

    let mut characters = Vec::new();
    for entry in std::fs::read_dir(&char_dir).context("Failed to read characters directory")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "md") {
            let content = std::fs::read_to_string(&path)?;
            if let Some(c) = parse_character(&content) {
                characters.push(c);
            }
        }
    }
    Ok(characters)
}

fn parse_character(content: &str) -> Option<Character> {
    let mut name = String::new();
    let mut fields: HashMap<String, String> = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            name = trimmed[3..].trim().to_string();
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let kv = &trimmed[2..];
            if let Some((k, v)) = kv.split_once(": ") {
                fields.insert(k.trim().to_string(), v.trim().to_string());
            } else if let Some((k, v)) = kv.split_once("：") {
                fields.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(Character {
        name,
        identity: fields.remove("身份").unwrap_or_default(),
        personality: fields.remove("性格").unwrap_or_default(),
        abilities: fields.remove("能力").unwrap_or_default(),
        relations: fields.remove("关系").unwrap_or_default(),
        speech_style: fields.remove("说话风格").unwrap_or_default(),
    })
}
