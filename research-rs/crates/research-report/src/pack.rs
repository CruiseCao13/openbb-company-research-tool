use anyhow::Result;
use research_core::run_folder::RunFolder;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;

fn add_dir(zip: &mut zip::ZipWriter<File>, base: &Path, path: &Path) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            add_dir(zip, base, &p)?;
        } else {
            let mut f = File::open(&p)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            let name = p.strip_prefix(base)?.to_string_lossy().replace('\\', "/");
            zip.start_file(name, SimpleFileOptions::default())?;
            zip.write_all(&buf)?;
        }
    }
    Ok(())
}

pub fn pack_run(folder: &RunFolder, ticker: &str) -> Result<std::path::PathBuf> {
    let out = folder.pack.join(format!("{}_research_pack.zip", ticker));
    let file = File::create(&out)?;
    let mut zip = zip::ZipWriter::new(file);
    add_dir(&mut zip, &folder.root, &folder.root)?;
    zip.finish()?;
    Ok(out)
}
