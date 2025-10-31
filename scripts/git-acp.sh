#!/bin/bash
# Git Add, Commit, Push - Convenience script
# Usage: ./scripts/git-acp.sh "Your commit message"

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 \"Your commit message\""
    exit 1
fi

COMMIT_MESSAGE="$1"

echo "🔄 Staging all changes..."
git add -A

echo "📝 Committing with message: $COMMIT_MESSAGE"
git commit -m "$COMMIT_MESSAGE"

echo "🚀 Pushing to remote..."
git push

echo "✅ Done! All changes have been committed and pushed."

