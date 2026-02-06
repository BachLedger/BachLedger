// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.4.0;

abstract contract ICallee {
    function Adder(uint256 x, uint256 y) public virtual returns(uint256);
}

contract Caller {
    function crossCall(address addr, uint256 x, uint256 y) public returns(uint256) {
        ICallee callee = ICallee(addr);
        return callee.Adder(x, y);
    }
}
