use crate::database::models::{ConfigVarFinder, Provider, ProviderConfig};
use crate::utils;

use anyhow::{Result, bail};
use inquire::Text;
use sqlx::sqlite::SqlitePool;
use tracing::error;

pub async fn update(
    pool: &SqlitePool,
    provider_config_id: &str,
    skip_confirmation: bool,
) -> Result<()> {
    enum AwsSessionTokenAction {
        Keep,
        Set(String),
        Clear,
    }

    let id = match provider_config_id.parse::<i64>() {
        Ok(value) => value,
        Err(_) => {
            bail!("Invalid Provider configuration ID, must be a valid integer");
        }
    };

    let config = match ProviderConfig::fetch_by_id(pool, id).await {
        Ok(Some(config)) => config,
        Ok(None) => {
            bail!("Provider configuration (id='{}') not found", id);
        }
        Err(e) => {
            error!("{}", e.to_string());
            bail!("DB Operation Failure")
        }
    };

    let provider = match Provider::fetch_by_id(pool, config.provider_id.clone()).await {
        Ok(Some(provider)) => provider,
        Ok(None) => {
            bail!(
                "Provider '{}' from provider config '{}' was not found",
                config.provider_id,
                config.display_name
            );
        }
        Err(e) => {
            error!("{}", e.to_string());
            bail!("DB Operation Failure")
        }
    };

    let existing_config_vars = config.get_config_vars(pool).await?;

    println!("\n{:<35}: {}", "Provider Configuration Name", config.display_name);
    println!("{:<35}: {}", "ID", config.id);
    println!("{:<35}: {}\n", "Provider", config.provider_id);

    let mut required_key_updates: Vec<(String, String)> = Vec::new();
    let required_keys = provider.get_required_config_vars();
    for key in required_keys {
        let prompt = format!(
            "Enter value for {} (leave blank to keep current):",
            key
        );
        let input = Text::new(&prompt).prompt()?;
        let value = if input.trim().is_empty() {
            match existing_config_vars.get_value(&key) {
                Some(v) => v.to_string(),
                None => {
                    bail!("Missing current value for required key '{}'", key);
                }
            }
        } else {
            input.trim().to_string()
        };
        required_key_updates.push((key, value));
    }

    let mut aws_session_token_action = AwsSessionTokenAction::Keep;
    if provider.id == "aws" {
        let prompt =
            "Enter value for SESSION_TOKEN (optional). Leave blank to keep current, type NONE to clear:";
        let input = Text::new(prompt).prompt()?;
        let session_input = input.trim();

        if session_input.eq_ignore_ascii_case("none") {
            aws_session_token_action = AwsSessionTokenAction::Clear;
        } else if !session_input.is_empty() {
            aws_session_token_action = AwsSessionTokenAction::Set(session_input.to_string());
        }
    }

    if !(utils::user_confirmation(
        skip_confirmation,
        "Confirm updating this Provider configuration?",
    )?) {
        return Ok(());
    }

    for (key, value) in required_key_updates {
        config.upsert_config_var(pool, &key, &value).await?;
    }

    match aws_session_token_action {
        AwsSessionTokenAction::Keep => {}
        AwsSessionTokenAction::Set(token) => {
            config.upsert_config_var(pool, "SESSION_TOKEN", &token).await?;
            config
                .delete_config_var_by_key(pool, "AWS_SESSION_TOKEN")
                .await?;
        }
        AwsSessionTokenAction::Clear => {
            config.delete_config_var_by_key(pool, "SESSION_TOKEN").await?;
            config
                .delete_config_var_by_key(pool, "AWS_SESSION_TOKEN")
                .await?;
        }
    }

    println!("Provider configuration updated successfully.");
    Ok(())
}
