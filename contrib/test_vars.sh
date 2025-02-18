# No shebang, this file should not be executed.
# shellcheck disable=SC2148
#
# disable verify unused vars, despite the fact that they are used when sourced
# shellcheck disable=SC2034

# Test all these features with "std" enabled.
FEATURES_WITH_STD="serde"

# Test all these features without "std" or "alloc" enabled.
FEATURES_WITHOUT_STD="alloc serde"

# Run these examples.
EXAMPLES="hexy:std wrap_array_fmt_traits:std serde:std,serde"
