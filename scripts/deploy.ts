import 'dotenv/config'
import {
    deployContract,
    executeContract, initialize,
    instantiateContract,
    queryContract,
    recover,
    setTimeoutDuration,
    uploadContract,
} from "./helpers.js"
import { LCDClient, LocalTerra, Wallet } from "@terra-money/terra.js"
import {testnet, bombay, local} from './deploy_configs.js';
import { join } from "path"

const ARTIFACTS_PATH = "../artifacts/"

async function main() {
    let terra: LCDClient | LocalTerra
    let wallet: Wallet
    let deployConfig: Config

    if (process.env.NETWORK === "testnet") {
        terra = new LCDClient({
            URL: String(process.env.LCD_CLIENT_URL),
            chainID: String(process.env.CHAIN_ID)
        })
        wallet = recover(terra, process.env.WALLET!)
        //wallet = initialize(terra)
        //Account Address: terra1ulx8rulpwcx86v4v0awxgadhxhqz95v9s8jkhd
        //MnemonicKey: cradle similar beef van gift start afford blush cry tobacco angry tuna buzz purpose cloud silver atom behind crouch vault pill afraid huge risk

        deployConfig = testnet

    } else  if (process.env.NETWORK === "bombay") {
        terra = new LCDClient({
            URL: String(process.env.LCD_CLIENT_URL),
            chainID: String(process.env.CHAIN_ID)
        })
        wallet = recover(terra, process.env.WALLET!)
        deployConfig = bombay
    } else {
        console.log("NETWORK:" +process.env.NETWORK)
        terra = new LocalTerra()
        wallet = (terra as LocalTerra).wallets.test1
        setTimeoutDuration(0)
        deployConfig = local
    }
    console.log(`Wallet address from seed: ${wallet.key.accAddress}`)

    if (!deployConfig.astroTokenContractAddress) {
        console.log(`Please deploy the CW20-base ASTRO token, and then set this address in the deploy config before running this script...`)
        return
    }

    /*************************************** Register Pairs Contract *****************************************/
    console.log("Register Pairs Contract...")
    let pairCodeID = await uploadContract(terra, wallet, join(ARTIFACTS_PATH, 'astroport_pair.wasm')!)
    deployConfig.factoryInitMsg.config.pair_configs[0].code_id = pairCodeID

    let pairStableCodeID = await uploadContract(terra, wallet, join(ARTIFACTS_PATH, 'astroport_pair_stable.wasm')!)
    deployConfig.factoryInitMsg.config.pair_configs[0].code_id = pairStableCodeID
    console.log("Code Pair Contract: " + pairCodeID + " Code Stable Pair Contract: " + pairStableCodeID)

    /*************************************** Deploy Factory Contract *****************************************/
    console.log("Deploying Factory...")
    const addressFactoryContract = await deployContract(
        terra,
        wallet,
        join(ARTIFACTS_PATH, 'astroport_factory.wasm'),
        deployConfig.factoryInitMsg.config
    )
    console.log("Address Factory Contract: " + addressFactoryContract)

    // /*************************************** Deploy Router Contract *****************************************/
    console.log("Deploying Router...")
    const addressRouterContract = await deployContract(
        terra,
        wallet,
        join(ARTIFACTS_PATH, 'astroport_router.wasm'),
        {
            astroport_factory: addressFactoryContract,
        },
    )
    console.log("Address Router Contract: " + addressRouterContract)

    /*************************************** Deploy Vesting Contract *****************************************/
    console.log("Deploying Vesting...")
    const addressVestingContract = await deployContract(
        terra,
        wallet,
        join(ARTIFACTS_PATH, 'astroport_vesting.wasm'),
        {
            owner: wallet.key.accAddress,
            token_addr: deployConfig.astroTokenContractAddress,
            genesis_time: String(Date.now())
        },
    )
    console.log("Address Vesting Contract: " + addressVestingContract)

    /*************************************** Deploy Staking Contract *****************************************/
    console.log("Deploying Staking...")
    const addressStakingContract = await deployContract(
        terra,
        wallet,
        join(ARTIFACTS_PATH, 'astroport_staking.wasm'),
        deployConfig.stakingInitMsg.config
    )
    console.log("Address Staking Contract: " + addressStakingContract)


    /*************************************** Deploy Maker Contract *****************************************/
    console.log("Deploying Maker...")
    const addressMakerContract = await deployContract(
        terra,
        wallet,
        join(ARTIFACTS_PATH, 'maker.wasm'),
        {
            factory_contract: String(addressFactoryContract),
            staking_contract: String(addressStakingContract),
            astro_token_contract: String(deployConfig.astroTokenContractAddress),
        }
    )
    console.log("Address Gauge Contract: " + addressMakerContract)
    /*************************************** Deploy Gauge Contract *****************************************/
    console.log("Deploying Gauge...")

    //deployConfig.gaugeInitMsg.config.token = deployConfig.astroTokenContractAddress;
    deployConfig.gaugeInitMsg.config.dev_addr = wallet.key.accAddress;

    const addressGaugeContract = await deployContract(
        terra,
        wallet,
        join(ARTIFACTS_PATH, 'astroport_gauge.wasm'),
        deployConfig.gaugeInitMsg.config
    )
    console.log("Address Gauge Contract: " + addressGaugeContract)


    /*************************************** Setting tokens to Vesting Contract *****************************************/

    const vestingAccounts = (
        deployConfig.registerVestingAccounts.register_vesting_accounts.vesting_accounts
    ).map(account => ({
        ...account,
        address: addressGaugeContract,
    }));

    deployConfig.registerVestingAccounts.register_vesting_accounts.vesting_accounts = vestingAccounts

    const { registerVestingAccounts } = deployConfig;

    console.log("Setting Vesting...")
    await executeContract(
        terra,
        wallet,
        addressVestingContract,
        registerVestingAccounts,
    )

    // const addressVestingContract = "terra1kyl8f2xkd63cga8szgkejdyvxay7mc7qpdc3c5"
    // const addressGaugeContract = "terra1qjrvlf27upqhqnrqmmu2y205ed2c3tc87dnku3"

    //console.log("Vesting accounts setup: ", await queryContract(terra, addressVestingContract, { "vesting_accounts": { } }))

    console.log("FINISH")
}

main().catch(console.log)