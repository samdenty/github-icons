# Worker Secret variables

`GITHUB_TOKEN` -> Default token used for lookups. If the user has access to private repos, the token could be leaked via the `headers` field on `/OWNER/PRIVATE_REPO/all`.
