use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use iceoryx2_payload_gen::{adapter, gen};

use crate::cli::{Cli, Lang};

pub fn generate(cli: Cli) -> anyhow::Result<()> {
    if cli.input.is_file() {
        generate_from_single_file(&cli.input, &cli.output, cli.lang, &cli.service_prefix)
    } else if cli.input.is_dir() {
        generate_from_directory(&cli.input, &cli.output, cli.lang, &cli.service_prefix)
    } else {
        bail!(
            "input '{}' is neither a file nor a directory",
            cli.input.display()
        )
    }
}

fn generate_from_single_file(
    input: &Path,
    output_arg: &Path,
    lang: Lang,
    service_prefix: &str,
) -> anyhow::Result<()> {
    ensure_supported_interface_file(input)?;

    let generated = render_messages(input, lang, service_prefix)?;
    if generated.len() != 1 {
        bail!(
            "single-file input must produce exactly one payload file; '{}' produced {} payload types",
            input.display(),
            generated.len()
        );
    }

    let file_ext = lang_extension(lang);
    let default_file_name = format!("{}.{}", generated[0].name.to_ascii_lowercase(), file_ext);
    let out_path = resolve_single_file_output_path(output_arg, &default_file_name)?;
    write_generated_file(&out_path, &generated[0].content)?;
    println!("Generated: {}", out_path.display());

    Ok(())
}

fn generate_from_directory(
    input_dir: &Path,
    output_arg: &Path,
    lang: Lang,
    service_prefix: &str,
) -> anyhow::Result<()> {
    let output_dir = resolve_directory_output_path(output_arg)?;
    std::fs::create_dir_all(&output_dir)?;

    let interface_files = collect_interface_files(input_dir)?;
    if interface_files.is_empty() {
        bail!(
            "no .msg/.srv interface files found in directory '{}'",
            input_dir.display()
        );
    }

    let file_ext = lang_extension(lang);
    for input in interface_files {
        for generated in render_messages(&input, lang, service_prefix)? {
            let out_path = output_dir.join(format!(
                "{}.{}",
                generated.name.to_ascii_lowercase(),
                file_ext
            ));
            write_generated_file(&out_path, &generated.content)?;
            println!("Generated: {}", out_path.display());
        }
    }

    Ok(())
}

fn render_messages(
    input: &Path,
    lang: Lang,
    service_prefix: &str,
) -> anyhow::Result<Vec<GeneratedMessage>> {
    let ir = adapter::adapt_file(input)
        .with_context(|| format!("failed to parse interface '{}'", input.display()))?;

    let mut generated = Vec::with_capacity(ir.messages.len());
    for msg in &ir.messages {
        let service_name = format!("{service_prefix}{}", msg.name);
        let content = match lang {
            Lang::Rust => gen::rust::generate(msg, &service_name),
            Lang::Cpp => gen::cpp::generate(msg, &service_name),
            Lang::Python => gen::python::generate(msg, &service_name),
        };
        generated.push(GeneratedMessage {
            name: msg.name.clone(),
            content,
        });
    }
    Ok(generated)
}

fn lang_extension(lang: Lang) -> &'static str {
    match lang {
        Lang::Rust => "rs",
        Lang::Cpp => "hpp",
        Lang::Python => "py",
    }
}

fn resolve_single_file_output_path(
    output_arg: &Path,
    default_file_name: &str,
) -> anyhow::Result<PathBuf> {
    if output_arg.exists() {
        if output_arg.is_dir() {
            return Ok(output_arg.join(default_file_name));
        }
        if output_arg.is_file() {
            return Ok(output_arg.to_path_buf());
        }
        bail!(
            "output path '{}' exists but is not writable",
            output_arg.display()
        );
    }

    if output_arg.extension().is_some() {
        Ok(output_arg.to_path_buf())
    } else {
        Ok(output_arg.join(default_file_name))
    }
}

fn resolve_directory_output_path(output_arg: &Path) -> anyhow::Result<PathBuf> {
    if output_arg.exists() {
        if output_arg.is_dir() {
            return Ok(output_arg.to_path_buf());
        }
        bail!(
            "directory input requires directory output; '{}' is not a directory",
            output_arg.display()
        );
    }

    if output_arg.extension().is_some() {
        bail!(
            "directory input does not support output file path with suffix: '{}'",
            output_arg.display()
        );
    }

    Ok(output_arg.to_path_buf())
}

fn collect_interface_files(input_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(input_dir)
        .with_context(|| format!("failed to read '{}'", input_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && is_supported_interface_file(&path) {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn ensure_supported_interface_file(path: &Path) -> anyhow::Result<()> {
    if is_supported_interface_file(path) {
        return Ok(());
    }
    bail!(
        "unsupported input interface '{}'; expected .msg or .srv",
        path.display()
    );
}

fn is_supported_interface_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("msg" | "srv")
    )
}

fn write_generated_file(path: &Path, content: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(path, content)?;
    Ok(())
}

#[derive(Debug)]
struct GeneratedMessage {
    name: String,
    content: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn single_file_output_with_suffix_is_treated_as_file() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("custom.rs");
        let resolved = resolve_single_file_output_path(&output, "pose.rs").unwrap();
        assert_eq!(resolved, output);
    }

    #[test]
    fn single_file_output_without_suffix_is_treated_as_directory() {
        let dir = tempdir().unwrap();
        let output_dir = dir.path().join("generated");
        let resolved = resolve_single_file_output_path(&output_dir, "pose.rs").unwrap();
        assert_eq!(resolved, output_dir.join("pose.rs"));
    }

    #[test]
    fn directory_input_rejects_output_with_suffix() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("generated.rs");
        let err = resolve_directory_output_path(&output).unwrap_err();
        assert!(err
            .to_string()
            .contains("does not support output file path with suffix"));
    }

    #[test]
    fn directory_input_accepts_existing_directory_even_with_dot_in_name() {
        let dir = tempdir().unwrap();
        let output_dir = dir.path().join("generated.v1");
        std::fs::create_dir_all(&output_dir).unwrap();

        let resolved = resolve_directory_output_path(&output_dir).unwrap();
        assert_eq!(resolved, output_dir);
    }

    #[test]
    fn directory_scan_collects_only_msg_and_srv_files() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.msg"), "float64 x\n").unwrap();
        std::fs::write(dir.path().join("b.srv"), "int32 a\n---\nint32 b\n").unwrap();
        std::fs::write(dir.path().join("c.txt"), "ignored\n").unwrap();

        let files = collect_interface_files(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].file_name().and_then(|s| s.to_str()), Some("a.msg"));
        assert_eq!(files[1].file_name().and_then(|s| s.to_str()), Some("b.srv"));
    }

    #[test]
    fn single_file_mode_rejects_multi_message_interfaces() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("add_two_ints.srv");
        let output = dir.path().join("out.rs");
        std::fs::write(&input, "int32 a\n---\nint32 sum\n").unwrap();

        let err = generate_from_single_file(&input, &output, Lang::Rust, "").unwrap_err();
        assert!(err
            .to_string()
            .contains("must produce exactly one payload file"));
    }
}
