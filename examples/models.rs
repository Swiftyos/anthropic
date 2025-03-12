use anthropic_api::{models::*, Credentials};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load credentials from environment variables
    let credentials = Credentials::from_env();

    // List all available models
    println!("Listing all available models:");
    let models = ModelList::builder()
        .credentials(credentials.clone())
        // Add the limit to the request if desired
        // .limit(5u32)
        .create()
        .await?;

    println!("Available models:");
    for model in &models.data {
        println!("- {} ({})", model.display_name, model.id);
    }

    // Get details for a specific model
    if let Some(first_model) = models.data.first() {
        println!("\nGetting details for model: {}", first_model.id);
        let model_details = Model::builder(&first_model.id)
            .credentials(credentials)
            .create()
            .await?;

        println!("Model details:");
        println!("  ID: {}", model_details.id);
        println!("  Name: {}", model_details.display_name);
        println!("  Created at: {}", model_details.created_at);
        println!("  Type: {}", model_details.model_type);
    }

    Ok(())
}
