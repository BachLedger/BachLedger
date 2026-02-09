// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title MedicalRecord
 * @notice Smart contract for managing patient medical records on the blockchain
 * @dev Implements role-based access control for doctors and patients
 *
 * Key Features:
 * - Role management (admin, doctors, patients)
 * - Encrypted medical record storage (only hash stored on-chain)
 * - Access logging for HIPAA compliance
 * - Patient-controlled data sharing
 */
contract MedicalRecord {
    // =========================================================================
    // State Variables
    // =========================================================================

    /// @notice Contract administrator
    address public admin;

    /// @notice Registered doctors
    mapping(address => bool) public doctors;

    /// @notice Registered patients
    mapping(address => bool) public patients;

    /// @notice Medical record structure
    struct Record {
        bytes32 dataHash;       // Hash of encrypted medical data (stored off-chain)
        address doctor;         // Doctor who created the record
        uint256 timestamp;      // Block timestamp when record was created
        bool isEncrypted;       // Whether the data is encrypted
        string recordType;      // Type of record (e.g., "diagnosis", "prescription", "lab_result")
    }

    /// @notice Patient address => list of their medical records
    mapping(address => Record[]) public records;

    /// @notice Patient => Doctor => Access permission
    mapping(address => mapping(address => bool)) public accessPermissions;

    /// @notice Patient => Record index => List of addresses who accessed
    mapping(address => mapping(uint256 => address[])) public accessHistory;

    // =========================================================================
    // Events
    // =========================================================================

    /// @notice Emitted when a new medical record is added
    event RecordAdded(
        address indexed patient,
        address indexed doctor,
        bytes32 dataHash,
        uint256 recordIndex,
        string recordType
    );

    /// @notice Emitted when a record is accessed
    event RecordAccessed(
        address indexed patient,
        address indexed accessor,
        uint256 recordIndex,
        uint256 timestamp
    );

    /// @notice Emitted when a doctor is registered
    event DoctorRegistered(address indexed doctor, address indexed registeredBy);

    /// @notice Emitted when a patient is registered
    event PatientRegistered(address indexed patient);

    /// @notice Emitted when access permission is granted
    event AccessGranted(address indexed patient, address indexed doctor);

    /// @notice Emitted when access permission is revoked
    event AccessRevoked(address indexed patient, address indexed doctor);

    // =========================================================================
    // Modifiers
    // =========================================================================

    modifier onlyAdmin() {
        require(msg.sender == admin, "MedicalRecord: caller is not admin");
        _;
    }

    modifier onlyDoctor() {
        require(doctors[msg.sender], "MedicalRecord: caller is not a doctor");
        _;
    }

    modifier onlyPatient() {
        require(patients[msg.sender], "MedicalRecord: caller is not a patient");
        _;
    }

    modifier onlyAuthorized(address patient) {
        require(
            msg.sender == patient ||
            msg.sender == admin ||
            accessPermissions[patient][msg.sender],
            "MedicalRecord: not authorized to access this patient's records"
        );
        _;
    }

    // =========================================================================
    // Constructor
    // =========================================================================

    constructor() {
        admin = msg.sender;
    }

    // =========================================================================
    // Admin Functions
    // =========================================================================

    /// @notice Register a new doctor
    /// @param doctor Address of the doctor to register
    function registerDoctor(address doctor) external onlyAdmin {
        require(doctor != address(0), "MedicalRecord: invalid doctor address");
        require(!doctors[doctor], "MedicalRecord: doctor already registered");

        doctors[doctor] = true;
        emit DoctorRegistered(doctor, msg.sender);
    }

    /// @notice Remove a doctor's registration
    /// @param doctor Address of the doctor to remove
    function removeDoctor(address doctor) external onlyAdmin {
        require(doctors[doctor], "MedicalRecord: doctor not registered");

        doctors[doctor] = false;
    }

    /// @notice Transfer admin role
    /// @param newAdmin Address of the new admin
    function transferAdmin(address newAdmin) external onlyAdmin {
        require(newAdmin != address(0), "MedicalRecord: invalid admin address");
        admin = newAdmin;
    }

    // =========================================================================
    // Patient Functions
    // =========================================================================

    /// @notice Register as a patient
    function registerAsPatient() external {
        require(!patients[msg.sender], "MedicalRecord: already registered as patient");

        patients[msg.sender] = true;
        emit PatientRegistered(msg.sender);
    }

    /// @notice Grant a doctor access to your records
    /// @param doctor Address of the doctor to grant access
    function grantAccess(address doctor) external onlyPatient {
        require(doctors[doctor], "MedicalRecord: not a registered doctor");
        require(!accessPermissions[msg.sender][doctor], "MedicalRecord: access already granted");

        accessPermissions[msg.sender][doctor] = true;
        emit AccessGranted(msg.sender, doctor);
    }

    /// @notice Revoke a doctor's access to your records
    /// @param doctor Address of the doctor to revoke access
    function revokeAccess(address doctor) external onlyPatient {
        require(accessPermissions[msg.sender][doctor], "MedicalRecord: access not granted");

        accessPermissions[msg.sender][doctor] = false;
        emit AccessRevoked(msg.sender, doctor);
    }

    // =========================================================================
    // Doctor Functions
    // =========================================================================

    /// @notice Add a medical record for a patient
    /// @param patient Address of the patient
    /// @param dataHash Hash of the encrypted medical data
    /// @param isEncrypted Whether the data is encrypted
    /// @param recordType Type of the medical record
    function addRecord(
        address patient,
        bytes32 dataHash,
        bool isEncrypted,
        string calldata recordType
    ) external onlyDoctor {
        require(patients[patient], "MedicalRecord: not a registered patient");
        require(accessPermissions[patient][msg.sender], "MedicalRecord: no access permission");
        require(dataHash != bytes32(0), "MedicalRecord: invalid data hash");

        Record memory newRecord = Record({
            dataHash: dataHash,
            doctor: msg.sender,
            timestamp: block.timestamp,
            isEncrypted: isEncrypted,
            recordType: recordType
        });

        records[patient].push(newRecord);
        uint256 recordIndex = records[patient].length - 1;

        emit RecordAdded(patient, msg.sender, dataHash, recordIndex, recordType);
    }

    // =========================================================================
    // View Functions
    // =========================================================================

    /// @notice Get the number of records for a patient
    /// @param patient Address of the patient
    /// @return Number of records
    function getRecordCount(address patient) external view returns (uint256) {
        return records[patient].length;
    }

    /// @notice Get a specific record for a patient
    /// @param patient Address of the patient
    /// @param index Index of the record
    /// @return Record data
    function getRecord(address patient, uint256 index)
        external
        view
        onlyAuthorized(patient)
        returns (Record memory)
    {
        require(index < records[patient].length, "MedicalRecord: record index out of bounds");
        return records[patient][index];
    }

    /// @notice Check if a doctor has access to a patient's records
    /// @param patient Address of the patient
    /// @param doctor Address of the doctor
    /// @return Whether access is granted
    function hasAccess(address patient, address doctor) external view returns (bool) {
        return accessPermissions[patient][doctor];
    }

    /// @notice Log record access (called when viewing records)
    /// @param patient Address of the patient
    /// @param recordIndex Index of the record accessed
    function logAccess(address patient, uint256 recordIndex) external onlyAuthorized(patient) {
        require(recordIndex < records[patient].length, "MedicalRecord: record index out of bounds");

        accessHistory[patient][recordIndex].push(msg.sender);
        emit RecordAccessed(patient, msg.sender, recordIndex, block.timestamp);
    }

    /// @notice Get access history for a record
    /// @param patient Address of the patient
    /// @param recordIndex Index of the record
    /// @return List of addresses who accessed the record
    function getAccessHistory(address patient, uint256 recordIndex)
        external
        view
        onlyAuthorized(patient)
        returns (address[] memory)
    {
        return accessHistory[patient][recordIndex];
    }
}
