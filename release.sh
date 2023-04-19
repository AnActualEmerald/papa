#!/usr/bin/env bash

# 'inspired' by https://github.com/orhun/git-cliff/blob/main/release.sh

if [ -z "$1" ]; then
  echo "Please provide a tag"
  echo "Usage: ./release.sh v[X.Y.Z]"
  exit
fi

echo "Prep for release $1..."

# update version
msg="# managed by release.sh"
sed -E -i "s/^version = .* $msg$/version = \"${1#v}\" $msg/" papa*/Cargo.toml

# generate changelog
git cliff --tag "$1" --prepend CHANGELOG.md 

git add -A && git commit -m "chore(release): prep for $1"
git show

changelog=$(git cliff --unrelease --strip all)
git tag -s -a "$1" -m "Release $1" -m "$changelog"
git tag -v "$1"
echo "Done! (ready to 'git push' and 'git push --tags')"