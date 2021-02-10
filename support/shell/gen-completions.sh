#!/bin/bash

# This file is part of Eruption.

# Eruption is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.

# Eruption is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.

# You should have received a copy of the GNU General Public License
# along with Eruption.  If not, see <http://www.gnu.org/licenses/>.

function gen_completions {
    ./target/debug/"$1" "completions" "bash" > "support/shell/completions/$LANG/$1.bash-completion"
    ./target/debug/"$1" "completions" "elvish" > "support/shell/completions/$LANG/$1.elvish-completion"
    ./target/debug/"$1" "completions" "fish" > "support/shell/completions/$LANG/$1.fish-completion"
    ./target/debug/"$1" "completions" "power-shell" > "support/shell/completions/$LANG/$1.powershell-completion"
    ./target/debug/"$1" "completions" "zsh" > "support/shell/completions/$LANG/$1.zsh-completion"
}

# supported locales
languages=('en_US')

for l in ${languages[@]}
do
    export LANG=$l
    mkdir -p "support/shell/completions/$LANG/"

    # gen_completions "eruption"
    gen_completions "eruption-debug-tool"
    # gen_completions "eruption-gui"
    gen_completions "eruption-netfx"
    gen_completions "eruption-process-monitor"
    gen_completions "eruptionctl"
done

exit 0