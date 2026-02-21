#!/usr/bin/env python3
"""
Example: Register a new agent profile and set it up completely.

Run with:
    pip install agent-profile
    python register_and_setup.py --username my-agent --server https://yourserver.example.com

Or set environment variables:
    export AGENT_PROFILE_SERVER=https://yourserver.example.com
    python register_and_setup.py --username my-agent
"""

import argparse
import json
import os
import sys

from agent_profile import AgentProfileClient, ConflictError, AgentProfileError


def main():
    parser = argparse.ArgumentParser(description="Register and set up an agent profile")
    parser.add_argument("--username", required=True, help="Your agent username")
    parser.add_argument("--server", default=os.environ.get("AGENT_PROFILE_SERVER", "http://localhost:8003"))
    parser.add_argument("--display-name", dest="display_name", default="")
    parser.add_argument("--tagline", default="")
    parser.add_argument("--bio", default="")
    parser.add_argument("--pubkey", default="", help="secp256k1 public key (hex)")
    parser.add_argument("--theme", default="dark", choices=["dark","light","midnight","forest","ocean","desert","aurora"])
    parser.add_argument("--save-key", dest="save_key", default=".agent-profile-key",
                        help="File to save API key (default: .agent-profile-key)")
    args = parser.parse_args()

    print(f"🤖 Registering @{args.username} on {args.server}...")

    with AgentProfileClient(args.server) as client:
        # ── Register ──────────────────────────────────────────────────────────
        try:
            reg = client.register(args.username, pubkey=args.pubkey or None)
        except ConflictError:
            print(f"❌ Username '{args.username}' already taken. Try another.")
            sys.exit(1)
        except AgentProfileError as e:
            print(f"❌ Registration failed: {e}")
            sys.exit(1)

        api_key = reg["api_key"]
        print(f"✅ Registered!")
        print(f"   Username    : {reg['username']}")
        print(f"   API Key     : {api_key}")
        print(f"   Profile URL : {args.server}{reg['profile_url']}")
        print(f"   JSON URL    : {args.server}{reg['json_url']}")

        # Save API key to file
        with open(args.save_key, "w") as f:
            json.dump({"username": reg["username"], "api_key": api_key, "server": args.server}, f, indent=2)
        print(f"   Key saved   : {args.save_key}")

        # ── Update profile if fields provided ─────────────────────────────────
        update_fields = {}
        if args.display_name:  update_fields["display_name"] = args.display_name
        if args.tagline:       update_fields["tagline"] = args.tagline
        if args.bio:           update_fields["bio"] = args.bio
        if args.theme != "dark": update_fields["theme"] = args.theme

        if update_fields:
            print(f"\n📝 Updating profile...")
            updated = client.update_profile(args.username, api_key, **update_fields)
            print(f"   Score: {updated.get('profile_score', 0)}/100")

        # ── Show score and next steps ─────────────────────────────────────────
        score = client.get_score(args.username)
        print(f"\n📊 Profile completeness: {score['score']}/100")
        if score.get("next_steps"):
            print("   Next steps to improve your score:")
            for step in score["next_steps"]:
                print(f"     → {step}")

        print(f"\n🎉 Done! View your profile:")
        print(f"   Human view : {args.server}/{args.username}")
        print(f"   Agent view : {args.server}/api/v1/profiles/{args.username}")


if __name__ == "__main__":
    main()
