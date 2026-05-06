use crate::integrations::providers::aws::{AwsInterface, interface::AwsClusterContext};

use anyhow::{Result, bail};
use aws_sdk_iam::error::SdkError;
use aws_sdk_iam::operation::get_instance_profile::GetInstanceProfileError;
use tracing::{error, info};

impl AwsInterface {
    pub async fn ensure_iam_profile(&self, context: &AwsClusterContext) -> Result<String> {
        let profile_name = context.iam_profile_name.clone();
        let role_name = context.iam_role_name.clone();

        let mut profile_id = None;
        let mut needs_role_attachment = true;

        match context
            .iam_client
            .get_instance_profile()
            .instance_profile_name(&profile_name)
            .send()
            .await
        {
            Ok(response) => {
                if let Some(profile) = response.instance_profile() {
                    let iam_profile_id = profile.instance_profile_id();
                    info!(
                        "Found existing IAM Profile (id='{}'), reconciling role attachments...",
                        iam_profile_id
                    );
                    profile_id = Some(iam_profile_id.to_string());

                    for existing_role in profile.roles() {
                        let existing_role_name = existing_role.role_name();
                        if existing_role_name == role_name {
                            needs_role_attachment = false;
                            continue;
                        }

                        match context
                            .iam_client
                            .remove_role_from_instance_profile()
                            .instance_profile_name(&profile_name)
                            .role_name(existing_role_name)
                            .send()
                            .await
                        {
                            Ok(_) => {
                                info!(
                                    "Removed unexpected IAM Role (name='{}') from IAM Profile (name='{}')",
                                    existing_role_name,
                                    profile_name
                                );
                            }
                            Err(e) => {
                                error!("{:?}", e);
                                bail!(
                                    "Failed to remove unexpected IAM Role (name='{}') from IAM Profile (name='{}')",
                                    existing_role_name,
                                    profile_name,
                                );
                            }
                        }
                    }
                }
            }
            Err(SdkError::ServiceError(service_err)) => match service_err.err() {
                GetInstanceProfileError::NoSuchEntityException(_) => {
                    info!(
                        "IAM Profile (name='{}') does not exist, will create it",
                        profile_name
                    );
                }
                _ => {
                    error!("{:?}", service_err);
                    bail!("Failure describing IAM Profile (name='{}')", profile_name);
                }
            },
            Err(e) => {
                error!("{:?}", e);
                bail!("Failure describing IAM Profile (name='{}')", profile_name);
            }
        };

        if profile_id.is_none() {
            let create_iam_profile_response = match context
                .iam_client
                .create_instance_profile()
                .instance_profile_name(&profile_name)
                .tags(
                    aws_sdk_iam::types::Tag::builder()
                        .key("Name")
                        .value(&profile_name)
                        .build()
                        .unwrap(),
                )
                .tags(
                    aws_sdk_iam::types::Tag::builder()
                        .key(context.cluster_id_tag.key().unwrap())
                        .value(context.cluster_id_tag.value().unwrap())
                        .build()
                        .unwrap(),
                )
                .send()
                .await
            {
                Ok(response) => {
                    info!("Created IAM Profile (name='{}')", profile_name);
                    response
                }
                Err(e) => {
                    error!("{:?}", e);
                    bail!("Failed to create IAM Profile (name='{}')", profile_name);
                }
            };

            profile_id = Some(
                create_iam_profile_response
                    .instance_profile()
                    .unwrap()
                    .instance_profile_id()
                    .to_string(),
            );
            needs_role_attachment = true;
        }

        if needs_role_attachment {
            match context
                .iam_client
                .add_role_to_instance_profile()
                .instance_profile_name(&profile_name)
                .role_name(&role_name)
                .send()
                .await
            {
                Ok(_) => {
                    info!(
                        "Ensured IAM Role (name='{}') is attached to IAM Profile (name='{}')",
                        role_name, profile_name
                    );
                }
                Err(e) => {
                    error!("{:?}", e);
                    bail!(
                        "Failed to attach IAM Role (name='{}') to IAM Profile (name='{}')",
                        role_name,
                        profile_name,
                    );
                }
            }
        } else {
            info!(
                "IAM Profile (name='{}') already has the expected IAM Role attached",
                profile_name
            );
        }

        Ok(profile_id.unwrap())
    }

    pub async fn cleanup_iam_profile(&self, context: &AwsClusterContext) -> Result<()> {
        let profile_name = context.iam_profile_name.clone();
        info!("Cleaning up IAM Profile (name='{}')...", profile_name);

        match context
            .iam_client
            .get_instance_profile()
            .instance_profile_name(&profile_name)
            .send()
            .await
        {
            Ok(response) => {
                if let Some(instance_profile) = response.instance_profile() {
                    let iam_profile_id = instance_profile.instance_profile_id();
                    info!(
                        "Found existing IAM Profile (id='{}'), proceeding with deletion...",
                        iam_profile_id
                    );

                    for role in instance_profile.roles() {
                        let role_name = role.role_name();
                        let _remove_role_response = match context
                            .iam_client
                            .remove_role_from_instance_profile()
                            .instance_profile_name(&profile_name)
                            .role_name(role_name)
                            .send()
                            .await
                        {
                            Ok(response) => {
                info!(
                                    "Removed IAM Role (name='{}') from IAM Profile (name='{}')",
                                    role_name, profile_name
                                );
                                response
                            }
                            Err(e) => {
                                error!("{:?}", e);
                                bail!(
                                    "Failure removing IAM Role (name='{}') from IAM Profile (name='{}')",
                                    role_name,
                                    profile_name
                                );
                            }
                        };
                    }
                } else {
                    info!(
                        "IAM Profile (name='{}') does not exist, skipping deletion...",
                        profile_name
                    );
                    return Ok(());
                }
            }
            Err(SdkError::ServiceError(service_err)) => match service_err.err() {
                GetInstanceProfileError::NoSuchEntityException(_) => {
                    info!(
                        "IAM Profile (name='{}') does not exist, skipping deletion...",
                        profile_name
                    );
                    return Ok(());
                }
                _ => {
                    error!("{:?}", service_err);
                    bail!("Failure describing IAM Profile '{}'", profile_name);
                }
            },
            Err(e) => {
                error!("{:?}", e);
                bail!("Failure describing IAM Profile '{}'", profile_name);
            }
        }

        let _delete_instance_profile_response = match context
            .iam_client
            .delete_instance_profile()
            .instance_profile_name(&profile_name)
            .send()
            .await
        {
            Ok(response) => {
                info!("Successfully deleted IAM Profile (name='{}')", profile_name);
                response
            }
            Err(e) => {
                error!("{:?}", e);
                bail!("Failed to delete IAM Profile (name='{}')", profile_name);
            }
        };

        Ok(())
    }
}
