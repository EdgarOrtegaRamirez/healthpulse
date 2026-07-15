// Code health analyzer - scans files for issues

use std::collections::HashMap;

use walkdir::WalkDir;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct Issue {
    pub file: String,
    pub severity: Severity,
    pub category: Category,
    pub message: String,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Category {
    Complexity,
    Length,
    Style,
    TestCoverage,
    Maintainability,
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub total_files: usize,
    pub total_lines: u64,
    pub total_issues: usize,
    pub critical: usize,
    pub warnings: usize,
    pub infos: usize,
    pub issues: Vec<Issue>,
    pub file_stats: HashMap<String, FileStats>,
}

#[derive(Debug, Clone)]
pub struct FileStats {
    pub lines: u64,
    pub functions: u64,
    pub complexity: u64,
    pub test_match: bool,
}

pub fn analyze(path: &str, config: &Config) -> AnalysisResult {
    let mut result = AnalysisResult {
        total_files: 0,
        total_lines: 0,
        total_issues: 0,
        critical: 0,
        warnings: 0,
        infos: 0,
        issues: Vec::new(),
        file_stats: HashMap::new(),
    };

    let entries = collect_entries(path, config);

    for entry in &entries {
        let is_test = is_test_file(entry);
        let (stats, issues) = analyze_file(entry, config, is_test);

        result.total_files += 1;
        result.total_lines += stats.lines;

        for issue in &issues {
            result.total_issues += 1;
            match issue.severity {
                Severity::Critical => result.critical += 1,
                Severity::Warning => result.warnings += 1,
                Severity::Info => result.infos += 1,
            }
        }

        result.issues.extend(issues);

        result
            .file_stats
            .insert(entry.path().to_string_lossy().to_string(), stats);
    }

    // Check test coverage ratio
    check_test_ratio(&entries, config, &mut result);

    result
}

fn collect_entries(path: &str, config: &Config) -> Vec<walkdir::DirEntry> {
    WalkDir::new(path)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| !config.should_ignore(e.path()))
        .filter(|e| !should_skip_dir(&e.path().to_string_lossy()))
        .collect()
}

fn should_skip_dir(path: &str) -> bool {
    let skip_dirs = [
        ".git",
        "node_modules",
        "vendor",
        "target",
        "dist",
        ".cache",
        ".tox",
    ];
    skip_dirs.iter().any(|skip| path.contains(skip))
}

fn is_test_file(entry: &walkdir::DirEntry) -> bool {
    let path = entry.path();
    let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let parent_dir = path.parent().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default();

    // Check if the file name itself matches test patterns
    let is_test_name = file_name.contains("_test")
        || file_name.contains("test_")
        || file_name.starts_with("test-");

    // Check if in a directory named exactly "tests"
    let is_in_tests_dir = parent_dir == "tests";

    is_test_name || is_in_tests_dir
}

fn analyze_file(
    entry: &walkdir::DirEntry,
    config: &Config,
    is_test: bool,
) -> (FileStats, Vec<Issue>) {
    let path = entry.path();
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            return (
                FileStats {
                    lines: 0,
                    functions: 0,
                    complexity: 0,
                    test_match: is_test,
                },
                Vec::new(),
            );
        }
    };

    let lines: Vec<_> = content.lines().collect();
    let line_count = lines.len() as u64;

    let mut issues = Vec::new();

    // Check file length
    if line_count > config.max_file_length as u64 && !is_test {
        issues.push(Issue {
            file: path.to_string_lossy().to_string(),
            severity: Severity::Warning,
            category: Category::Length,
            message: format!(
                "File is too long ({} lines, max {})",
                line_count, config.max_file_length
            ),
            line: None,
        });
    }

    let function_count = count_functions(&content);
    let complexity = estimate_complexity(&content);

    // Check function lengths (Go-specific)
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("func ") {
            if let Some(end) = find_function_end(&lines[i..]) {
                let func_length = end + 1;
                if func_length > config.max_function_length as usize && !is_test {
                    issues.push(Issue {
                        file: path.to_string_lossy().to_string(),
                        severity: Severity::Info,
                        category: Category::Length,
                        message: format!(
                            "Function at line {} is {} lines (max {})",
                            i + 1,
                            func_length,
                            config.max_function_length
                        ),
                        line: Some(i + 1),
                    });
                }
            }
        }
    }

    // Also check Python function definitions
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("def ") {
            if let Some(end) = find_python_function_end(&lines[i..]) {
                let func_length = end + 1;
                if func_length > config.max_function_length as usize && !is_test {
                    issues.push(Issue {
                        file: path.to_string_lossy().to_string(),
                        severity: Severity::Info,
                        category: Category::Length,
                        message: format!(
                            "Function at line {} is {} lines (max {})",
                            i + 1,
                            func_length,
                            config.max_function_length
                        ),
                        line: Some(i + 1),
                    });
                }
            }
        }
    }

    // Check high complexity
    if complexity > config.max_complexity as u64 * 5 {
        issues.push(Issue {
            file: path.to_string_lossy().to_string(),
            severity: Severity::Warning,
            category: Category::Complexity,
            message: format!("File has high complexity ({})", complexity),
            line: None,
        });
    }

    let stats = FileStats {
        lines: line_count,
        functions: function_count as u64,
        complexity,
        test_match: is_test,
    };

    (stats, issues)
}

fn count_functions(content: &str) -> usize {
    content
        .lines()
        .filter(|l| l.trim().starts_with("func "))
        .count()
}

fn estimate_complexity(content: &str) -> u64 {
    let mut complexity: u64 = 1;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("if ")
            || trimmed.starts_with("else if ")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("case ")
            || trimmed.starts_with("&&")
            || trimmed.starts_with("||")
            || trimmed.starts_with("switch")
        {
            complexity += 1;
        }
    }
    complexity
}

fn find_function_end(lines: &[&str]) -> Option<usize> {
    let mut brace_count = 0;
    let mut started = false;

    for (i, line) in lines.iter().enumerate() {
        for ch in line.chars() {
            if ch == '{' {
                brace_count += 1;
                started = true;
            } else if ch == '}' {
                brace_count -= 1;
            }
        }

        if started && brace_count <= 0 {
            return Some(i);
        }
    }

    None
}

fn find_python_function_end(lines: &[&str]) -> Option<usize> {
    if lines.is_empty() {
        return None;
    }

    let mut indent_level: Option<usize> = None;

    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            continue;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if i == 1 {
            // Determine the function's base indent from the def line
            let base_spaces = lines[0].len() - lines[0].trim_start().len();
            indent_level = Some(base_spaces);
        }

        if let Some(base) = indent_level {
            let current_spaces = line.len() - line.trim_start().len();
            if current_spaces <= base && !trimmed.starts_with('#') {
                return Some(i - 1);
            }
        }
    }

    Some(lines.len() - 1)
}

fn check_test_ratio(
    entries: &[walkdir::DirEntry],
    config: &Config,
    result: &mut AnalysisResult,
) {
    let total_files: usize = entries.iter().filter(|e| !is_test_file(e)).count();
    let test_files: usize = entries.iter().filter(|e| is_test_file(e)).count();

    if total_files > 0 && (test_files as f64 / total_files as f64) < config.min_test_ratio {
        result.issues.push(Issue {
            file: String::new(),
            severity: Severity::Warning,
            category: Category::TestCoverage,
            message: format!(
                "Test ratio is low ({:.0}%, recommended >= {:.0}%)",
                (test_files as f64 / total_files as f64) * 100.0,
                config.min_test_ratio * 100.0
            ),
            line: None,
        });
    }
}