# brew install librsvg imagemagick

find ~ -path ~/.Trash -prune -o -path ~/Library -prune -o -type d \
  -name .git \
  -exec sh -c './repo.sh {}' \;
