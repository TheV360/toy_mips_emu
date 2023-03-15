#!/bin/bash
set -euo pipefail

trunk build --release
git subtree push --prefix www/dist origin gh-pages
