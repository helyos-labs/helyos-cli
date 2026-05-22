use std::path::Path;

use crate::output;

pub async fn cni(bin_dir: &str, version: &str) -> anyhow::Result<()> {
    let arch = if cfg!(target_arch = "x86_64") {
        "amd64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        anyhow::bail!("unsupported architecture");
    };

    let filename = format!("cni-plugins-linux-{arch}-v{version}.tgz");
    let url = format!(
        "https://github.com/containernetworking/plugins/releases/download/v{version}/{filename}"
    );

    println!("Downloading CNI plugins v{version} for linux/{arch}...");
    println!("  URL: {url}");

    let bin_path = Path::new(bin_dir);
    std::fs::create_dir_all(bin_path)?;

    // Download the tarball
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("download failed: {e}"))?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "download failed with status {}: {}",
            resp.status(),
            resp.status().canonical_reason().unwrap_or("unknown")
        );
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| anyhow::anyhow!("failed to read download: {e}"))?;

    // Extract the tarball
    let tar_gz = flate2::read::GzDecoder::new(std::io::Cursor::new(&bytes));
    let mut archive = tar::Archive::new(tar_gz);

    let mut installed = Vec::new();
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if name.is_empty() {
            continue;
        }

        let dest = bin_path.join(&name);
        entry.unpack(&dest)?;

        // Ensure executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&dest)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&dest, perms)?;
        }

        installed.push(name);
    }

    installed.sort();
    println!("\nInstalled {} CNI plugins to {bin_dir}:", installed.len());
    for name in &installed {
        println!("  - {name}");
    }

    // Verify required plugins
    let required = ["bridge", "loopback", "host-local"];
    let missing: Vec<&&str> = required
        .iter()
        .filter(|r| !installed.iter().any(|i| i == **r))
        .collect();

    if missing.is_empty() {
        output::print_success("All required CNI plugins installed successfully");
    } else {
        println!(
            "\nWarning: missing required plugins: {}",
            missing.iter().map(|m| **m).collect::<Vec<_>>().join(", ")
        );
    }

    Ok(())
}
