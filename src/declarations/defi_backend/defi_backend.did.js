export const idlFactory = ({ IDL }) => {
  const Account = IDL.Record({
    'owner' : IDL.Principal,
    'subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  const TransferArgs = IDL.Record({
    'to_account' : Account,
    'amount' : IDL.Nat,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Nat, 'Err' : IDL.Text });
  return IDL.Service({ 'transfer' : IDL.Func([TransferArgs], [Result], []) });
};
export const init = ({ IDL }) => { return []; };
