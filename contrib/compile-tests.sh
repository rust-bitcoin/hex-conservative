#!/usr/bin/env bash
#
# Checks intended compile failures.

set -euo pipefail

REPO_DIR=$(git rev-parse --show-toplevel)

compile_check() {
    local dir_path=$1
    local expected_status=$2
    local result=0
    local output
    # All crates in the specified subdirectory must compile or fail based on expected_status
    for dir in "$dir_path"/*; do
        if [ -d "$dir" ]; then
            pushd "$dir" > /dev/null
            echo "Compiling $dir"
            output=$(cargo build 2>&1)
            if [ "$?" -ne "$expected_status" ]; then
                if [ "$expected_status" -eq 0 ]; then
                    echo "$output"
                    echo "error: compile-tests/$dir/ failed to compile"
                else
                    echo "error: compile-tests/$dir/ compiled when it should not have"
                fi
                result=1
            fi
            popd > /dev/null
        fi
    done
    return "$result"
}

# Check that all files in compiletests pass or fail as expected
cd "$REPO_DIR"/tests/compile-tests
if compile_check "pass" 0 && compile_check "fail" 101; then
    echo "Compile tests passed"
    exit 0
else
    echo "Compile tests failed"
    exit 1
fi
