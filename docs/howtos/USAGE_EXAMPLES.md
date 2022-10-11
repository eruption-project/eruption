# Table of Contents

- [Table of Contents](#table-of-contents)
  - [Switching between Slots and Profiles](#switching-between-slots-and-profiles)
  - [Adding Shell Completions](#adding-shell-completions)

## Switching between Slots and Profiles

```sh
eruptionctl switch profile /var/lib/eruption/profiles/swirl-perlin-rainbow.profile
```

Short form:

```sh
eruptionctl switch profile swirl-perlin-rainbow.profile
```

```sh
eruptionctl switch slot 4
```

## Adding Shell Completions

In this example we will use the path `~/.eruption-completion.bash` for the completions.  
Eruption supports completions through [clap-complete](https://docs.rs/clap_complete/latest/clap_complete/).  
Currently supported shells are: bash, zsh, fish, elvish and powershell.

Bash example:

```sh
eruptionctl completions bash > ~/.eruption-completion.bash
echo "[ -f ~/.eruption-completion.bash -a -r ~/.eruption-completion.bash ] && . ~/.eruption-completion.bash" >> ~/.bashrc
```
