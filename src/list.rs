use chrono::NaiveDateTime;
use totebag::extractor::Entry;

pub fn print_long_format(entry: Entry) {
    println!("{}", format_long_format(entry));
}

fn format_long_format(entry: Entry) -> String {
    let r1 = format_unix_mode(entry.unix_mode);
    let r2 = format_size(entry.compressed_size, entry.original_size);
    let r3 = format_date(entry.date);
    format!("{} {} {} {}", r1, r2, r3, entry.name)
}

fn format_date(date: Option<NaiveDateTime>) -> String {
    match date {
        Some(d) => d.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => "                    ".to_string(),
    }
}

fn format_size(compressed: Option<u64>, original: Option<u64>) -> String {
    let formatter = humansize::make_format(humansize::DECIMAL);
    match (compressed, original) {
        (Some(c), Some(o)) => format!("{:>10}/{:>10}", formatter(c), formatter(o)),
        (Some(c), None) => format!("{:>10}/         ", formatter(c)),
        (None, Some(o)) => format!("          /{:>10}", formatter(o)),
        (None, None) => "          /          ".to_string(),
    }
}

fn format_unix_mode(mode: Option<u32>) -> String {
    if let Some(mode) = mode {
        format!(
            "-{}{}{}",
            format_mode(mode >> 6 & 0x7),
            format_mode(mode >> 3 & 0x7),
            format_mode(mode >> 0 & 0x7)
        )
    } else {
        "----------".to_string()
    }
}

fn format_mode(mode: u32) -> String {
    match mode {
        0 => "---",
        1 => "--x",
        2 => "-w-",
        3 => "-wx",
        4 => "r--",
        5 => "r-x",
        6 => "rw-",
        7 => "rwx",
        _ => "???",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry() {
        let entry = Entry {
            name: "Cargo.toml".to_string(),
            compressed_size: Some(100),
            original_size: Some(200),
            unix_mode: Some(0o644),
            date: Some(
                NaiveDateTime::parse_from_str("2021-02-03 04:05:10", "%Y-%m-%d %H:%M:%S").unwrap(),
            ),
        };
        assert_eq!(
            format_long_format(entry),
            "-rw-r--r--      100 B/     200 B 2021-02-03 04:05:10 Cargo.toml"
        );
    }
}
