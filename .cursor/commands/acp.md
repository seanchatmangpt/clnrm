# Git Add, Commit, Push

Quick git workflow: stage all changes, auto generate a message, and push to remote.

## Command
When you type `/acp`, I will:
1. Stage all changes (`git add -A`)
2. Auto gen msg
3. Commit with your message (`git commit -m "<message>"`)
4. Push to remote (`git push`)

## What It Does
- **Stage all changes** - `git add -A`
- **Commit** - Create a commit with your provided message
- **Push** - Push to `origin/master` (or current branch)

## Use When
- After making code changes
- Ready to commit and push
- Want a quick git workflow

## Example
```
/acp

# I'll prompt: "Enter commit message:"
# You type: "Add telemetry validation infrastructure"
# Result: All changes staged, committed, and pushed
```

## Safety
- Always prompts for commit message (never empty)
- Uses `git add -A` to catch all changes
- Pushes to current branch

