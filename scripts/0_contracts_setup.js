const chainConfig = require('./config/chain').defaultChain;

const fs = require('fs');

const { SigningCosmWasmClient } = require('@cosmjs/cosmwasm-stargate');
const { DirectSecp256k1HdWallet, coin } = require('@cosmjs/proto-signing');
const { calculateFee, GasPrice } = require('@cosmjs/stargate');

// wasm folder
const wasmFolder = `${__dirname}/../artifacts`;

// gas price
const gasPrice = GasPrice.fromString(`0.025${chainConfig.denom}`);
// deployer info
let deployerWallet, deployerClient, deployerAccount;

const CONTROLLER = "aura1s9e6r0qv8nvfgzhdw9z23rpvgzzdwavu2qfjdd";
const RECEIVER = "aura1s9e6r0qv8nvfgzhdw9z23rpvgzzdwavu2qfjdd";

/// @dev Store the contract source code on chain
/// @param `wasm_name` - The name of the wasm file
/// @return `storeCodeResponse` - The response of the store code transaction
async function store_contract(wasm_name) {
    const uploadFee = calculateFee(2600000, gasPrice);
    const contractCode = fs.readFileSync(`${wasmFolder}/${wasm_name}.wasm`);

    console.log("Uploading contract code...");
    const storeCodeResponse = await deployerClient.upload(deployerAccount.address, contractCode, uploadFee, 'Upload nft_launchapad contract code');

    console.log("  transactionHash: ", storeCodeResponse.transactionHash);
    console.log("  codeId: ", storeCodeResponse.codeId);
    console.log("  gasWanted / gasUsed: ", storeCodeResponse.gasWanted, " / ", storeCodeResponse.gasUsed);

    return storeCodeResponse;
}

/// @dev Instantiate contract base on the code id and instantiate message of contract
/// @param `_codeID` - The code id of the contract
/// @param `instantiateMsg` - The instantiate message of the contract
/// @return `instantiateResponse` - The response of the instantiate transaction
async function instantiate(contract_code_id, instantiateMsg) {
    console.log("Instantiating contract...");

    //Instantiate the contract
    const instantiateResponse = await deployerClient.instantiate(
        deployerAccount.address,
        Number(contract_code_id),
        instantiateMsg,
        "instantiation contract",
        "auto",
    );
    console.log("  transactionHash: ", instantiateResponse.transactionHash);
    console.log("  contractAddress: ", instantiateResponse.contractAddress);
    console.log("  gasWanted / gasUsed: ", instantiateResponse.gasWanted, " / ", instantiateResponse.gasUsed);

    return instantiateResponse;
}

/// @dev Execute a message to the contract
/// @param `userClient` - The client of the user who execute the message
/// @param `userAccount` -  The account of the user who execute the message
/// @param `contract` - The address of the contract
/// @param `executeMsg` - The message that will be executed
/// @return `executeResponse` - The response of the execute transaction
async function execute(userClient, userAccount, contract, executeMsg, native_amount = 0, native_denom = chainConfig.denom) {
    console.log("Executing message to contract...");

    const memo = "execute a message";

    let executeResponse;

    // if the native amount is not 0, then send the native token to the contract
    if (native_amount != 0) {
        executeResponse = await userClient.execute(
            userAccount.address,
            contract,
            executeMsg,
            "auto",
            memo,
            [coin(native_amount, native_denom)],
        );
    } else {
        executeResponse = await userClient.execute(
            userAccount.address,
            contract,
            executeMsg,
            "auto",
            memo,
        );
    }


    console.log("  transactionHash: ", executeResponse.transactionHash);
    console.log("  gasWanted / gasUsed: ", executeResponse.gasWanted, " / ", executeResponse.gasUsed);

    return executeResponse;
}

/// @dev Query information from the contract
/// @param `userClient` - The client of the user who execute the message
/// @param `contract` - The address of the contract
/// @param `queryMsg` - The message that will be executed
/// @return `queryResponse` - The response of the query
async function query(userClient, contract, queryMsg) {
    console.log("Querying contract...");

    const queryResponse = await userClient.queryContractSmart(contract, queryMsg);

    console.log("  Querying successful");

    return queryResponse;
}

async function main() {
    // ***************************
    // SETUP INFORMATION FOR USERS
    // ***************************
    // connect deployer wallet to chain and get admin account
    deployerWallet = await DirectSecp256k1HdWallet.fromMnemonic(
        chainConfig.deployer_mnemonic,
        {
            prefix: chainConfig.prefix
        }
    );
    deployerClient = await SigningCosmWasmClient.connectWithSigner(chainConfig.rpcEndpoint, deployerWallet, { gasPrice });
    deployerAccount = (await deployerWallet.getAccounts())[0];

    // ****************
    // EXECUTE CONTRACT
    // ****************
    // Price feed contract
    console.log("Storing price feed contract code...");
    let storeCodeResponse = await store_contract("price_feed");
    let price_feed_code_id = storeCodeResponse.codeId;

    // prepare instantiate message for price feed contract
    let priceFeedInstantiateMsg = {
        "controller": CONTROLLER,
        "decimals": 6,
        "description": "AURA / VND",
    };

    let priceFeedInstantiateResponse = await instantiate(price_feed_code_id, priceFeedInstantiateMsg);

    console.log(priceFeedInstantiateResponse);

    // Price collector contract
    console.log("Storing price collector contract code...");
    storeCodeResponse = await store_contract("price_collector");
    let price_collector_code_id = storeCodeResponse.codeId;

    // prepare instantiate message for minter contract
    let priceCollectorInstantiateMsg = {
        "price_feed": priceFeedInstantiateResponse.contractAddress,
        "decimals": 6,
    }

    let priceCollectorInstantiateResponse = await instantiate(price_collector_code_id, priceCollectorInstantiateMsg);

    console.log(priceCollectorInstantiateResponse);

    // update price feed controller
    let updatePriceFeedControllerMsg = {
        "update_controller": {
            "controller": priceCollectorInstantiateResponse.contractAddress,
        },
    }

    let updatePriceFeedControllerResponse = await execute(deployerClient, deployerAccount, priceFeedInstantiateResponse.contractAddress, updatePriceFeedControllerMsg);
    console.log(updatePriceFeedControllerResponse);

    // update new price feeder
    let feeder_1 = "aura1s9e6r0qv8nvfgzhdw9z23rpvgzzdwavu2qfjdd";
    let updatePriceFeederMsg = {
        "update_price_feeder": {
            "price_feeder": feeder_1,
            "status": true
        },
    }

    let updatePriceFeederResponse = await execute(deployerClient, deployerAccount, priceCollectorInstantiateResponse.contractAddress, updatePriceFeederMsg);
    console.log(updatePriceFeederResponse);
}

main();
