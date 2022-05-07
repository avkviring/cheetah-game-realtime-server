use std::fs::{read_to_string, DirEntry, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::{env, fs};

use deflate::write::GzEncoder;
use deflate::CompressionOptions;
use itertools::Itertools;
use tar::Builder;

fn main() {
    let args: Vec<String> = env::args().collect();
    let help = "use cheetah-hosting-configurator with args <source> <target>";
    let source = &args.get(1).expect(help);
    let target = &args.get(2).expect(help);
    create_config_maps_for_services(source, target);
}

fn create_config_maps_for_services(source: &str, target: &str) {
    fs::read_dir(source)
        .unwrap()
        .flatten()
        .filter(|service| service.path().is_dir())
        .map(|service| {
            (
                service.file_name().to_str().unwrap().to_string(),
                fs::read_dir(service.path()).unwrap(),
            )
        })
        .for_each(|(name, configs)| {
            let name = name.as_str();
            configs
                .filter_map(|config| create_config_map(config.unwrap(), name))
                .for_each(|(name, content)| {
                    let mut file =
                        File::create(Path::new(target).join(format!("{}.yaml", name))).unwrap();
                    file.write_all(content.as_bytes()).unwrap();
                })
        });
}

fn create_config_map(configuration: DirEntry, service_name: &str) -> Option<(String, String)> {
    let config_map_name = format!(
        "cheetah-{}-{}",
        service_name,
        configuration
            .file_name()
            .to_str()
            .unwrap()
            .replace(".yaml", "")
    );
    let mut builder = String::new();
    builder += "apiVersion: v1\n";
    builder += "kind: ConfigMap\n";
    builder += "metadata:\n";
    builder += format!(" name: \"{}\"\n", config_map_name).as_str();

    if configuration.path().is_dir() || configuration.path().is_symlink() {
        builder += "binaryData:\n";
        let binary_config = get_binary_config(&configuration);
        builder += format!(
            " {}.tgz: \"{}\"\n",
            configuration.file_name().into_string().unwrap(),
            binary_config
        )
        .as_str();
    } else if let Some(ext) = configuration.path().extension() {
        if ext.to_str().unwrap() == "yaml" {
            builder += "data:\n";
            let string_config = get_string_config(&configuration)
                .split('\n')
                .map(|l| format!("     {}", l))
                .join("\n");
            builder += format!(
                " {}: |-\n {}",
                configuration.file_name().into_string().unwrap(),
                string_config
            )
            .as_str();
        } else {
            return None;
        }
    }

    Some((config_map_name, builder))
}
fn get_string_config(configuration: &DirEntry) -> String {
    read_to_string(configuration.path()).unwrap()
}

fn get_binary_config(configuration: &DirEntry) -> String {
    let mut archive_buffer = BufWriter::new(Vec::new());
    {
        let mut tar = Builder::new(&mut archive_buffer);
        tar.append_dir_all(".", configuration.path().to_str().unwrap())
            .unwrap();
        tar.finish().unwrap();
    }
    let tar_binary_content = archive_buffer.into_inner().unwrap();
    let mut gzipped = BufWriter::new(Vec::new());
    let mut encoder = GzEncoder::new(&mut gzipped, CompressionOptions::high());
    encoder.write_all(tar_binary_content.as_slice()).unwrap();
    encoder.finish().unwrap();
    base64::encode(gzipped.into_inner().unwrap())
}
