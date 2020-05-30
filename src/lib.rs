use std::env;

use futures::stream::Stream;
use tokio::runtime::Runtime;
use hubcaps::search::{IssuesSort, SearchIssuesOptions};
use hubcaps::{Credentials, Github, Result, SortDirection};
use serde::Serialize;

#[derive(Serialize, Debug, PartialEq)]
pub enum State {
    Open,
    Closed
}

#[derive(Serialize, Debug, PartialEq)]
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
            .filter(move |issue| { is_merged_pull_request(&issue, &current_username) })
            .take(limit())
            .collect()
    })?;
    Ok(prs.into_iter().map(parse_pr).collect())
}

fn parse_pr(pr: hubcaps::search::IssuesItem) -> PullRequest {
    let state = if pr.state == "closed" {
        State::Closed
    } else if pr.state == "open" {
        State::Open
    } else {
        panic!(format!("Unknown state '{}'", pr.state));
    };

    let url_parts = pr.html_url.split('/').collect::<Vec<&str>>();
    // Assuming that we always have the same GitHub URL, going to `unwrap` here.
    let project = url_parts.get(4).expect("Unable to find 'project' part of the URL");

    PullRequest {
        title: pr.title,
        body: pr.body.unwrap_or_default(),
        project: project.to_string(),
        html_url: pr.html_url,
        state,
        closed_at: pr.closed_at.unwrap_or_default(),
    }
}

fn is_merged_pull_request(issue: &hubcaps::search::IssuesItem, username: &str) -> bool {
    !issue.html_url.contains(&username) &&
        issue.pull_request.is_some() &&
        issue.state != "open"
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
    #[should_panic(expected = "Unknown state \'foo\'")]
    fn parse_pr_with_incorrect_state() {
        let res = build_issues_item(String::new(), String::from("foo"), true);
        parse_pr(res);
    }

    #[test]
    #[should_panic(expected = "Unable to find 'project' part of the URL")]
    fn parse_pr_with_incorrect_html_url() {
        let res = build_issues_item(String::new(), String::from("open"), true);
        parse_pr(res);
    }

    #[test]
    fn parse_pr_with_correct_data() {
        let res = build_issues_item(
            String::from("https://github.com/softprops/hubcaps/pull/245"),
            String::from("open"),
            true
        );
        assert_eq!(parse_pr(res), PullRequest {
            title: String::new(),
            body: String::new(),
            project: String::from("hubcaps"),
            html_url: String::from("https://github.com/softprops/hubcaps/pull/245"),
            state: State::Open,
            closed_at: String::new(),
        });
    }

    #[test]
    fn test_is_merged_pull_request() {
        let closed_pr = build_issues_item(String::from("https://github.com/softprops/hubcaps/pull/245"), String::from("closed"), true);
        assert!(is_merged_pull_request(&closed_pr, "lola"));

        let own_pr = build_issues_item(String::from("https://github.com/lola/lolas-project/pull/245"), String::from("closed"), true);
        assert!(!is_merged_pull_request(&own_pr, "lola"));

        let open_pr = build_issues_item(String::from("https://github.com/softprops/hubcaps/pull/245"), String::from("open"), true);
        assert!(!is_merged_pull_request(&open_pr, "lola"));

        let issue = build_issues_item(String::from("https://github.com/softprops/hubcaps/pull/245"), String::from("closed"), false);
        assert!(!is_merged_pull_request(&issue, "lola"));
    }

    fn build_issues_item(html_url: String, state: String, pull_request: bool) -> hubcaps::search::IssuesItem {
        let pull_request = if pull_request {
            Some(hubcaps::search::PullRequestInfo {
                url: String::new(),
                html_url: String::new(),
                diff_url: String::new(),
                patch_url: String::new(),
            })
        } else {
            None
        };
        hubcaps::search::IssuesItem {
            url: String::new(),
            repository_url: String::new(),
            labels_url: String::new(),
            comments_url: String::new(),
            events_url: String::new(),
            html_url,
            id: 1,
            number: 1,
            title: String::new(),
            user: FAKE_USER,
            labels: Vec::new(),
            state,
            locked: false,
            assignee: None,
            assignees: Vec::new(),
            comments: 1,
            created_at: String::new(),
            updated_at: String::new(),
            closed_at: None,
            pull_request,
            body: None,
        }
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
