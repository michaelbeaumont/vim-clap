mod download;

use anyhow::{anyhow, Context, Result};
use curl::easy::{Easy, List};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

const USER: &str = "liuchengxu";
const REPO: &str = "vim-clap";

/// This command is only invoked when user uses the prebuilt binary, more specifically, exe in
/// vim-clap/bin/maple.
#[derive(StructOpt, Debug, Clone)]
pub struct CheckRelease {
    /// Download if the local version mismatches the latest remote version.
    #[structopt(long)]
    pub download: bool,
}

impl CheckRelease {
    pub fn check_new_release(&self, local_tag: &str) -> Result<()> {
        println!("Retriving the latest remote release info...");
        let remote_release = latest_remote_release()?;
        let remote_tag = remote_release.tag_name;
        let remote_version = extract_remote_version_number(&remote_tag);
        let local_version = extract_local_version_number(local_tag);
        if remote_version != local_version {
            if self.download {
                println!(
                    "New maple release {} is avaliable, downloading...",
                    remote_tag
                );
                self.download_prebuilt_binary(&remote_tag)?;
                println!("Latest version {} download completed", remote_tag);
            } else {
                println!(
                    "New maple release {} is avaliable, please download it from {} or rerun with --download flag.",
                    remote_tag,
                    download::to_download_url(&remote_tag)?
                );
            }
        } else {
            println!("No newer release, current maple version: {}", remote_tag);
        }
        Ok(())
    }

    fn download_prebuilt_binary(&self, version: &str) -> Result<()> {
        let exe_dir = std::env::current_exe()?;
        let bin_dir = exe_dir
            .parent()
            .context("Couldn't get the parent of current exe")?;
        if !bin_dir.ends_with("bin") {
            return Err(anyhow!(
                "Current exe has to be under vim-clap/bin directory"
            ));
        }
        let temp_file = download::download_prebuilt_binary_to_a_tempfile(version)?;
        #[cfg(windows)]
        let bin_path = bin_dir.join("maple.exe");
        #[cfg(not(windows))]
        let bin_path = bin_dir.join("maple");
        // Move the downloaded binary to bin/maple
        std::fs::rename(temp_file, bin_path)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoteRelease {
    pub tag_name: String,
}

fn get_latest_release_info() -> Result<Vec<u8>> {
    let mut dst = Vec::new();
    let mut handle = Easy::new();
    handle.url(&format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        USER, REPO
    ))?;
    let mut headers = List::new();
    headers.append(&format!("User-Agent: {}", USER))?;
    headers.append("Accept: application/json")?;
    handle.http_headers(headers)?;

    {
        let mut transfer = handle.transfer();
        transfer.write_function(|data| {
            dst.extend_from_slice(data);
            Ok(data.len())
        })?;

        transfer.perform()?;
    }

    Ok(dst)
}

pub fn latest_remote_release() -> Result<RemoteRelease> {
    let data = get_latest_release_info()?;
    let release: RemoteRelease = serde_json::from_slice(&data).unwrap();
    Ok(release)
}

/// remote: "v0.13"
#[inline]
fn extract_remote_version_number(remote_tag: &str) -> u32 {
    let v = remote_tag.split('.').collect::<Vec<_>>();
    v[1].parse().expect("Couldn't extract remote version")
}

/// local: "v0.13-4-g58738c0"
#[inline]
fn extract_local_version_number(local_tag: &str) -> u32 {
    let tag = local_tag.split('-').collect::<Vec<_>>();
    extract_remote_version_number(tag[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version_number() {
        let tag = "v0.13-4-g58738c0";
        assert_eq!(13u32, extract_local_version_number(tag));
        let tag = "v0.13";
        assert_eq!(13u32, extract_local_version_number(tag));
    }
}
