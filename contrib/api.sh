#!/usr/bin/env bash
#
# Script for querying the API.
#
# Shellcheck can't search dynamic paths
# shellcheck source=/dev/null

set -euo pipefail

file=""                 # File name of the all-features API text file.
crate_full_name="hex_conservative" # Full crate name using underscores e.g., `bitcoin_primitives`.

# Set to false to turn off verbose output.
flag_verbose=false

usage() {
    cat <<EOF
Usage:

    ./api.sh COMMAND

CMD
  - types             Show all public types (structs and enums)
  - types_no_err      Show all public types (structs and enums) excluding error types.
EOF
}

main() {
    if [ "$#" -lt 1 ]; then
        usage
        exit 1
    fi

    local _cmd="${1:---help}"
    if [[ "$_cmd" == "-h" || "$_cmd" == "--help" ]]; then
        usage
        exit 1
    fi

    check_required_commands

    file="./api/all-features.txt"

    case $_cmd in
	types)
            structs_and_enums
            ;;

	types_no_err)
            structs_and_enums_no_err
            ;;

        traits)
            traits
            ;;

        *)
            err "Error: unknown cmd $_cmd"
            ;;
    esac
}

# Print all public structs and enums.
structs_and_enums() {
    grep -oP 'pub (struct|enum) \K[\w:]+(?:<[^>]+>)?(?=\(|;| |$)' "$file" | sed "s/^${crate_full_name}:://"
}

# Print all public structs and enums excluding error types.
structs_and_enums_no_err() {
    grep -oP 'pub (struct|enum) \K[\w:]+(?:<[^>]+>)?(?=\(|;| |$)' "$file" | sed "s/^${crate_full_name}:://" | grep -v Error
}

# Print all public traits.
traits() {
    grep -oP '^pub trait \K[\w:]+' "$file" | sed "s/^${crate_full_name}:://" | sed 's/:$//'
}

# Check all the commands we use are present in the current environment.
check_required_commands() {
    need_cmd grep
}

say() {
    echo "api: $1"
}

say_err() {
    say "$1" >&2
}

verbose_say() {
    if [ "$flag_verbose" = true ]; then
	say "$1"
    fi
}

err() {
    echo "$1" >&2
    exit 1
}

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1
    then err "need '$1' (command not found)"
    fi
}

#
# Main script
#
main "$@"
exit 0
