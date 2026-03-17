#!/bin/bash
set -e

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

# 检查参数数量
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

# 使用 "$@" 或 "\"$1\"" 等方式引用带空格的参数
ORG="$1"
EMAIL="$2"
TYPE="$3"
OUTPUT="$4"

cargo run -p license-issuer issue --org "$ORG" --email "$EMAIL" --type "$TYPE" -o "$OUTPUT"
