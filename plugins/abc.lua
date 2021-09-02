game.register_command("a","b","c",string.format([[
    chat.broadcast("Hello!")
    chat.send_to_id(sender_id, "what's up!")
    block_id = world.get_block(0, 0, 0)
    chat.send_to_id(sender_id, "The block at 0, 0, 0 is currently ID " .. block_id)
    username = players.get_username(sender_id)
    chat.send_to_id(sender_id, "Your username is " .. username)
    value = storage.get_value("test")
    chat.send_to_id(sender_id, "Value is: " .. value)
    world.set_block(0, 0, 0, 35)
    logger.info("This")
    logger.warn("is")
    logger.error("a")
    logger.debug("message")
]]))
game.register_command("b", "c", "a", string.format([[
    storage.new_value("test", "a")
]]))