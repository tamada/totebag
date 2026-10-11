
use builtin;
use str;

set edit:completion:arg-completer[totebag] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'totebag'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'totebag'= {
            cand -C 'Prefix every entry in the archive with DIR (archive mode). For example, -C root stores src/main.rs as root/src/main.rs. By default entries keep their own paths.'
            cand --rebase-dir 'Prefix every entry in the archive with DIR (archive mode). For example, -C root stores src/main.rs as root/src/main.rs. By default entries keep their own paths.'
            cand --dir 'Prefix every entry in the archive with DIR (archive mode). For example, -C root stores src/main.rs as root/src/main.rs. By default entries keep their own paths.'
            cand -B 'Store entries relative to DIR (archive mode). For example, -B testdata/sample stores testdata/sample/src/main.rs as src/main.rs. DIR is stripped before -C is applied. Every target must be under DIR.'
            cand --base-dir 'Store entries relative to DIR (archive mode). For example, -B testdata/sample stores testdata/sample/src/main.rs as src/main.rs. DIR is stripped before -C is applied. Every target must be under DIR.'
            cand -i 'Specify the ignore type.'
            cand --ignore-types 'Specify the ignore type.'
            cand -L 'Specify the compression level. [possible values: 0-9 (none to finest)] For more details of level of each compression method, see README.'
            cand --level 'Specify the compression level. [possible values: 0-9 (none to finest)] For more details of level of each compression method, see README.'
            cand -f 'Specify the format for listing entries in the archive file.'
            cand --output-format 'Specify the format for listing entries in the archive file.'
            cand --log 'Specify the log level'
            cand -m 'Mode of operation.'
            cand --mode 'Mode of operation.'
            cand -F 'Specify the archive format for listing mode (default auto). available on list and extract modes.'
            cand --from 'Specify the archive format for listing mode (default auto). available on list and extract modes.'
            cand -o 'Output file in archive mode, or output directory in extraction mode'
            cand --output 'Output file in archive mode, or output directory in extraction mode'
            cand --to-archive-name-dir 'extract files to DEST/ARCHIVE_NAME directory (extract mode).'
            cand -n 'No recursive directory (archive mode).'
            cand --no-recursive 'No recursive directory (archive mode).'
            cand --overwrite 'Overwrite existing files.'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
