use colored::*;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};
use toml;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub remote_clash_binary_url: String,
    pub remote_config_url: String,
    pub clash_config_root: String,
}

pub fn setup_default_config(path: &str) {
    let default_config = Config {
        remote_clash_binary_url: String::from(""),
        remote_config_url: String::from(""),
        // Reference to clash config: https://github.com/Dreamacro/clash/wiki/configuration
        clash_config_root: String::from("~/.config/clash"),
    };
    let config = toml::to_string(&default_config).unwrap();
    fs::write(path, config).unwrap();
}

pub fn parse_config(path: &str) -> Config {
    let config = fs::read_to_string(path).unwrap();
    let config: Config = toml::from_str(&config).unwrap();
    config
}

pub fn sudo_check(prefix: &str) {
    // Check for sudo privilege and try to escalate if not
    if sudo::check() != sudo::RunningAs::Root {
        println!("{} Sudo required, enter password below", prefix.yellow());
        sudo::escalate_if_needed().unwrap();
    }
}

pub fn download_file(url: &str, path: &str) {
    println!(
        "{} Downloading from {}",
        "download:".blue(),
        url.underline().yellow()
    );
    let mut resp = reqwest::blocking::get(url).unwrap();
    let mut file = fs::File::create(path).unwrap();
    resp.copy_to(&mut file).unwrap();
    println!(
        "{} Downloaded to {}",
        "download:".blue(),
        path.underline().yellow()
    );
}

pub fn move_file(from: &str, to: &str, prefix: &str) {
    fs::rename(from, to).unwrap();
    println!("{} Moved to {}", prefix.green(), to.underline().yellow());
}

pub fn delete_file(path: &str, prefix: &str) {
    fs::remove_file(&path).unwrap();
    println!("{} Removed {}", prefix.green(), path.underline().yellow());
}

pub fn extract_gzip(gzip_path: &str, filename: &str, prefix: &str) {
    let mut archive = GzDecoder::new(fs::File::open(gzip_path).unwrap());
    let mut file = fs::File::create(filename).unwrap();
    io::copy(&mut archive, &mut file).unwrap();
    fs::remove_file(gzip_path).unwrap();
    println!(
        "{} Extracted to {}",
        prefix.green(),
        filename.underline().yellow()
    );
}

#[derive(Debug)]
pub enum ClashrupConfigError {
    ConfigMissingError,
    RemoteClashBinaryUrlMissingError,
    RemoteConfigUrlMissingError,
}

pub fn validate_clashrup_config(path: &str, prefix: &str) -> Result<Config, ClashrupConfigError> {
    // Create clashrup default config if not exists
    let config_path = Path::new(path);
    if !config_path.exists() {
        setup_default_config(path);
        return Err(ClashrupConfigError::ConfigMissingError);
    }

    // Parse config file and validate if urls are defined
    println!(
        "{} Reading config from {}",
        prefix.cyan(),
        path.underline().yellow()
    );
    let config = parse_config(path);
    if config.remote_clash_binary_url.is_empty() {
        return Err(ClashrupConfigError::RemoteClashBinaryUrlMissingError);
    }
    if config.remote_config_url.is_empty() {
        return Err(ClashrupConfigError::RemoteConfigUrlMissingError);
    }
    return Ok(config);
}

/**
 * Create a systemd service file for running clash
 *
 * Reference: https://github.com/Dreamacro/clash/wiki/Running-Clash-as-a-service
 */
pub fn create_clash_service(clash_binary_path: &str, clash_config_path: &str, prefix: &str) {
    let service = format!(
        "[Unit]
Description=Clash - A rule-based tunnel in Go.
After=network.target

[Service]
Type=simple
ExecStart={clash_binary_path} -d {clash_config_path}
Restart=always

[Install]
WantedBy=multi-user.target",
        clash_binary_path = clash_binary_path,
        clash_config_path = clash_config_path
    );
    let service_path = "/etc/systemd/system/clash.service";
    fs::write(service_path, service).unwrap();
    println!(
        "{} Created clash.service at {}",
        prefix.green(),
        service_path.underline().yellow()
    );
}
