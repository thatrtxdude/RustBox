use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::error::Error;

pub fn update_discord_activity(format_title: &str) -> Result<(), Box<dyn Error>> {
    let mut client = DiscordIpcClient::new("1206034100977406012")?;
    client.connect()?;

    let payload = activity::Activity::new()
        .state("Listening to music")
        .details(format_title);

    client.set_activity(payload)?;
    Ok(())
}
