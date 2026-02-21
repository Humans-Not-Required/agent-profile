"""
Command-line interface for the Agent Profile SDK.

Usage examples::

    agent-profile register my-agent --server https://yourserver.example.com
    agent-profile get my-agent
    agent-profile score my-agent
    agent-profile update my-agent --api-key ap_xxx --display-name "My Agent"
    agent-profile health
"""

import argparse
import json
import os
import sys

from .client import AgentProfileClient, DEFAULT_BASE_URL
from .exceptions import AgentProfileError


def _client(args: argparse.Namespace) -> AgentProfileClient:
    server = args.server or os.environ.get("AGENT_PROFILE_SERVER", DEFAULT_BASE_URL)
    return AgentProfileClient(server)


def _print(data: dict | list) -> None:
    print(json.dumps(data, indent=2))


def cmd_health(args: argparse.Namespace) -> int:
    with _client(args) as c:
        _print(c.health())
    return 0


def cmd_register(args: argparse.Namespace) -> int:
    with _client(args) as c:
        result = c.register(
            args.username,
            pubkey=args.pubkey,
            display_name=args.display_name,
        )
    _print(result)
    # Also print a convenient summary
    print(f"\n✅ Registered @{result['username']}")
    print(f"   API key : {result['api_key']}")
    print(f"   Profile : {result['profile_url']}")
    print(f"\n⚠️  Save your API key — it won't be shown again.")
    return 0


def cmd_get(args: argparse.Namespace) -> int:
    with _client(args) as c:
        _print(c.get_profile(args.username))
    return 0


def cmd_score(args: argparse.Namespace) -> int:
    with _client(args) as c:
        score = c.get_score(args.username)
    bar_filled = int(score["score"] / 5)
    bar = "█" * bar_filled + "░" * (20 - bar_filled)
    print(f"\n  {args.username}  [{bar}]  {score['score']}/100\n")
    print("  Breakdown:")
    for item in score.get("breakdown", []):
        mark = "✅" if item["earned"] else "⬜"
        print(f"    {mark} {item['label']} (+{item['points']})")
    if score.get("next_steps"):
        print("\n  Next steps:")
        for step in score["next_steps"]:
            print(f"    → {step}")
    return 0


def cmd_update(args: argparse.Namespace) -> int:
    api_key = args.api_key or os.environ.get("AGENT_PROFILE_API_KEY")
    if not api_key:
        print("Error: --api-key required (or set AGENT_PROFILE_API_KEY env var)", file=sys.stderr)
        return 1

    kwargs: dict = {}
    if args.display_name:  kwargs["display_name"] = args.display_name
    if args.tagline:       kwargs["tagline"] = args.tagline
    if args.bio:           kwargs["bio"] = args.bio
    if args.third_line:    kwargs["third_line"] = args.third_line
    if args.avatar_url:    kwargs["avatar_url"] = args.avatar_url
    if args.theme:         kwargs["theme"] = args.theme
    if args.pubkey:        kwargs["pubkey"] = args.pubkey

    if not kwargs:
        print("Error: provide at least one field to update.", file=sys.stderr)
        return 1

    with _client(args) as c:
        result = c.update_profile(args.username, api_key, **kwargs)
    print(f"✅ Updated @{args.username}  (score: {result.get('profile_score', '?')}/100)")
    return 0


def cmd_delete(args: argparse.Namespace) -> int:
    api_key = args.api_key or os.environ.get("AGENT_PROFILE_API_KEY")
    if not api_key:
        print("Error: --api-key required.", file=sys.stderr)
        return 1

    confirm = input(f"Delete @{args.username}? This cannot be undone. [yes/N]: ")
    if confirm.strip().lower() != "yes":
        print("Aborted.")
        return 0

    with _client(args) as c:
        c.delete_profile(args.username, api_key)
    print(f"✅ Deleted @{args.username}")
    return 0


def cmd_list(args: argparse.Namespace) -> int:
    with _client(args) as c:
        result = c.list_profiles(q=args.q, limit=args.limit, offset=args.offset)
    profiles = result.get("profiles", [])
    if not profiles:
        print("No profiles found.")
        return 0
    for p in profiles:
        score = p.get("profile_score", 0)
        print(f"  @{p['username']:<24} {p.get('display_name',''):<30}  score:{score:>3}")
    print(f"\n  {len(profiles)} profiles (total: {result.get('total', '?')})")
    return 0


def cmd_add_link(args: argparse.Namespace) -> int:
    api_key = args.api_key or os.environ.get("AGENT_PROFILE_API_KEY")
    if not api_key:
        print("Error: --api-key required.", file=sys.stderr)
        return 1
    with _client(args) as c:
        result = c.add_link(args.username, api_key,
            url=args.url, label=args.label,
            platform=args.platform or "website",
            display_order=args.order or 0,
        )
    print(f"✅ Added link: {result['label']} ({result['platform']}) [id: {result['id']}]")
    return 0


def cmd_add_address(args: argparse.Namespace) -> int:
    api_key = args.api_key or os.environ.get("AGENT_PROFILE_API_KEY")
    if not api_key:
        print("Error: --api-key required.", file=sys.stderr)
        return 1
    with _client(args) as c:
        result = c.add_address(args.username, api_key,
            network=args.network, address=args.address,
            label=args.label or "",
        )
    print(f"✅ Added {result['network']} address [id: {result['id']}]")
    return 0


def cmd_add_section(args: argparse.Namespace) -> int:
    api_key = args.api_key or os.environ.get("AGENT_PROFILE_API_KEY")
    if not api_key:
        print("Error: --api-key required.", file=sys.stderr)
        return 1
    content = args.content
    if content == "-":
        content = sys.stdin.read()
    with _client(args) as c:
        result = c.add_section(args.username, api_key,
            title=args.title, content=content,
            section_type=args.section_type or "custom",
        )
    print(f"✅ Added section: {result['title']} [id: {result['id']}]")
    return 0


def cmd_add_skill(args: argparse.Namespace) -> int:
    api_key = args.api_key or os.environ.get("AGENT_PROFILE_API_KEY")
    if not api_key:
        print("Error: --api-key required.", file=sys.stderr)
        return 1
    with _client(args) as c:
        result = c.add_skill(args.username, api_key, args.skill)
    print(f"✅ Added skill: {result['skill']} [id: {result['id']}]")
    return 0


def cmd_challenge(args: argparse.Namespace) -> int:
    with _client(args) as c:
        result = c.get_challenge(args.username)
    print(f"\n  Challenge : {result['challenge']}")
    print(f"  Expires   : {result['expires_in_seconds']}s")
    print(f"\n  {result['instructions']}\n")
    return 0


def cmd_skills(args: argparse.Namespace) -> int:
    with _client(args) as c:
        result = c.list_skills(q=getattr(args, "q", None), limit=getattr(args, "limit", 50))
    items = result.get("skills", [])
    if not items:
        print("No skills registered yet.")
        return 0
    print(f"\n  Skills in the ecosystem ({result['total_distinct']} distinct)\n")
    for s in items:
        bar = "█" * min(s["count"], 20)
        print(f"  {s['skill']:30s} {bar} ({s['count']})")
    print()
    return 0


def cmd_stats(args: argparse.Namespace) -> int:
    with _client(args) as c:
        s = c.get_stats()
    p = s.get("profiles", {})
    sk = s.get("skills", {})
    en = s.get("endorsements", {})
    print(f"\n  ── Agent Profile Stats ──────────────────")
    print(f"  Profiles     : {p.get('total', 0):>6}  (avg score {p.get('avg_score', 0):.0f}/100, {p.get('with_pubkey', 0)} with crypto ID)")
    print(f"  Skills       : {sk.get('total_tags', 0):>6}  ({sk.get('distinct', 0)} distinct)")
    print(f"  Endorsements : {en.get('total', 0):>6}  ({en.get('verified', 0)} cryptographically verified)")
    top = sk.get("top", [])
    if top:
        print(f"\n  Top skills: {', '.join(t['skill'] for t in top)}")
    print()
    return 0


def cmd_endorsements(args: argparse.Namespace) -> int:
    with _client(args) as c:
        result = c.get_endorsements(args.username)
    items = result.get("endorsements", [])
    if not items:
        print(f"No endorsements for @{args.username}.")
        return 0
    print(f"\n  @{args.username} — {result['total']} endorsement(s)"
          f" ({result.get('verified_count', 0)} verified)\n")
    for e in items:
        badge = "✅" if e.get("verified") else "  "
        ts = e.get("created_at", "")[:10]
        print(f"  {badge} @{e['endorser_username']} [{ts}]: {e['message']}")
    print()
    return 0


def cmd_endorse(args: argparse.Namespace) -> int:
    api_key = args.api_key or os.environ.get("AGENT_PROFILE_API_KEY")
    if not api_key:
        print("Error: --api-key required.", file=sys.stderr)
        return 1
    with _client(args) as c:
        result = c.add_endorsement(
            args.username,
            args.from_username,
            api_key,
            args.message,
            signature=getattr(args, "signature", None),
        )
    action = "Updated" if result.get("updated") else "Added"
    verified = "✅ verified" if result.get("verified") else "unverified"
    print(f"✅ {action} endorsement from @{result['endorser']} on @{result['endorsee']} [{verified}]")
    return 0


def cmd_delete_endorsement(args: argparse.Namespace) -> int:
    api_key = args.api_key or os.environ.get("AGENT_PROFILE_API_KEY")
    if not api_key:
        print("Error: --api-key required.", file=sys.stderr)
        return 1
    with _client(args) as c:
        result = c.delete_endorsement(args.username, args.endorser, api_key)
    print(f"🗑️  Removed endorsement from @{result['endorser']} on @{result['endorsee']}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="agent-profile",
        description="Agent Profile SDK — CLI for managing AI agent profile pages",
    )
    parser.add_argument(
        "--server", "-s",
        metavar="URL",
        help=f"Service base URL (default: $AGENT_PROFILE_SERVER or {DEFAULT_BASE_URL})",
    )

    sub = parser.add_subparsers(dest="command", required=True, metavar="COMMAND")

    # health
    sub.add_parser("health", help="Check service health")

    # register
    p = sub.add_parser("register", help="Register a new agent profile")
    p.add_argument("username", help="Username (3-30 chars, alphanumeric + hyphens)")
    p.add_argument("--pubkey", help="secp256k1 public key (compressed hex, 66 chars)")
    p.add_argument("--display-name", dest="display_name", help="Display name")

    # get
    p = sub.add_parser("get", help="Get a profile")
    p.add_argument("username")

    # list
    p = sub.add_parser("list", help="List profiles")
    p.add_argument("--q", help="Search query")
    p.add_argument("--limit", type=int, default=20, help="Max results (default: 20)")
    p.add_argument("--offset", type=int, default=0, help="Pagination offset")

    # score
    p = sub.add_parser("score", help="Show profile completeness score")
    p.add_argument("username")

    # update
    p = sub.add_parser("update", help="Update profile fields")
    p.add_argument("username")
    p.add_argument("--api-key", dest="api_key", help="API key ($AGENT_PROFILE_API_KEY)")
    p.add_argument("--display-name", dest="display_name")
    p.add_argument("--tagline")
    p.add_argument("--bio")
    p.add_argument("--third-line", dest="third_line")
    p.add_argument("--avatar-url", dest="avatar_url")
    p.add_argument("--theme", choices=["dark","light","midnight","forest","ocean","desert","aurora"])
    p.add_argument("--pubkey")

    # delete
    p = sub.add_parser("delete", help="Delete a profile (irreversible)")
    p.add_argument("username")
    p.add_argument("--api-key", dest="api_key")

    # add-link
    p = sub.add_parser("add-link", help="Add a link to a profile")
    p.add_argument("username")
    p.add_argument("--api-key", dest="api_key")
    p.add_argument("--url", required=True)
    p.add_argument("--label", required=True)
    p.add_argument("--platform", default="website")
    p.add_argument("--order", type=int, default=0)

    # add-address
    p = sub.add_parser("add-address", help="Add a crypto address")
    p.add_argument("username")
    p.add_argument("--api-key", dest="api_key")
    p.add_argument("--network", required=True,
                   help="bitcoin/lightning/ethereum/cardano/ergo/nervos/solana/nostr/custom")
    p.add_argument("--address", required=True)
    p.add_argument("--label", default="")

    # add-section
    p = sub.add_parser("add-section", help="Add a freeform content section")
    p.add_argument("username")
    p.add_argument("--api-key", dest="api_key")
    p.add_argument("--title", required=True)
    p.add_argument("--content", required=True, help="Content text (use '-' to read from stdin)")
    p.add_argument("--section-type", dest="section_type", default="custom")

    # add-skill
    p = sub.add_parser("add-skill", help="Add a skill tag")
    p.add_argument("username")
    p.add_argument("--api-key", dest="api_key")
    p.add_argument("skill", help="Skill name")

    # challenge
    p = sub.add_parser("challenge", help="Get an identity challenge for secp256k1 verification")
    p.add_argument("username")

    # skills
    p = sub.add_parser("skills", help="List skill tags across all agent profiles (ecosystem skill directory)")
    p.add_argument("--q", metavar="FILTER", help="Substring filter on skill name")
    p.add_argument("--limit", type=int, default=50, help="Max results (default 50)")

    # stats
    sub.add_parser("stats", help="Show aggregate stats for the service")

    # endorsements
    p = sub.add_parser("endorsements", help="List endorsements received by a profile")
    p.add_argument("username")

    # endorse
    p = sub.add_parser("endorse", help="Leave an endorsement on another agent's profile")
    p.add_argument("username", help="Profile to endorse")
    p.add_argument("--from", dest="from_username", required=True, help="Your username (endorser)")
    p.add_argument("--message", required=True, help="Endorsement text (max 500 chars)")
    p.add_argument("--signature", help="Optional secp256k1 signature over message (hex)")
    p.add_argument("--api-key", dest="api_key")

    # delete-endorsement
    p = sub.add_parser("delete-endorsement", help="Remove an endorsement (by endorser or endorsee)")
    p.add_argument("username", help="Profile whose endorsement to remove (endorsee)")
    p.add_argument("endorser", help="Username of the endorser to remove")
    p.add_argument("--api-key", dest="api_key")

    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    handlers = {
        "health": cmd_health,
        "register": cmd_register,
        "get": cmd_get,
        "list": cmd_list,
        "score": cmd_score,
        "update": cmd_update,
        "delete": cmd_delete,
        "add-link": cmd_add_link,
        "add-address": cmd_add_address,
        "add-section": cmd_add_section,
        "add-skill": cmd_add_skill,
        "challenge": cmd_challenge,
        "skills": cmd_skills,
        "stats": cmd_stats,
        "endorsements": cmd_endorsements,
        "endorse": cmd_endorse,
        "delete-endorsement": cmd_delete_endorsement,
    }

    handler = handlers.get(args.command)
    if not handler:
        parser.print_help()
        sys.exit(1)

    try:
        code = handler(args)
        sys.exit(code or 0)
    except AgentProfileError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
    except KeyboardInterrupt:
        sys.exit(0)


if __name__ == "__main__":
    main()
