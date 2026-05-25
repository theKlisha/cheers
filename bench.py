#!/usr/bin/env python3
"""Quick benchmark: cheers vs stockfish via fastchess."""

import re
import subprocess
import sys

ROUNDS = 10
TC = "1+0.1"
ENGINE = "./target/release/cheers"
BOOK = "books/2moves_v2.pgn"


def build():
    result = subprocess.run(
        ["cargo", "build", "--release"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(result.stderr, file=sys.stderr)
        sys.exit(1)


def run_match():
    cmd = [
        "fastchess",
        "-engine", f"cmd=stockfish", "name=Stockfish",
        "-engine", f"cmd={ENGINE}", "name=Cheers",
        "-each", f"tc={TC}",
        "-rounds", str(ROUNDS),
        "-repeat",
        "-openings", f"file={BOOK}", "format=pgn", "order=random",
        "-resign", "movecount=3", "score=600",
        "-draw", "movenumber=5", "movecount=3", "score=0",
        "-output", "format=fastchess",
    ]
    proc = subprocess.Popen(cmd, text=True, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    lines = []
    for line in proc.stdout:
        print(line, end="", flush=True)
        lines.append(line)
    proc.wait()
    return "".join(lines)


def parse(output):
    scores = re.search(
        r"Games:\s*(\d+),\s*Wins:\s*(\d+),\s*Losses:\s*(\d+),\s*Draws:\s*(\d+)",
        output,
    )
    elo = re.search(r"Elo:\s*([-\d.inf]+)\s*\+/-\s*([\d.nan]+)", output)
    los = re.search(r"LOS:\s*([\d.]+)\s*%", output)

    if not scores:
        return None

    return {
        "games":   int(scores.group(1)),
        "wins":    int(scores.group(2)),
        "losses":  int(scores.group(3)),
        "draws":   int(scores.group(4)),
        "elo":     elo.group(1) if elo else "?",
        "elo_err": elo.group(2) if elo else "?",
        "los":     los.group(1) if los else "?",
    }


def compliance_check():
    result = subprocess.run(
        ["fastchess", "--compliance", ENGINE],
        capture_output=True,
        text=True,
    )
    print(result.stdout, end="")
    if result.returncode != 0:
        print("Compliance check failed", file=sys.stderr)
        sys.exit(1)
    print()


def main():
    print(f"Building cheers (release)...")
    build()

    print("Running UCI compliance check...")
    compliance_check()

    print(f"Running {ROUNDS * 2} games vs Stockfish (tc={TC})...\n")
    output = run_match()

    r = parse(output)
    if not r:
        print("Could not parse fastchess output:")
        print(output)
        sys.exit(1)

    w, l, d, g = r["wins"], r["losses"], r["draws"], r["games"]
    # scores are from Stockfish's perspective -- flip for Cheers
    cheers_pts = l + d * 0.5
    print(f"Cheers vs Stockfish ({TC}, {g} games)")
    print(f"  W/L/D:  {l} / {w} / {d}  (cheers wins / stockfish wins / draws)")
    print(f"  Points: {cheers_pts:.1f} / {g}  ({100 * cheers_pts / g:.1f}%)")
    print(f"  Elo:    {r['elo']} +/- {r['elo_err']}")
    print(f"  LOS:    {r['los']}%")


if __name__ == "__main__":
    main()
