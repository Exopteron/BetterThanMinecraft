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
    set_block(cmd_args[1], cmd_args[2], cmd_args[3], 1)
    blockid = get_block(cmd_args[1], cmd_args[2], cmd_args[3])
    chat_to_id(sender_id, "Block id is: " .. blockid)
end
]]))
register_command("fill", "x1, y1, z1, x2, y2, z2, id", "Fill an area with blocks.", string.format([[
  if #cmd_args < 7 then
    return_number = 1
  else
    x1 = tonumber(cmd_args[1])
    y1 = tonumber(cmd_args[2])
    z1 = tonumber(cmd_args[3])
    x2 = tonumber(cmd_args[4])
    y2 = tonumber(cmd_args[5])
    z2 = tonumber(cmd_args[6])
    id = tonumber(cmd_args[7])
    
    for xpos = x1, x2 do
      for ypos = z1, z2 do
        for zpos = y1, y2 do
          set_block(xpos, zpos, ypos, id)
        end
      end
    end
  end
]]))