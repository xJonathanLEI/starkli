# Shell completions

Automatic shell completions are critical to good user experience for CLI tools. Starkli supports shell completions for the following shells:

- _bash_
- _elvish_
- _fish_
- _powershell_
- _zsh_

When [installing with `starkliup`](./installation.md#using-starkliup), shell completions are automatically set up for the following shells:

- _bash_
- _zsh_

and you don't need to configure them yourself.

Otherwise, you can generate shell completion files by running the following command:

```console
starkli completions <SHELL>
```

which prints the content of the completion file to the standard output. For example, to generate the completion file for _fish_:

```console
starkli completions fish
```

You can pipe the output into a completion file and place it at the folder expected by your shell. You might need to restart your shell session for the changes to take effect.
