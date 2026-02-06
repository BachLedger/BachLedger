// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

contract Factory {

    function bytesCopy(bytes memory src, bytes memory dst, uint begin) internal pure returns (bytes memory){
        uint len = src.length < 31 ? src.length : 31;
        for(uint i = 0; i < len; i++){
            dst[begin + i] = src[i];
        }
        return dst;
    }

    function create(uint8 rtType, string calldata name, bytes calldata code) public returns (address addr){
        assert(0 < rtType && rtType < 8);

	    bytes memory bytesname = bytes(name);
	    bytes memory bytesCode = code;
        bytes memory custom = new bytes(32);
        custom[0] = bytes1(rtType);
        custom = bytesCopy(bytesname, custom, 1);

        assembly {
            //mstore8(add(custom, 32), rtType)
            addr := create2(rtType, add(bytesCode,0x20), mload(bytesCode), mload(add(custom, 32)))
        }
    }
}
