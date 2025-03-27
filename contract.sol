// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract CredentialSystem {
    enum CredentialType {
        WorkExperience,
        Education,
        Certification,
        ProjectContribution,
        SkillEndorsement
    }

    struct Credential {
        address issuedTo;
        address issuedBy;
        CredentialType credentialType;
        string metadata;
        uint256 timestamp;
    }

    // Mapping: user => array of credentials
    mapping(address => Credential[]) public credentials;

    // Mapping: keccak(user, issuer, type) => metadata
    mapping(bytes32 => string) public requests;

    event CredentialRequested(address indexed user, address indexed issuer, CredentialType credentialType, string metadata);
    event CredentialIssued(address indexed user, address indexed issuer, CredentialType credentialType);

    // Modifier for identity check (stubbed for now)
    modifier hasIdentity(address _account) {
        require(_hasIdentity(_account), "Account lacks on-chain identity");
        _;
    }

    // User requests credential from issuer (and pays)
    function requestCredential(address issuer, CredentialType credentialType, string calldata metadata)
        external
        payable
        hasIdentity(msg.sender)
        hasIdentity(issuer)
    {
        require(msg.value >= 0.002 ether, "Insufficient fee");

        bytes32 key = _requestKey(msg.sender, issuer, credentialType);
        requests[key] = metadata;

        // Transfer fee to issuer
        (bool success, ) = issuer.call{value: msg.value}("");
        require(success, "Transfer failed");

        emit CredentialRequested(msg.sender, issuer, credentialType, metadata);
    }

    // Issuer approves and issues credential to user
    function issueCredential(address user, CredentialType credentialType)
        external
        hasIdentity(msg.sender)
        hasIdentity(user)
    {
        bytes32 key = _requestKey(user, msg.sender, credentialType);
        string memory metadata = requests[key];
        require(bytes(metadata).length > 0, "No request found");

        credentials[user].push(Credential({
            issuedTo: user,
            issuedBy: msg.sender,
            credentialType: credentialType,
            metadata: metadata,
            timestamp: block.timestamp
        }));

        delete requests[key];

        emit CredentialIssued(user, msg.sender, credentialType);
    }

    // Get credentials for a user
    function getCredentials(address user) external view returns (Credential[] memory) {
        return credentials[user];
    }

    // Internal helpers

    function _requestKey(address user, address issuer, CredentialType credentialType)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(abi.encodePacked(user, issuer, credentialType));
    }

    // Stub: Replace with proper identity integration (e.g. ENS, World ID, etc.)
    function _hasIdentity(address account) internal pure returns (bool) {
        return account != address(0); // basic placeholder
    }
}
