
compile:
	rm -rf target
	solcjs BRC20_Prog.sol --bin --abi
	solcjs BRC20_Controller.sol --bin --abi
	cat BRC20_Prog_sol_BRC20_Prog.abi | python3 -m json.tool > ../BRC20_Prog.abi
	cat BRC20_Controller_sol_BRC20_Controller.abi | python3 -m json.tool > ../BRC20_Controller.abi
	cp BRC20_Prog_sol_BRC20_Prog.bin ../BRC20_Prog.bin
	cp BRC20_Controller_sol_BRC20_Controller.bin  ../BRC20_Controller.bin
	mkdir -p target
	mv *.abi target/
	mv *.bin target/
