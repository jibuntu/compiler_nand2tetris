#!/bin/bash
# tokenizerをテストするためのshell script
compiler="../target/debug/compiler"
comparer="../../../../tools/TextComparer.sh"

function test () {
    printf $1": "
    $compiler $1 > out.xml
    sh $comparer $2 out.xml
    rm out.xml
}

cargo build
test ../../ArrayTest/Main.jack ../../ArrayTest/MainT.xml
test ../../Square/Main.jack ../../Square/MainT.xml
test ../../Square/SquareGame.jack ../../Square/SquareGameT.xml
test ../../Square/Square.jack ../../Square/SquareT.xml