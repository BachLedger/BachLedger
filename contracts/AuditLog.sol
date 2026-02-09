// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title AuditLog
 * @notice Immutable audit logging for medical blockchain compliance
 * @dev Provides HIPAA-compliant audit trail for all medical data access
 *
 * Key Features:
 * - Immutable audit entries (cannot be modified or deleted)
 * - Categorized events for different types of access
 * - Time-based querying for compliance reports
 * - Integration with AccessControl for role-based logging
 */
contract AuditLog {
    // =========================================================================
    // Enums
    // =========================================================================

    /// @notice Type of audit event
    enum EventType {
        RECORD_CREATE,      // Medical record created
        RECORD_READ,        // Medical record accessed/viewed
        RECORD_UPDATE,      // Medical record updated
        ACCESS_GRANT,       // Access permission granted
        ACCESS_REVOKE,      // Access permission revoked
        LOGIN,              // User login
        LOGOUT,             // User logout
        ROLE_CHANGE,        // User role changed
        EMERGENCY_ACCESS,   // Emergency override access
        EXPORT_DATA,        // Data exported
        CONSENT_UPDATE      // Patient consent updated
    }

    /// @notice Severity level of the event
    enum Severity {
        INFO,       // Informational
        WARNING,    // Warning (unusual but not critical)
        CRITICAL    // Critical (security-relevant)
    }

    // =========================================================================
    // Structs
    // =========================================================================

    /// @notice Audit log entry structure
    struct AuditEntry {
        uint256 id;             // Unique entry ID
        uint256 timestamp;      // Block timestamp
        address actor;          // Who performed the action
        address subject;        // Who/what was affected (e.g., patient)
        EventType eventType;    // Type of event
        Severity severity;      // Severity level
        bytes32 resourceId;     // Identifier of the resource (e.g., record hash)
        string details;         // Additional details (JSON string)
        bytes32 txHash;         // Transaction hash for reference
    }

    // =========================================================================
    // State Variables
    // =========================================================================

    /// @notice Total number of audit entries
    uint256 public entryCount;

    /// @notice All audit entries by ID
    mapping(uint256 => AuditEntry) public entries;

    /// @notice Entries by actor (address => entry IDs)
    mapping(address => uint256[]) public entriesByActor;

    /// @notice Entries by subject (address => entry IDs)
    mapping(address => uint256[]) public entriesBySubject;

    /// @notice Entries by event type (eventType => entry IDs)
    mapping(EventType => uint256[]) public entriesByType;

    /// @notice Entries by date (day timestamp => entry IDs)
    mapping(uint256 => uint256[]) public entriesByDate;

    /// @notice Authorized loggers (contracts/addresses that can create logs)
    mapping(address => bool) public authorizedLoggers;

    /// @notice Contract owner
    address public owner;

    /// @notice Access control contract reference
    address public accessControl;

    // =========================================================================
    // Events
    // =========================================================================

    /// @notice Emitted when a new audit entry is created
    event AuditEntryCreated(
        uint256 indexed id,
        address indexed actor,
        address indexed subject,
        EventType eventType,
        Severity severity
    );

    /// @notice Emitted when a logger is authorized
    event LoggerAuthorized(address indexed logger);

    /// @notice Emitted when a logger is deauthorized
    event LoggerDeauthorized(address indexed logger);

    // =========================================================================
    // Modifiers
    // =========================================================================

    modifier onlyOwner() {
        require(msg.sender == owner, "AuditLog: caller is not owner");
        _;
    }

    modifier onlyAuthorized() {
        require(
            authorizedLoggers[msg.sender] || msg.sender == owner,
            "AuditLog: caller is not authorized"
        );
        _;
    }

    // =========================================================================
    // Constructor
    // =========================================================================

    constructor(address _accessControl) {
        owner = msg.sender;
        accessControl = _accessControl;
        authorizedLoggers[msg.sender] = true;
    }

    // =========================================================================
    // Admin Functions
    // =========================================================================

    /// @notice Authorize an address to create audit logs
    /// @param logger Address to authorize
    function authorizeLogger(address logger) external onlyOwner {
        require(logger != address(0), "AuditLog: invalid logger address");
        require(!authorizedLoggers[logger], "AuditLog: already authorized");

        authorizedLoggers[logger] = true;
        emit LoggerAuthorized(logger);
    }

    /// @notice Remove authorization from a logger
    /// @param logger Address to deauthorize
    function deauthorizeLogger(address logger) external onlyOwner {
        require(authorizedLoggers[logger], "AuditLog: not authorized");

        authorizedLoggers[logger] = false;
        emit LoggerDeauthorized(logger);
    }

    /// @notice Update access control contract reference
    /// @param _accessControl New access control contract address
    function setAccessControl(address _accessControl) external onlyOwner {
        require(_accessControl != address(0), "AuditLog: invalid address");
        accessControl = _accessControl;
    }

    /// @notice Transfer ownership
    /// @param newOwner New owner address
    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "AuditLog: invalid owner");
        owner = newOwner;
    }

    // =========================================================================
    // Logging Functions
    // =========================================================================

    /// @notice Create a new audit log entry
    /// @param actor Address that performed the action
    /// @param subject Address affected by the action
    /// @param eventType Type of event
    /// @param severity Severity level
    /// @param resourceId Resource identifier
    /// @param details Additional details (JSON string)
    /// @return entryId The ID of the created entry
    function log(
        address actor,
        address subject,
        EventType eventType,
        Severity severity,
        bytes32 resourceId,
        string calldata details
    ) external onlyAuthorized returns (uint256 entryId) {
        entryId = entryCount++;

        AuditEntry storage entry = entries[entryId];
        entry.id = entryId;
        entry.timestamp = block.timestamp;
        entry.actor = actor;
        entry.subject = subject;
        entry.eventType = eventType;
        entry.severity = severity;
        entry.resourceId = resourceId;
        entry.details = details;
        entry.txHash = blockhash(block.number - 1);

        // Index the entry
        entriesByActor[actor].push(entryId);
        if (subject != address(0)) {
            entriesBySubject[subject].push(entryId);
        }
        entriesByType[eventType].push(entryId);

        // Index by date (day granularity)
        uint256 dayTimestamp = (block.timestamp / 1 days) * 1 days;
        entriesByDate[dayTimestamp].push(entryId);

        emit AuditEntryCreated(entryId, actor, subject, eventType, severity);
    }

    /// @notice Log a record creation event
    function logRecordCreate(
        address doctor,
        address patient,
        bytes32 recordHash,
        string calldata details
    ) external onlyAuthorized returns (uint256) {
        return this.log(doctor, patient, EventType.RECORD_CREATE, Severity.INFO, recordHash, details);
    }

    /// @notice Log a record access event
    function logRecordAccess(
        address accessor,
        address patient,
        bytes32 recordHash,
        string calldata details
    ) external onlyAuthorized returns (uint256) {
        return this.log(accessor, patient, EventType.RECORD_READ, Severity.INFO, recordHash, details);
    }

    /// @notice Log an emergency access event
    function logEmergencyAccess(
        address accessor,
        address patient,
        bytes32 recordHash,
        string calldata reason
    ) external onlyAuthorized returns (uint256) {
        return this.log(accessor, patient, EventType.EMERGENCY_ACCESS, Severity.CRITICAL, recordHash, reason);
    }

    // =========================================================================
    // Query Functions
    // =========================================================================

    /// @notice Get an audit entry by ID
    /// @param id Entry ID
    /// @return The audit entry
    function getEntry(uint256 id) external view returns (AuditEntry memory) {
        require(id < entryCount, "AuditLog: entry does not exist");
        return entries[id];
    }

    /// @notice Get entry IDs for an actor
    /// @param actor Actor address
    /// @return Array of entry IDs
    function getEntriesByActor(address actor) external view returns (uint256[] memory) {
        return entriesByActor[actor];
    }

    /// @notice Get entry IDs for a subject
    /// @param subject Subject address
    /// @return Array of entry IDs
    function getEntriesBySubject(address subject) external view returns (uint256[] memory) {
        return entriesBySubject[subject];
    }

    /// @notice Get entry IDs by event type
    /// @param eventType Event type
    /// @return Array of entry IDs
    function getEntriesByEventType(EventType eventType) external view returns (uint256[] memory) {
        return entriesByType[eventType];
    }

    /// @notice Get entry IDs for a specific date
    /// @param timestamp Any timestamp within the desired day
    /// @return Array of entry IDs
    function getEntriesByDate(uint256 timestamp) external view returns (uint256[] memory) {
        uint256 dayTimestamp = (timestamp / 1 days) * 1 days;
        return entriesByDate[dayTimestamp];
    }

    /// @notice Get entries within a time range
    /// @param startTime Start timestamp
    /// @param endTime End timestamp
    /// @param maxResults Maximum number of results to return
    /// @return ids Array of entry IDs
    /// @return count Actual number of entries found
    function getEntriesInRange(
        uint256 startTime,
        uint256 endTime,
        uint256 maxResults
    ) external view returns (uint256[] memory ids, uint256 count) {
        require(startTime <= endTime, "AuditLog: invalid time range");

        ids = new uint256[](maxResults);
        count = 0;

        for (uint256 i = 0; i < entryCount && count < maxResults; i++) {
            if (entries[i].timestamp >= startTime && entries[i].timestamp <= endTime) {
                ids[count] = i;
                count++;
            }
        }
    }

    /// @notice Get the total number of entries for compliance reporting
    /// @return Total entry count
    function getTotalEntries() external view returns (uint256) {
        return entryCount;
    }

    /// @notice Get entry count by type
    /// @param eventType Event type to count
    /// @return Count of entries of this type
    function getEntryCountByType(EventType eventType) external view returns (uint256) {
        return entriesByType[eventType].length;
    }
}
