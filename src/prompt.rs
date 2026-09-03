use std::io::{self, Write};

pub fn ask(question: &str, default: Option<&str>) -> String {
    loop {
        match default {
            Some(d) => print!("{question} [{d}]: "),
            None => print!("{question}: "),
        }
        io::stdout().flush().ok();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            return default.unwrap_or("").to_string();
        }
        let line = line.trim();
        if line.is_empty() {
            if let Some(d) = default {
                return d.to_string();
            }
            continue;
        }
        return line.to_string();
    }
}

pub fn ask_bool(question: &str, default: bool) -> bool {
    let hint = if default { "Y/n" } else { "y/N" };
    loop {
        print!("{question} [{hint}]: ");
        io::stdout().flush().ok();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            return default;
        }
        let line = line.trim().to_lowercase();
        match line.as_str() {
            "" => return default,
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => continue,
        }
    }
}

/// Presents a numbered list and lets the user pick zero or more entries,
/// e.g. "1,3" or "all" or blank for none.
pub fn multi_select(prompt: &str, options: &[String]) -> Vec<usize> {
    println!("{prompt}");
    for (i, opt) in options.iter().enumerate() {
        println!("  {}) {}", i + 1, opt);
    }
    print!("Select (comma-separated numbers, \"all\", or blank for none): ");
    io::stdout().flush().ok();
    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        return Vec::new();
    }
    let line = line.trim();
    if line.is_empty() {
        return Vec::new();
    }
    if line.eq_ignore_ascii_case("all") {
        return (0..options.len()).collect();
    }
    let mut chosen = Vec::new();
    for part in line.split(',') {
        let part = part.trim();
        if let Ok(n) = part.parse::<usize>() {
            if n >= 1 && n <= options.len() {
                chosen.push(n - 1);
            }
        }
    }
    chosen
}
