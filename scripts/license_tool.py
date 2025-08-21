#!/usr/bin/env python3
"""
Simple vendor tool for reStrike VTA licensing

Requires: pip install pynacl
"""
import argparse
import base64
import json
import os
import sys
import time
import uuid
import hashlib
from typing import Optional

try:
    from nacl import signing
    from nacl import encoding
except Exception as e:
    print("Please install dependencies: pip install pynacl")
    raise

PRODUCT_ID_DEFAULT = "re-strike-vta"
LICENSE_SALT = b"rst_vta_license_v1"


def b64e(data: bytes) -> str:
    return base64.b64encode(data).decode('utf-8')


def b64d(txt: str) -> bytes:
    return base64.b64decode(txt.encode('utf-8'))


def month_plan_to_seconds(plan: str) -> Optional[int]:
    if plan == 'perpetual':
        return None
    mapping = {
        '1m': 1,
        '12m': 12,
        '36m': 36,
        '60m': 60,
    }
    months = mapping.get(plan)
    if months is None:
        return 12 * 30 * 24 * 3600
    return months * 30 * 24 * 3600


def compute_machine_hash(raw_uid: str) -> str:
    h = hashlib.sha256()
    h.update(raw_uid.encode('utf-8'))
    h.update(LICENSE_SALT)
    return h.hexdigest()


def cmd_gen_keys(args):
    sk = signing.SigningKey.generate()
    vk = sk.verify_key
    # Export private in libsodium seed format; we will emit PKCS8-like via base64 of raw seed for simplicity
    sk_b = sk.encode(encoder=encoding.RawEncoder())
    vk_b = vk.encode(encoder=encoding.RawEncoder())
    print("Private key (seed, base64) - KEEP SECRET:\n" + b64e(sk_b))
    print("Public key (base64) - embed in app:\n" + b64e(vk_b))
    if args.out:
        with open(args.out, 'w', encoding='utf-8') as f:
            f.write(b64e(sk_b))
        print(f"Wrote private key to {args.out}")


def cmd_derive_pub(args):
    sk_b64 = args.sk or (open(args.sk_file, 'r', encoding='utf-8').read().strip() if args.sk_file else None)
    if not sk_b64:
        print("Provide --sk or --sk-file")
        sys.exit(1)
    sk = signing.SigningKey(b64d(sk_b64))
    vk_b = sk.verify_key.encode(encoder=encoding.RawEncoder())
    print(b64e(vk_b))


def cmd_fingerprint(args):
    if not args.uid:
        print("Provide --uid <raw_machine_uid>")
        sys.exit(1)
    print(compute_machine_hash(args.uid))


def cmd_issue(args):
    sk_b64 = args.sk or (open(args.sk_file, 'r', encoding='utf-8').read().strip() if args.sk_file else None)
    if not sk_b64:
        print("Provide --sk or --sk-file for private key")
        sys.exit(1)
    machine_hash = args.mh
    if not machine_hash and args.uid:
        machine_hash = compute_machine_hash(args.uid)
    if not machine_hash:
        print("Provide --mh or --uid to compute machine hash")
        sys.exit(1)

    plan = args.plan
    expires_sec = month_plan_to_seconds(plan)
    now = int(time.time())
    expires_at = None if expires_sec is None else now + expires_sec

    payload = {
        "product_id": args.product or PRODUCT_ID_DEFAULT,
        "machine_hash": machine_hash,
        "issued_at": now,
        "expires_at": expires_at,
        "plan": plan,
        "features": [],
        "nonce": str(uuid.uuid4()),
        "version": 1,
    }

    # Sign compact canonical JSON
    payload_bytes = json.dumps(payload, separators=(',', ':'), sort_keys=True).encode('utf-8')
    sk = signing.SigningKey(b64d(sk_b64))
    sig = sk.sign(payload_bytes).signature
    token = {
        "payload": payload,
        "signature": b64e(sig),
    }
    token_str = json.dumps(token, indent=2)
    if args.out:
        with open(args.out, 'w', encoding='utf-8') as f:
            f.write(token_str)
        print(f"Wrote license to {args.out}")
    else:
        print(token_str)


def cmd_embed_pub(args):
    sk_b64 = args.sk or (open(args.sk_file, 'r', encoding='utf-8').read().strip() if args.sk_file else None)
    if not sk_b64:
        print("Provide --sk or --sk-file")
        sys.exit(1)
    sk = signing.SigningKey(b64d(sk_b64))
    vk_b64 = b64e(sk.verify_key.encode(encoder=encoding.RawEncoder()))
    print("Replace LICENSE_PUBKEY_B64 in plugin_license.rs with:\n")
    print(vk_b64)


def build_parser():
    p = argparse.ArgumentParser(description='reStrike VTA License Tool (Python)')
    sub = p.add_subparsers(dest='cmd', required=True)

    g = sub.add_parser('gen-keys', help='Generate Ed25519 keypair')
    g.add_argument('--out', help='Save private key (base64 seed) to file')
    g.set_defaults(func=cmd_gen_keys)

    d = sub.add_parser('derive-pub', help='Derive public key from private')
    d.add_argument('--sk', help='Private key (base64 seed)')
    d.add_argument('--sk-file', help='Private key file path')
    d.set_defaults(func=cmd_derive_pub)

    f = sub.add_parser('fingerprint', help='Compute machine_hash from raw UID')
    f.add_argument('--uid', required=True, help='Raw machine UID string')
    f.set_defaults(func=cmd_fingerprint)

    i = sub.add_parser('issue', help='Issue a signed license token')
    i.add_argument('--sk', help='Private key (base64 seed)')
    i.add_argument('--sk-file', help='Private key file path')
    i.add_argument('--mh', help='Machine hash (hex)')
    i.add_argument('--uid', help='Raw machine UID (compute hash automatically)')
    i.add_argument('--plan', default='12m', choices=['1m','12m','36m','60m','perpetual'])
    i.add_argument('--product', default=PRODUCT_ID_DEFAULT)
    i.add_argument('--out', help='Output file path for license JSON')
    i.set_defaults(func=cmd_issue)

    e = sub.add_parser('embed-pub', help='Print public key for embedding into app')
    e.add_argument('--sk', help='Private key (base64 seed)')
    e.add_argument('--sk-file', help='Private key file path')
    e.set_defaults(func=cmd_embed_pub)

    return p


def main():
    parser = build_parser()
    args = parser.parse_args()
    args.func(args)


if __name__ == '__main__':
    main()


