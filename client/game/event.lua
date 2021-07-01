-- An event bus.
local EventBus = {}

function EventBus:new()
    local o = {
        handlers = {},
        nextHandlerID = 0,
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function EventBus:trigger(eventName, event)
    local handlers = self.handlers[eventName] or {}
    for i=1,#handlers do
        handlers[i].callback(event)
    end
end

function EventBus:registerHandler(eventName, handlerFunction)
    local handlers = self.handlers[eventName] or {}
    self.handlers[eventName] = handlers
    handlers[#handlers + 1] = {
        callback = handlerFunction,
        id = self.nextHandlerID
    }
    self.nextHandlerID = self.nextHandlerID + 1
    return self.nextHandlerID - 1
end

function EventBus:deregisterHandler(id)
    for _, handlerList in pairs(self.handlers) do
        for i, handler in ipairs(handlerList) do
            if handler.id == id then
                table.remove(handlerList, i)
                return
            end
        end
    end
end

return EventBus
