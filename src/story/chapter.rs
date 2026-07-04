use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Chapter {
    pub number: u32,
    pub title: String,
    pub content: String,
}

#[derive(Debug)]
pub struct ChapterMeta {
    pub number: u32,
    pub title: String,
    pub path: PathBuf,
}

pub fn list_chapters(story_path: &Path) -> Result<Vec<ChapterMeta>> {
    let chapters_dir = story_path.join("chapters");
    if !chapters_dir.exists() {
        return Ok(Vec::new());
    }

    let mut chapters = Vec::new();
    for entry in fs::read_dir(&chapters_dir)? {
        let entry = entry?;
        let path = entry.path();
        let filename = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let parts: Vec<&str> = filename.splitn(2, '-').collect();
        if parts.is_empty() {
            continue;
        }
        let number = parts[0].parse::<u32>().unwrap_or(0);
        let title = parts.get(1).unwrap_or(&"").to_string();

        chapters.push(ChapterMeta {
            number,
            title,
            path,
        });
    }
    chapters.sort_by_key(|c| c.number);
    Ok(chapters)
}

pub fn read_chapter(story_path: &Path, chapter_num: u32) -> Result<Option<Chapter>> {
    let chapters_dir = story_path.join("chapters");
    if !chapters_dir.exists() {
        return Ok(None);
    }

    for entry in fs::read_dir(&chapters_dir)? {
        let entry = entry?;
        let path = entry.path();
        let filename = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let parts: Vec<&str> = filename.splitn(2, '-').collect();
        if parts.is_empty() {
            continue;
        }
        let num = parts[0].parse::<u32>().unwrap_or(0);
        if num != chapter_num {
            continue;
        }

        let title = parts.get(1).unwrap_or(&"").to_string();
        let content = fs::read_to_string(&path)?;
        return Ok(Some(Chapter {
            number: chapter_num,
            title,
            content,
        }));
    }
    Ok(None)
}

pub fn write_chapter(story_path: &Path, chapter: &Chapter) -> Result<()> {
    let chapters_dir = story_path.join("chapters");
    fs::create_dir_all(&chapters_dir)?;

    let number_str = format!("{:03}", chapter.number);
    let filename = format!("{}-{}.md", number_str, chapter.title);
    let path = chapters_dir.join(&filename);

    fs::write(&path, &chapter.content)?;
    Ok(())
}

pub fn read_summary(story_path: &Path) -> Result<String> {
    let path = story_path.join("summary.md");
    if !path.exists() {
        return Ok(String::new());
    }
    fs::read_to_string(&path).context("Failed to read summary.md")
}

pub fn append_summary(story_path: &Path, new_summary: &str) -> Result<()> {
    let path = story_path.join("summary.md");

    let mut existing = String::new();
    if path.exists() {
        existing = fs::read_to_string(&path)?;
    }

    let updated = format!("{}{}\n", existing, new_summary.trim());
    fs::write(&path, updated)?;
    Ok(())
}

pub fn get_next_chapter_number(story_path: &Path) -> u32 {
    match list_chapters(story_path) {
        Ok(chapters) => {
            chapters.last().map(|c| c.number + 1).unwrap_or(1)
        }
        Err(_) => 1,
    }
}
