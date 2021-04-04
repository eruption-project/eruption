-- This file is part of Eruption.

-- Eruption is free software: you can redistribute it and/or modify
-- it under the terms of the GNU General Public License as published by
-- the Free Software Foundation, either version 3 of the License, or
-- (at your option) any later version.

-- Eruption is distributed in the hope that it will be useful,
-- but WITHOUT ANY WARRANTY without even the implied warranty of
-- MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
-- GNU General Public License for more details.

-- You should have received a copy of the GNU General Public License
-- along with Eruption.  If not, see <http://www.gnu.org/licenses/>.

require "declarations"
require "utilities"
require "debug"

-- global state variables --
color_map = {}
ticks = 0
split_column = 0
easing_state = 1
alpha = 0

max_effect_ttl = target_fps * 4
effect_ttl = max_effect_ttl

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do
        color_map[i] = 0x00000000
    end
end

function on_key_down(key_index)
    split_column = trunc(key_index / max_keys_per_col)
    easing_state = 0

	effect_ttl = max_effect_ttl
end

function on_tick(delta)
    if effect_ttl <= 0 then return end

    ticks = ticks + delta

    if easing_state == 0 then
        alpha = clamp(alpha + ((alpha + 5) * 0.35) + 1, 0, 255)
    elseif easing_state == 1 then
        alpha = clamp(alpha - ((alpha + 5) * 0.35) + 1, 0, 255)
    elseif easing_state == 2 then
        alpha = 0
    end

    -- left wave
    for i = split_column, 0, -1 do
        for j = 1, max_keys_per_col do
            if easing_state == 0 then
                local local_alpha

                if alpha <= 0 then
                    local_alpha = 0
                else
                    local_alpha = clamp(alpha - lerp(0, 255, sin(i / wave_length)), 0, 255)
                end

                local index = n(cols_topology[j + (i * max_keys_per_col)]) + 1
                color_map[index] = hsla_to_color((i * hue_multiplier), color_saturation, color_lightness, local_alpha)

                if alpha >= 255 then
                    easing_state = 1
                end
            elseif easing_state == 1 then
                local local_alpha

                if alpha <= 0 then
                    local_alpha = 0
                    easing_state = 2
                else
                    local_alpha = clamp(alpha + lerp(0, 255, sin(i / wave_length)), 0, 255)
                end

                local index = n(cols_topology[j + (i * max_keys_per_col)]) + 1
                color_map[index] = hsla_to_color((i * hue_multiplier), color_saturation, color_lightness, local_alpha)
            elseif easing_state == 2 then
                local index = n(cols_topology[j + (i * max_keys_per_col)]) + 1
                color_map[index] = 0x00000000
            else
                error("Invalid easing_state")
            end
        end
    end

    -- right wave
    for i = split_column, num_cols, 1 do
        for j = 1, max_keys_per_col do
            if easing_state == 0 then
                local local_alpha

                if alpha <= 0 then
                    local_alpha = 0
                else
                    local_alpha = clamp(alpha - lerp(255, 0, sin(i / wave_length)), 0, 255)
                end

                local index = n(cols_topology[j + (i * max_keys_per_col)]) + 1
                color_map[index] = hsla_to_color((i * hue_multiplier), color_saturation, color_lightness, local_alpha)

                if alpha >= 255 then
                    easing_state = 1
                end
            elseif easing_state == 1 then
                local local_alpha

                if alpha <= 0 then
                    local_alpha = 0
                    easing_state = 2
                else
                    local_alpha = clamp(alpha + lerp(255, 0, sin(i / wave_length)), 0, 255)
                end

                local index = n(cols_topology[j + (i * max_keys_per_col)]) + 1
                color_map[index] = hsla_to_color((i * hue_multiplier), color_saturation, color_lightness, local_alpha)
            elseif easing_state == 2 then
                local index = n(cols_topology[j + (i * max_keys_per_col)]) + 1
                color_map[index] = 0x00000000
            else
                error("Invalid easing_state")
            end
        end
    end

    submit_color_map(color_map)

    effect_ttl = effect_ttl - 1
end
