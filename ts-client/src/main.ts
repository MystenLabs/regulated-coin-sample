import {SuiClient} from "@mysten/sui.js/client";
import {
    ADMIN_SECRET_KEY, COIN_TYPE,
    DENY_CAP_ID,
    SUI_DENY_LIST_OBJECT_ID,
    SUI_NETWORK,
    TREASURY_CAP_ID
} from "./config";
import {TransactionBlock} from '@mysten/sui.js/transactions';
import {program} from "commander";
import {fromB64} from "@mysten/sui.js/utils";
import {Ed25519Keypair} from "@mysten/sui.js/keypairs/ed25519";


const run = async () => {

    program
        .name('stablecoin-utility')
        .description('CLI to manage your Stablecoin')
        .version('0.0.1');

    program.command('deny-list-add')
        .description('Adds an address to the deny list')
        .requiredOption('--address <address>', 'Address to add')

        .action((options) => {
            console.log("Executing Addition to Deny List");
            console.log("Address to add in deny list: ", options.address);
            const txb = new TransactionBlock();

            txb.moveCall({
                target: `0x2::coin::deny_list_add`,
                arguments: [
                    txb.object(SUI_DENY_LIST_OBJECT_ID),
                    txb.object(DENY_CAP_ID),
                    txb.pure.address(options.address),
                ],
                typeArguments: [COIN_TYPE],
            });

            executeTx(txb);
        });


    program.command('deny-list-remove')
        .description('Removes an address from the deny list')
        .requiredOption('--address <address>', 'Address to add')
        .requiredOption('--deny_list <address>', 'Deny List Object ID')

        .action((options) => {
            console.log("Executing Removal from Deny List");
            console.log("Address to Remove in deny list: ", options.address);

            if(!DENY_CAP_ID) throw new Error("DENY_CAP_ID environment variable is not set. Are you sure you have the ownership of the deny list object?");

            const txb = new TransactionBlock();

            txb.moveCall({
                target: `0x2::coin::deny_list_remove`,
                arguments: [
                    txb.object(SUI_DENY_LIST_OBJECT_ID),
                    txb.object(DENY_CAP_ID),
                    txb.pure.address(options.address),
                ],
                typeArguments: [COIN_TYPE],
            });

            executeTx(txb);
        });


    program.command('mint-and-transfer')
        .description('mints coins and transfers to an address')
        .requiredOption('--amount <amount>', 'How many coins to mint')
        .requiredOption('--address <address>', 'Address to send coins')

        .action((options) => {
            console.log("Executing Mint new coins and transfer to address   ");

            console.log("Amount to mint: ", options.amount);
            console.log("Address to send coins: ", options.address);
            console.log("TREASURY_CAP_ID: ", TREASURY_CAP_ID);

            if(!TREASURY_CAP_ID) throw new Error("TREASURY_CAP_ID environment variable is not set");

            const txb = new TransactionBlock();

            txb.moveCall({
                target: `0x2::coin::mint_and_transfer`,
                arguments: [
                    txb.object(TREASURY_CAP_ID),
                    txb.pure(options.amount),
                    txb.pure.address(options.address),
                ],
                typeArguments: [COIN_TYPE],
            });

            executeTx(txb);
        });



    program.command('burn')
        .description('mints coins and transfers to an address')
        .requiredOption('--coin <address>', 'The coin to burn')
        .action((options) => {
            console.log("Executing Burn coin");
            console.log("Coin to burn: ", options.coin);

            if(!TREASURY_CAP_ID) throw new Error("TREASURY_CAP_ID environment variable is not set");

            const txb = new TransactionBlock();

            txb.moveCall({
                target: `0x2::coin::burn`,
                arguments: [
                    txb.object(TREASURY_CAP_ID),
                    txb.pure(options.coin),
                ],
                typeArguments: [COIN_TYPE],
            });

            executeTx(txb);
        });

    program.command('help')
        .description('prints help')
        .action((options) => {
            console.log("Help for stablecoin-utility");
            program.outputHelp();
        });

    program.parse();

};

run();

async function executeTx(txb: TransactionBlock) {

    console.log("Connecting to SUI network: ", SUI_NETWORK);
    const suiClient = new SuiClient({url: SUI_NETWORK});

    if(!ADMIN_SECRET_KEY) throw new Error("ADMIN_SECRET_KEY environment variable is not set");

    let adminPrivateKeyArray = Uint8Array.from(
        Array.from(fromB64(ADMIN_SECRET_KEY))
    );
    const adminKeypair = Ed25519Keypair.fromSecretKey(
        adminPrivateKeyArray.slice(1)
    );

    txb.setGasBudget(1000000000);

    suiClient.signAndExecuteTransactionBlock({
        signer: adminKeypair,
        transactionBlock: txb,
        requestType: 'WaitForLocalExecution',
        options: {
            showEvents: true,
            showEffects: true,
            showObjectChanges: true,
            showBalanceChanges: true,
            showInput: true,
        }
    }).then((res) => {

        const status = res?.effects?.status.status;

        console.log("TxDigest = ", res?.digest);
        console.log("Status = ", status);

        if (status === "success") {
            console.log("Transaction executed successfully");
        }
        if (status == "failure") {
            console.log("Transaction Error = ", res?.effects?.status);
        }
    });

}
