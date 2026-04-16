---
name: git
category: version-control
description: Distributed version control system for tracking changes in source code
tags: [version-control, vcs, repository, commit, branch, merge, clone, rebase, cherry-pick]
author: oxo-call built-in
source_url: "https://git-scm.com/docs"
---

## Concepts

- Git commands use subcommands: 'git <subcommand> [options] [args]'. Always include the subcommand (clone, commit, push, pull, add, etc.) as the first argument.
- Staging area (index) is between working tree and repository. 'git add' stages changes; 'git commit' commits only what is staged. Use 'git commit -a' to stage all tracked-file changes and commit in one step.
- Branches are lightweight pointers to commits. Use 'git checkout -b <branch>' or 'git switch -c <branch>' to create and switch to a new branch.
- Remote operations: 'git clone' copies a remote repo; 'git fetch' downloads remote changes without merging; 'git pull' = fetch + merge; 'git push' uploads local commits to remote.
- Undo operations: 'git restore <file>' discards unstaged changes; 'git restore --staged <file>' unstages; 'git revert <sha>' creates a new undo commit (safe for shared history); 'git reset --hard' is destructive.
- git rm removes files from both working tree and index. Use 'git rm --cached' to stop tracking a file without deleting it from disk. Deletion via git rm is staged and committed, making it part of history.
- cherry-pick applies specific commits from one branch to another without merging; useful for backporting fixes.
- bisect performs binary search through commit history to find which commit introduced a bug.
- worktree allows multiple working directories from a single repository; useful for working on multiple branches simultaneously.
- submodule includes external repositories within a parent repository; requires special handling for updates.
- sparse-checkout reduces working tree to a subset of tracked files; useful for large repositories.
- reflog records all reference updates; useful for recovering "lost" commits after reset or rebase.

## Pitfalls

- For 'git clone', always include the repository URL as the last argument, e.g. 'git clone --depth 1 https://github.com/user/repo.git'. Omitting the URL will cause an error.
- For 'git commit', do not mix 'git add' and 'git commit' in a single args string with '&&'. Use 'git commit -a -m "message"' to stage all tracked changes and commit in one command.
- Commit messages with spaces must be quoted: 'git commit -m "fix: correct index off-by-one"'. Multi-word messages without quotes will be interpreted as multiple arguments.
- 'git reset --hard HEAD~N' and 'git push --force' permanently rewrite history — never use on shared/public branches without team consensus.
- 'git rm -r --force .' removes all tracked files recursively without confirmation. Always verify the scope with 'git status' before running destructive git rm commands.
- git push requires specifying the remote and branch the first time: 'git push -u origin main'. After that, 'git push' uses the cached upstream.
- cherry-pick can cause conflicts if the commit depends on previous changes; resolve conflicts with --continue or abort with --abort.
- bisect requires a known good commit and a known bad commit; mark commits with 'git bisect good/bad' during the search.
- worktree directories must be outside the main repository; cannot create worktree inside existing repo directory.
- submodule updates require 'git submodule update --init --recursive' after cloning; collaborators may miss submodules without this.
- sparse-checkout requires cone mode for best performance; non-cone mode has limitations and is not recommended.
- reflog entries expire after 90 days by default; recover lost commits quickly before garbage collection removes them.

## Examples

### clone a repository with shallow history (last commit only) on a specific branch
**Args:** `clone --depth 1 --branch main https://github.com/user/repo.git`
**Explanation:** --depth 1 limits history to 1 commit; --branch specifies the branch; URL is required

### stage all changes and commit with a message
**Args:** `commit -a -m "fix: resolve null pointer in parser"`
**Explanation:** -a stages all modified tracked files automatically; -m provides the commit message inline

### push the current branch to origin and set upstream tracking
**Args:** `push -u origin main`
**Explanation:** -u sets the upstream so future 'git push' without args works; 'origin' is the remote name

### create and switch to a new branch
**Args:** `checkout -b feature/new-api`
**Explanation:** -b creates the branch and switches to it in one step

### view the commit log with one-line summaries and branch graph
**Args:** `log --oneline --graph --decorate --all`
**Explanation:** --oneline for compact output; --graph shows branch topology; --decorate shows ref names

### show unstaged and staged changes
**Args:** `diff HEAD`
**Explanation:** diff HEAD shows all uncommitted changes (both staged and unstaged) against the last commit

### stash current working tree changes to switch branches cleanly
**Args:** `stash push -m "WIP: experiment with new feature"`
**Explanation:** stash push saves working tree changes; -m provides a descriptive stash message

### rebase current branch onto main to update with upstream changes
**Args:** `rebase origin/main`
**Explanation:** replays current branch commits on top of origin/main; resolve conflicts if they arise

### stop tracking a file without deleting it from disk
**Args:** `rm --cached secrets.env`
**Explanation:** --cached removes from index only, leaving the file on disk; commit afterward to record removal

### pull latest changes from remote and rebase local commits on top
**Args:** `pull --rebase origin main`
**Explanation:** --rebase keeps history linear by replaying local commits after the fetched commits

### cherry-pick a specific commit from another branch
**Args:** `cherry-pick a1b2c3d`
**Explanation:** applies commit a1b2c3d to current branch; useful for backporting fixes without full merge

### cherry-pick a range of commits
**Args:** `cherry-pick a1b2c3d^..e4f5g6h`
**Explanation:** applies all commits from a1b2c3d to e4f5g6h inclusive; ^ includes the first commit in the range

### find which commit introduced a bug using bisect
**Args:** `bisect start`
**Explanation:** starts binary search; then use 'git bisect bad' and 'git bisect good <commit>' to narrow down; 'git bisect reset' when done

### create a new worktree for parallel branch work
**Args:** `worktree add ../hotfix-branch hotfix/urgent-fix`
**Explanation:** creates separate working directory at ../hotfix-branch; allows simultaneous work on multiple branches

### initialize and update submodules after cloning
**Args:** `submodule update --init --recursive`
**Explanation:** initializes and updates all submodules recursively; required after cloning repos with submodules

### add a submodule to the repository
**Args:** `submodule add https://github.com/user/lib.git libs/lib`
**Explanation:** adds external repo as submodule at libs/lib; commits .gitmodules file; collaborators must run submodule update

### enable sparse-checkout for large repositories
**Args:** `sparse-checkout init --cone`
**Explanation:** enables sparse-checkout in cone mode; use 'git sparse-checkout set <dir>' to specify which directories to checkout

### view reflog to find lost commits
**Args:** `reflog`
**Explanation:** shows all reference updates; use commit hashes from reflog to recover lost commits after reset or rebase

### restore a file to a specific commit version
**Args:** `restore --source=HEAD~5 -- config.py`
**Explanation:** restores config.py to its state from 5 commits ago; --source specifies the commit to restore from
