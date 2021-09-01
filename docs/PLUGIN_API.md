# Lua Plugin API
Documentation for my experimental Lua Plugin API.

# Command documentation:
Example command:
```lua
register_command("luasay", "(message)", "Send a message from lua!", string.format([[
if #cmd_args < 1 then
    return_number = 1
else
    chat_broadcast(get_username(sender_id) .. " says: " .. table.concat(cmd_args, " "))
    set_block(0, 0, 0, 50)
end
]]))
```

## register_command documentation:
To register a new command, do:
```
register_command(command: string, arguments: string, description: string, function: string)
```
So, for creating a command called `test`, that will print "Hello World!", you would do:

```
register_command("test", "", "Print \"Hello, world!\"", string.format([[
    chat_to_id(sender_id, "Hello World!")
]]))
```
Functions usable within a registered command:
```
chat_broadcast(msg: string) - Broadcast a chat message
chat_to_id(id: i8, msg: string) - Send a chat message to an ID.
set_block(x: i16, y: i16, z: i16, id: u8) - Set a block in the world.
get_block(x: i16, y: i16, z: i16) => u8 - Get a block ID from coordinates.
get_username(id: i8) => string - Get the username corresponding to an ID.
get_id(username: string) => i8 - Get the ID corresponding to a username.
get_perm_level(id: i8) => usize - Get the permission level of an ID.
```
Global variables accessible within a registered command:
```
cmd_args - Table of arguments passed to the command.
return_number - Return code of the command ( 0 - no error, 1 - syntax error, 2 - permissions error, 3 - generic error.)
sender_id - ID of the command executor.
```