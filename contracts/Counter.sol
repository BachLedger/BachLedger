// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title Counter
 * @dev A simple counter contract for demonstrating state changes on BachLedger
 */
contract Counter {
    uint256 private _count;

    event CounterIncremented(uint256 newValue);
    event CounterDecremented(uint256 newValue);
    event CounterReset(uint256 previousValue);

    constructor() {
        _count = 0;
    }

    /**
     * @dev Increments the counter by 1
     */
    function increment() public {
        _count += 1;
        emit CounterIncremented(_count);
    }

    /**
     * @dev Decrements the counter by 1
     */
    function decrement() public {
        require(_count > 0, "Counter: cannot decrement below zero");
        _count -= 1;
        emit CounterDecremented(_count);
    }

    /**
     * @dev Resets the counter to 0
     */
    function reset() public {
        uint256 previousValue = _count;
        _count = 0;
        emit CounterReset(previousValue);
    }

    /**
     * @dev Returns the current count value
     */
    function get() public view returns (uint256) {
        return _count;
    }

    /**
     * @dev Adds a specific value to the counter
     */
    function add(uint256 value) public {
        _count += value;
        emit CounterIncremented(_count);
    }
}
