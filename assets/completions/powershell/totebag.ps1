
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'totebag' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'totebag'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'totebag' {
            [CompletionResult]::new('-C', '-C ', [CompletionResultType]::ParameterName, 'Prefix every entry in the archive with DIR (archive mode). For example, -C root stores src/main.rs as root/src/main.rs. By default entries keep their own paths.')
            [CompletionResult]::new('--rebase-dir', '--rebase-dir', [CompletionResultType]::ParameterName, 'Prefix every entry in the archive with DIR (archive mode). For example, -C root stores src/main.rs as root/src/main.rs. By default entries keep their own paths.')
            [CompletionResult]::new('--dir', '--dir', [CompletionResultType]::ParameterName, 'Prefix every entry in the archive with DIR (archive mode). For example, -C root stores src/main.rs as root/src/main.rs. By default entries keep their own paths.')
            [CompletionResult]::new('-B', '-B ', [CompletionResultType]::ParameterName, 'Store entries relative to DIR (archive mode). For example, -B testdata/sample stores testdata/sample/src/main.rs as src/main.rs. DIR is stripped before -C is applied. Every target must be under DIR.')
            [CompletionResult]::new('--base-dir', '--base-dir', [CompletionResultType]::ParameterName, 'Store entries relative to DIR (archive mode). For example, -B testdata/sample stores testdata/sample/src/main.rs as src/main.rs. DIR is stripped before -C is applied. Every target must be under DIR.')
            [CompletionResult]::new('-i', '-i', [CompletionResultType]::ParameterName, 'Specify the ignore type.')
            [CompletionResult]::new('--ignore-types', '--ignore-types', [CompletionResultType]::ParameterName, 'Specify the ignore type.')
            [CompletionResult]::new('-L', '-L ', [CompletionResultType]::ParameterName, 'Specify the compression level. [possible values: 0-9 (none to finest)] For more details of level of each compression method, see README.')
            [CompletionResult]::new('--level', '--level', [CompletionResultType]::ParameterName, 'Specify the compression level. [possible values: 0-9 (none to finest)] For more details of level of each compression method, see README.')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Specify the format for listing entries in the archive file.')
            [CompletionResult]::new('--output-format', '--output-format', [CompletionResultType]::ParameterName, 'Specify the format for listing entries in the archive file.')
            [CompletionResult]::new('--log', '--log', [CompletionResultType]::ParameterName, 'Specify the log level')
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'Mode of operation.')
            [CompletionResult]::new('--mode', '--mode', [CompletionResultType]::ParameterName, 'Mode of operation.')
            [CompletionResult]::new('-F', '-F ', [CompletionResultType]::ParameterName, 'Specify the archive format for listing mode (default auto). available on list and extract modes.')
            [CompletionResult]::new('--from', '--from', [CompletionResultType]::ParameterName, 'Specify the archive format for listing mode (default auto). available on list and extract modes.')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Output file in archive mode, or output directory in extraction mode')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Output file in archive mode, or output directory in extraction mode')
            [CompletionResult]::new('--to-archive-name-dir', '--to-archive-name-dir', [CompletionResultType]::ParameterName, 'extract files to DEST/ARCHIVE_NAME directory (extract mode).')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'No recursive directory (archive mode).')
            [CompletionResult]::new('--no-recursive', '--no-recursive', [CompletionResultType]::ParameterName, 'No recursive directory (archive mode).')
            [CompletionResult]::new('--overwrite', '--overwrite', [CompletionResultType]::ParameterName, 'Overwrite existing files.')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
