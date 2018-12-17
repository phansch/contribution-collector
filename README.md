# myprs

Fetches a list of your open source PRs.

'open source' meaning that it excludes contributions to your own
repositories.

## Usage

```
GH_TOKEN=your_github_token LIMIT=15
```

* `LIMIT` is optional, default is 20.
* `GH_TOKEN` is required and can be created [here][token].

## Nice to haves

* Tests, maybe with [mockito][mockito]

[token]: https://github.com/settings/tokens
[mockito]: https://github.com/lipanski/mockito
