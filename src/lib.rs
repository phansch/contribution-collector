use std::env;

use futures::stream::Stream;
use tokio::runtime::Runtime;
use hubcaps::search::{IssuesSort, SearchIssuesOptions};
use hubcaps::{Credentials, Github, Result, SortDirection};

/// Fetch all my open and recently closed PRs
pub fn fetch() -> Result<()> {
    let mut rt = Runtime::new()?;
    let gh = Github::new(
        concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
        Credentials::Token(env::var("GH_TOKEN").unwrap())
    );
    let current_user = rt.block_on(gh.users().authenticated())?;
    let current_username = current_user.login;
    println!("current_username: {:?}", current_username);
    rt.block_on({
        let my_prs = gh
                     .search()
                     .issues()
                     .iter(
                         format!("author:{}", &current_username),
                         &SearchIssuesOptions::builder().sort(IssuesSort::Updated).per_page(100).order(SortDirection::Desc).build(),
                     )
                     .filter(move |res| {
                         !res.html_url.contains(&current_username) && res.pull_request.is_some()
                     })
                     .take(limit());
            my_prs.for_each(|res| {
                let text = format!(
                    "Title:     {:?}\n\
                    Body:      {:?}\n\
                    HTML URL:  {:?}\n\
                    URL:       {:?}\n\
                    State:     {:?}\n\
                    Closed at: {:?}\n\
                    ---------\n",
                    res.title, res.body, res.html_url, res.url,
                    res.state, res.closed_at
                );
                println!("{}", text);
                Ok(())
            })
    })?;
    Ok(())
}

/// Set the limit to ENV['LIMIT'] or 20 if not set or can't be parsed
fn limit() -> u64 {
    match env::var("LIMIT") {
        Ok(value) => value.parse::<u64>().unwrap_or(20),
        Err(_) => 20
    }
}
