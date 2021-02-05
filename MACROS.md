# Writing Custom Macros for Eruption

## Introduction

If you want to implement custom macros for Eruption, you need to complete the following steps. Please replace every occurrence of
`mygame` with a name of your choice.

* Create a new Lua macros file for example by copying an existing file, and using it as a starting point:

  ```bash
   $ sudo cp -v /usr/share/eruption/scripts/lib/macros/user-macros.lua /usr/share/eruption/scripts/lib/macros/mygame.lua
  ```

* Edit the newly created file and perform customizations

  ```bash
   $ sudoedit /usr/share/eruption/scripts/lib/macros/mygame.lua
  ```

* To wire-up the newly created Lua file with an existing profile,
  add the following configuration stanza to the `.profile` file:

  ```toml
  [[config.Macros]]
  type = 'string'
  name = 'requires'
  value = 'macros/mygame'
  ```

* Decide whether you just need a simple key remapping, or if you want to inject complex sequences of keystrokes
* Implement simple key remapping using the table based remapping infrastructure
* Write complex macro sequences as Lua functions that perform calls to `inject_key(...)` or `inject_key_with_delay(...)`

## Important Remarks

The functions `inject_key(...)` and `inject_key_with_delay(...)` will consume the original key event!
If you don't perform a call to one of the aforementioned functions, the original keystroke will be
delivered to the system as-is.

## Examples

The delay (in milliseconds) uses the **first** call to `inject_key_with_delay(...)` as a baseline, so you have to increase `millis` with each consecutive call.

In this example, the second 'l' will be sent to the system `200ms` after the first 'l', since the difference relative to the the baseline is `800ms-600ms == 200ms`.

```lua
  inject_key_with_delay(38, true, 600)  	-- 'l' down
  inject_key_with_delay(38, false, 700)  	-- 'l' up

  inject_key_with_delay(38, true, 800)  	-- 'l' down
  inject_key_with_delay(38, false, 900)  	-- 'l' up
```

A Lua function, that slowly types the string 'Hello!':

```lua
function easyshift_macro_3()
  debug("Executing: 'easyshift_macro_3'")

  inject_key_with_delay(42, true, 0)      -- shift down

  inject_key_with_delay(35, true, 100)  	-- 'h' down
  inject_key_with_delay(35, false, 200)  	-- 'h' up

  inject_key_with_delay(42, false, 300) 	-- shift up

  inject_key_with_delay(18, true, 400)  	-- 'e' down
  inject_key_with_delay(18, false, 500)  	-- 'e' up

  inject_key_with_delay(38, true, 600)  	-- 'l' down
  inject_key_with_delay(38, false, 700)  	-- 'l' up

  inject_key_with_delay(38, true, 800)  	-- 'l' down
  inject_key_with_delay(38, false, 900)  	-- 'l' up

  inject_key_with_delay(24, true, 1000)  	-- 'o' down
  inject_key_with_delay(24, false, 1100) 	-- 'o' up

  inject_key_with_delay(42, true, 1200)  	-- shift down

  inject_key_with_delay(2, true, 1300)  	-- '1' down
  inject_key_with_delay(2, false, 1400)  	-- '1' up

  inject_key_with_delay(42, false, 1500) 	-- shift up
end
```
