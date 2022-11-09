use octocrab::{models::pulls, params, Octocrab};
use tokio;

async fn process_pr(
    octocrab: &octocrab::Octocrab,
    pr: &pulls::PullRequest,
) -> octocrab::Result<()> {
    match pr.author_association {
        Some(ref association) => {
            if association.contains("CONTRIBUTOR") {
                // Do something.
                match pr.labels {
                    Some(ref labels) => {
                        let mut need_community_label = true;
                        let mut need_version_label = true;
                        for label in labels {
                            if label.name == "community" {
                                need_community_label = false;
                            }
                            if label.name == "version: 7X_ALPHA"
                                || label.name == "version: 6X_STABLE"
                                || label.name == "version: 5X_STABLE"
                            {
                                need_version_label = false;
                            }
                        }

                        if !need_community_label && !need_version_label {
                            return Ok(());
                        }

                        let mut labels_to_append: Vec<String> = vec![];
                        if need_community_label {
                            labels_to_append.push(String::from("community"));
                        }

                        if need_version_label {
                            if pr.base.ref_field == "main" {
                                labels_to_append.push(String::from("version: 7X_ALPHA"));
                            } else if pr.base.ref_field == "6X_STABLE" {
                                labels_to_append.push(String::from("version: 6X_STABLE"));
                            } else if pr.base.ref_field == "5X_STABLE" {
                                labels_to_append.push(String::from("version: 5X_STABLE"));
                            }
                        }

                        println!(
                            "Adding labels: {:?} to #{:?} - #{:?}",
                            labels_to_append, pr.number, pr.title
                        );

                        let _ = octocrab
                            .issues("greenplum-db", "gpdb")
                            .add_labels(pr.number, &labels_to_append[..])
                            .await?;
                        return Ok(());
                    }
                    None => return Ok(()),
                }
            } else {
                return Ok(());
            }
        }
        None => return Ok(()),
    }
}

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
    let octocrab = Octocrab::builder().personal_token(token).build()?;

    let pull_requests = octocrab
        .pulls("greenplum-db", "gpdb")
        .list()
        .state(params::State::Open)
        .sort(params::pulls::Sort::Created)
        .direction(params::Direction::Descending)
        // We only label the newest created 50 pull-requests.
        .per_page(50)
        .send()
        .await?;

    for pr in pull_requests {
        process_pr(&octocrab, &pr).await?;
    }

    Ok(())
}
