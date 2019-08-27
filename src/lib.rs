use std::env;

use futures::stream::Stream;
use tokio::runtime::Runtime;
use hubcaps::search::{IssuesSort, SearchIssuesOptions};
use hubcaps::{Credentials, Github, Result, SortDirection};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub enum State {
    Open,
    Closed
}

#[derive(Serialize, Debug)]
pub struct PullRequest {
    pub title: String,
    pub body: String,
    pub project: String,
    pub html_url: String,
    pub state: State,
    pub closed_at: String,
}

/// Fetch all my open and recently closed PRs
pub fn fetch() -> Result<Vec<PullRequest>> {
    let mut rt = Runtime::new()?;
    let gh = Github::new(
        concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
        Credentials::Token(env::var("GH_TOKEN").expect("GH_TOKEN env variable not set!"))
    );
    let current_user = rt.block_on(gh.users().authenticated())?;
    let current_username = current_user.login;
    let prs = rt.block_on({
        gh
            .search()
            .issues()
            .iter(
                format!("author:{}", &current_username),
                &SearchIssuesOptions::builder().sort(IssuesSort::Updated).per_page(100).order(SortDirection::Desc).build(),
            )
            .filter(move |res| {
                !res.html_url.contains(&current_username) && res.pull_request.is_some() && res.state != "open"
            })
            .take(limit())
            .collect()
    })?;
    Ok(
        prs.into_iter().map(|res| {
            let state = if res.state == "closed" {
                State::Closed
            } else if res.state == "open" {
                State::Open
            } else {
                panic!(format!("Unknown state '{}'", res.state));
            };
            let url_parts = res.html_url.split('/').collect::<Vec<&str>>();
            // Assuming that we always have the same GitHub URL, going to `unwrap` here.
            let project = url_parts.get(4).expect("Unable to find 'project' part of the URL");

            PullRequest {
                title: res.title,
                body: res.body.unwrap_or_default(),
                project: project.to_string(),
                html_url: res.html_url,
                state,
                closed_at: res.closed_at.unwrap_or_default(),
            }
        }).collect()
    )
}

/// Set the limit to ENV['LIMIT'] or 20 if not set or can't be parsed
fn limit() -> u64 {
    match env::var("LIMIT") {
        Ok(value) => value.parse::<u64>().unwrap_or(20),
        Err(_) => 20
    }
}
