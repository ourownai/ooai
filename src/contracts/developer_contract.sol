// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@chainlink/contracts/src/v0.8/ChainlinkClient.sol";

contract ProjectManagement is ERC721, Ownable, ChainlinkClient {
    using Chainlink for Chainlink.Request;

    struct Rewards {
        uint256 concept;
        uint256 testing;
        uint256 production;
    }

    struct Task {
        string id;
        string title;
        string description;
        string status;
        Rewards rewards;
    }

    struct Contributor {
        string name;
        string email;
        address walletAddress;
    }

    struct Contribution {
        string taskId;
        string contributorId;
        string commitHash;
        string mergeStage;
    }

    mapping(string => Task) public tasks;
    mapping(string => Contributor) public contributors;
    mapping(string => Contribution[]) public contributions;
    mapping(uint256 => string) public nftIdToContributionId;

    uint256 private nftCounter;
    address private oracleAddress;
    bytes32 private jobId;
    uint256 private fee;

    event TaskAdded(string taskId);
    event ContributorAdded(string contributorId);
    event ContributionAdded(string contributionId, string taskId, string contributorId);
    event NFTMinted(uint256 nftId, string contributionId);

    constructor(address _oracleAddress, string memory _jobId, uint256 _fee) ERC721("ProjectNFT", "PNFT") {
        setPublicChainlinkToken();
        oracleAddress = _oracleAddress;
        jobId = stringToBytes32(_jobId);
        fee = _fee;
    }

    function addTask(string memory _id, string memory _title, string memory _description, string memory _status, uint256 _concept, uint256 _testing, uint256 _production) public onlyOwner {
        Rewards memory newRewards = Rewards(_concept, _testing, _production);
        tasks[_id] = Task(_id, _title, _description, _status, newRewards);
        emit TaskAdded(_id);
    }

    function addContributor(string memory _id, string memory _name, string memory _email, address _walletAddress) public onlyOwner {
        contributors[_id] = Contributor(_name, _email, _walletAddress);
        emit ContributorAdded(_id);
    }

    function addContribution(string memory _taskId, string memory _contributorId, string memory _commitHash, string memory _mergeStage) public {
        require(bytes(tasks[_taskId].id).length != 0, "Task does not exist.");
        require(bytes(contributors[_contributorId].name).length != 0, "Contributor does not exist.");

        string memory contributionId = string(abi.encodePacked(_taskId, "-", _contributorId, "-", _commitHash));
        Contribution memory newContribution = Contribution(_taskId, _contributorId, _commitHash, _mergeStage);
        contributions[_taskId].push(newContribution);
        emit ContributionAdded(contributionId, _taskId, _contributorId);
    }

    function updateTaskStatus(string memory _taskId, string memory _status) public onlyOwner {
        require(bytes(tasks[_taskId].id).length != 0, "Task does not exist.");
        tasks[_taskId].status = _status;
    }

    function mintNFT(string memory _contributionId) public {
        Contribution memory contribution = getContributionById(_contributionId);
        require(bytes(contribution.taskId).length != 0, "Contribution does not exist.");

        Chainlink.Request memory request = buildChainlinkRequest(jobId, address(this), this.fulfillNFTMinting.selector);
        request.add("contributionId", _contributionId);
        sendChainlinkRequestTo(oracleAddress, request, fee);
    }

    function fulfillNFTMinting(bytes32 _requestId, bool _isValid) public recordChainlinkFulfillment(_requestId) {
        string memory contributionId = requestIdToContributionId[_requestId];
        require(bytes(contributionId).length != 0, "Invalid contribution ID.");

        if (_isValid) {
            uint256 newNftId = nftCounter;
            nftCounter++;
            _safeMint(msg.sender, newNftId);
            nftIdToContributionId[newNftId] = contributionId;
            emit NFTMinted(newNftId, contributionId);
        }
    }

    function getContributionById(string memory _contributionId) private view returns (Contribution memory) {
        string[] memory parts = splitString(_contributionId, "-");
        require(parts.length == 3, "Invalid contribution ID format.");

        string memory taskId = parts[0];
        string memory contributorId = parts[1];
        string memory commitHash = parts[2];

        Contribution[] memory taskContributions = contributions[taskId];
        for (uint256 i = 0; i < taskContributions.length; i++) {
            if (compareStrings(taskContributions[i].contributorId, contributorId) &&
                compareStrings(taskContributions[i].commitHash, commitHash)) {
                return taskContributions[i];
            }
        }

        return Contribution("", "", "", "");
    }

    function splitString(string memory _str, string memory _delimiter) private pure returns (string[] memory) {
        // Split string implementation
    }

    function compareStrings(string memory _a, string memory _b) private pure returns (bool) {
        // Compare strings implementation
    }

    function stringToBytes32(string memory _str) private pure returns (bytes32) {
        // Convert string to bytes32 implementation
    }

    mapping(bytes32 => string) private requestIdToContributionId;
}