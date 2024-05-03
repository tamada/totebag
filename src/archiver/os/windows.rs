pub fn create(target: &PathBuf) -> SimpleFileOptions {
    let metadata = std::fs::metadata(&target).unwrap();
    let mod_time = DateTime::try_from(
        OffsetDateTime::from(metadata.modified().unwrap()));

    SimpleFileOptions::default()
        .last_modified_time(mod_time.unwrap())
        .compression_method(zip::CompressionMethod::Stored)
}
