# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_matrix_global_optspecs
    string join \n s/speed= d/density= f/fps= c/color= p/preset= charset= chars= trail= truecolor= list-presets list-charsets list-palettes h/help V/version
end

function __fish_matrix_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_matrix_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_matrix_using_subcommand
    set -l cmd (__fish_matrix_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c matrix -n "__fish_matrix_needs_command" -s s -l speed -d 'Fall speed multiplier (0.05–8)' -r
complete -c matrix -n "__fish_matrix_needs_command" -s d -l density -d 'Column density as fraction of width (0.05–1)' -r
complete -c matrix -n "__fish_matrix_needs_command" -s f -l fps -d 'Frame rate cap' -r
complete -c matrix -n "__fish_matrix_needs_command" -s c -l color -d 'Color palette' -r
complete -c matrix -n "__fish_matrix_needs_command" -s p -l preset -d 'Named preset' -r
complete -c matrix -n "__fish_matrix_needs_command" -l charset -d 'Glyph charset' -r
complete -c matrix -n "__fish_matrix_needs_command" -l chars -d 'Custom glyph string (overrides --charset)' -r
complete -c matrix -n "__fish_matrix_needs_command" -l trail -d 'Trail length factor (0.1–1)' -r
complete -c matrix -n "__fish_matrix_needs_command" -l truecolor -d 'Force truecolor on/off' -r -f -a "true\t''
false\t''"
complete -c matrix -n "__fish_matrix_needs_command" -l list-presets -d 'List available presets and exit'
complete -c matrix -n "__fish_matrix_needs_command" -l list-charsets -d 'List available charsets and exit'
complete -c matrix -n "__fish_matrix_needs_command" -l list-palettes -d 'List available palettes and exit'
complete -c matrix -n "__fish_matrix_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c matrix -n "__fish_matrix_needs_command" -s V -l version -d 'Print version'
complete -c matrix -n "__fish_matrix_needs_command" -f -a "completions" -d 'Generate shell completions to stdout'
complete -c matrix -n "__fish_matrix_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c matrix -n "__fish_matrix_using_subcommand completions" -s h -l help -d 'Print help'
complete -c matrix -n "__fish_matrix_using_subcommand help; and not __fish_seen_subcommand_from completions help" -f -a "completions" -d 'Generate shell completions to stdout'
complete -c matrix -n "__fish_matrix_using_subcommand help; and not __fish_seen_subcommand_from completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
