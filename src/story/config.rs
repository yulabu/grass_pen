use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct StoryConfig {
    pub title: String,
    pub genre: String,
    pub style: String,
    pub pov: String,
    pub world_setting: String,
}

pub fn read_config(story_path: &Path) -> Result<StoryConfig> {
    let content = std::fs::read_to_string(story_path.join("config.md"))?;
    let mut map: HashMap<String, String> = HashMap::new();

    let sections: Vec<&str> = content.split("\n### ").collect();
    for section in sections {
        let trimmed = section.trim().trim_start_matches("### ");
        if let Some((key, value)) = trimmed.split_once('\n') {
            map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    Ok(StoryConfig {
        title: map.remove("书名").unwrap_or_default(),
        genre: map.remove("体裁").unwrap_or_default(),
        style: map.remove("风格").unwrap_or_default(),
        pov: map.remove("视角").unwrap_or_default(),
        world_setting: map.remove("世界观").unwrap_or_default(),
    })
}
