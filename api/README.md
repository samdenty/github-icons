# Worker Secret variables

`GITHUB_TOKEN` -> Default token used for lookups (if the user has access to no private repositories, the token could be leaked via the `headers` field on `/OWNER/PRIVATE_REPO/all`).
