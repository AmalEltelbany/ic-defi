import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Account {
  'owner' : Principal,
  'subaccount' : [] | [Uint8Array | number[]],
}
export type Result = { 'Ok' : bigint } |
  { 'Err' : string };
export interface TransferArgs { 'to_account' : Account, 'amount' : bigint }
export interface _SERVICE { 'transfer' : ActorMethod<[TransferArgs], Result> }
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
