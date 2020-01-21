#/bin/bash

path_to_riscv_tests="../riscv-tests"
path_to_objcopy="/opt/riscv/bin/riscv64-unknown-elf-objcopy"
out_directory="$(pwd)/tests"

if [ ! -e $out_directory ]; then
    mkdir $out_directory
fi

pushd $path_to_riscv_tests

for file in $(ls -F ./isa | grep -v / | grep -v Makefile | grep -v .dump | cut -d"*" -f1)
do
	echo isa/${file} to ${out_directory}/${file}
	$path_to_objcopy -O binary ./isa/${file} ${out_directory}/${file}
done

popd
