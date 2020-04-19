#!/bin/bash

PATH_TO_RISCV_TESTS=../riscv-tests

for file in $(ls ${PATH_TO_RISCV_TESTS}/isa/rv*u*-*-* | grep -v uf- | grep -v ud- | grep -v .dump)
do
  cargo run --release $file -n 2>&1 | grep 'Running\|Test\|ECALL'
done

