// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.4.21;

contract sm2 {

    function number2Bytes(uint256 num) internal pure returns (bytes memory) {
        bytes memory b = new bytes(32);

        assembly{
            mstore(add(b, 32), num)
        }

        return b;
    }

    function verify(bytes memory pubKey, bytes memory message, bytes memory signature) public view returns (bytes32[2] memory) {
        bytes32[2] memory output;

        /* style 1 */
        //uint256 uPkLen  = pubKey.length;
        //uint256 uMsgLen = message.length;
        //uint256 uSigLen = signature.length;

        //bytes memory pkLen = new bytes(32);
        //bytes memory msgLen = new bytes(32);
        //bytes memory sigLen = new bytes(32);

        //assembly {
        //    mstore(add(pkLen, 32), uPkLen)
        //    mstore(add(msgLen, 32), uMsgLen)
        //    mstore(add(sigLen, 32), uSigLen)
        //}

        //bytes memory input = bytes.concat(pkLen, pubKey, msgLen, message, sigLen, signature);

        /* style 2 */
        bytes memory pkLen  = number2Bytes(pubKey.length);
        bytes memory msgLen = number2Bytes(message.length);
        bytes memory sigLen = number2Bytes(signature.length);
        bytes memory input  = bytes.concat(pkLen, pubKey, msgLen, message, sigLen, signature);

        /* style 3 */
        //bytes memory input = abi.encodePacked(pubKey.length, pubKey, message.length, message, signature.length, signature);
        uint256 inPutLen = input.length;

        assembly {
            if iszero(staticcall(0, 0x03ef, add(input, 32), inPutLen, output, 0x40)) {
                revert(0, 0)
            }
        }

        return output;
    }
}