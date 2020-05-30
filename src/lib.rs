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
            .filter(move |issue| {
                !issue.html_url.contains(&current_username) && issue.pull_request.is_some() && issue.state != "open"
            })
            .take(limit())
            .collect()
    })?;
    Ok(
        prs.into_iter().map(|issue| {
            parse_pr(issue)
        }).collect()
    )
}

fn parse_pr(res: hubcaps::search::IssuesItem) -> PullRequest {
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
}

/// Set the limit to ENV['LIMIT'] or 20 if not set or can't be parsed
fn limit() -> u64 {
    match env::var("LIMIT") {
        Ok(value) => value.parse::<u64>().unwrap_or(20),
        Err(_) => 20
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Unknown state \'\'")]
    fn parse_pr_with_incorrect_state() {
        let res = hubcaps::search::IssuesItem {
            url: String::new(),
            repository_url: String::new(),
            labels_url: String::new(),
            comments_url: String::new(),
            events_url: String::new(),
            html_url: "some url".to_string(),
            id: 1,
            number: 1,
            title: String::new(),
            user: FAKE_USER,
            labels: Vec::new(),
            state: String::new(),
            locked: false,
            assignee: None,
            assignees: Vec::new(),
            comments: 1,
            created_at: String::new(),
            updated_at: String::new(),
            closed_at: None,
            pull_request: None,
            body: None,
        };
        parse_pr(res);
    }

    const FAKE_USER: hubcaps::users::User = hubcaps::users::User {
        login: String::new(),
        id: 1,
        avatar_url: String::new(),
        gravatar_id: String::new(),
        url: String::new(),
        html_url: String::new(),
        followers_url: String::new(),
        following_url: String::new(),
        gists_url: String::new(),
        starred_url: String::new(),
        subscriptions_url: String::new(),
        organizations_url: String::new(),
        repos_url: String::new(),
        events_url: String::new(),
        received_events_url: String::new(),
        site_admin: false,
    };
}
