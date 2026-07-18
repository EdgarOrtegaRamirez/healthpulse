// Output formatting for healthpulse analyzer results

use crate::analyzer::{AnalysisResult, Category, Severity};
use crate::config::Config;
use colored::Colorize;

pub enum Output {
    Text,
    Json,
    Markdown,
}

impl Output {
    pub fn render(&self, result: &AnalysisResult, _config: &Config) -> String {
        match self {
            Output::Text => self.render_text(result),
            Output::Json => self.render_json(result),
            Output::Markdown => self.render_markdown(result),
        }
    }

    fn render_text(&self, result: &AnalysisResult) -> String {
        let mut output = String::new();

        // Header
        output.push_str("╔══════════════════════════════════════════════╗\n");
        output.push_str("║           Code Health Report                  ║\n");
        output.push_str("╚══════════════════════════════════════════════╝\n\n");

        // Summary
        output.push_str(&format!("  Files scanned:    {}\n", result.total_files));
        output.push_str(&format!("  Total lines:      {}\n\n", result.total_lines));

        // Issues summary
        output.push_str("  Issues:\n");
        output.push_str(&format!(
            "    🚨 Critical:  {}\n",
            result.critical.to_string().red().bold()
        ));
        output.push_str(&format!(
            "    ⚠️  Warning:  {}\n",
            result.warnings.to_string().yellow()
        ));
        output.push_str(&format!(
            "    ℹ️  Info:     {}\n\n",
            result.infos.to_string().cyan()
        ));

        if result.total_issues == 0 {
            output.push_str("  ✅ No issues found! Great code health!\n\n");
        } else {
            // Group issues by category
            let complexity_issues: Vec<_> = result
                .issues
                .iter()
                .filter(|i| i.category == Category::Complexity)
                .collect();
            let length_issues: Vec<_> = result
                .issues
                .iter()
                .filter(|i| i.category == Category::Length)
                .collect();
            let test_issues: Vec<_> = result
                .issues
                .iter()
                .filter(|i| i.category == Category::TestCoverage)
                .collect();

            if !complexity_issues.is_empty() {
                output.push_str(&format!("  📊 Complexity ({})\n", complexity_issues.len()));
                for issue in &complexity_issues[..complexity_issues.len().min(5)] {
                    let line_str = issue
                        .line
                        .map(|l| format!(" (line {})", l))
                        .unwrap_or_default();
                    output.push_str(&format!("    - {}{}\n", issue.message, line_str));
                }
                output.push('\n');
            }

            if !length_issues.is_empty() {
                output.push_str(&format!("  📏 Length ({})\n", length_issues.len()));
                for issue in &length_issues[..length_issues.len().min(5)] {
                    let line_str = issue
                        .line
                        .map(|l| format!(" (line {})", l))
                        .unwrap_or_default();
                    output.push_str(&format!("    - {}{}\n", issue.message, line_str));
                }
                output.push('\n');
            }

            if !test_issues.is_empty() {
                output.push_str(&format!("  🧪 Tests ({})\n", test_issues.len()));
                for issue in &test_issues {
                    output.push_str(&format!("    - {}\n", issue.message));
                }
                output.push('\n');
            }
        }

        // File table
        output.push_str("  Files analyzed:\n");
        output.push_str(&format!(
            "    {:<45} {:>8} {:>8}\n",
            "File", "Lines", "Complexity"
        ));
        output.push_str(&format!("    {:─<45} {:─<8} {:─<8}\n", "", "", ""));

        let mut files: Vec<_> = result.file_stats.iter().collect();
        files.sort_by_key(|a| std::cmp::Reverse(a.1.lines));

        for (path, stats) in &files[..files.len().min(20)] {
            output.push_str(&format!(
                "    {:<45} {:>8} {:>8}\n",
                truncate(path, 45),
                stats.lines,
                stats.complexity
            ));
        }

        output
    }

    fn render_json(&self, result: &AnalysisResult) -> String {
        let json = serde_json::json!({
            "files_scanned": result.total_files,
            "total_lines": result.total_lines,
            "issues": {
                "total": result.total_issues,
                "critical": result.critical,
                "warnings": result.warnings,
                "info": result.infos,
            },
            "issues_list": result.issues.iter().map(|i| serde_json::json!({
                "file": i.file,
                "severity": match i.severity {
                    Severity::Critical => "critical",
                    Severity::Warning => "warning",
                    Severity::Info => "info",
                },
                "category": format!("{:?}", i.category).to_lowercase(),
                "message": i.message,
                "line": i.line,
            })).collect::<Vec<_>>(),
            "files": result.file_stats.iter().map(|(path, stats)| serde_json::json!({
                "path": path,
                "lines": stats.lines,
                "functions": stats.functions,
                "complexity": stats.complexity,
            })).collect::<Vec<_>>(),
        });

        serde_json::to_string_pretty(&json).unwrap_or_default()
    }

    fn render_markdown(&self, result: &AnalysisResult) -> String {
        let mut output = String::new();

        output.push_str("# Code Health Report\n\n");

        output.push_str("## Summary\n\n");
        output.push_str("| Metric | Value |\n|--------|-------|\n");
        output.push_str(&format!("| Files scanned | {} |\n", result.total_files));
        output.push_str(&format!("| Total lines | {} |\n", result.total_lines));
        output.push_str(&format!("| Critical issues | {} |\n", result.critical));
        output.push_str(&format!("| Warnings | {} |\n", result.warnings));
        output.push_str(&format!("| Info | {} |\n", result.infos));

        if !result.issues.is_empty() {
            output.push_str("\n## Issues\n\n");

            let complexity_issues: Vec<_> = result
                .issues
                .iter()
                .filter(|i| i.category == Category::Complexity)
                .collect();
            if !complexity_issues.is_empty() {
                output.push_str("### Complexity\n\n");
                output.push_str("| File | Message | Line |\n");
                output.push_str("|------|---------|------|\n");
                for issue in &complexity_issues {
                    let line = issue.line.map(|l| l.to_string()).unwrap_or_default();
                    output.push_str(&format!(
                        "| `{}` | {} | {} |\n",
                        issue.file, issue.message, line
                    ));
                }
                output.push('\n');
            }

            let length_issues: Vec<_> = result
                .issues
                .iter()
                .filter(|i| i.category == Category::Length)
                .collect();
            if !length_issues.is_empty() {
                output.push_str("### Length\n\n");
                output.push_str("| File | Message | Line |\n");
                output.push_str("|------|---------|------|\n");
                for issue in &length_issues {
                    let line = issue.line.map(|l| l.to_string()).unwrap_or_default();
                    output.push_str(&format!(
                        "| `{}` | {} | {} |\n",
                        issue.file, issue.message, line
                    ));
                }
                output.push('\n');
            }
        }

        output
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("...{}", &s[s.len() - max_len + 4..])
    } else {
        s.to_string()
    }
}
