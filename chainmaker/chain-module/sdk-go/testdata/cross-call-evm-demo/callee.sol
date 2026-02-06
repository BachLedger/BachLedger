// SPDX-License-Identifier: GPL-3.0

pragma solidity > 0.5.21;

contract Callee {
    function Adder(uint256 x, uint256 y) public returns(uint256) {
        return x + y;
    }
}
