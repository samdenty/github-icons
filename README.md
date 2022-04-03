#

- Be able to change icon for a repo
- All icons are downloaded and cached
- ability to upload an icon for a repo slug
  - `git-icons set samdenty/gqless ./icon.png`
  - `git-icons set ~/Projects/gqless ./icon.png`
- sync
  - `git-icons sync`
  - `git-icons sync ~/Projects/gqless`
  - Sync runs repo-icons again on every repository in cache
  - Only downloads new icons
  - If the default hasn't been changed then updates it to the new first icon returned
