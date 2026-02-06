// SPDX-License-Identifier: GPL-3.0

pragma solidity >= 0.8.0;

contract CrossCall {
    string[8] params;

    function cross_call(address callee, string calldata method, string calldata time, string calldata name, string calldata hash) public {
        
	//CrossVMCall is reserved key word
        params[0] = "CrossVMCall";
        params[1] = method;
        params[2] = "time";
        params[3] = time;
        params[4] = "file_name";
        params[5] = name;
        params[6] = "file_hash";
        params[7] = hash;

	callee.call("");
    }
}
