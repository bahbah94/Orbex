import {getPolkadotApi} from "./client";
import {web3FromSource} from "@polkadot/extension-dapp";

export async function deposit(assetId: number, amount: bigint, account: string){
    const api = getPolkadotApi();
    const injector = await web3FromSource("subwallet-js");

    const tx = (await api).tx.assets.deposit(assetId,amount);
    console.log("TX:",tx.toHex());

    const res = tx.signAndSend(account, { signer: injector.signer });
    console.log("RES:",res);
    return res;
}

export async function withdraw(assetId: number, amount: bigint, account: string) {
    const api = await getPolkadotApi();
    const injector = await web3FromSource("subwallet-js");

    const tx = api.tx.exchange.withdraw(assetId, amount);

    return tx.signAndSend(account, { signer: injector.signer });
  }