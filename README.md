[![Build Status](https://travis-ci.com/phansch/prs.svg?branch=master)](https://travis-ci.com/phansch/prs)

# prs

Fetches a list of your open source PRs.

'open source' meaning that it excludes contributions to your own
repositories.

## Usage

```
GH_TOKEN=your_github_token LIMIT=15 cargo run
```

* `LIMIT` is optional, default is 20.
* `GH_TOKEN` is required and can be created [here][token].

[token]: https://github.com/settings/tokens
