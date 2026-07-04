use anyhow::{Context, Result};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GeneralOutline {
    pub title: String,
    pub volumes: Vec<Volume>,
}

#[derive(Debug, Clone)]
pub struct Volume {
    pub title: String,
    pub chapters: Vec<ChapterRef>,
}

#[derive(Debug, Clone)]
pub struct ChapterRef {
    pub number: u32,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct OutlineItem {
    pub chapter_number: u32,
    pub title: String,
    pub points: Vec<String>,
}

pub fn read_general_outline(story_path: &Path) -> Result<GeneralOutline> {
    let path = story_path.join("总纲.md");
    let content =
        std::fs::read_to_string(&path).context("总纲.md not found")?;

    let mut title = String::new();
    let mut volumes = Vec::new();
    let mut current_volume: Option<Volume> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
            title = trimmed[2..].trim().replace(" 总纲", "").to_string();
        } else if trimmed.starts_with("## ") {
            if let Some(vol) = current_volume.take() {
                volumes.push(vol);
            }
            current_volume = Some(Volume {
                title: trimmed[3..].trim().to_string(),
                chapters: Vec::new(),
            });
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let text = &trimmed[2..];
            if let Some((chapter_part, title_part)) = text.split_once(' ') {
                let num = chapter_part
                    .trim_start_matches("第")
                    .trim_end_matches("章")
                    .trim()
                    .parse::<u32>();
                if let Ok(num) = num {
                    if let Some(vol) = &mut current_volume {
                        vol.chapters.push(ChapterRef {
                            number: num,
                            title: title_part.trim().to_string(),
                        });
                    }
                }
            }
        }
    }

    if let Some(vol) = current_volume {
        volumes.push(vol);
    }

    Ok(GeneralOutline { title, volumes })
}

pub fn read_chapter_outline(story_path: &Path, chapter_num: u32) -> Result<Option<OutlineItem>> {
    let outlines_dir = story_path.join("outlines");
    if !outlines_dir.exists() {
        return Ok(None);
    }

    for entry in std::fs::read_dir(&outlines_dir)? {
        let entry = entry?;
        let path = entry.path();
        let filename = path.file_stem().unwrap_or_default().to_string_lossy();

        let parts: Vec<&str> = filename.splitn(2, '-').collect();
        if parts.is_empty() {
            continue;
        }
        let file_num = parts[0].parse::<u32>().unwrap_or(0);
        if file_num != chapter_num {
            continue;
        }

        let title = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
        let content = std::fs::read_to_string(&path)?;
        let points: Vec<String> = content
            .lines()
            .filter(|l| {
                let t = l.trim();
                t.starts_with("- ") || t.starts_with("* ")
            })
            .map(|l| l.trim()[2..].to_string())
            .filter(|p| !p.is_empty())
            .collect();

        return Ok(Some(OutlineItem {
            chapter_number: chapter_num,
            title,
            points,
        }));
    }

    Ok(None)
}

pub fn get_last_chapter_number(outline: &GeneralOutline) -> u32 {
    outline
        .volumes
        .iter()
        .flat_map(|v| v.chapters.iter())
        .map(|c| c.number)
        .max()
        .unwrap_or(0)
}
