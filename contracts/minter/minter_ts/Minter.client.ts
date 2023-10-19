/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin, StdFee } from "@cosmjs/amino";
import { Uint128, Logo, EmbeddedLogo, Binary, InstantiateMsg, InstantiateMsg1, Cw20Coin, InstantiateMarketingInfo, MinterResponse, ExecuteMsg, QueryMsg, ExchangingInfoResponse, String, ReceiverResponse } from "./Minter.types";
export interface MinterReadOnlyInterface {
  contractAddress: string;
  owner: () => Promise<String>;
  receiver: () => Promise<ReceiverResponse>;
  exchangingInfo: () => Promise<ExchangingInfoResponse>;
}
export class MinterQueryClient implements MinterReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.owner = this.owner.bind(this);
    this.receiver = this.receiver.bind(this);
    this.exchangingInfo = this.exchangingInfo.bind(this);
  }

  owner = async (): Promise<String> => {
    return this.client.queryContractSmart(this.contractAddress, {
      owner: {}
    });
  };
  receiver = async (): Promise<ReceiverResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      receiver: {}
    });
  };
  exchangingInfo = async (): Promise<ExchangingInfoResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      exchanging_info: {}
    });
  };
}
export interface MinterInterface extends MinterReadOnlyInterface {
  contractAddress: string;
  sender: string;
  exchange: ({
    amount,
    expectedReceived
  }: {
    amount: Uint128;
    expectedReceived: Uint128;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  withdraw: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
}
export class MinterClient extends MinterQueryClient implements MinterInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.exchange = this.exchange.bind(this);
    this.withdraw = this.withdraw.bind(this);
  }

  exchange = async ({
    amount,
    expectedReceived
  }: {
    amount: Uint128;
    expectedReceived: Uint128;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      exchange: {
        amount,
        expected_received: expectedReceived
      }
    }, fee, memo, _funds);
  };
  withdraw = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      withdraw: {}
    }, fee, memo, _funds);
  };
}