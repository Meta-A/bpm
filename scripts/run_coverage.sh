#!/bin/bash

# exist the script if any command fails
set -e

# Check if SKIP_CRATE is set
if [ -z "$SKIP_CRATE" ]; then
  echo "No crate to skip. Running tarpaulin for all crates."
else
  echo "Skipping crate: $SKIP_CRATE"
fi

# Find all crates by looking for Cargo.toml files
for crate in $(find packages -name Cargo.toml -exec dirname {} \;); do
  CRATE_NAME=$(basename $crate)

  # Skip the crate if SKIP_CRATE is set and matches the current crate
  if [ -n "$SKIP_CRATE" ] && [ "$CRATE_NAME" = "$SKIP_CRATE" ]; then
    echo "Skipping tarpaulin for $CRATE_NAME"
    continue
  fi

  echo "Running coverage for crate: $CRATE_NAME"
  mkdir -p "$crate/tarpaulin_temp"
  mkdir -p "$crate/coverage"
  
  cargo tarpaulin --manifest-path $crate/Cargo.toml --out lcov \
    --output-dir $crate/coverage --exclude-files "/target/**/*" "/tarpaulin_temp/**/*" \
    --target-dir $crate/tarpaulin_temp --skip-clean || echo "Tarpaulin failed for $CRATE_NAME"

  # Check if the coverage report exists and upload it
  if [ -f "$crate/coverage/lcov.info" ]; then
    echo "Uploading coverage for $CRATE_NAME"
    coveralls --lcov-file "$crate/coverage/lcov.info" --repo-token $COVERALLS_REPO_TOKEN
  else
    echo "No lcov.info file found for $CRATE_NAME"
  fi

  # Generate a badge link and store it in a markdown file for the crate
  echo "Generating badge for $CRATE_NAME"
  echo "[![Coverage Status](https://coveralls.io/repos/github/Meta-A/bbpm/${CRATE_NAME}/badge.svg?branch=main)](https://coveralls.io/github/Meta-A/bbpm/${CRATE_NAME}?branch=main)" > "${CRATE_NAME}.md"
  echo "Badge for $CRATE_NAME saved in ${CRATE_NAME}.md"
done
