/// SPDX-License-Identifier: MIT
// solhint-disable not-rely-on-time
pragma solidity 0.8.23;

import {IERC721} from "@openzeppelin/contracts/token/ERC721/IERC721.sol";

import {IRootManager} from "../root-manager/IRootManager.sol";
import {IZKPTLC} from "../common-interface/IZKPTLC.sol";
import {IAsset} from "../common-interface/IAsset.sol";
import {ITransfer} from "../common-interface/ITransfer.sol";
import {IMerkleProof} from "../common-interface/IMerkleProof.sol";
import {TransferLib} from "../utils/TransferLib.sol";

contract NFTAtomicSwap is IZKPTLC {
    using TransferLib for ITransfer.Transfer;

    address public rootManagerAddress;
    mapping(bytes32 => uint256) public instanceSetAt;

    constructor(address _rootManagerAddress) {
        rootManagerAddress = _rootManagerAddress;
    }

    function computeInstance(
        address from,
        address to,
        address nftContract,
        uint256 tokenId,
        ITransfer.Transfer memory transfer
    ) public pure returns (bytes32) {
        bytes32 tc = transfer.transferCommitment();
        return keccak256(abi.encodePacked(tc, from, to, nftContract, tokenId));
    }

    function deposit(
        address to,
        address nftContract,
        uint256 tokenId,
        ITransfer.Transfer memory transfer
    ) external {
        bytes32 instance = computeInstance(
            msg.sender,
            to,
            nftContract,
            tokenId,
            transfer
        );
        require(instanceSetAt[instance] == 0, "Duplicate instance");
        IERC721(nftContract).transferFrom(msg.sender, address(this), tokenId);
        instanceSetAt[instance] = block.timestamp;
    }

    function withdraw(
        address from,
        address nftContract,
        uint256 tokenId,
        ITransfer.Transfer memory transfer,
        IMerkleProof.EvidenceWithMerkleProof memory proof
    ) external {
        bytes32 instance = computeInstance(
            from,
            msg.sender,
            nftContract,
            tokenId,
            transfer
        );
        require(instanceSetAt[instance] != 0, "Instance does not exist");
        instanceSetAt[instance] = 0;
        _verifyExistence(transfer, proof);
        IERC721(nftContract).transferFrom(address(this), msg.sender, tokenId);
    }

    function cancell(
        address to,
        address nftContract,
        uint256 tokenId,
        ITransfer.Transfer memory transfer
    ) external {
        bytes32 instance = computeInstance(
            msg.sender,
            to,
            nftContract,
            tokenId,
            transfer
        );
        require(instanceSetAt[instance] != 0, "Instance does not exist");
        require(
            instanceSetAt[instance] + 4 days < block.timestamp,
            "Instance is not expired"
        );
        instanceSetAt[instance] = 0;
        IERC721(nftContract).transferFrom(address(this), msg.sender, tokenId);
    }

    function _verifyExistence(
        ITransfer.Transfer memory transfer,
        IMerkleProof.EvidenceWithMerkleProof memory proof
    ) internal view {
        if (transfer.transferCommitment() != proof.leaf.transferCommitment) {
            revert("Transfer commitment does not match");
        }
        IRootManager(rootManagerAddress).verifyEvidenceMerkleProof(proof);
    }

    struct Witness {
        ITransfer.Transfer transfer;
        IMerkleProof.EvidenceWithMerkleProof proof;
        address from;
        address to;
        address nftContract;
        uint256 tokenId;
    }

    function verifyCondition(
        bytes32 instance,
        bytes memory witness
    ) external view returns (IAsset.AssetsDelta memory toOperatorDelta) {
        Witness memory w = abi.decode(witness, (Witness));
        bytes32 expectedInstance = computeInstance(
            w.from,
            w.to,
            w.nftContract,
            w.tokenId,
            w.transfer
        );
        if (instance != expectedInstance) {
            revert("Invalid instance");
        }
        _verifyExistence(w.transfer, w.proof);
        return toOperatorDelta; // return zero
    }
}
