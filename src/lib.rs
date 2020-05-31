use std::env;

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

pub fn project_name_from_url(url: &str) -> &str {
    let url_parts = url.split('/').collect::<Vec<&str>>();
    // Assuming that we always have the same GitHub URL, going to `unwrap` here.
    url_parts.get(4).expect("Unable to find 'project' part of the URL")
}

pub fn is_merged_pull_request(issue: &hubcaps::search::IssuesItem, username: &str) -> bool {
    !issue.html_url.contains(&username) &&
        issue.pull_request.is_some() &&
        issue.state != "open"
}

/// Set the limit to ENV['LIMIT'] or 20 if not set or can't be parsed
pub fn limit() -> u64 {
    match env::var("LIMIT") {
        Ok(value) => value.parse::<u64>().unwrap_or(20),
        Err(_) => 20
    }
}

#[cfg(feature = "test-utilities")]
pub mod tests_util {
    pub fn build_issues_item(html_url: String, state: String, pull_request: bool) -> hubcaps::search::IssuesItem {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tests_util::*;

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

}
