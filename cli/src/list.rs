use totebag::archiver::ArchiveEntries;

pub fn print_archive_result(result: ArchiveEntries) {
    let f = humansize::make_format(humansize::DECIMAL);
    let total = result.total();
    let rate = if total == 0 {
        0.0
    } else {
        result.compressed as f64 / total as f64 * 100.0
    };
    println!(
        "archived: {} ({} entries, {:>10} / {:>10}, {:.2}%)",
        result.archive_file.display(),
        result.len(),
        f(result.compressed),
        f(result.total()),
        rate
    );
}
