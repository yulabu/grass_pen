use anyhow::{Context, Result};
use std::path::Path;

use crate::llm::LlmClient;
use crate::story::{
    Character, Chapter, StoryConfig, append_summary, get_next_chapter_number,
    read_chapter, read_chapter_outline, read_characters, read_config, read_summary,
    write_chapter,
};

pub struct WriterAgent {
    llm: LlmClient,
}

impl WriterAgent {
    pub fn new(llm: LlmClient) -> Self {
        Self { llm }
    }

    pub async fn generate_chapter(&self, story_path: &Path, chapter_num: u32) -> Result<Chapter> {
        let config = read_config(story_path)?;
        let characters = read_characters(story_path)?;
        let summary = read_summary(story_path)?;
        let outline = read_chapter_outline(story_path, chapter_num)?;

        let chapter_title = outline
            .as_ref()
            .map(|o| o.title.clone())
            .unwrap_or_else(|| format!("第{}章", chapter_num));

        let outline_points = outline
            .as_ref()
            .map(|o| o.points.clone())
            .unwrap_or_default();

        let prev_chapter = if chapter_num > 1 {
            read_chapter(story_path, chapter_num - 1)?
        } else {
            None
        };

        let prev_prev_chapter = if chapter_num > 2 {
            read_chapter(story_path, chapter_num - 2)?
        } else {
            None
        };

        let system_prompt = build_system_prompt(&config, &characters);
        let user_prompt = build_user_prompt(
            &chapter_title,
            chapter_num,
            &summary,
            prev_prev_chapter.as_ref(),
            prev_chapter.as_ref(),
            &outline_points,
        );

        let content = self
            .llm
            .generate_chapter(&system_prompt, &user_prompt)
            .await
            .context("Failed to generate chapter")?;

        let chapter = Chapter {
            number: chapter_num,
            title: chapter_title,
            content,
        };

        write_chapter(story_path, &chapter)?;

        let chapter_summary = self
            .llm
            .generate_summary(&chapter.content)
            .await
            .unwrap_or_else(|e| {
                eprintln!("Warning: failed to generate summary: {}", e);
                String::from("(summary generation failed)")
            });

        append_summary(
            story_path,
            &format!("第{}章 {}: {}", chapter_num, chapter.title, chapter_summary),
        )?;

        Ok(chapter)
    }

    pub async fn generate_next(&self, story_path: &Path) -> Result<Chapter> {
        let next = get_next_chapter_number(story_path);
        self.generate_chapter(story_path, next).await
    }
}

fn build_system_prompt(config: &StoryConfig, characters: &[Character]) -> String {
    let mut prompt = String::new();

    prompt.push_str("你是专业的小说作家。严格按照以下设定写作，不得偏离。\n\n");
    prompt.push_str("## 作品设定\n\n");
    prompt.push_str(&format!("- 书名：{}\n", config.title));
    prompt.push_str(&format!("- 体裁：{}\n", config.genre));
    prompt.push_str(&format!("- 风格：{}\n", config.style));
    prompt.push_str(&format!("- 视角：{}\n\n", config.pov));

    prompt.push_str("## 世界观\n\n");
    prompt.push_str(&config.world_setting);
    prompt.push_str("\n\n");

    prompt.push_str("## 人物设定\n\n");
    for c in characters {
        prompt.push_str(&format!("### {}\n", c.name));
        if !c.identity.is_empty() {
            prompt.push_str(&format!("- 身份：{}\n", c.identity));
        }
        if !c.personality.is_empty() {
            prompt.push_str(&format!("- 性格：{}\n", c.personality));
        }
        if !c.abilities.is_empty() {
            prompt.push_str(&format!("- 能力：{}\n", c.abilities));
        }
        if !c.relations.is_empty() {
            prompt.push_str(&format!("- 关系：{}\n", c.relations));
        }
        if !c.speech_style.is_empty() {
            prompt.push_str(&format!("- 说话风格：{}\n", c.speech_style));
        }
        prompt.push('\n');
    }

    prompt.push_str("## 硬性规则\n\n");
    prompt.push_str("1. 不允许新增上述人物设定中没有的具名角色。路人用\"路人甲\"\"店小二\"等泛称。\n");
    prompt.push_str("2. 不允许修改或新增世界观设定。\n");
    prompt.push_str(&format!(
        "3. 全程保持{}视角，不得跳转。\n",
        config.pov
    ));
    prompt.push_str("4. 每个角色的说话风格必须严格匹配人物设定。\n");
    prompt.push_str("5. 情节严格遵循本章大纲，不得自行添加大纲外的重大事件。\n");
    prompt.push_str("6. 不要写\"本章完\"或以任何方式标榜结束。\n");
    prompt.push_str("7. 对话、动作、心理描写均衡，避免纯对话或纯叙述。\n");

    prompt
}

fn build_user_prompt(
    chapter_title: &str,
    chapter_num: u32,
    summary: &str,
    prev_prev: Option<&Chapter>,
    prev: Option<&Chapter>,
    outline_points: &[String],
) -> String {
    let mut prompt = String::new();

    if !summary.is_empty() {
        prompt.push_str("## 已写章节摘要\n\n");
        prompt.push_str(summary);
        prompt.push_str("\n\n");
    }

    if let Some(ch) = prev_prev {
        prompt.push_str("## 最近章节原文\n\n");
        prompt.push_str(&format!("### 第{}章 {}\n\n", ch.number, ch.title));
        prompt.push_str(&ch.content);
        prompt.push_str("\n\n---\n\n");
    }

    if let Some(ch) = prev {
        if prev_prev.is_none() {
            prompt.push_str("## 最近章节原文\n\n");
        }
        prompt.push_str(&format!("### 第{}章 {}\n\n", ch.number, ch.title));
        prompt.push_str(&ch.content);
        prompt.push_str("\n\n---\n\n");
    }

    if !outline_points.is_empty() {
        prompt.push_str("## 本章大纲\n\n");
        for point in outline_points {
            prompt.push_str(&format!("- {}\n", point));
        }
        prompt.push('\n');
    }

    prompt.push_str(&format!(
        "请写出第{}章《{}》的完整正文。",
        chapter_num, chapter_title
    ));

    prompt
}
