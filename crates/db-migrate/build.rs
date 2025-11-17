use std::{env, fs, path::Path};

fn main() {
    // Invalidate the build when any migration file changes so sqlx::migrate! picks up new files.
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mig_dir = Path::new(&manifest_dir).join("migrations_sqlx");
    println!("cargo:rerun-if-changed={}", mig_dir.display());

    if let Ok(entries) = fs::read_dir(&mig_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                println!("cargo:rerun-if-changed={}", p.display());
            }
        }
    }
}
