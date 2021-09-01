register_command("luasay", "(message)", "Send a message from lua!", string.format([[
if #cmd_args < 1 then
    return_number = 1
else
    chat_broadcast(get_username(sender_id) .. " says: " .. table.concat(cmd_args, " "))
    set_block(0, 0, 0, 50)
end
]]))

register_command("getblock", "(x) (y) (z)", "Get block id!", string.format([[
if #cmd_args < 3 then
    return_number = 1
else
    plevel = get_perm_level(sender_id)
    chat_to_id(sender_id, "Your permission level is " .. plevel)
    id = get_id(get_username(sender_id))
    chat_to_id(sender_id, "Your id is " .. id)
    blockid = get_block(cmd_args[1], cmd_args[2], cmd_args[3])
    chat_to_id(sender_id, "Block id is: " .. blockid)
end
]]))