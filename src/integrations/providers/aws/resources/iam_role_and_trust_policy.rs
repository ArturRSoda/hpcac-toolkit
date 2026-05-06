use crate::integrations::providers::aws::{AwsInterface, interface::AwsClusterContext};

use anyhow::{Result, bail};
use aws_sdk_iam::error::SdkError;
use aws_sdk_iam::operation::detach_role_policy::DetachRolePolicyError;
use aws_sdk_iam::operation::get_role::GetRoleError;
use tracing::{error, info};

const SSM_MANAGED_POLICY_ARN: &str = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore";
const EC2_ASSUME_ROLE_TRUST_POLICY: &str = r#"{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Principal": {
                "Service": [
                    "ec2.amazonaws.com",
                    "ssm.amazonaws.com"
                ]
            },
            "Action": "sts:AssumeRole"
        }
    ]
}"#;

impl AwsInterface {
    pub async fn ensure_iam_role_and_trust_policies(
        &self,
        context: &AwsClusterContext,
    ) -> Result<String> {
        let role_name = context.iam_role_name.clone();

        let role_id = match context
            .iam_client
            .get_role()
            .role_name(&role_name)
            .send()
            .await
        {
            Ok(response) => {
                if let Some(role) = response.role() {
                    let iam_role_id = role.role_id();
                    info!(
                        "Found existing IAM Role (id='{}'), reconciling configuration...",
                        iam_role_id
                    );
                    iam_role_id.to_string()
                } else {
                    bail!("Unexpected empty IAM Role response for role '{}'", role_name);
                }
            }
            Err(SdkError::ServiceError(service_err)) => match service_err.err() {
                GetRoleError::NoSuchEntityException(_) => {
                    info!(
                        "IAM Role (name='{}') does not exist, will create it",
                        role_name
                    );

                    let create_iam_role_response = match context
                        .iam_client
                        .create_role()
                        .role_name(&role_name)
                        .assume_role_policy_document(EC2_ASSUME_ROLE_TRUST_POLICY)
                        .description("Role for EC2 instances to use Systems Manager")
                        .tags(
                            aws_sdk_iam::types::Tag::builder()
                                .key("Name")
                                .value(&role_name)
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
                            info!("Created IAM role (name='{}')", role_name);
                            response
                        }
                        Err(e) => {
                            error!("{:?}", e);
                            bail!("Failed to create IAM role (name='{}')", role_name);
                        }
                    };

                    create_iam_role_response
                        .role()
                        .unwrap()
                        .role_id()
                        .to_string()
                }
                _ => {
                    error!("{:?}", service_err);
                    bail!("Failure describing IAM Role (name='{}')", role_name);
                }
            },
            Err(e) => {
                error!("{:?}", e);
                bail!("Failure describing IAM Role (name='{}')", role_name);
            }
        };

        match context
            .iam_client
            .update_assume_role_policy()
            .role_name(&role_name)
            .policy_document(EC2_ASSUME_ROLE_TRUST_POLICY)
            .send()
            .await
        {
            Ok(_) => {
                info!("Ensured IAM Role '{}' trust policy is up to date", role_name);
            }
            Err(e) => {
                error!("{:?}", e);
                bail!(
                    "Failed to update trust policy for IAM Role (name='{}')",
                    role_name
                );
            }
        }

        match context
            .iam_client
            .attach_role_policy()
            .role_name(&role_name)
            .policy_arn(SSM_MANAGED_POLICY_ARN)
            .send()
            .await
        {
            Ok(_) => {
                info!("Attached SSM Policy to IAM Role '{}'", role_name);
            }
            Err(e) => {
                error!("{:?}", e);
                bail!(
                    "Failed to attach SSM Policy to IAM Role (name='{}')",
                    role_name
                );
            }
        }

        Ok(role_id)
    }

    pub async fn cleanup_trust_policies_and_iam_role(
        &self,
        context: &AwsClusterContext,
    ) -> Result<()> {
        let role_name = context.iam_role_name.clone();
        info!("Cleaning up IAM Role (name='{}')...", role_name);

        match context
            .iam_client
            .get_role()
            .role_name(&role_name)
            .send()
            .await
        {
            Ok(response) => {
                if let Some(role) = response.role() {
                    let iam_role_id = role.role_id();
                    info!(
                        "Found existing IAM Role (id='{}'), proceeding with deletion...",
                        iam_role_id
                    );
                } else {
                    info!(
                        "IAM Role (name='{}') does not exist, skipping deletion...",
                        role_name
                    );
                    return Ok(());
                }
            }
            Err(SdkError::ServiceError(service_err)) => match service_err.err() {
                GetRoleError::NoSuchEntityException(_) => {
                    info!(
                        "IAM Role (name='{}') does not exist, skipping deletion...",
                        role_name
                    );
                    return Ok(());
                }
                _ => {
                    error!("{:?}", service_err);
                    bail!("Failure describing IAM Role '{}'", role_name);
                }
            },
            Err(e) => {
                error!("{:?}", e);
                bail!("Failure describing IAM Role '{}'", role_name);
            }
        }

        match context
            .iam_client
            .detach_role_policy()
            .role_name(&role_name)
            .policy_arn(SSM_MANAGED_POLICY_ARN)
            .send()
            .await
        {
            Ok(_) => {
                info!(
                    "Detached SSM managed policy from IAM Role (name='{}')",
                    role_name
                );
            }
            Err(SdkError::ServiceError(service_err)) => match service_err.err() {
                DetachRolePolicyError::NoSuchEntityException(_) => {
                    info!(
                        "SSM managed policy was already absent from IAM Role (name='{}'), continuing...",
                        role_name
                    );
                }
                _ => {
                    error!("{:?}", service_err);
                    bail!(
                        "Failure detaching SSM managed policy from IAM Role (name='{}')",
                        role_name
                    );
                }
            },
            Err(e) => {
                error!("{:?}", e);
                bail!(
                    "Failure detaching SSM managed policy from IAM Role (name='{}')",
                    role_name
                );
            }
        };

        let _delete_iam_role_response = match context
            .iam_client
            .delete_role()
            .role_name(&role_name)
            .send()
            .await
        {
            Ok(response) => {
                info!("Successfully deleted IAM Role (name='{}')", role_name);
                response
            }
            Err(e) => {
                error!("{:?}", e);
                bail!("Failed to delete IAM Role (name='{}')", role_name);
            }
        };

        Ok(())
    }
}
