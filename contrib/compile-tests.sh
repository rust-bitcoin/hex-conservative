#!/bin/bash

even_capacity() {
    cd even-capacity
    if cargo run > /dev/null 2>&1; then
        cd ..
        return 0
    else
        cd ..
        return 1
    fi
}

odd_capacity() {
    cd odd-capacity
    if cargo run > /dev/null 2>&1; then
        cd ..
        return 0
    else
        cd ..
        return 1
    fi
}

# Check that an even capacity BufEncoder compiles and an odd BufEncoder capacity fails to compile
cd tests/compiletests
if ! even_capacity; then
    echo "even-capacity BufEncoder failed to compile"
    exit 1
elif odd_capacity; then
    echo "odd-capacity BufEncoder compiled when it should not have"
    exit 1
else
    echo "BufEncoder capacity compile tests passed"
    exit 0
fi
