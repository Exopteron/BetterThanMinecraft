# Lua Plugin API
Documentation for my experimental Lua Plugin API.

# Command documentation:
Example command:
```lua
game.register_command("luasay", "(message)", "Send a message from lua!", string.format([[
if #cmd_args < 1 then
    return_number = 1
else
    chat.broadcast(players.get_username(sender_id) .. " says: " .. table.concat(cmd_args, " "))
end
]]))
```

## register_command documentation:
To register a new command, do:
```
game.register_command(command: string, arguments: string, description: string, function: string)
```
So, for creating a command called `test`, that will print "Hello World!", you would do:

```
game.register_command("test", "", "Print \"Hello, world!\"", string.format([[
    chat.send_to_id(sender_id, "Hello World!")
]]))
```
### Modules usable within a registered command:
Logger module:
```
logger.info(msg: string) - Info log
logger.warn(msg: string) - Warn log
logger.debug(msg: string) - Debug log
logger.error(msg: String) - Error log
```
Chat module:
```
chat.broadcast(msg: string) - Broadcast a chat message
chat.send_to_id(id: i8, msg: string) - Send a chat message to an ID
```
World module:
```
world.get_block(x: i16, y: i16, z: i16) - Get a block in the world
world.set_block(x: i16, y: i16, z: i16, id: u8) - Set a block in the world
```
Players module:
```
players.get_username(id: i8) - Get the username corresponding to an ID.
players.get_id(username: string) - Get the ID corresponding to a username.
players.perm_level(id: i8) - Get the permission level of an ID.
```
Global variables accessible within a registered command:
```
cmd_args - Table of arguments passed to the command.
return_number - Return code of the command ( 0 - no error, 1 - syntax error, 2 - permissions error, 3 - generic error.)
sender_id - ID of the command executor.
```