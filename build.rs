use clap::{Command, CommandFactory};
use clap_complete::Shell;
use std::fs::File;
use std::path::Path;

include!("src/cli.rs");

fn generate(s: Shell, app: &mut Command, appname: &str, outdir: &Path, file: String) {
    let destfile = outdir.join(file);
    std::fs::create_dir_all(destfile.parent().unwrap()).unwrap();
    let mut dest = File::create(destfile).unwrap();
    
    clap_complete::generate(s, app, appname, &mut dest);
}

fn parse_cargo_toml() -> toml::Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let file = match std::fs::read_to_string(path) {
        Ok(f) => f,
        Err(e) => panic!("{}", e),
    };

    file.parse().unwrap()
}

fn main() {
    let table = parse_cargo_toml();
    let appname = table["package"]["name"].as_str().unwrap();

    let mut app = CliOpts::command();
    app.set_bin_name(appname);

    let outdir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/completions/");

    generate(Shell::Bash, &mut app, appname, &outdir, format!("bash/{}", appname));
    generate(Shell::Elvish, &mut app, appname, &outdir, format!("elvish/{}", appname));
    generate(Shell::Fish, &mut app, appname, &outdir, format!("fish/{}", appname));
    generate(Shell::PowerShell, &mut app, appname, &outdir, format!("powershell/{}", appname));
    generate(Shell::Zsh, &mut app, appname, &outdir, format!("zsh/_{}", appname));
}
