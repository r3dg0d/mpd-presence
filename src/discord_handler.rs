use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::error::Error;

pub struct DiscordHandler {
    client: DiscordIpcClient,
}

impl DiscordHandler {
    pub fn new(app_id: &str) -> Result<Self, Box<dyn Error>> {
        let mut client = DiscordIpcClient::new(app_id)?;
        client.connect()?;
        Ok(Self { client })
    }

    pub fn update_presence(
        &mut self,
        details: &str,
        state: &str,
        album: &str,
        art_url: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let mut assets = activity::Assets::new()
            .large_text(album);
            
        if let Some(url) = art_url {
            assets = assets.large_image(url);
        } else {
            // value "default" or similar if you have a default asset uploaded
            // assets = assets.large_image("default");
        }

        let payload = activity::Activity::new()
            .details(details)
            .state(state)
            .assets(assets);

        self.client.set_activity(payload)?;
        Ok(())
    }
}
