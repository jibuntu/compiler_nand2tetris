#!/bin/bash
# compilation engineをテストするためのshell script
compiler="../target/debug/compiler"
comparer="../../../../tools/TextComparer.sh"


function test () {
    echo $1": "
    compile_result=$($compiler $1 out.xml)
    result=$(sh $comparer $2 out.xml)

    printf "\x1b[31m" # 文字を赤色にする
    if [[ $result == *"Comparison ended successfully"* ]]; then
        printf "\x1b[32m" # 文字を黄緑色にする
    fi

    if [[ $compile_result != "" ]]; then
        echo "    "$compile_result
    fi

    echo "    "$result
    printf "\x1b[0m" # 文字色を戻す
    rm out.xml
}

function t () {
    test "xml/"$1.jack "xml/"$1.xml
}


cargo build

t "test"
t "class"
t "class_var_dec"
t "subroutine_dec"
t "var_dec"
