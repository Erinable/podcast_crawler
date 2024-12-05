use tokio;
use podcast_crawler::infrastructure;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize app
    let state = infrastructure::initialize().await?;
    let result = state.repositories.podcast_rank.print_podcast_details().await?;
    println!("{:#?}", result);
    Ok(())
}
