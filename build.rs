use std::env;
use std::fs;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut config_path = Path::new(&root_dir).join("config");

    if !config_path.exists() {
        fs::create_dir(&config_path)?;
    }

    for &prefix in ["api_key", "backend", "domain"].iter() {
        let file = format!("{}_default.txt", prefix);
        config_path.push(file);
        if !config_path.exists() {
            fs::File::create(&config_path)?;
        }
        config_path.pop();
    }

    Ok(())
}
