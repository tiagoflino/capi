use anyhow::{anyhow, Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const OV_VERSION: &str = env!("OV_VERSION");
const OV_SHORT: &str = env!("OV_SHORT");
const OV_SONAME: &str = env!("OV_SONAME");

const OV_BASE_URL: &str =
    "https://storage.openvinotoolkit.org/repositories/openvino_genai/packages";

fn sentinel_lib() -> String {
    #[cfg(target_os = "linux")]
    { format!("libopenvino_genai.so.{}", OV_SONAME) }
    #[cfg(target_os = "windows")]
    { "openvino_genai.dll".to_string() }
}

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

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    let launcher_path = env::current_exe()
        .context("Cannot determine launcher path")?;
    let launcher_dir = launcher_path.parent()
        .ok_or_else(|| anyhow!("Cannot determine launcher directory"))?;

    let engine_name = engine_binary_name();
    let engine_path = launcher_dir.join(&engine_name);

    if !engine_path.exists() {
        return Err(anyhow!(
            "Engine binary '{}' not found at {}. Is Capi installed correctly?",
            engine_name,
            launcher_dir.display()
        ));
    }

    let bundled_lib_dir = launcher_dir.join("lib");
    let user_lib_dir = default_user_lib_dir()?;

    let lib_dir = if has_openvino_libs(&bundled_lib_dir) {
        bundled_lib_dir
    } else if has_openvino_libs(&user_lib_dir) {
        user_lib_dir
    } else {
        eprintln!(
            "OpenVINO GenAI {} not found. Downloading essential libraries...",
            OV_VERSION
        );
        fs::create_dir_all(&user_lib_dir)
            .context("Failed to create library directory")?;
        download_openvino(&user_lib_dir).await?;
        eprintln!("OpenVINO GenAI {} ready.", OV_VERSION);
        user_lib_dir
    };

    set_library_path(&lib_dir);
    exec_engine(&engine_path, &args)
}

fn has_openvino_libs(dir: &Path) -> bool {
    if !dir.exists() {
        return false;
    }
    dir.join(sentinel_lib()).exists()
}

fn engine_binary_name() -> String {
    #[cfg(target_os = "linux")]
    { "capi-engine".to_string() }
    #[cfg(target_os = "windows")]
    { "capi-engine.exe".to_string() }
}

fn default_user_lib_dir() -> Result<PathBuf> {
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
}

fn set_library_path(lib_dir: &Path) {
    #[cfg(target_os = "linux")]
    {
        let current = env::var("LD_LIBRARY_PATH").unwrap_or_default();
        let lib_path = lib_dir.to_string_lossy();
        if !current.contains(lib_path.as_ref()) {
            let new_path = if current.is_empty() {
                lib_path.to_string()
            } else {
                format!("{}:{}", lib_path, current)
            };
            env::set_var("LD_LIBRARY_PATH", &new_path);
        }
    }
    #[cfg(target_os = "windows")]
    {
        let current = env::var("PATH").unwrap_or_default();
        let lib_path = lib_dir.to_string_lossy();
        if !current.contains(lib_path.as_ref()) {
            let new_path = format!("{};{}", lib_path, current);
            env::set_var("PATH", &new_path);
        }
    }
}

fn exec_engine(engine_path: &Path, args: &[String]) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = Command::new(engine_path)
            .args(args)
            .exec();
        Err(anyhow!("Failed to exec engine: {}", err))
    }

    #[cfg(windows)]
    {
        let status = Command::new(engine_path)
            .args(args)
            .status()
            .context("Failed to launch engine")?;
        std::process::exit(status.code().unwrap_or(1));
    }
}

async fn download_openvino(dest_dir: &Path) -> Result<()> {
    let url = build_download_url()?;
    eprintln!("  Downloading from: {}", url);

    let response = reqwest::get(&url)
        .await
        .context("Failed to download OpenVINO package")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Download failed: HTTP {}. URL: {}",
            response.status(),
            url
        ));
    }

    let total_size = response.content_length().unwrap_or(0);
    let pb = if total_size > 0 {
        let pb = indicatif::ProgressBar::new(total_size);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("  [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );
        Some(pb)
    } else {
        None
    };

    use futures_util::StreamExt;
    let mut bytes = Vec::with_capacity(total_size as usize);
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Error reading download stream")?;
        if let Some(pb) = &pb {
            pb.inc(chunk.len() as u64);
        }
        bytes.extend_from_slice(&chunk);
    }
    if let Some(pb) = &pb {
        pb.finish_and_clear();
    }

    eprintln!("  Extracting essential libraries...");

    #[cfg(target_os = "linux")]
    extract_tar_gz(&bytes, dest_dir)?;

    #[cfg(target_os = "windows")]
    extract_zip(&bytes, dest_dir)?;

    fs::write(dest_dir.join(".version"), OV_VERSION)
        .context("Failed to write version marker")?;

    Ok(())
}

fn build_download_url() -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        let distro = detect_linux_distro()?;
        Ok(format!(
            "{}/{}/linux/openvino_genai_{}_{}_{}.tar.gz",
            OV_BASE_URL, OV_SHORT, distro, OV_VERSION, "x86_64",
        ))
    }
    #[cfg(target_os = "windows")]
    {
        Ok(format!(
            "{}/{}/windows/openvino_genai_windows_{}_{}.zip",
            OV_BASE_URL, OV_SHORT, OV_VERSION, "x86_64",
        ))
    }
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

    let major: u32 = version_id
        .split('.')
        .next()
        .and_then(|v| v.parse().ok())
        .unwrap_or(22);

    match id.as_str() {
        "ubuntu" => {
            let ver = if major >= 24 { "24" } else { "22" };
            Ok(format!("ubuntu{}", ver))
        }
        _ => Ok("ubuntu22".to_string()),
    }
}

#[cfg(target_os = "linux")]
fn extract_tar_gz(data: &[u8], dest_dir: &Path) -> Result<()> {
    let decoder = flate2::read::GzDecoder::new(data);
    let mut archive = tar::Archive::new(decoder);

    for entry in archive.entries().context("Failed to read tar entries")? {
        let mut entry = entry.context("Failed to read tar entry")?;

        let path_string = {
            let path = entry.path().context("Failed to get entry path")?;
            path.to_string_lossy().into_owned()
        };

        let basename = match Path::new(&path_string).file_name() {
            Some(name) => name.to_string_lossy().into_owned(),
            None => continue,
        };

        let is_essential = ESSENTIAL_PREFIXES_LINUX.iter().any(|prefix| {
            basename.starts_with(prefix)
        });
        let is_essential = is_essential || basename == "plugins.xml";

        if is_essential && !entry.header().entry_type().is_dir() {
            let dest = dest_dir.join(&basename);
            let mut dest_file = fs::File::create(&dest)
                .with_context(|| format!("Failed to create {}", dest.display()))?;

            std::io::copy(&mut entry, &mut dest_file)
                .with_context(|| format!("Failed to write {}", dest.display()))?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))?;
            }

            eprintln!("    {}", basename);
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn extract_zip(data: &[u8], dest_dir: &Path) -> Result<()> {
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
        });
        let is_essential = is_essential || basename == "plugins.xml";

        if is_essential && !file.is_dir() {
            let dest = dest_dir.join(&basename);
            let mut dest_file = fs::File::create(&dest)
                .with_context(|| format!("Failed to create {}", dest.display()))?;

            std::io::copy(&mut file, &mut dest_file)
                .with_context(|| format!("Failed to write {}", dest.display()))?;

            eprintln!("    {}", basename);
        }
    }

    Ok(())
}
