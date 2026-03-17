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
# cargo run -p license-issuer issue --org "ShixiangWang" --email "w_shixiang@163.com" --type academic -o ~/Downloads/oxo-call-license.json
cargo run -p license-issuer issue --org $1 --email $2 --type $3 -o $4

