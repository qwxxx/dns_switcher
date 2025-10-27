use std::process::Command;

use anyhow::{Context, Result, anyhow};
use regex::Regex;

fn extract_dns_configuration(output: &str) -> Option<String> {
    const MARKER: &str = "DNS configuration (for scoped queries)";

    output.find(MARKER).map(|pos| {
        let start = pos + MARKER.len();
        let remaining = &output[start..];
        remaining.to_string()
    })
}

pub fn get_dns_from_system() -> Result<Vec<String>> {
    let output = Command::new("scutil").arg("--dns").output()?;
    if !output.status.success() {
        let error_str = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(error_str.to_string()));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let dns_configuration = extract_dns_configuration(&output_str)
        .context("Could not find dns configuration in command output")?;

    let re = Regex::new(r"nameserver\[\d+\] : (\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap();

    let mut ips = Vec::new();
    for cap in re.captures_iter(&dns_configuration) {
        ips.push(cap[1].to_string());
    }

    Ok(ips)
}

pub fn set_dns_to_system(dns_list: &Vec<String>) -> Result<()> {
    /* let apple_script = format!(
        r#"
        do shell script "sudo networksetup -setdnsservers \"WI-FI\" {}""#,
        if dns_list.is_empty() {
            "Empty".to_string()
        } else {
            dns_list.join(" ")
        }
    );

    Command::new("osascript")
        .arg("-e")
        .arg(&apple_script)
        .output()?;*/
    let mut command = Command::new("networksetup");
    command.arg("-setdnsservers").arg("Wi-Fi");

    if dns_list.is_empty() {
        command.arg("Empty");
    } else {
        for dns in dns_list {
            command.arg(dns);
        }
    }

    command.output()?;
    Ok(())
}
