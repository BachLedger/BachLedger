// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title AssetToken
 * @author BachLedger Team
 * @notice A fully decentralized, permissionless ERC-20 token with open minting
 * @dev Implements ERC-20 standard with mint/burn extensions. No access control.
 *
 * Design Decisions:
 * - Full ERC-20 compatibility for wallet/DEX integration
 * - Open minting: anyone can mint tokens (permissionless)
 * - Self-burn only: users can only burn their own tokens
 * - No supply cap: unlimited minting
 * - No owner/admin: fully decentralized
 * - No pausable/reentrancy guard: minimal attack surface
 * - Initial supply: 0
 */
contract AssetToken {
    // ============ State Variables ============

    /// @notice Token name
    string private constant _name = "AssetToken";

    /// @notice Token symbol
    string private constant _symbol = "AST";

    /// @notice Token decimals (18 = Ethereum standard)
    uint8 private constant _decimals = 18;

    /// @notice Total supply of tokens in circulation
    uint256 private _totalSupply;

    /// @notice Mapping of account balances
    mapping(address => uint256) private _balances;

    /// @notice Mapping of allowances: owner => spender => amount
    mapping(address => mapping(address => uint256)) private _allowances;

    // ============ Events ============

    /**
     * @notice Emitted when tokens are transferred
     * @param from The sender address (address(0) for mints)
     * @param to The recipient address (address(0) for burns)
     * @param value The amount transferred
     */
    event Transfer(address indexed from, address indexed to, uint256 value);

    /**
     * @notice Emitted when an allowance is set
     * @param owner The token owner
     * @param spender The approved spender
     * @param value The approved amount
     */
    event Approval(address indexed owner, address indexed spender, uint256 value);

    /**
     * @notice Emitted when new tokens are minted
     * @param to The recipient of minted tokens
     * @param amount The amount minted
     */
    event Mint(address indexed to, uint256 amount);

    /**
     * @notice Emitted when tokens are burned
     * @param from The address whose tokens were burned
     * @param amount The amount burned
     */
    event Burn(address indexed from, uint256 amount);

    // ============ ERC-20 View Functions ============

    /**
     * @notice Returns the token name
     * @return The token name "AssetToken"
     */
    function name() external pure returns (string memory) {
        return _name;
    }

    /**
     * @notice Returns the token symbol
     * @return The token symbol "AST"
     */
    function symbol() external pure returns (string memory) {
        return _symbol;
    }

    /**
     * @notice Returns the number of decimals
     * @return The number of decimals (18)
     */
    function decimals() external pure returns (uint8) {
        return _decimals;
    }

    /**
     * @notice Returns the total supply of tokens
     * @return The total supply
     */
    function totalSupply() external view returns (uint256) {
        return _totalSupply;
    }

    /**
     * @notice Returns the balance of an account
     * @param account The address to query
     * @return The balance of the account
     */
    function balanceOf(address account) external view returns (uint256) {
        return _balances[account];
    }

    /**
     * @notice Returns the allowance of a spender for an owner
     * @param owner The token owner
     * @param spender The approved spender
     * @return The remaining allowance
     */
    function allowance(address owner, address spender) external view returns (uint256) {
        return _allowances[owner][spender];
    }

    // ============ ERC-20 State-Changing Functions ============

    /**
     * @notice Transfers tokens to a recipient
     * @dev Emits a {Transfer} event
     * @param to The recipient address
     * @param amount The amount to transfer
     * @return True if successful
     */
    function transfer(address to, uint256 amount) external returns (bool) {
        _transfer(msg.sender, to, amount);
        return true;
    }

    /**
     * @notice Approves a spender to spend tokens on behalf of the caller
     * @dev Emits an {Approval} event
     * @param spender The address to approve
     * @param amount The amount to approve
     * @return True if successful
     */
    function approve(address spender, uint256 amount) external returns (bool) {
        _approve(msg.sender, spender, amount);
        return true;
    }

    /**
     * @notice Transfers tokens from one address to another using allowance
     * @dev Emits a {Transfer} event. Requires sufficient allowance.
     * @param from The sender address
     * @param to The recipient address
     * @param amount The amount to transfer
     * @return True if successful
     */
    function transferFrom(address from, address to, uint256 amount) external returns (bool) {
        _spendAllowance(from, msg.sender, amount);
        _transfer(from, to, amount);
        return true;
    }

    // ============ Extended Functions ============

    /**
     * @notice Mints new tokens to the specified address
     * @dev Anyone can call this function (permissionless minting).
     *      Emits {Mint} and {Transfer} events.
     * @param to The recipient address
     * @param amount The amount to mint
     */
    function mint(address to, uint256 amount) external {
        require(to != address(0), "AssetToken: mint to zero address");
        require(amount > 0, "AssetToken: mint amount is zero");

        _totalSupply += amount;
        _balances[to] += amount;

        emit Mint(to, amount);
        emit Transfer(address(0), to, amount);
    }

    /**
     * @notice Burns tokens from the caller's balance
     * @dev Users can only burn their own tokens.
     *      Emits {Burn} and {Transfer} events.
     * @param amount The amount to burn
     */
    function burn(uint256 amount) external {
        require(amount > 0, "AssetToken: burn amount is zero");
        require(_balances[msg.sender] >= amount, "AssetToken: burn exceeds balance");

        _balances[msg.sender] -= amount;
        _totalSupply -= amount;

        emit Burn(msg.sender, amount);
        emit Transfer(msg.sender, address(0), amount);
    }

    // ============ Safe Allowance Functions (ERC-20 Extension) ============

    /**
     * @notice Increases the allowance of a spender
     * @dev Mitigates the ERC-20 approve race condition.
     *      Emits an {Approval} event.
     * @param spender The address to increase allowance for
     * @param addedValue The amount to add to the allowance
     * @return True if successful
     */
    function increaseAllowance(address spender, uint256 addedValue) external returns (bool) {
        _approve(msg.sender, spender, _allowances[msg.sender][spender] + addedValue);
        return true;
    }

    /**
     * @notice Decreases the allowance of a spender
     * @dev Mitigates the ERC-20 approve race condition.
     *      Emits an {Approval} event.
     * @param spender The address to decrease allowance for
     * @param subtractedValue The amount to subtract from the allowance
     * @return True if successful
     */
    function decreaseAllowance(address spender, uint256 subtractedValue) external returns (bool) {
        uint256 currentAllowance = _allowances[msg.sender][spender];
        require(currentAllowance >= subtractedValue, "AssetToken: decreased allowance below zero");
        _approve(msg.sender, spender, currentAllowance - subtractedValue);
        return true;
    }

    // ============ Internal Functions ============

    /**
     * @dev Internal transfer implementation
     * @param from The sender address
     * @param to The recipient address
     * @param amount The amount to transfer
     */
    function _transfer(address from, address to, uint256 amount) internal {
        require(from != address(0), "AssetToken: transfer from zero address");
        require(to != address(0), "AssetToken: transfer to zero address");

        uint256 fromBalance = _balances[from];
        require(fromBalance >= amount, "AssetToken: transfer exceeds balance");

        _balances[from] = fromBalance - amount;
        _balances[to] += amount;

        emit Transfer(from, to, amount);
    }

    /**
     * @dev Internal approve implementation
     * @param owner The token owner
     * @param spender The approved spender
     * @param amount The approved amount
     */
    function _approve(address owner, address spender, uint256 amount) internal {
        require(owner != address(0), "AssetToken: approve from zero address");
        require(spender != address(0), "AssetToken: approve to zero address");

        _allowances[owner][spender] = amount;

        emit Approval(owner, spender, amount);
    }

    /**
     * @dev Spends allowance on behalf of an owner
     * @param owner The token owner
     * @param spender The spender using the allowance
     * @param amount The amount to spend
     */
    function _spendAllowance(address owner, address spender, uint256 amount) internal {
        uint256 currentAllowance = _allowances[owner][spender];
        if (currentAllowance != type(uint256).max) {
            require(currentAllowance >= amount, "AssetToken: insufficient allowance");
            _approve(owner, spender, currentAllowance - amount);
        }
    }
}
