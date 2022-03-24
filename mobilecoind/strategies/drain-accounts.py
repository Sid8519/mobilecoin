#!/usr/bin/env python3

# Copyright (c) 2018-2021 The MobileCoin Foundation

"""
The purpose of this strategy is to drain funds from one account to another.
This is one way to transfer tokens to a fog account, which were created after bootstrap.
This is the case for minted tokens (which fog distro cannot transfer).

It zips the source accounts and dest accounts together, and each source account sends
its entire balance (less fee) to the dest account

Example setup and usage:
```
    python3 drain-accounts.py --key-dir ../../../target/sample_data/master/keys/ --dest-key-dir ../../../target/sample_data/master/fog_keys/ --fee 20
```
"""
import argparse
import concurrent.futures
import grpc
import mobilecoind_api_pb2
import mobilecoind_api_pb2_grpc
import os
import time
from accounts import *
from google.protobuf.empty_pb2 import Empty


def parse_args() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mobilecoind-host",
                        default="localhost",
                        type=str,
                        help="Mobilecoind host")
    parser.add_argument("--mobilecoind-port",
                        default="4444",
                        type=str,
                        help="Mobilecoind port")
    parser.add_argument("--key-dir",
                        type=str,
                        help="Path to directory of account_keys")
    parser.add_argument("--dest-key-dir",
                        type=str,
                        help="Path to directory of account_keys")
    parser.add_argument("--max-seconds",
                        type=int,
                        default=40,
                        help="Number of seconds to wait for a tx to clearn")
    parser.add_argument("--fee",
                        type=int,
                        default=20,
                        help="Amount less than the balance that we attempt to send")
    return parser.parse_args()


def run_test(stub, amount, monitor_id, dest, max_seconds):
    resp = stub.GetBalance(
        mobilecoind_api_pb2.GetBalanceRequest(monitor_id=monitor_id))
    starting_balance = resp.balance
    print("Starting balance prior to transfer:", starting_balance)
    tx_stats = {}
    sync_start = time.time()
    wait_for_accounts_sync(stub, [monitor_id], 3)
    print("Time to sync:", time.time() - sync_start)
    tx_resp = stub.SendPayment(
        mobilecoind_api_pb2.SendPaymentRequest(
            sender_monitor_id=monitor_id,
            sender_subaddress=0,
            outlay_list=[
                mobilecoind_api_pb2.Outlay(
                    value=amount,
                    receiver=dest,
                )
            ],
            fee=0,
            tombstone=0,
        ))

    tx_stats[0] = {
        'start': time.time(),
        'time_delta': None,
        'tombstone': tx_resp.sender_tx_receipt.tombstone,
        'block_delta': None,
        'status': TransferStatus.pending,
        'receipt': tx_resp,
    }
    stats = poll(monitor_id, tx_stats, stub)
    # FIXME: Move max seconds check inside polling
    assert tx_stats[0]['time_delta'] < max_seconds, "Did not clear in time"
    assert tx_stats[0]['status'] == TransferStatus.success, "Transfer did not succeed"
    return stats


if __name__ == '__main__':
    args = parse_args()
    print(args)

    stub = connect(args.mobilecoind_host, args.mobilecoind_port)
    source_accounts = [
        load_key_and_register("{}/{}".format(args.key_dir, k), stub)
        for k in sorted(
            filter(lambda x: x.endswith(".json"), os.listdir(args.key_dir)))
    ]

    dest_addresses = [
        with b58_pubfile as open(k, "r"):
            b58_pubfile.read()
        for k in sorted(
            filter(lambda x: x.endswith(".b58pub"), os.listdir(args.key_dir)))
    ]

    # Go through each account and have all their friends transact to them
    for i, (src_account, dest) in enumerate(zip(accounts, dest_addresses)):
        wait_for_accounts_sync(stub, src.monitor_id, 3)
        # Get starting balance
        resp = stub.GetBalance(
            mobilecoind_api_pb2.GetBalanceRequest(monitor_id=account_data.monitor_id))
        balance = resp.balance
        print("Starting balance for account", i, ":", resp)

        # Note: due to the transaction fee, we can't assume we have enough funds
        # to divide equally among all our friends, so add an extra factor.
        amount = balance - args.fee

        # Create a pool of transfers to all other accounts
        print("Transferring", amount, "each to", len(accounts), "accounts")

        run_test(stub, amount, src_account, dest, args.max_second)
        print("Test", i, "succeeded:", stats)

    print("All transfers successful")
