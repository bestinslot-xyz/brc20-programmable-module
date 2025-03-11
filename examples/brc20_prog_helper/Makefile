
compile:
	rm -rf output
	solcjs BRC20_Prog.sol --bin --abi
	find . -type f -name "*.abi" -exec sh -c 'python3 -m json.tool "$$1" > "$$1.tmp" && mv "$$1.tmp" "$$1"' _ {} \;
	mkdir -p output
	mv *.abi output/
	mv *.bin output/
