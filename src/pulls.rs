use prs::{limit, project_name_from_url, is_merged_pull_request, State, PullRequest};

use std::env;

use futures::stream::Stream;
use tokio::runtime::Runtime;
use hubcaps::search::{IssuesSort, SearchIssuesOptions};
use hubcaps::{Credentials, Github, Result, SortDirection};

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
    Ok(prs.into_iter().map(parse_issue_item).collect())
}

fn parse_issue_item(pr: hubcaps::search::IssuesItem) -> PullRequest {
    let state = if pr.state == "closed" {
        State::Closed
    } else if pr.state == "open" {
        State::Open
    } else {
        panic!(format!("Unknown state '{}'", pr.state));
    };

    let project = project_name_from_url(&pr.html_url);

    PullRequest {
        title: pr.title,
        body: pr.body.unwrap_or_default(),
        project: project.to_string(),
        html_url: pr.html_url,
        state,
        closed_at: pr.closed_at.unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prs::tests_util::*;

    #[test]
    #[should_panic(expected = "Unknown state \'foo\'")]
    fn parse_pr_with_incorrect_state() {
        let res = build_issues_item(String::new(), String::from("foo"), true);
        parse_issue_item(res);
    }

    #[test]
    #[should_panic(expected = "Unable to find 'project' part of the URL")]
    fn parse_pr_with_incorrect_html_url() {
        let res = build_issues_item(String::new(), String::from("open"), true);
        parse_issue_item(res);
    }

    #[test]
    fn parse_pr_with_correct_data() {
        let res = build_issues_item(
            String::from("https://github.com/softprops/hubcaps/pull/245"),
            String::from("open"),
            true
        );
        assert_eq!(parse_issue_item(res), PullRequest {
            title: String::new(),
            body: String::new(),
            project: String::from("hubcaps"),
            html_url: String::from("https://github.com/softprops/hubcaps/pull/245"),
            state: State::Open,
            closed_at: String::new(),
        });
    }
}
