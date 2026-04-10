#!/bin/bash
set -euo pipefail

# Run cargo run -p license-issuer issue --help 
# Issue (sign) a new license file
#
# This command issues a new license file.
#
# Usage: license-issuer issue [OPTIONS] --org <ORG>

# Options:
#       --private-key <PRIVATE_KEY>  Path to private key file containing a Base64-encoded 32-byte seed. Alternatively set OXO_LICENSE_PRIVATE_KEY environment variable
#       --org <ORG>                  Full legal name of the organization (or individual for academic)
#       --email <EMAIL>              Contact e-mail address (optional)
#       --type <TYPE>                License type: "academic" or "commercial" [default: commercial]
#       --issued-at <ISSUED_AT>      Issue date in YYYY-MM-DD format (defaults to today)
#   -o, --output <OUTPUT>            Write the signed license JSON to this file (defaults to stdout)
#   -h, --help                       Print help

# Example:
# ./issue-license.sh "Shixiang Wang" "w_shixiang@163.com" academic ~/Downloads/oxo-call-license.json

if [ $# -lt 4 ]; then
    echo "Usage: $0 <org_name> <email> <type> <output_file>"
    echo "  org_name:    Full legal name of the organization (or individual for academic)"
    echo "  email:       Contact e-mail address"
    echo "  type:        License type: academic or commercial"
    echo "  output_file: Path to write the signed license JSON"
    echo ""
    echo "Example:"
    echo "  $0 \"Shixiang Wang\" \"w_shixiang@163.com\" academic ~/Downloads/oxo-call-license.json"
    exit 1
fi

ORG="$1"
EMAIL="$2"
TYPE="$3"
OUTPUT="$4"

# Validate that arguments are non-empty
if [ -z "$ORG" ]; then
    echo "Error: org_name must not be empty" >&2
    exit 1
fi
if [ -z "$EMAIL" ]; then
    echo "Error: email must not be empty" >&2
    exit 1
fi
if [ "$TYPE" != "academic" ] && [ "$TYPE" != "commercial" ]; then
    echo "Error: type must be 'academic' or 'commercial', got '$TYPE'" >&2
    exit 1
fi
if [ -z "$OUTPUT" ]; then
    echo "Error: output_file must not be empty" >&2
    exit 1
fi

# Verify cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed or not in PATH" >&2
    exit 1
fi

cargo run -p license-issuer -- issue --org "$ORG" --email "$EMAIL" --type "$TYPE" -o "$OUTPUT"
