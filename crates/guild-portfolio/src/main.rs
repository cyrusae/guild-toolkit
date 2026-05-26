use anyhow::Context;
use clap::{Parser, Subcommand};
use guild_core::data::{Difficulty, ProjectRegistry, ProjectStatus};
use pulldown_cmark::{Event, Parser as MarkdownParser, Tag, TagEnd};
use std::fs;
use std::path::{Path, PathBuf};

/// Generate a portfolio site from your project repos.
#[derive(Parser)]
#[command(name = "guild-portfolio", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate the portfolio site
    Build {
        /// Output directory for generated site
        #[arg(short, long, default_value = "./site")]
        output: String,
    },
    /// Remove generated output
    Clean {
        /// Output directory to clean
        #[arg(short, long, default_value = "./site")]
        output: String,
    },
}

const STYLE_CSS: &str = r#":root {
    --bg-color: #0b0f19;
    --card-bg: #151c2c;
    --border-color: #222d44;
    --text-primary: #f8fafc;
    --text-secondary: #94a3b8;
    --accent-primary: #6366f1;
    --accent-secondary: #a855f7;
    --accent-hover: #4f46e5;
    
    /* Badges */
    --status-not-started-bg: rgba(75, 85, 99, 0.1);
    --status-not-started-text: #9ca3af;
    --status-not-started-border: #4b5563;
    
    --status-in-progress-bg: rgba(249, 115, 22, 0.1);
    --status-in-progress-text: #fdba74;
    --status-in-progress-border: #ea580c;
    
    --status-under-review-bg: rgba(59, 130, 246, 0.1);
    --status-under-review-text: #93c5fd;
    --status-under-review-border: #2563eb;
    
    --status-complete-bg: rgba(16, 185, 129, 0.1);
    --status-complete-text: #6ee7b7;
    --status-complete-border: #059669;

    --diff-beginner-bg: rgba(20, 184, 166, 0.1);
    --diff-beginner-text: #5eead4;
    --diff-beginner-border: #0d9488;
    
    --diff-intermediate-bg: rgba(245, 158, 11, 0.1);
    --diff-intermediate-text: #fcd34d;
    --diff-intermediate-border: #d97706;
    
    --diff-advanced-bg: rgba(244, 63, 94, 0.1);
    --diff-advanced-text: #fda4af;
    --diff-advanced-border: #e11d48;
}

body {
    background-color: var(--bg-color);
    color: var(--text-primary);
    font-family: 'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    margin: 0;
    padding: 0;
    line-height: 1.6;
    -webkit-font-smoothing: antialiased;
}

.container {
    max-width: 1000px;
    margin: 0 auto;
    padding: 4rem 2rem;
}

.portfolio-header {
    margin-bottom: 4rem;
    text-align: center;
}

.gradient-text {
    font-size: 3.5rem;
    font-weight: 700;
    background: linear-gradient(135deg, var(--accent-primary) 0%, var(--accent-secondary) 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    margin: 0 0 1rem 0;
    letter-spacing: -0.025em;
}

.subtitle {
    color: var(--text-secondary);
    font-size: 1.2rem;
    margin: 0;
}

.handle {
    color: var(--accent-primary);
    font-weight: 600;
}

.projects-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 2rem;
    margin-bottom: 4rem;
}

.project-card {
    background-color: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 16px;
    padding: 2rem;
    display: flex;
    flex-direction: column;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;
}

.project-card:hover {
    transform: translateY(-4px);
    border-color: var(--accent-primary);
    box-shadow: 0 12px 30px -10px rgba(99, 102, 241, 0.15);
}

.project-header {
    margin-bottom: 1.5rem;
}

.project-title {
    font-size: 1.5rem;
    margin: 0 0 0.75rem 0;
    font-weight: 600;
}

.project-title a {
    color: var(--text-primary);
    text-decoration: none;
    transition: color 0.2s ease;
}

.project-title a:hover {
    color: var(--accent-primary);
}

.badges {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
}

.badge {
    display: inline-block;
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: capitalize;
    border: 1px solid transparent;
}

/* Status badge overrides */
.status-not-started {
    background-color: var(--status-not-started-bg);
    color: var(--status-not-started-text);
    border-color: var(--status-not-started-border);
}
.status-in-progress {
    background-color: var(--status-in-progress-bg);
    color: var(--status-in-progress-text);
    border-color: var(--status-in-progress-border);
}
.status-under-review {
    background-color: var(--status-under-review-bg);
    color: var(--status-under-review-text);
    border-color: var(--status-under-review-border);
}
.status-complete {
    background-color: var(--status-complete-bg);
    color: var(--status-complete-text);
    border-color: var(--status-complete-border);
}

/* Difficulty badge overrides */
.diff-beginner {
    background-color: var(--diff-beginner-bg);
    color: var(--diff-beginner-text);
    border-color: var(--diff-beginner-border);
}
.diff-intermediate {
    background-color: var(--diff-intermediate-bg);
    color: var(--diff-intermediate-text);
    border-color: var(--diff-intermediate-border);
}
.diff-advanced {
    background-color: var(--diff-advanced-bg);
    color: var(--diff-advanced-text);
    border-color: var(--diff-advanced-border);
}

.project-description {
    color: var(--text-secondary);
    font-size: 0.95rem;
    margin: 0 0 2rem 0;
    flex-grow: 1;
    display: -webkit-box;
    -webkit-line-clamp: 4;
    -webkit-box-orient: vertical;
    overflow: hidden;
}

.view-project-link {
    color: var(--accent-primary);
    text-decoration: none;
    font-weight: 600;
    font-size: 0.9rem;
    display: inline-flex;
    align-items: center;
    transition: color 0.2s ease;
}

.view-project-link:hover {
    color: var(--accent-secondary);
}

.portfolio-footer {
    text-align: center;
    color: var(--text-secondary);
    font-size: 0.85rem;
    border-top: 1px solid var(--border-color);
    padding-top: 2rem;
    margin-top: 4rem;
}

.portfolio-footer code {
    background-color: var(--card-bg);
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
}

/* Project Detail Page */
.project-page-header {
    margin-bottom: 3rem;
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 2rem;
}

.back-link {
    color: var(--text-secondary);
    text-decoration: none;
    font-size: 0.9rem;
    display: inline-block;
    margin-bottom: 1.5rem;
    transition: color 0.2s ease;
}

.back-link:hover {
    color: var(--accent-primary);
}

.project-page-title {
    font-size: 2.5rem;
    margin: 0 0 1rem 0;
    font-weight: 700;
    letter-spacing: -0.02em;
}

.project-meta {
    display: flex;
    gap: 0.75rem;
}

/* Readme Markdown Styling */
.readme-content {
    background-color: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 16px;
    padding: 3rem;
}

.readme-content h1, 
.readme-content h2, 
.readme-content h3, 
.readme-content h4 {
    color: var(--text-primary);
    margin-top: 2rem;
    margin-bottom: 1rem;
    font-weight: 600;
}

.readme-content h1 { font-size: 2rem; border-bottom: 1px solid var(--border-color); padding-bottom: 0.5rem; }
.readme-content h2 { font-size: 1.5rem; }
.readme-content h3 { font-size: 1.25rem; }

.readme-content p {
    color: var(--text-secondary);
    margin-bottom: 1.5rem;
}

.readme-content a {
    color: var(--accent-primary);
    text-decoration: none;
}

.readme-content a:hover {
    text-decoration: underline;
}

.readme-content code {
    background-color: rgba(99, 102, 241, 0.1);
    color: #a5b4fc;
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    font-family: monospace;
    font-size: 0.9em;
}

.readme-content pre {
    background-color: #0b0f19;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.5rem;
    overflow-x: auto;
    margin-bottom: 1.5rem;
}

.readme-content pre code {
    background-color: transparent;
    color: var(--text-primary);
    padding: 0;
    border-radius: 0;
    font-size: 0.85em;
}

.readme-content ul, 
.readme-content ol {
    color: var(--text-secondary);
    margin-bottom: 1.5rem;
    padding-left: 2rem;
}

.readme-content li {
    margin-bottom: 0.5rem;
}

/* Simple animations */
@keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
}

.animate-fade-in {
    animation: fadeIn 0.5s ease-out forwards;
}

@media (max-width: 768px) {
    .container {
        padding: 2rem 1rem;
    }
    .gradient-text {
        font-size: 2.5rem;
    }
    .readme-content {
        padding: 1.5rem;
    }
}
"#;

fn resolve_path(p: &str) -> PathBuf {
    if let Some(stripped) = p.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return Path::new(&home).join(stripped);
        }
    } else if p == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home);
        }
    }
    PathBuf::from(p)
}

fn slugify(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn escape_html(s: &str) -> String {
    let mut escaped = String::new();
    for c in s.chars() {
        match c {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

fn extract_first_paragraph(markdown: &str) -> String {
    let parser = MarkdownParser::new(markdown);
    let mut in_paragraph = false;
    let mut paragraph_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) if paragraph_text.is_empty() => {
                in_paragraph = true;
            }
            Event::End(TagEnd::Paragraph) if in_paragraph => {
                break;
            }
            Event::Text(text) | Event::Code(text) if in_paragraph => {
                paragraph_text.push_str(&text);
            }
            Event::SoftBreak | Event::HardBreak if in_paragraph => {
                paragraph_text.push('\n');
            }
            _ => {}
        }
    }
    paragraph_text.trim().to_string()
}

fn markdown_to_html(markdown: &str) -> String {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);
    options.insert(pulldown_cmark::Options::ENABLE_SMART_PUNCTUATION);

    let parser = MarkdownParser::new_ext(markdown, options);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    html_output
}

fn get_status_labels(status: &ProjectStatus) -> (&'static str, &'static str) {
    match status {
        ProjectStatus::NotStarted => ("not-started", "Not Started"),
        ProjectStatus::InProgress => ("in-progress", "In Progress"),
        ProjectStatus::UnderReview => ("under-review", "Under Review"),
        ProjectStatus::Complete => ("complete", "Complete"),
    }
}

fn get_difficulty_labels(diff: &Difficulty) -> (&'static str, &'static str) {
    use Difficulty::*;
    match diff {
        Beginner => ("beginner", "Beginner"),
        Intermediate => ("intermediate", "Intermediate"),
        Advanced => ("advanced", "Advanced"),
    }
}

fn run_generator(output_dir: &str) -> anyhow::Result<()> {
    // 1. Load apprentice details
    let (apprentice_name, apprentice_handle) = if let Ok(config) = guild_core::GuildConfig::load() {
        (config.user.name, config.user.handle)
    } else {
        ("Apprentice".to_string(), "apprentice".to_string())
    };

    // 2. Load project registry
    let registry = ProjectRegistry::load().context("Failed to load project registry")?;

    // 3. Create output directory
    let output_path = Path::new(output_dir);
    fs::create_dir_all(output_path)
        .with_context(|| format!("Failed to create output directory: {}", output_dir))?;

    // 4. Collect project READMEs
    let mut projects_data = Vec::new();
    for project in registry.projects {
        let resolved = resolve_path(&project.path);
        if !resolved.exists() || !resolved.is_dir() {
            guild_core::output::warn(&format!(
                "Skipping project '{}': path '{}' does not exist or is not a directory",
                project.name, project.path
            ));
            continue;
        }

        let readme_path = resolved.join("README.md");
        let readme_path_lower = resolved.join("readme.md");
        let target_readme = if readme_path.exists() && readme_path.is_file() {
            Some(readme_path)
        } else if readme_path_lower.exists() && readme_path_lower.is_file() {
            Some(readme_path_lower)
        } else {
            None
        };

        let readme_path = match target_readme {
            Some(path) => path,
            None => {
                guild_core::output::warn(&format!(
                    "Skipping project '{}': no README.md found in '{}'",
                    project.name, project.path
                ));
                continue;
            }
        };

        let content = match fs::read_to_string(&readme_path) {
            Ok(c) => c,
            Err(e) => {
                guild_core::output::warn(&format!(
                    "Skipping project '{}': failed to read '{}': {}",
                    project.name,
                    readme_path.display(),
                    e
                ));
                continue;
            }
        };

        projects_data.push((project, content));
    }

    // 5. Generate and write style.css
    let css_path = output_path.join("style.css");
    fs::write(&css_path, STYLE_CSS)
        .with_context(|| format!("Failed to write CSS stylesheet: {}", css_path.display()))?;

    // 6. Generate project detail HTML files and build project cards list
    let mut project_cards = Vec::new();
    for (project, readme_content) in &projects_data {
        let slug = slugify(&project.name);
        let project_filename = format!("{}.html", slug);
        let project_file_path = output_path.join(&project_filename);

        // Convert readme to HTML
        let readme_html = markdown_to_html(readme_content);

        // Extract first paragraph for card description
        let raw_description = extract_first_paragraph(readme_content);
        let escaped_description = if raw_description.is_empty() {
            "No description available.".to_string()
        } else {
            escape_html(&raw_description)
        };

        // Render project detail page
        let (status_class, status_label) = get_status_labels(&project.status);
        let (diff_class, diff_label) = get_difficulty_labels(&project.difficulty);

        let detail_html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{project_name} - Project Details</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@300;400;500;600;700&display=swap" rel="stylesheet">
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="container animate-fade-in">
        <header class="project-page-header">
            <a href="index.html" class="back-link">&larr; Back to Portfolio</a>
            <h1 class="project-page-title">{project_name}</h1>
            <div class="project-meta">
                <span class="badge status-badge status-{status_class}">{status_label}</span>
                <span class="badge diff-badge diff-{diff_class}">{diff_label}</span>
            </div>
        </header>

        <main class="readme-content">
            {readme_html}
        </main>

        <footer class="portfolio-footer">
            <p>Generated with <code>guild-portfolio</code></p>
        </footer>
    </div>
</body>
</html>"#,
            project_name = escape_html(&project.name),
            status_class = status_class,
            status_label = status_label,
            diff_class = diff_class,
            diff_label = diff_label,
            readme_html = readme_html
        );

        fs::write(&project_file_path, detail_html).with_context(|| {
            format!(
                "Failed to write project detail page: {}",
                project_file_path.display()
            )
        })?;

        // Format card HTML for index page
        let card_html = format!(
            r#"            <article class="project-card">
                <div class="project-header">
                    <h2 class="project-title"><a href="{project_filename}">{project_name_escaped}</a></h2>
                    <div class="badges">
                        <span class="badge status-badge status-{status_class}">{status_label}</span>
                        <span class="badge diff-badge diff-{diff_class}">{diff_label}</span>
                    </div>
                </div>
                <p class="project-description">{escaped_description}</p>
                <a class="view-project-link" href="{project_filename}">View Project README &rarr;</a>
            </article>"#,
            project_filename = project_filename,
            project_name_escaped = escape_html(&project.name),
            status_class = status_class,
            status_label = status_label,
            diff_class = diff_class,
            diff_label = diff_label,
            escaped_description = escaped_description
        );

        project_cards.push(card_html);
    }

    // 7. Generate and write index.html
    let projects_html = if project_cards.is_empty() {
        "            <div style=\"text-align: center; width: 100%; grid-column: 1 / -1; color: var(--text-secondary);\">No projects have been successfully generated yet. Check that paths in projects.toml exist and contain README.md files.</div>".to_string()
    } else {
        project_cards.join("\n")
    };

    let index_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{apprentice_name}'s Guild Portfolio</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@300;400;500;600;700&display=swap" rel="stylesheet">
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="container animate-fade-in">
        <header class="portfolio-header">
            <h1 class="gradient-text">{apprentice_name}'s Portfolio</h1>
            <p class="subtitle">A collection of apprentice projects by <span class="handle">@{apprentice_handle}</span></p>
        </header>

        <main class="projects-grid">
{projects_html}
        </main>

        <footer class="portfolio-footer">
            <p>Generated with <code>guild-portfolio</code></p>
        </footer>
    </div>
</body>
</html>"#,
        apprentice_name = escape_html(&apprentice_name),
        apprentice_handle = escape_html(&apprentice_handle),
        projects_html = projects_html
    );

    let index_path = output_path.join("index.html");
    fs::write(&index_path, index_html)
        .with_context(|| format!("Failed to write index.html: {}", index_path.display()))?;

    guild_core::output::success(&format!(
        "Successfully generated portfolio for {} projects in '{}'",
        projects_data.len(),
        output_dir
    ));

    Ok(())
}

fn run_clean(output_dir: &str) -> anyhow::Result<()> {
    let path = Path::new(output_dir);
    if path.exists() {
        if path.is_dir() {
            fs::remove_dir_all(path)
                .with_context(|| format!("Failed to clean output directory: {}", output_dir))?;
            guild_core::output::success(&format!("Successfully cleaned directory: {}", output_dir));
        } else {
            anyhow::bail!("Output path '{}' exists but is not a directory", output_dir);
        }
    } else {
        guild_core::output::info(&format!(
            "Directory '{}' does not exist, nothing to clean",
            output_dir
        ));
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build { output } => run_generator(&output),
        Commands::Clean { output } => run_clean(&output),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_resolve_path() {
        let _guard = ENV_LOCK.lock().unwrap();
        let original_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", "/users/mockapprentice");
        }

        assert_eq!(
            resolve_path("~/projects/a"),
            PathBuf::from("/users/mockapprentice/projects/a")
        );
        assert_eq!(resolve_path("~"), PathBuf::from("/users/mockapprentice"));
        assert_eq!(
            resolve_path("/absolute/path"),
            PathBuf::from("/absolute/path")
        );
        assert_eq!(
            resolve_path("relative/path"),
            PathBuf::from("relative/path")
        );

        if let Some(h) = original_home {
            unsafe {
                std::env::set_var("HOME", h);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("My Project"), "my-project");
        assert_eq!(slugify("Project-123_4.5"), "project-123-4-5");
        assert_eq!(slugify("  Special   Characters!@# "), "special-characters");
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(
            escape_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
        assert_eq!(escape_html("Hello & World"), "Hello &amp; World");
    }

    #[test]
    fn test_extract_first_paragraph() {
        let markdown = "\
# Header 1

This is the first paragraph.
It spans multiple lines.

This is the second paragraph.
";
        assert_eq!(
            extract_first_paragraph(markdown),
            "This is the first paragraph.\nIt spans multiple lines."
        );

        let markdown_no_para = "\
# Only Headers
## No Paragraphs
";
        assert_eq!(extract_first_paragraph(markdown_no_para), "");
    }

    #[test]
    fn test_pipeline_integration() {
        let _guard = ENV_LOCK.lock().unwrap();
        let original_home = std::env::var("HOME").ok();
        let dir = tempfile::tempdir().unwrap();
        let mock_home = dir.path();

        unsafe {
            std::env::set_var("HOME", mock_home);
        }

        // 1. Create guild structure
        let guild_dir = mock_home.join(".guild");
        let data_dir = guild_dir.join("data");
        std::fs::create_dir_all(&data_dir).unwrap();

        // 2. Write config.toml
        let config_path = guild_dir.join("config.toml");
        std::fs::write(
            &config_path,
            r#"[user]
name = "Jane Doe"
handle = "janedoe"
"#,
        )
        .unwrap();

        // 3. Create mock projects
        let alpha_dir = mock_home.join("project-alpha");
        let beta_dir = mock_home.join("project-beta");
        let gamma_dir = mock_home.join("project-gamma");
        std::fs::create_dir_all(&alpha_dir).unwrap();
        std::fs::create_dir_all(&beta_dir).unwrap();
        std::fs::create_dir_all(&gamma_dir).unwrap();

        // Write READMEs
        std::fs::write(
            alpha_dir.join("README.md"),
            r#"# Project Alpha

This is the description of project alpha.

Some other section.
"#,
        )
        .unwrap();

        std::fs::write(
            beta_dir.join("README.md"),
            r#"# Project Beta

This is the description of project beta.

Another section.
"#,
        )
        .unwrap();

        // 4. Write projects.toml registry
        let projects_toml = format!(
            r#"[[projects]]
name = "Project Alpha"
path = "{}"
status = "complete"
difficulty = "beginner"

[[projects]]
name = "Project Beta"
path = "{}"
status = "inprogress"
difficulty = "intermediate"

[[projects]]
name = "Project Gamma"
path = "{}"
status = "notstarted"
difficulty = "beginner"

[[projects]]
name = "Project Delta"
path = "{}"
status = "notstarted"
difficulty = "advanced"
"#,
            alpha_dir.display(),
            beta_dir.display(),
            gamma_dir.display(),
            mock_home.join("project-delta").display()
        );
        std::fs::write(data_dir.join("projects.toml"), projects_toml).unwrap();

        // 5. Run the generator pipeline
        let site_dir = mock_home.join("site");
        run_generator(&site_dir.to_string_lossy()).unwrap();

        // 6. Verify files generated
        assert!(site_dir.join("index.html").exists());
        assert!(site_dir.join("style.css").exists());
        assert!(site_dir.join("project-alpha.html").exists());
        assert!(site_dir.join("project-beta.html").exists());
        assert!(!site_dir.join("project-gamma.html").exists());
        assert!(!site_dir.join("project-delta.html").exists());

        // Verify content in index.html
        let index_content = std::fs::read_to_string(site_dir.join("index.html")).unwrap();
        assert!(index_content.contains("Jane Doe's Portfolio"));
        assert!(index_content.contains("@janedoe"));
        assert!(index_content.contains("project-alpha.html"));
        assert!(index_content.contains("Project Alpha"));
        assert!(index_content.contains("This is the description of project alpha."));
        assert!(index_content.contains("project-beta.html"));
        assert!(index_content.contains("Project Beta"));
        assert!(index_content.contains("This is the description of project beta."));

        // Verify content in project detail html
        let alpha_detail = std::fs::read_to_string(site_dir.join("project-alpha.html")).unwrap();
        assert!(alpha_detail.contains("Project Alpha"));
        assert!(alpha_detail.contains("This is the description of project alpha."));

        // Cleanup
        if let Some(h) = original_home {
            unsafe {
                std::env::set_var("HOME", h);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn test_markdown_to_html() {
        let markdown = "# Title\n\nHello **world** with `code`.";
        let html = markdown_to_html(markdown);
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("Hello <strong>world</strong> with <code>code</code>."));
    }

    #[test]
    fn test_clean_integration() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let mock_dir = dir.path().join("my-site");
        std::fs::create_dir_all(&mock_dir).unwrap();
        std::fs::write(mock_dir.join("some-file.html"), "content").unwrap();

        assert!(mock_dir.exists());
        run_clean(&mock_dir.to_string_lossy()).unwrap();
        assert!(!mock_dir.exists());
    }
}
