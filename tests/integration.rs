// Tests for the code health analyzer

use healthpulse::analyzer::{analyze, Category};
use healthpulse::config::Config;
use std::env;
use std::fs;

#[test]
fn test_analyze_empty_dir() {
    let tmp = env::temp_dir().join("healthpulse_test_empty");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    let config = Config::default();
    let result = analyze(tmp.to_str().unwrap(), &config);

    assert_eq!(result.total_files, 0);
    assert_eq!(result.total_issues, 0);
    assert_eq!(result.critical, 0);
}

#[test]
fn test_analyze_finds_long_file() {
    let tmp = env::temp_dir().join("healthpulse_test_long");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    // Create a file that exceeds max_file_length (500 lines default)
    let content: String = (0..600)
        .map(|i| format!("line {} - this is content\n", i))
        .collect();
    let file_path = tmp.join("big.go");
    fs::write(&file_path, content).unwrap();

    let mut config = Config::default();
    config.max_file_length = 500;

    let result = analyze(tmp.to_str().unwrap(), &config);

    assert_eq!(result.total_files, 1);
    
    // Debug: print all issues
    for issue in &result.issues {
        eprintln!("ISSUE: {:?} - {}", issue.category, issue.message);
    }
    
    assert!(result.total_issues > 0, "Expected at least 1 issue, got {}", result.total_issues);

    let has_length_issue = result
        .issues
        .iter()
        .any(|i| i.category == Category::Length && i.message.contains("too long"));
    assert!(
        has_length_issue,
        "Should find a length issue for big file. Issues: {:?}",
        result.issues
    );
}

#[test]
fn test_analyze_finds_complex_file() {
    let tmp = env::temp_dir().join("healthpulse_test_complex");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    let mut content = String::new();
    for i in 0..200 {
        content.push_str(&format!("if i == {} {{\n", i));
        content.push_str("    for j := 0; j < 10; j++ {\n");
    }
    content.push_str(&"}\n".repeat(200));

    let file_path = tmp.join("complex.go");
    fs::write(&file_path, content).unwrap();

    let mut config = Config::default();
    config.max_complexity = 10;

    let result = analyze(tmp.to_str().unwrap(), &config);

    assert_eq!(result.total_files, 1);

    let has_complexity_issue = result
        .issues
        .iter()
        .any(|i| i.category == Category::Complexity && i.message.contains("high complexity"));
    assert!(
        has_complexity_issue,
        "Should find a complexity issue for complex file"
    );
}

#[test]
fn test_analyze_finds_function_too_long() {
    let tmp = env::temp_dir().join("healthpulse_test_func");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    let mut content = String::from("package main\n\nfunc longFunc() {\n");
    for _ in 0..80 {
        content.push_str("    // function body line\n");
    }
    content.push_str("}\n");

    let file_path = tmp.join("long.go");
    fs::write(&file_path, content).unwrap();

    let mut config = Config::default();
    config.max_function_length = 50;

    let result = analyze(tmp.to_str().unwrap(), &config);

    assert_eq!(result.total_files, 1);

    let has_func_issue = result
        .issues
        .iter()
        .any(|i| i.category == Category::Length && i.message.contains("Function at line 3"));
    assert!(has_func_issue, "Should find a long function issue. Issues: {:?}", result.issues);
}

#[test]
fn test_analyze_respects_test_files() {
    let tmp = env::temp_dir().join("healthpulse_test_ignored");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    let content: String = (0..600)
        .map(|i| format!("func Test{}(t *testing.T) {{}}\n", i))
        .collect();
    let file_path = tmp.join("big_test.go");
    fs::write(&file_path, content).unwrap();

    let mut config = Config::default();
    config.max_file_length = 500;

    let result = analyze(tmp.to_str().unwrap(), &config);

    assert_eq!(result.total_files, 1);

    let has_length_issue = result
        .issues
        .iter()
        .any(|i| i.category == Category::Length && i.message.contains("too long"));
    assert!(
        !has_length_issue,
        "Test files should not trigger length issues. Issues: {:?}",
        result.issues
    );
}

#[test]
fn test_analyze_ignores_git_dir() {
    let tmp = env::temp_dir().join("healthpulse_test_git");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    fs::write(tmp.join("main.go"), "package main\n\nfunc main() {}\n").unwrap();
    let git_dir = tmp.join(".git");
    fs::create_dir(&git_dir).unwrap();
    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();

    let config = Config::default();
    let result = analyze(tmp.to_str().unwrap(), &config);

    assert_eq!(result.total_files, 1);
    assert!(!result.issues.iter().any(|i| i.file.contains(".git")));
}

#[test]
fn test_file_stats_collected() {
    let tmp = env::temp_dir().join("healthpulse_test_stats");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    fs::write(tmp.join("main.go"), "package main\n\nfunc main() {}\n").unwrap();

    let config = Config::default();
    let result = analyze(tmp.to_str().unwrap(), &config);

    assert_eq!(result.total_files, 1);
    assert_eq!(result.total_lines, 3);
    assert!(result.file_stats.keys().any(|k| k.ends_with("main.go")));
}

#[test]
fn test_config_load() {
    let tmp = env::temp_dir().join("healthpulse_test_config");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    let config_content = r#"{
        "max_complexity": 20,
        "max_function_length": 100,
        "max_file_length": 1000,
        "min_test_ratio": 0.5,
        "ignore": ["vendor"]
    }"#;

    let config_path = tmp.join("config.json");
    fs::write(&config_path, config_content).unwrap();

    let mut config = Config::default();
    config.load(config_path.to_str().unwrap()).unwrap();

    assert_eq!(config.max_complexity, 20);
    assert_eq!(config.max_function_length, 100);
    assert_eq!(config.max_file_length, 1000);
    assert_eq!(config.min_test_ratio, 0.5);
    assert!(config.ignore.contains(&"vendor".to_string()));
}

#[test]
fn test_config_ignore_pattern() {
    let config = Config {
        ignore: vec!["target".to_string(), "node_modules".to_string()],
        ..Default::default()
    };

    assert!(config.should_ignore(std::path::Path::new("target/main.go")));
    assert!(config.should_ignore(std::path::Path::new("node_modules/pkg/main.js")));
    assert!(!config.should_ignore(std::path::Path::new("src/main.go")));
}

#[test]
fn test_test_ratio_warning() {
    let tmp = env::temp_dir().join("healthpulse_test_ratio");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    for i in 0..10 {
        fs::write(tmp.join(format!("source{}.go", i)), "package main\n").unwrap();
    }

    let mut config = Config::default();
    config.min_test_ratio = 0.3;

    let result = analyze(tmp.to_str().unwrap(), &config);

    let has_test_warning = result
        .issues
        .iter()
        .any(|i| i.category == Category::TestCoverage);
    assert!(
        has_test_warning,
        "Should warn about low test ratio. Issues: {:?}",
        result.issues
    );
}