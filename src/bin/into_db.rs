use std::{fs, time, env};
use mongodb::Client;
use holo_dev::Song;
use tracing::info;

const SOURCE: &'static str = "intermediate/original_music_list.json";

#[tokio::main]
async  fn main() -> anyhow::Result<()> {
    let start = time::Instant::now();

    dotenvy::dotenv()?;
    tracing_subscriber::fmt().init();

    let url = env::var("DB_URL")?;
    let mongo = Client::with_uri_str(url).await?;
    let db_name = env::var("DB_NAME")?;
    let db = mongo.database(&db_name);
    let collection = db.collection::<Song>("original_musics");

    let b = fs::read(SOURCE)?;
    let musics = serde_json::from_slice::<Vec<Song>>(&b)?;

    let result = collection.insert_many(musics).await?;

    info!(
        "inserted {} items in {}milsecs", 
        result.inserted_ids.len(),
        start.elapsed().as_millis()
    );
    Ok(())
}
