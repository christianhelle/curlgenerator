#[cfg(windows)]
use std::path::Path;
use std::process::Command;
use time::OffsetDateTime;

fn main() {
    // Get Cargo.toml version as fallback
    let cargo_version = env!("CARGO_PKG_VERSION");

    // Get git information
    let git_tag = get_git_output(&["git", "describe", "--tags", "--abbrev=0"])
        .filter(|s| !s.is_empty() && s != "unknown");
    let git_commit =
        get_git_output(&["git", "rev-parse", "--short", "HEAD"]).filter(|s| !s.is_empty());

    // Determine version: prefer git tag, fallback to Cargo.toml version
    let version = if let Some(tag) = &git_tag {
        tag.strip_prefix('v').unwrap_or(tag).to_string()
    } else {
        cargo_version.to_string()
    };

    let git_tag_display = git_tag.unwrap_or_else(|| format!("v{}", cargo_version));
    let git_commit_display = git_commit.unwrap_or_else(|| "unknown".to_string());

    // Get current timestamp
    let now = OffsetDateTime::now_utc();
    let build_date = format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    );

    // Set environment variables for build
    println!("cargo:rustc-env=VERSION={}", version);
    println!("cargo:rustc-env=GIT_TAG={}", git_tag_display);
    println!("cargo:rustc-env=GIT_COMMIT={}", git_commit_display);
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    // Rerun if git changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/tags");
    println!("cargo:rerun-if-changed=.git/packed-refs");

    // Convert PNG to ICO and embed icon on Windows
    #[cfg(windows)]
    {
        let png_path = Path::new("resources/icon.png");
        let ico_path = Path::new("resources/icon.ico");

        // Convert PNG to ICO if the ICO doesn't exist or PNG is newer
        if !ico_path.exists() || needs_rebuild(png_path, ico_path) {
            println!("cargo:warning=Converting icon.png to icon.ico");
            if let Err(e) = convert_png_to_ico(png_path, ico_path) {
                println!("cargo:warning=Failed to convert icon: {}", e);
            }
        }

        // Embed the icon
        if ico_path.exists() {
            println!("cargo:rerun-if-changed=resources/icon.png");
            println!("cargo:rerun-if-changed=resources/icon.rc");
            embed_resource::compile("resources/icon.rc", embed_resource::NONE);
        }
    }
}

fn get_git_output(args: &[&str]) -> Option<String> {
    Command::new(args[0])
        .args(&args[1..])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

#[cfg(windows)]
fn needs_rebuild(src: &Path, dest: &Path) -> bool {
    use std::fs;

    if let (Ok(src_meta), Ok(dest_meta)) = (fs::metadata(src), fs::metadata(dest)) {
        if let (Ok(src_time), Ok(dest_time)) = (src_meta.modified(), dest_meta.modified()) {
            return src_time > dest_time;
        }
    }
    true
}

#[cfg(windows)]
fn convert_png_to_ico(png_path: &Path, ico_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use ico::{IconDir, IconImage, ResourceType};
    use image::ImageReader;

    // Load PNG
    let img = ImageReader::open(png_path)?.decode()?;

    // Create icon directory
    let mut icon_dir = IconDir::new(ResourceType::Icon);

    // Resize to common icon sizes and add to icon directory
    for size in [16, 32, 48, 64, 128, 256] {
        let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);

        let rgba = resized.to_rgba8();
        let icon_image = IconImage::from_rgba_data(size, size, rgba.into_raw());
        icon_dir.add_entry(ico::IconDirEntry::encode(&icon_image)?);
    }

    // Write ICO file
    let file = std::fs::File::create(ico_path)?;
    icon_dir.write(file)?;

    Ok(())
}
