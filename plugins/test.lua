register_command("luasay", "(message)", "Send a message from lua!", string.format([[
if #cmd_args < 1 then
    return_number = 1
else
    chat_broadcast(get_username(sender_id) .. " says: " .. table.concat(cmd_args, " "))
    set_block(0, 0, 0, 50)
end
]]))