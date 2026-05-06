use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
};

pub fn load_env_files() {
    let mut candidates = Vec::new();
    if let Some(path) = env::var_os("STATUS_ENV_FILE") {
        candidates.push(PathBuf::from(path));
    }
    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd.join("status-backend").join(".env"));
        candidates.push(cwd.join(".env"));
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("..").join("..").join(".env"));
            candidates.push(dir.join("..").join(".env"));
            candidates.push(dir.join(".env"));
        }
    }

    let mut seen = HashSet::new();
    for path in candidates {
        let normalized = normalize_path(&path);
        if !seen.insert(normalized) {
            continue;
        }
        load_one_env_file(&path);
    }
}

fn load_one_env_file(path: &Path) {
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line).trim();
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() || key.contains(char::is_whitespace) || env::var_os(key).is_some() {
            continue;
        }
        env::set_var(key, parse_env_value(value.trim()));
    }
}

fn parse_env_value(raw: &str) -> String {
    if raw.len() >= 2 {
        let bytes = raw.as_bytes();
        if bytes.first() == Some(&b'"') && bytes.last() == Some(&b'"') {
            return unescape_double_quoted(&raw[1..raw.len() - 1]);
        }
        if bytes.first() == Some(&b'\'') && bytes.last() == Some(&b'\'') {
            return raw[1..raw.len() - 1].to_string();
        }
    }
    raw.split_once(" #")
        .map(|(value, _)| value.trim_end())
        .unwrap_or(raw)
        .to_string()
}

fn unescape_double_quoted(raw: &str) -> String {
    let mut out = String::new();
    let mut chars = raw.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

fn normalize_path(path: &Path) -> String {
    path.components()
        .collect::<PathBuf>()
        .to_string_lossy()
        .replace('\\', "/")
        .to_lowercase()
}
