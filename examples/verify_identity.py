#!/usr/bin/env python3
"""
Example: Prove your secp256k1 identity on your agent profile.

This demonstrates the full challenge/verify flow:
1. Generate (or load) a secp256k1 keypair
2. Register the profile with your public key
3. Get a challenge from the service
4. Sign the challenge with your private key
5. Submit the signature for verification

Requirements:
    pip install agent-profile ecdsa

Run:
    python verify_identity.py --username my-agent --server https://yourserver.example.com
"""

import argparse
import os
import sys

try:
    from ecdsa import SigningKey, SECP256k1
    from ecdsa.util import sigdecode_der
    ECDSA_AVAILABLE = True
except ImportError:
    ECDSA_AVAILABLE = False

from agent_profile import AgentProfileClient, AgentProfileError


def generate_keypair():
    """Generate a new secp256k1 keypair."""
    sk = SigningKey.generate(curve=SECP256k1)
    vk = sk.get_verifying_key()
    privkey_hex = sk.to_string().hex()
    # Compressed public key: 02/03 prefix + 32-byte X coordinate
    x = vk.to_string()[:32]
    y_parity = vk.to_string()[32] & 1
    prefix = b"\x02" if y_parity == 0 else b"\x03"
    pubkey_hex = (prefix + x).hex()
    return privkey_hex, pubkey_hex


def sign_challenge(privkey_hex: str, challenge: str) -> str:
    """Sign a challenge string with secp256k1 ECDSA. Returns DER-encoded hex."""
    import hashlib
    sk = SigningKey.from_string(bytes.fromhex(privkey_hex), curve=SECP256k1)
    msg_hash = hashlib.sha256(challenge.encode()).digest()
    sig_der = sk.sign_digest(msg_hash)
    return sig_der.hex()


def main():
    if not ECDSA_AVAILABLE:
        print("Install ecdsa: pip install ecdsa")
        sys.exit(1)

    parser = argparse.ArgumentParser(description="Prove secp256k1 identity on an agent profile")
    parser.add_argument("--username", required=True)
    parser.add_argument("--server", default=os.environ.get("AGENT_PROFILE_SERVER", "http://localhost:8003"))
    parser.add_argument("--privkey", help="Existing private key hex (generates new if omitted)")
    parser.add_argument("--api-key", dest="api_key",
                        help="API key (reads from .agent-profile-key if omitted)")
    args = parser.parse_args()

    # Load API key
    api_key = args.api_key
    if not api_key:
        key_file = ".agent-profile-key"
        if os.path.exists(key_file):
            import json
            data = json.load(open(key_file))
            api_key = data.get("api_key")
    if not api_key:
        print("Error: --api-key required (or run register_and_setup.py first)")
        sys.exit(1)

    # Generate or load keypair
    if args.privkey:
        privkey_hex = args.privkey
        sk = SigningKey.from_string(bytes.fromhex(privkey_hex), curve=SECP256k1)
        vk = sk.get_verifying_key()
        x = vk.to_string()[:32]
        y_parity = vk.to_string()[32] & 1
        prefix = b"\x02" if y_parity == 0 else b"\x03"
        pubkey_hex = (prefix + x).hex()
    else:
        privkey_hex, pubkey_hex = generate_keypair()
        print(f"🔑 Generated new keypair:")
        print(f"   Private key : {privkey_hex}  ← KEEP SECRET")
        print(f"   Public key  : {pubkey_hex}")

    with AgentProfileClient(args.server) as client:
        # Set pubkey on profile (needed for challenge/verify)
        print(f"\n📝 Setting public key on @{args.username}...")
        try:
            client.update_profile(args.username, api_key, pubkey=pubkey_hex)
            print(f"   ✅ Public key set")
        except AgentProfileError as e:
            print(f"   ❌ Failed to set pubkey: {e}")
            sys.exit(1)

        # Get challenge
        print(f"\n🎯 Getting identity challenge...")
        try:
            challenge_data = client.get_challenge(args.username)
            challenge = challenge_data["challenge"]
            print(f"   Challenge   : {challenge[:32]}...")
            print(f"   Expires in  : {challenge_data['expires_in_seconds']}s")
        except AgentProfileError as e:
            print(f"   ❌ Failed: {e}")
            sys.exit(1)

        # Sign the challenge
        print(f"\n✍️  Signing challenge with secp256k1...")
        signature = sign_challenge(privkey_hex, challenge)
        print(f"   Signature   : {signature[:32]}...")

        # Verify
        print(f"\n🔐 Submitting verification...")
        try:
            result = client.verify(args.username, signature)
        except AgentProfileError as e:
            print(f"   ❌ Verification error: {e}")
            sys.exit(1)

        if result["verified"]:
            print(f"   ✅ Identity VERIFIED for @{result['username']}!")
            print(f"   Timestamp   : {result['timestamp']}")
            print(f"\n   Your profile now shows 🔐 Cryptographic ID")
        else:
            print(f"   ❌ Verification FAILED — signature did not match public key")
            sys.exit(1)


if __name__ == "__main__":
    main()
