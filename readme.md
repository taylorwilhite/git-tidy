# git-tidy
git-tidy is a simple command line utility to help easily clean up your git branches. By default it will remove all branches merged to main while protecting `master` and `develop`, and additional protected branches can be added in configuration, either at a project or global level.

Supported flags are:
--merged - only show merged branches
--older-than=30d - filter by age
--dry-run - preview without deleting
--force - skip confirmations (use carefully)
--keep-pattern=feature/\* - regex to protect certain branches
