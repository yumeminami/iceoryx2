use std::path::Path;

use iceoryx2_payload_gen::{adapter, gen};

use crate::cli::{Cli, Lang};

pub fn generate(cli: Cli) -> anyhow::Result<()> {
    std::fs::create_dir_all(&cli.output)?;

    let mut had_error = false;
    for input in &cli.input {
        if let Err(e) = process_file(input, &cli.output, cli.lang, &cli.service_prefix) {
            eprintln!("error: {}: {e}", input.display());
            had_error = true;
        }
    }

    if had_error {
        std::process::exit(1);
    }
    Ok(())
}

fn process_file(
    input: &Path,
    output_dir: &Path,
    lang: Lang,
    service_prefix: &str,
) -> anyhow::Result<()> {
    let ir = adapter::adapt_file(input)?;

    for msg in &ir.messages {
        let service_name = format!("{service_prefix}{}", msg.name);
        let (content, ext) = match lang {
            Lang::Rust => (gen::rust::generate(msg, &service_name), "rs"),
            Lang::Cpp => (gen::cpp::generate(msg, &service_name), "hpp"),
            Lang::Python => (gen::python::generate(msg, &service_name), "py"),
        };

        let out_path = output_dir.join(format!("{}.{}", msg.name.to_lowercase(), ext));
        std::fs::write(&out_path, &content)?;
        println!("Generated: {}", out_path.display());
    }

    Ok(())
}
