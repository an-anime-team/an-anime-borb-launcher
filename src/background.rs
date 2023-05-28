use anime_launcher_sdk::anime_game_core::installer::downloader::Downloader;
use anime_launcher_sdk::anime_game_core::minreq;

#[derive(Debug, Clone)]
pub struct Background {
    pub uri: String,
    pub hash: String
}

#[cached::proc_macro::cached(result)]
pub fn get_background_info() -> anyhow::Result<Background> {
    let response = minreq::get("https://media-cdn-zspms.kurogame.net/pnswebsite/website2.0/json/G167/MainMenu.json")
        .send()?;

    let json = serde_json::from_slice::<serde_json::Value>(response.as_bytes())?;

    let uri = match json["pcTopPicture"]["coverImage"].as_str() {
        Some(uri) => uri.to_owned(),
        None => anyhow::bail!("Failed to get background picture uri")
    };

    // Not really a hash but we have nothing else
    let hash = uri.split('/').last().unwrap_or_default().to_owned();

    Ok(Background {
        uri,
        hash
    })
}

pub fn download_background() -> anyhow::Result<()> {
    tracing::debug!("Downloading background picture");

    let info = get_background_info()?;

    if crate::BACKGROUND_FILE.exists() && crate::BACKGROUND_HASH_FILE.as_path().exists() {
        let hash = std::fs::read_to_string(crate::BACKGROUND_HASH_FILE.as_path())?;

        if hash == info.hash {
            tracing::debug!("Background picture is already downloaded. Skipping");

            return Ok(());
        }
    }

    let mut downloader = Downloader::new(info.uri)?
        .with_continue_downloading(false);

    if let Err(err) = downloader.download(crate::BACKGROUND_FILE.as_path(), |_, _| {}) {
        anyhow::bail!(err);
    }

    std::fs::write(crate::BACKGROUND_HASH_FILE.as_path(), info.hash)?;

    Ok(())
}
