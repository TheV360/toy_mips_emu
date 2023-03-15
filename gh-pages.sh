#!/bin/bash
set -euo pipefail

git stash push -u --message "temporary push-to-gh-pages stash"
trunk build --release

sed -i -e '/\/www\/dist/d' ./.gitignore

# https://stackoverflow.com/a/40178818/

git add .gitignore
git add www/dist/

git push -d origin gh-pages || true # i am lazy
git commit -m "temporary push-to-gh-pages commit"
git subtree push --prefix www/dist origin gh-pages
# git push origin "$(git subtree split --prefix www/dist master)":gh-pages --force
git reset HEAD~

git checkout .gitignore

git stash pop
