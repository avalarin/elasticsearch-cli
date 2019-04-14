#!/usr/bin/env bash

set -ex

publish() {
    brew bump-formula-pr --strict --tag=v${TRAVIS_TAG} --revision=${TRAVIS_COMMIT} ${PROJECT_NAME}
}

main() {
    publish
}

main