#!/bin/bash
# compilation engineをテストするためのshell script
compiler="../target/debug/compiler"
comparer="../../../../tools/TextComparer.sh"

function test () {
    printf $1": "
    $compiler $1 out.xml
    sh $comparer $2 out.xml
    rm out.xml
}

cargo build
test xml/test.jack xml/test.xml
