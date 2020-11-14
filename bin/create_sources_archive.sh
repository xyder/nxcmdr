#!/usr/bin/env bash


temp_release_dir=./releases/temp
target_archive=./releases/sources.tar.gz

if [[ $(git status --porcelain ./crates | wc -l) -gt 0 ]]; then
    echo "Changes detected in ./crates . Commit or revert them."
    exit 1
fi

if [[ $(git status --porcelain ./src | wc -l) -gt 0 ]]; then
    echo "Changes detected in ./src . Commit or revert them."
    exit 1
fi

# reset temp release directory
rm -fr $temp_release_dir && mkdir -p $temp_release_dir

# copy sources
rsync -avm --include='*.rs,*.toml' --exclude 'zlf_*' ./crates $temp_release_dir > /dev/null
rsync -avm --include='*.rs,*.toml' --exclude 'zlf_*' ./src $temp_release_dir > /dev/null

# # copy license and readme
cp README.md LICENSE $temp_release_dir

# # create archive
tar -czvf $target_archive -C $temp_release_dir . > /dev/null

# # cleanup
rm -fr $temp_release_dir

echo "Created release archive $target_archive"
