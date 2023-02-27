// SPDX-License-Identifier: MIT

pragma solidity ^0.8.16;

import {IBonsaiProxy} from "./IBonsaiProxy.sol";
import {IBonsaiApp} from "./IBonsaiApp.sol";

/// @title A starter application using Bonsai through the on-chain proxy.
/// @dev This contract demonstrates one pattern for offloading the computation of an expensive
//       or difficult to implement function to a RISC Zero guest running on Bonsai.
contract QuadraticFunding is IBonsaiApp {
    // Address of the Bonsai proxy contract.
    IBonsaiProxy public immutable bonsai_proxy;
    // Image ID of the associated RISC Zero guest program.
    bytes32 public immutable image_id;

    struct Grant {
        uint256 id;
        uint256 totalDonated;
        address payable recipient;
    }

    mapping(uint256 => uint256[]) public donations;
    mapping(uint256 => Grant) public grants;
    uint256 public grantCount;

    uint256 public matchingPool;

    event CalculateFibonacciCallback(uint256 indexed n, uint256 result);

    // Initialize the contract, binding it to a specified Bonsai proxy and RISC Zero guest image.
    constructor(IBonsaiProxy _bonsai_proxy, bytes32 _image_id) {
        bonsai_proxy = _bonsai_proxy;
        image_id = _image_id;
    }

    function addGrant(address payable recipient) external {
        grants[grantCount] = Grant(grantCount, 0, recipient);
        grantCount++;
    }

    function donate(uint256 grantId) external payable {
        donations[grantId].push(msg.value);
        grants[grantId].totalDonated += msg.value;
    }

    function donateMatching() external payable {
        matchingPool += msg.value;
    }

    function payout() external {
        uint256[][] memory grantDonations;
        for (uint256 i = 0; i < grantCount; i++) {
            grantDonations[i] = donations[i];
        }
        bonsai_proxy.submit_request(image_id, abi.encode(grantDonations, matchingPool), address(this));
    }

    /// @notice Callback function to be called by the Bonsai proxy when the result is ready.
    /// @param _image_id The verified image ID for the RISC Zero guest that produced the journal.
    ///        It must be checked to match the specific image ID of the associated RISC Zero guest.
    /// @param journal Data committed by the guest program with the results and important context.
    function callback(bytes32 _image_id, bytes calldata journal) external {
        // Require that caller is the trusted proxy contract and guest program.
        require(msg.sender == address(bonsai_proxy), "calls must come from Bonsai");
        require(_image_id == image_id, "journal must be expected guest");
        (uint256 n, uint256 result) = abi.decode(journal, (uint256, uint256));
    }
}
