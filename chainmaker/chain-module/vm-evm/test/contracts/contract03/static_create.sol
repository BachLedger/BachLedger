// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.7.0 <0.9.0;

contract D {
    uint public x;
    constructor(uint a) payable {
        x = a;
    }

    function get() public view returns(uint){
        return x;
    }
}

contract C {
    function bytesCopy(bytes memory src, bytes memory dst, uint begin) internal pure returns (bytes memory){
        uint len = src.length < 31 ? src.length : 31;
        for(uint i = 0; i < len; i++){
            dst[begin + i] = src[i];
        }
        return dst;
    }

    function createDSalted(uint arg, string calldata name) public {
	    bytes memory bytesname = bytes(name);
        bytes memory custom = new bytes(32);

        uint8 rtTypeEvm = 5;
        custom[0] = bytes1(rtTypeEvm);
        custom = bytesCopy(bytesname, custom, 1);

        bytes32 n;
	    assembly{
	        n := mload(add(custom, 32))
	    }

        D d = new D{salt: n}(arg);
    }
}
