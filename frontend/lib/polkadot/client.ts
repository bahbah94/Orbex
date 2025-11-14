import {ApiPromise, WsProvider} from "@polkadot/api"
import { env } from "../env";

let api: ApiPromise | null = null;

export async function getPolkadotApi() {
  if (api) return api;

  const wsProvider = new WsProvider("ws://127.0.0.1:9944");
  api = await ApiPromise.create({provider: wsProvider});
  return api;
}

