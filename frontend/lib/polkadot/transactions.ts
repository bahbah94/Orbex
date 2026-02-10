import {getPolkadotApi} from "./client";
import {web3FromSource} from "@polkadot/extension-dapp";

type OrderSide = "Buy" | "Sell";
type OrderType = "Limit" | "Market";

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

    const tx = api.tx.assets.withdraw(assetId, amount);

    return tx.signAndSend(account, { signer: injector.signer });
  }

export async function placeOrder(side: OrderSide,price: bigint, quantity: bigint, order_type: OrderType, account: string) {
    const api = await getPolkadotApi();
    const injector = await web3FromSource("subwallet-js");

    const tx = api.tx.assets.placeOrder(side,price,quantity,order_type);

    return tx.signAndSend(account, {signer: injector.signer});

}

export async function cancelOrder(order_id: bigint, account: string) {
    const api = await getPolkadotApi();
    const injector = await web3FromSource("subwallet-js");

    const tx = api.tx.assets.cancelOrder(order_id);

    return tx.signAndSend(account, {signer: injector.signer});

}

