mod agent;
mod llm;
mod story;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use agent::WriterAgent;
use llm::LlmClient;

#[derive(Parser)]
#[command(name = "grass_pen", about = "小说写作 Agent")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化一个新故事
    Story {
        #[command(subcommand)]
        cmd: StoryCmd,
    },
    /// 生成章节
    Generate {
        /// 故事名称 (stories/ 下的文件夹名)
        story: String,
        /// 指定章节号，不指定则自动续写下一章
        #[arg(short, long)]
        chapter: Option<u32>,
    },
    /// 查看故事信息
    Info {
        /// 故事名称
        story: String,
    },
}

#[derive(Subcommand)]
enum StoryCmd {
    /// 初始化故事模板
    Init { name: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Story { cmd } => match cmd {
            StoryCmd::Init { name } => {
                cmd_init_story(&name)?;
                println!("故事 {} 初始化完成", name);
            }
        },
        Commands::Generate { story, chapter } => {
            let api_key = std::env::var("OPENCODE_API_KEY").expect(
                "请设置 OPENCODE_API_KEY 环境变量或在项目根目录创建 .env 文件:\n  OPENCODE_API_KEY=sk_xxx",
            );
            let llm = LlmClient::new(&api_key);
            let agent = WriterAgent::new(llm);
            let story_path = PathBuf::from("stories").join(&story);
            if !story_path.exists() {
                anyhow::bail!("故事 {} 不存在，请先运行 `grass_pen story init {}`", story, story);
            }

            let chapter = if let Some(num) = chapter {
                agent.generate_chapter(&story_path, num).await?
            } else {
                agent.generate_next(&story_path).await?
            };

            let path = story_path
                .join("chapters")
                .join(format!("{:03}-{}.md", chapter.number, chapter.title));
            println!("第{}章已生成: {}", chapter.number, path.display());
        }
        Commands::Info { story } => {
            let story_path = PathBuf::from("stories").join(&story);
            if !story_path.exists() {
                anyhow::bail!("故事 {} 不存在", story);
            }

            let config = story::read_config(&story_path)?;
            let chapters = story::list_chapters(&story_path)?;
            let outline = story::read_general_outline(&story_path).ok();
            let characters = story::read_characters(&story_path)?;

            println!("书名: {}", config.title);
            println!("体裁: {}", config.genre);
            println!("风格: {}", config.style);
            println!("视角: {}", config.pov);
            println!("已写章节: {} 章", chapters.len());
            if let Some(ol) = &outline {
                let total: u32 = ol.volumes.iter().flat_map(|v| &v.chapters).count() as u32;
                println!("大纲规划: {} 章", total);
            }
            println!("人物: {} 个", characters.len());
            for c in &characters {
                println!("  - {}", c.name);
            }
        }
    }

    Ok(())
}

fn cmd_init_story(name: &str) -> Result<()> {
    use std::fs;

    let story_path = PathBuf::from("stories").join(name);
    fs::create_dir_all(story_path.join("characters"))?;
    fs::create_dir_all(story_path.join("outlines"))?;
    fs::create_dir_all(story_path.join("chapters"))?;

    let config = format!(
        "\
### 书名
{name}

### 体裁


### 风格


### 视角
第三人称

### 世界观

"
    );
    fs::write(story_path.join("config.md"), config)?;

    fs::write(
        story_path.join("总纲.md"),
        format!("# {name} 总纲\n\n## 第一卷\n\n"),
    )
    .unwrap_or_else(|_| {
        eprintln!("警告: 无法创建总纲文件，请检查是否有写入权限。");
    });

    fs::write(story_path.join("summary.md"), "")?;
    fs::write(
        story_path.join("伏笔管理.md"),
        "## 待植入\n\n## 已植入\n",
    )?;

    Ok(())
}
