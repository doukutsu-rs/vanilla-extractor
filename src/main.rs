use std::env;

use vanilla_extractor::vanilla::VanillaExtractor;

fn main() {
    let vanilla_ext_exe = env::var("VANILLA_EXT_EXE").unwrap_or("Doukutsu.exe".to_owned());
    let vanilla_ext_outdir = env::var("VANILLA_EXT_OUTDIR").unwrap_or("data".to_owned());

    if let Some(vanilla_extractor) = VanillaExtractor::from(vanilla_ext_exe, vanilla_ext_outdir) {
        let result = vanilla_extractor.extract_data();
        if let Err(e) = result {
            eprintln!("Failed to extract vanilla data: {}", e);
        }
    }
}
