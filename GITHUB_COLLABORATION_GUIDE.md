# BEACON Project - Simple GitHub Collaboration Guide for 3 Developers

## Table of Contents

1. [Getting Started with Git & GitHub](#getting-started-with-git--github)
2. [Fork Workflow - Safe Development](#fork-workflow---safe-development)
3. [Basic Git Commands](#basic-git-commands)
4. [Simple Branching Strategy](#simple-branching-strategy)
5. [Development Workflow](#development-workflow)
6. [Pull Request Process](#pull-request-process)
7. [Code Review Guidelines](#code-review-guidelines)
8. [Common Problems & Solutions](#common-problems--solutions)
9. [Best Practices for 3 Developers](#best-practices-for-3-developers)

## Getting Started with Git & GitHub

### What You Need

- Git installed on your computer ([Download here](https://git-scm.com/))
- GitHub account
- Access to the BEACON project repository

### First Time Setup

1. **Configure Git with your information:**

```bash
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

2. **Fork the main repository:**

- Go to https://github.com/M-Chethipuzha/Beacon
- Click the "Fork" button in the top right
- This creates your own copy of the project

3. **Clone YOUR fork to your computer:**

```bash
git clone https://github.com/YOUR-USERNAME/Beacon.git
cd Beacon
```

4. **Add the main repository as "upstream":**

```bash
git remote add upstream https://github.com/M-Chethipuzha/Beacon.git
```

5. **Verify your remotes:**

```bash
git remote -v
# Should show:
# origin    https://github.com/YOUR-USERNAME/Beacon.git (fetch)
# origin    https://github.com/YOUR-USERNAME/Beacon.git (push)
# upstream  https://github.com/M-Chethipuzha/Beacon.git (fetch)
# upstream  https://github.com/M-Chethipuzha/Beacon.git (push)
```

## Fork Workflow - Safe Development

### Why Use Forks?

🛡️ **Safety First:**

- You can't accidentally break the main repository
- Experiment freely without affecting others
- Main repo stays clean and stable
- Easy to sync with team changes

### Fork vs Clone - What's the Difference?

**Fork** = Your own copy of the repository on GitHub  
**Clone** = Downloading a repository to your computer

```
Main Repo (M-Chethipuzha/Beacon)
    ↓ Fork
Your Fork (YOUR-USERNAME/Beacon)
    ↓ Clone
Your Computer (Local copy)
```

### Setting Up Your Fork

#### Step 1: Fork on GitHub

1. Go to the main BEACON repository
2. Click "Fork" button
3. Choose your GitHub account
4. Wait for fork to be created

#### Step 2: Clone Your Fork

```bash
# Clone YOUR fork (not the main repo)
git clone https://github.com/YOUR-USERNAME/Beacon.git
cd Beacon

# Add main repo as upstream
git remote add upstream https://github.com/M-Chethipuzha/Beacon.git
```

#### Step 3: Verify Setup

```bash
git remote -v
# origin = your fork (where you push)
# upstream = main repo (where you pull updates)
```

### Daily Fork Workflow

#### **Morning Routine - Stay Updated**

```bash
# Get latest changes from main repo
git checkout main
git fetch upstream
git merge upstream/main

# Push updates to your fork
git push origin main
```

#### **Starting New Work**

```bash
# Make sure you're on updated main
git checkout main
git fetch upstream
git merge upstream/main

# Create feature branch
git checkout -b feature-your-task-name

# Push branch to YOUR fork
git push origin feature-your-task-name
```

#### **Regular Development**

```bash
# Work on your code
git add .
git commit -m "Add feature X"

# Push to YOUR fork (not main repo)
git push origin feature-your-task-name
```

#### **Creating Pull Request**

1. Go to GitHub.com
2. Navigate to YOUR fork
3. Click "New Pull Request"
4. Set: `YOUR-USERNAME:feature-branch` → `M-Chethipuzha:main`
5. Write description and request review

### Staying Synchronized

#### **Problem: Main repo has new changes**

```bash
# Fetch updates from main repo
git fetch upstream

# Switch to your main branch
git checkout main

# Merge updates from main repo
git merge upstream/main

# Push updates to your fork
git push origin main

# Update your feature branch
git checkout feature-your-branch
git merge main
```

#### **Problem: Your fork is behind**

```bash
# Quick sync command
git checkout main
git pull upstream main
git push origin main
```

## Basic Git Commands

### Essential Commands Every Developer Needs

#### **Check Status of Your Files**

```bash
git status
```

Shows what files you've changed, added, or need to commit.

#### **Create a New Branch**

```bash
git branch feature-name
git checkout feature-name
```

Or do both in one command:

```bash
git checkout -b feature-name
```

#### **Switch Between Branches**

```bash
git checkout main          # Go to main branch
git checkout feature-name  # Go to your feature branch
```

#### **See All Branches**

```bash
git branch -a
```

#### **Add Files to Commit**

```bash
git add filename.rs              # Add specific file
git add .                        # Add all changed files
git add crates/beacon-core/      # Add entire folder
```

#### **Commit Your Changes**

```bash
git commit -m "Your commit message"
```

#### **Push Your Branch to GitHub (Your Fork)**

```bash
git push origin feature-name
```

#### **Pull Latest Changes from Main Repository**

```bash
git fetch upstream
git checkout main
git merge upstream/main
git push origin main
```

#### **See Your Commit History**

```bash
git log --oneline
```

## Simple Branching Strategy

We use a simple 2-branch strategy perfect for 3 developers:

### Branch Types

**In Main Repository:**

1. **`main`** - Production-ready code (protected, only maintainers can merge)

**In Your Fork:**

1. **`main`** - Stays synced with main repository
2. **`feature-*`** - Your individual work branches
3. **`bugfix-*`** - Bug fix branches
4. **`docs-*`** - Documentation branches

### Branch Naming Convention

- `feature-api-endpoints` - Adding new API features
- `feature-consensus-fix` - Fixing consensus issues
- `feature-ui-dashboard` - Working on UI components
- `bugfix-memory-leak` - Fixing bugs
- `docs-setup-guide` - Documentation updates

## Development Workflow

### Step-by-Step Process

#### 1. **Start New Work (Fork Workflow)**

```bash
# Sync your fork with main repository
git checkout main
git fetch upstream
git merge upstream/main
git push origin main

# Create your feature branch
git checkout -b feature-your-task-name

# Push branch to your fork
git push origin feature-your-task-name
```

#### 2. **Do Your Work**

- Write code
- Test your changes
- Commit frequently with clear messages

```bash
# Check what you've changed
git status

# Add your changes
git add .

# Commit with a clear message
git commit -m "Add new blockchain consensus validation"
```

#### 3. **Push Your Work to Your Fork**

```bash
git push origin feature-your-task-name
```

#### 4. **Create Pull Request (Fork to Main)**

- Go to GitHub website (your fork)
- Click "New Pull Request"
- Set direction: `YOUR-USERNAME:feature-branch` → `M-Chethipuzha:main`
- Write clear title and description
- Request review from teammates

#### 5. **After Review & Approval**

- Merge your pull request
- Delete your feature branch
- Start next task

## Pull Request Process

### Creating a Good Pull Request

#### Title Format

```
[Component] Brief description of change

Examples:
[Consensus] Fix validation bug in block processing
[API] Add health check endpoint
[Storage] Optimize database query performance
```

#### Description Template

```markdown
## What This PR Does

Brief explanation of your changes

## How to Test

1. Steps to test your changes
2. Commands to run
3. Expected results

## Checklist

- [ ] Code compiles without errors
- [ ] Tested locally
- [ ] Updated documentation if needed
- [ ] No breaking changes
```

### Review Process

1. **Author creates PR** → requests review from 1 other developer
2. **Reviewer checks code** → approves or requests changes
3. **Author addresses feedback** → pushes updates
4. **Final approval** → merge to main
5. **Clean up** → delete feature branch

## Code Review Guidelines

### What to Look For

#### ✅ **Good Things to Check**

- Does the code do what the PR says?
- Are there any obvious bugs?
- Is the code easy to understand?
- Are variable names clear?
- Does it follow Rust best practices?
- Will this break anything?

#### ❌ **What NOT to Focus On**

- Personal coding style preferences
- Minor formatting (let tools handle this)
- Rewriting everything differently

### Review Comments

#### **Good Review Comments:**

```
"This function could return an error if the file doesn't exist.
Should we handle that case?"

"Great fix! This will solve the memory issue we've been seeing."

"Can you add a comment explaining what this calculation does?"
```

#### **Avoid These Comments:**

```
"I would have done this differently"
"This is wrong" (without explanation)
"Change everything"
```

## Common Problems & Solutions

### Problem: "I can't push my changes"

**Solution (Fork Workflow):**

```bash
# Main repository has new changes, sync your fork
git checkout main
git fetch upstream
git merge upstream/main
git push origin main

# Update your feature branch
git checkout your-feature-branch
git merge main

# Resolve any conflicts, then push to your fork
git push origin your-feature-branch
```

### Problem: "My fork is behind the main repository"

**Solution:**

```bash
# Quick sync your fork
git checkout main
git fetch upstream
git merge upstream/main
git push origin main

# If you have a feature branch, update it too
git checkout your-feature-branch
git merge main
```

### Problem: "I made changes to the wrong branch"

**Solution:**

```bash
# If you haven't committed yet
git stash                          # Save your changes
git checkout correct-branch        # Switch to right branch
git stash pop                      # Get your changes back

# If you already committed
git checkout correct-branch
git cherry-pick commit-hash        # Copy the commit to right branch
```

### Problem: "I want to undo my last commit"

**Solution:**

```bash
# Undo last commit but keep the changes
git reset --soft HEAD~1

# Undo last commit and throw away changes (BE CAREFUL!)
git reset --hard HEAD~1
```

### Problem: "Merge conflict"

**Solution:**

1. Git will mark conflicted files
2. Open the file and look for `<<<<<<< HEAD` markers
3. Choose which version to keep
4. Remove the conflict markers
5. Add and commit the resolved file

```bash
git add conflicted-file.rs
git commit -m "Resolve merge conflict"
```

## Best Practices for 3 Developers

### Communication

- **Talk before big changes** - Let others know what you're working on
- **Small, frequent commits** - Easier to review and understand
- **Clear commit messages** - Others need to understand what you did

### Code Organization

- **One feature per branch** - Don't mix unrelated changes
- **Test your code** - Make sure it works before pushing
- **Update documentation** - If you change behavior, update docs

### Daily Routine

#### **Start of Day (Fork Workflow):**

```bash
# Sync your fork with main repository
git checkout main
git fetch upstream
git merge upstream/main
git push origin main
# Check if teammates merged anything new
```

#### **Before Creating PR (Fork Workflow):**

```bash
# Make sure your code works with latest main
git checkout main
git fetch upstream
git merge upstream/main
git push origin main

git checkout your-feature-branch
git merge main
# Test everything still works
```

#### **End of Day:**

```bash
# Save your work to your fork (even if not finished)
git add .
git commit -m "WIP: working on feature X"
git push origin your-feature-branch
```

### Project-Specific Guidelines

#### **BEACON Rust Code**

- Run `cargo fmt` before committing
- Run `cargo test` to make sure tests pass
- Check `cargo clippy` for code issues

```bash
# Quick check before committing
cargo fmt
cargo clippy
cargo test
```

#### **File Organization**

- **`crates/beacon-core/`** - Core blockchain logic
- **`crates/beacon-api/`** - REST API endpoints
- **`crates/beacon-consensus/`** - Consensus algorithms
- **`crates/beacon-storage/`** - Database operations
- **`edge-gateway-scs/`** - Python edge gateway
- **`docs/`** - All documentation

### Handling Emergencies

#### **Critical Bug in Production**

1. Create `hotfix-critical-issue` branch from main
2. Make minimal fix
3. Test quickly
4. Create PR with "URGENT" label
5. Get immediate review
6. Merge ASAP

#### **Breaking Someone's Work**

1. **Don't panic!**
2. Immediately notify the team
3. Check what broke using `git log`
4. Either fix quickly or revert the problematic commit
5. Learn from what went wrong

### Success Tips

1. **Commit often** - Small commits are easier to track and revert
2. **Pull before push** - Always check for updates before pushing
3. **Read the code** - Understand what teammates are doing
4. **Ask questions** - Better to ask than break something
5. **Keep it simple** - Don't over-complicate solutions

### Tools That Help

- **VS Code** - Great Git integration
- **GitHub Desktop** - Visual interface for Git
- **GitKraken** - Visual Git client (optional)

---

## Quick Reference Card

### Most Used Commands (Fork Workflow)

```bash
# Daily workflow with fork
git fetch upstream && git checkout main && git merge upstream/main
git checkout -b feature-name
git add .
git commit -m "message"
git push origin feature-name

# Staying updated with main repository
git fetch upstream
git checkout main
git merge upstream/main
git push origin main

# Fixing mistakes
git reset --soft HEAD~1  # Undo last commit
git stash                # Save changes temporarily
git stash pop            # Get stashed changes back
```

### Fork Setup Quick Reference

```bash
# One-time setup
git clone https://github.com/YOUR-USERNAME/Beacon.git
git remote add upstream https://github.com/M-Chethipuzha/Beacon.git

# Daily sync
git fetch upstream
git checkout main
git merge upstream/main
git push origin main
```

Remember: **When in doubt, ask your teammates!** It's better to ask a question than to accidentally break the project.
