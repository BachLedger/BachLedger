pragma solidity > 0.4.11;

contract Storage {
    uint number;

    function set(uint x) public {
        number = x;
    }

    function get() public view returns (uint retVal) {
        return number;
    }
}


