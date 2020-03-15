#/bin/bash

path_to_riscv_tests="../riscv-tests"
out_directory="$(pwd)/tests"

if [ ! -e $out_directory ]; then
    mkdir $out_directory
fi

pushd $path_to_riscv_tests

for file in $(ls -F ./isa | grep -v / | grep -v Makefile | grep -v .dump | cut -d"*" -f1)
do
	echo isa/${file} to ${out_directory}/${file}
	cp isa/${file} ${out_directory}/${file}
done

popd
