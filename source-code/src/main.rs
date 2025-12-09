use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::{tempdir, TempDir};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: translator <hacker_file> [--verbose]");
        std::process::exit(1);
    }
    let file_path = &args[1];
    let verbose = args.len() > 2 && args[2] == "--verbose";
    let content = fs::read_to_string(file_path)?;
    let blocks = extract_blocks(&content, verbose);
    for (lang, code) in blocks {
        if verbose {
            println!("Executing {} code:\n{}", lang, code);
        }
        match execute_code(&lang, &code, verbose) {
            Ok(output) => println!("[{}] Output:\n{}", lang, output),
            Err(e) => eprintln!("[{}] Error: {}", lang, e),
        }
    }
    Ok(())
}

fn extract_blocks(content: &str, verbose: bool) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with("|> translator:") {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let lang = parts[1].trim().split('(').next().unwrap_or("").trim().to_string();
                if !lang.is_empty() {
                    let mut code = String::new();
                    i += 1;
                    let mut depth = 1;
                    while i < lines.len() && depth > 0 {
                        let code_line = lines[i];
                        for c in code_line.chars() {
                            if c == '(' {
                                depth += 1;
                            } else if c == ')' {
                                depth -= 1;
                            }
                        }
                        code.push_str(code_line);
                        code.push('\n');
                        i += 1;
                    }
                    if depth == 0 {
                        let code_trimmed = code.trim().to_string();
                        blocks.push((lang, code_trimmed));
                        if verbose {
                            println!("Extracted {} block", lang);
                        }
                    } else {
                        if verbose {
                            eprintln!("Unclosed block for {}", lang);
                        }
                    }
                    continue;
                }
            }
        }
        i += 1;
    }
    blocks
}

fn execute_code(lang: &str, code: &str, verbose: bool) -> Result<String, Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    if verbose {
        println!("Temp dir: {:?}", dir.path());
    }
    match lang.as_str() {
        "rust" => execute_rust(code, &dir, verbose),
        "java" => execute_java(code, &dir, verbose),
        "python" => execute_python(code, verbose),
        "go" => execute_go(code, &dir, verbose),
        _ => Err(format!("Unsupported language: {}", lang).into()),
    }
}

fn execute_rust(code: &str, dir: &TempDir, verbose: bool) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = dir.path().join("main.rs");
    fs::write(&file_path, code)?;
    let output = Command::new("rustc")
        .arg(&file_path)
        .arg("-o")
        .arg(dir.path().join("a.out"))
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string().into());
    }
    let run_output = Command::new(dir.path().join("a.out"))
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    if run_output.status.success() {
        Ok(String::from_utf8_lossy(&run_output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&run_output.stderr).to_string().into())
    }
}

fn execute_java(code: &str, dir: &TempDir, verbose: bool) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = dir.path().join("Main.java");
    fs::write(&file_path, code)?;
    let output = Command::new("javac")
        .arg(&file_path)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string().into());
    }
    let run_output = Command::new("java")
        .arg("-cp")
        .arg(dir.path())
        .arg("Main")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    if run_output.status.success() {
        Ok(String::from_utf8_lossy(&run_output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&run_output.stderr).to_string().into())
    }
}

fn execute_python(code: &str, verbose: bool) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("python")
        .arg("-c")
        .arg(code)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string().into())
    }
}

fn execute_go(code: &str, dir: &TempDir, verbose: bool) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = dir.path().join("main.go");
    fs::write(&file_path, code)?;
    let output = Command::new("go")
        .arg("run")
        .arg(&file_path)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string().into())
    }
}
