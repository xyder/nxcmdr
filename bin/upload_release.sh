#!/usr/bin/env bash


# external env vars: GITLAB_TOKEN, GITLAB_PROJECT_ID

VERSION=0.2.3
API_V4_URL=https://gitlab.com/api/v4

echo "Uploading macosx build .."
target_archive=nxc-$VERSION-x86_64-apple-darwin.tar.gz
curl --header "PRIVATE-TOKEN: $GITLAB_TOKEN" \
    --upload-file ./releases/$target_archive \
    $API_V4_URL/projects/$GITLAB_PROJECT_ID/packages/generic/nxcmdr/$VERSION/$target_archive > /dev/null

echo "Uploading linux build .."
target_archive=nxc-$VERSION-x86_64-unknown-linux-gnu.tar.gz
curl --header "PRIVATE-TOKEN: $GITLAB_TOKEN" \
    --upload-file ./releases/$target_archive \
    $API_V4_URL/projects/$GITLAB_PROJECT_ID/packages/generic/nxcmdr/$VERSION/$target_archive > /dev/null

echo "Uploading sources .."
target_archive=sources.tar.gz
curl --header "PRIVATE-TOKEN: $GITLAB_TOKEN" \
    --upload-file ./releases/$target_archive \
    $API_V4_URL/projects/$GITLAB_PROJECT_ID/packages/generic/nxcmdr/$VERSION/$target_archive > /dev/null

echo "Done!"
