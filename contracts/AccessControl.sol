// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title AccessControl
 * @notice Role-based access control for medical blockchain system
 * @dev Implements a flexible role system with hierarchical permissions
 *
 * Key Features:
 * - Predefined roles: ADMIN, DOCTOR, PATIENT, AUDITOR
 * - Role granting/revoking with admin control
 * - Role hierarchy support
 * - Multi-role support per address
 */
contract AccessControl {
    // =========================================================================
    // Role Definitions
    // =========================================================================

    /// @notice Administrator role - can manage all other roles
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN");

    /// @notice Doctor role - can create and access medical records
    bytes32 public constant DOCTOR_ROLE = keccak256("DOCTOR");

    /// @notice Patient role - owns medical records
    bytes32 public constant PATIENT_ROLE = keccak256("PATIENT");

    /// @notice Auditor role - can view audit logs
    bytes32 public constant AUDITOR_ROLE = keccak256("AUDITOR");

    /// @notice Pharmacy role - can access prescriptions
    bytes32 public constant PHARMACY_ROLE = keccak256("PHARMACY");

    /// @notice Insurance role - can access approved claims data
    bytes32 public constant INSURANCE_ROLE = keccak256("INSURANCE");

    /// @notice Lab role - can submit lab results
    bytes32 public constant LAB_ROLE = keccak256("LAB");

    // =========================================================================
    // State Variables
    // =========================================================================

    /// @notice Role => Address => Has role
    mapping(bytes32 => mapping(address => bool)) public roles;

    /// @notice Role => Admin role that can manage it
    mapping(bytes32 => bytes32) public roleAdmin;

    /// @notice Address of the super admin (can manage ADMIN_ROLE)
    address public superAdmin;

    /// @notice Role metadata
    struct RoleData {
        string name;
        string description;
        bool exists;
    }

    /// @notice Role hash => Role metadata
    mapping(bytes32 => RoleData) public roleMetadata;

    // =========================================================================
    // Events
    // =========================================================================

    /// @notice Emitted when a role is granted
    event RoleGranted(
        bytes32 indexed role,
        address indexed account,
        address indexed sender
    );

    /// @notice Emitted when a role is revoked
    event RoleRevoked(
        bytes32 indexed role,
        address indexed account,
        address indexed sender
    );

    /// @notice Emitted when role admin is changed
    event RoleAdminChanged(
        bytes32 indexed role,
        bytes32 indexed previousAdminRole,
        bytes32 indexed newAdminRole
    );

    /// @notice Emitted when a new role is created
    event RoleCreated(
        bytes32 indexed role,
        string name,
        bytes32 indexed adminRole
    );

    // =========================================================================
    // Modifiers
    // =========================================================================

    /// @notice Restrict to accounts with a specific role
    modifier onlyRole(bytes32 role) {
        require(hasRole(role, msg.sender), "AccessControl: account lacks required role");
        _;
    }

    /// @notice Restrict to super admin
    modifier onlySuperAdmin() {
        require(msg.sender == superAdmin, "AccessControl: caller is not super admin");
        _;
    }

    // =========================================================================
    // Constructor
    // =========================================================================

    constructor() {
        superAdmin = msg.sender;

        // Set up role hierarchy - ADMIN_ROLE is managed by itself (super admin)
        roleAdmin[ADMIN_ROLE] = ADMIN_ROLE;
        roleAdmin[DOCTOR_ROLE] = ADMIN_ROLE;
        roleAdmin[PATIENT_ROLE] = ADMIN_ROLE;
        roleAdmin[AUDITOR_ROLE] = ADMIN_ROLE;
        roleAdmin[PHARMACY_ROLE] = ADMIN_ROLE;
        roleAdmin[INSURANCE_ROLE] = ADMIN_ROLE;
        roleAdmin[LAB_ROLE] = ADMIN_ROLE;

        // Grant admin role to deployer
        roles[ADMIN_ROLE][msg.sender] = true;
        emit RoleGranted(ADMIN_ROLE, msg.sender, msg.sender);

        // Initialize role metadata
        _initializeRoleMetadata();
    }

    // =========================================================================
    // Internal Functions
    // =========================================================================

    function _initializeRoleMetadata() internal {
        roleMetadata[ADMIN_ROLE] = RoleData({
            name: "Administrator",
            description: "System administrator with full access",
            exists: true
        });

        roleMetadata[DOCTOR_ROLE] = RoleData({
            name: "Doctor",
            description: "Healthcare provider who can create and access medical records",
            exists: true
        });

        roleMetadata[PATIENT_ROLE] = RoleData({
            name: "Patient",
            description: "Patient who owns medical records",
            exists: true
        });

        roleMetadata[AUDITOR_ROLE] = RoleData({
            name: "Auditor",
            description: "Compliance auditor who can view audit logs",
            exists: true
        });

        roleMetadata[PHARMACY_ROLE] = RoleData({
            name: "Pharmacy",
            description: "Pharmacy that can access and fulfill prescriptions",
            exists: true
        });

        roleMetadata[INSURANCE_ROLE] = RoleData({
            name: "Insurance",
            description: "Insurance provider for claims processing",
            exists: true
        });

        roleMetadata[LAB_ROLE] = RoleData({
            name: "Laboratory",
            description: "Medical laboratory that can submit test results",
            exists: true
        });
    }

    // =========================================================================
    // Role Management
    // =========================================================================

    /// @notice Check if an account has a role
    /// @param role The role to check
    /// @param account The account to check
    /// @return Whether the account has the role
    function hasRole(bytes32 role, address account) public view returns (bool) {
        return roles[role][account];
    }

    /// @notice Get the admin role for a role
    /// @param role The role to get the admin for
    /// @return The admin role
    function getRoleAdmin(bytes32 role) public view returns (bytes32) {
        return roleAdmin[role];
    }

    /// @notice Grant a role to an account
    /// @param role The role to grant
    /// @param account The account to grant the role to
    function grantRole(bytes32 role, address account) external {
        require(
            hasRole(getRoleAdmin(role), msg.sender) || msg.sender == superAdmin,
            "AccessControl: sender must be role admin"
        );
        require(account != address(0), "AccessControl: cannot grant role to zero address");

        if (!roles[role][account]) {
            roles[role][account] = true;
            emit RoleGranted(role, account, msg.sender);
        }
    }

    /// @notice Revoke a role from an account
    /// @param role The role to revoke
    /// @param account The account to revoke the role from
    function revokeRole(bytes32 role, address account) external {
        require(
            hasRole(getRoleAdmin(role), msg.sender) || msg.sender == superAdmin,
            "AccessControl: sender must be role admin"
        );

        if (roles[role][account]) {
            roles[role][account] = false;
            emit RoleRevoked(role, account, msg.sender);
        }
    }

    /// @notice Renounce a role (self-revoke)
    /// @param role The role to renounce
    function renounceRole(bytes32 role) external {
        require(roles[role][msg.sender], "AccessControl: account does not have role");

        roles[role][msg.sender] = false;
        emit RoleRevoked(role, msg.sender, msg.sender);
    }

    /// @notice Set the admin role for a role
    /// @param role The role to set admin for
    /// @param adminRole The new admin role
    function setRoleAdmin(bytes32 role, bytes32 adminRole) external onlySuperAdmin {
        bytes32 previousAdminRole = roleAdmin[role];
        roleAdmin[role] = adminRole;
        emit RoleAdminChanged(role, previousAdminRole, adminRole);
    }

    /// @notice Transfer super admin role
    /// @param newSuperAdmin Address of the new super admin
    function transferSuperAdmin(address newSuperAdmin) external onlySuperAdmin {
        require(newSuperAdmin != address(0), "AccessControl: invalid super admin");
        superAdmin = newSuperAdmin;
    }

    // =========================================================================
    // Custom Role Creation
    // =========================================================================

    /// @notice Create a new custom role
    /// @param role The role identifier
    /// @param name Human-readable name
    /// @param description Description of the role
    /// @param adminRole The admin role that can manage this role
    function createRole(
        bytes32 role,
        string calldata name,
        string calldata description,
        bytes32 adminRole
    ) external onlyRole(ADMIN_ROLE) {
        require(!roleMetadata[role].exists, "AccessControl: role already exists");
        require(bytes(name).length > 0, "AccessControl: name cannot be empty");

        roleMetadata[role] = RoleData({
            name: name,
            description: description,
            exists: true
        });

        roleAdmin[role] = adminRole;
        emit RoleCreated(role, name, adminRole);
    }

    // =========================================================================
    // View Functions
    // =========================================================================

    /// @notice Get role metadata
    /// @param role The role to get metadata for
    /// @return name The role name
    /// @return description The role description
    function getRoleMetadata(bytes32 role)
        external
        view
        returns (string memory name, string memory description)
    {
        RoleData storage data = roleMetadata[role];
        return (data.name, data.description);
    }

    /// @notice Check if a role exists
    /// @param role The role to check
    /// @return Whether the role exists
    function roleExists(bytes32 role) external view returns (bool) {
        return roleMetadata[role].exists;
    }

    /// @notice Check multiple roles for an account
    /// @param account The account to check
    /// @param rolesToCheck Array of roles to check
    /// @return Array of booleans indicating which roles the account has
    function checkRoles(address account, bytes32[] calldata rolesToCheck)
        external
        view
        returns (bool[] memory)
    {
        bool[] memory results = new bool[](rolesToCheck.length);
        for (uint256 i = 0; i < rolesToCheck.length; i++) {
            results[i] = hasRole(rolesToCheck[i], account);
        }
        return results;
    }
}
