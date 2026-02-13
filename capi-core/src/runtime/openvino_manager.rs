use anyhow::{anyhow, Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

const OV_FULL_VERSION: &str = env!("OV_VERSION");
const OV_SHORT_VERSION: &str = env!("OV_SHORT");
#[allow(dead_code)]
const OV_SONAME: &str = env!("OV_SONAME");

const OV_BASE_URL: &str =
    "https://storage.openvinotoolkit.org/repositories/openvino_genai/packages";

#[cfg(target_os = "linux")]
const ESSENTIAL_PREFIXES_LINUX: &[&str] = &[
    "libopenvino",
    "libtbb",
    "libhwloc",
];

#[cfg(target_os = "windows")]
const ESSENTIAL_PREFIXES_WINDOWS: &[&str] = &[
    "openvino",
    "tbb",
];

pub struct OpenVinoManager {
    lib_dir: PathBuf,
}

impl OpenVinoManager {
    pub fn new() -> Result<Self> {
        let lib_dir = Self::default_lib_dir()?;
        Ok(Self { lib_dir })
    }

    pub fn with_lib_dir(lib_dir: PathBuf) -> Self {
        Self { lib_dir }
    }

    pub fn lib_dir(&self) -> &Path {
        &self.lib_dir
    }

    pub async fn ensure_installed(&self) -> Result<PathBuf> {
        if self.is_correct_version_installed() {
            info!(
                "OpenVINO GenAI {} already installed at {}",
                OV_FULL_VERSION,
                self.lib_dir.display()
            );
            return Ok(self.lib_dir.clone());
        }

        info!(
            "OpenVINO GenAI {} not found, downloading...",
            OV_FULL_VERSION
        );

        if self.lib_dir.exists() {
            fs::remove_dir_all(&self.lib_dir)
                .context("Failed to remove old OpenVINO installation")?;
        }
        fs::create_dir_all(&self.lib_dir)
            .context("Failed to create library directory")?;

        let url = self.download_url()?;
        info!("Downloading from: {}", url);

        self.download_and_extract(&url).await?;
        self.write_version_marker()?;

        info!(
            "OpenVINO GenAI {} installed successfully at {}",
            OV_FULL_VERSION,
            self.lib_dir.display()
        );

        Ok(self.lib_dir.clone())
    }

    pub fn configure_environment(&self) -> Result<()> {
        if !self.lib_dir.exists() {
            return Err(anyhow!(
                "OpenVINO library directory does not exist: {}",
                self.lib_dir.display()
            ));
        }

        #[cfg(target_os = "linux")]
        {
            let current = env::var("LD_LIBRARY_PATH").unwrap_or_default();
            let lib_path = self.lib_dir.to_string_lossy();
            if !current.contains(lib_path.as_ref()) {
                let new_path = if current.is_empty() {
                    lib_path.to_string()
                } else {
                    format!("{}:{}", lib_path, current)
                };
                env::set_var("LD_LIBRARY_PATH", &new_path);
                info!("Set LD_LIBRARY_PATH to include {}", lib_path);
            }
        }

        #[cfg(target_os = "windows")]
        {
            let current = env::var("PATH").unwrap_or_default();
            let lib_path = self.lib_dir.to_string_lossy();
            if !current.contains(lib_path.as_ref()) {
                let new_path = format!("{};{}", lib_path, current);
                env::set_var("PATH", &new_path);
                info!("Added {} to PATH", lib_path);
            }
        }

        Ok(())
    }

    fn is_correct_version_installed(&self) -> bool {
        let version_file = self.lib_dir.join(".version");
        match fs::read_to_string(&version_file) {
            Ok(version) => version.trim() == OV_FULL_VERSION,
            Err(_) => false,
        }
    }

    fn write_version_marker(&self) -> Result<()> {
        let version_file = self.lib_dir.join(".version");
        fs::write(&version_file, OV_FULL_VERSION)
            .context("Failed to write version marker")?;
        Ok(())
    }

    fn download_url(&self) -> Result<String> {
        let (os_name, extension) = Self::detect_os_package()?;
        Ok(format!(
            "{}/{}/{}/openvino_genai_{}_{}_{}.{}",
            OV_BASE_URL,
            OV_SHORT_VERSION,
            Self::os_url_segment(),
            os_name,
            OV_FULL_VERSION,
            "x86_64",
            extension,
        ))
    }

    fn detect_os_package() -> Result<(String, &'static str)> {
        #[cfg(target_os = "linux")]
        {
            let distro = Self::detect_linux_distro()?;
            Ok((distro, "tar.gz"))
        }

        #[cfg(target_os = "windows")]
        {
            Ok(("windows".to_string(), "zip"))
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Err(anyhow!("Unsupported operating system. OpenVINO GenAI is currently available for Linux and Windows."))
        }
    }

    fn os_url_segment() -> &'static str {
        #[cfg(target_os = "linux")]
        { "linux" }
        #[cfg(target_os = "windows")]
        { "windows" }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        { "unknown" }
    }

    #[cfg(target_os = "linux")]
    fn detect_linux_distro() -> Result<String> {
        let os_release = fs::read_to_string("/etc/os-release")
            .context("Failed to read /etc/os-release")?;

        let mut id = String::new();
        let mut version_id = String::new();

        for line in os_release.lines() {
            if let Some(val) = line.strip_prefix("ID=") {
                id = val.trim_matches('"').to_lowercase();
            }
            if let Some(val) = line.strip_prefix("VERSION_ID=") {
                version_id = val.trim_matches('"').to_string();
            }
        }

        let major_version: u32 = version_id
            .split('.')
            .next()
            .and_then(|v| v.parse().ok())
            .unwrap_or(22);

        match id.as_str() {
            "ubuntu" => {
                let ubuntu_version = if major_version >= 24 { "24" } else { "22" };
                Ok(format!("ubuntu{}", ubuntu_version))
            }
            "debian" => Ok("ubuntu22".to_string()),
            "fedora" | "rhel" | "centos" | "rocky" | "almalinux" => {
                Ok("ubuntu22".to_string())
            }
            _ => {
                warn!(
                    "Unknown Linux distribution '{}', defaulting to ubuntu22 package",
                    id
                );
                Ok("ubuntu22".to_string())
            }
        }
    }

    async fn download_and_extract(&self, url: &str) -> Result<()> {
        let response = reqwest::get(url)
            .await
            .context("Failed to download OpenVINO package")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download OpenVINO package: HTTP {}. URL: {}",
                response.status(),
                url
            ));
        }

        let bytes = response
            .bytes()
            .await
            .context("Failed to read response body")?;

        info!(
            "Downloaded {} bytes, extracting essential libraries...",
            bytes.len()
        );

        #[cfg(target_os = "linux")]
        self.extract_tar_gz(&bytes)?;

        #[cfg(target_os = "windows")]
        self.extract_zip(&bytes)?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn extract_tar_gz(&self, data: &[u8]) -> Result<()> {
        let decoder = flate2::read::GzDecoder::new(data);
        let mut archive = tar::Archive::new(decoder);

        for entry in archive.entries().context("Failed to read tar entries")? {
            let mut entry = entry.context("Failed to read tar entry")?;

            let path_string = {
                let path = entry.path().context("Failed to get entry path")?;
                path.to_string_lossy().into_owned()
            };

            let relative = match path_string.splitn(2, '/').nth(1) {
                Some(r) => r.to_string(),
                None => continue,
            };

            let basename = match Path::new(&relative).file_name() {
                Some(name) => name.to_string_lossy().into_owned(),
                None => continue,
            };

            let is_essential = ESSENTIAL_PREFIXES_LINUX.iter().any(|prefix| {
                basename.starts_with(prefix)
            }) || basename == "plugins.xml";

            if is_essential && !entry.header().entry_type().is_dir() {
                let dest = self.lib_dir.join(&basename);
                let mut dest_file = fs::File::create(&dest)
                    .with_context(|| format!("Failed to create {}", dest.display()))?;

                std::io::copy(&mut entry, &mut dest_file)
                    .with_context(|| format!("Failed to write {}", dest.display()))?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))?;
                }

                info!("  Extracted: {}", basename);
            }
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn extract_zip(&self, data: &[u8]) -> Result<()> {
        use std::io::Read;

        let reader = std::io::Cursor::new(data);
        let mut archive = zip::ZipArchive::new(reader)
            .context("Failed to open zip archive")?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .context("Failed to read zip entry")?;

            let path_str = file.name().to_string();

            let basename = match Path::new(&path_str).file_name() {
                Some(name) => name.to_string_lossy().into_owned(),
                None => continue,
            };

            let is_essential = ESSENTIAL_PREFIXES_WINDOWS.iter().any(|prefix| {
                basename.starts_with(prefix)
            }) || basename == "plugins.xml";

            if is_essential && !file.is_dir() {
                let dest = self.lib_dir.join(&basename);
                let mut dest_file = fs::File::create(&dest)
                    .with_context(|| format!("Failed to create {}", dest.display()))?;

                std::io::copy(&mut file, &mut dest_file)
                    .with_context(|| format!("Failed to write {}", dest.display()))?;

                info!("  Extracted: {}", basename);
            }
        }

        Ok(())
    }

    fn default_lib_dir() -> Result<PathBuf> {
        #[cfg(target_os = "linux")]
        {
            let home = dirs::home_dir()
                .ok_or_else(|| anyhow!("Cannot determine home directory"))?;
            Ok(home.join(".local/lib/capi/openvino"))
        }

        #[cfg(target_os = "windows")]
        {
            let local_data = dirs::data_local_dir()
                .ok_or_else(|| anyhow!("Cannot determine local app data directory"))?;
            Ok(local_data.join("capi").join("openvino"))
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Err(anyhow!("Unsupported operating system"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_format() {
        let manager = OpenVinoManager::with_lib_dir(PathBuf::from("/tmp/test"));
        let url = manager.download_url().unwrap();
        assert!(url.contains(OV_SHORT_VERSION));
        assert!(url.contains(OV_FULL_VERSION));
        assert!(url.contains("x86_64"));
        assert!(url.starts_with(OV_BASE_URL));
    }

    #[test]
    fn test_version_check_missing() {
        let manager = OpenVinoManager::with_lib_dir(PathBuf::from("/tmp/nonexistent_ov_test"));
        assert!(!manager.is_correct_version_installed());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_detect_linux_distro() {
        let distro = OpenVinoManager::detect_linux_distro().unwrap();
        assert!(
            distro.starts_with("ubuntu"),
            "Expected ubuntu-based package name, got: {}",
            distro
        );
    }
}
